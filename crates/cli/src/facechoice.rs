use anyhow::{bail, Result};
use riffle_core::candidate::{self, EyeFocus, Scored};
use riffle_core::decode::decode_rgb;
use riffle_core::eyes;
use riffle_core::faces::{self, Face};
use riffle_core::reader;
use riffle_core::scan;
use riffle_core::sharpness;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const USAGE: &str = "riffle-cli facechoice <dir|file>... [threads] [--out <out.tsv>] [--check]";

/// The header of the dump, the column names of `frame_lines`.
const HEADER: &str = "folder\tfile\taf\taf_x\taf_y\tset\tface\tx\ty\tw\th\tscore\tleft_eye\tright_eye\tnearest\teye_focus\tstate\tscored\tear\tyaw\tpitch\troll\tmesh_ms";

/// One face of a set, scored as the scan scores its judged face.
#[derive(Clone, Copy)]
struct Judged {
    face: Face,
    focus: EyeFocus,
    /// The ms `eyes::mesh_of` took; `None` when `meshes_face` turned it down.
    mesh_ms: Option<f64>,
}

/// Both face sets of one frame, in stored coordinates.
struct Frame {
    /// The crop detection's AF point; `None` without a trusted AF point.
    point: Option<(usize, usize)>,
    /// `detect_around_rgb` around the AF point; empty without one.
    crop: Vec<Judged>,
    /// `decode_whole` + `detect_whole_upright`.
    whole: Vec<Judged>,
}

/// Dump every face of both face sets of every RAW file given or in the
/// folders given (the face-choice plan's Step 2): one `frame_lines` row per
/// (file, set, face) after `HEADER`, to `--out` or stdout. With `--check`
/// each file's `af_rule` value is compared with
/// `scan::extract_analysis(path).cue.eye_focus` and the mismatches counted.
/// Errors and the totals go to stderr.
pub fn run(args: &[String]) -> Result<()> {
    let (mut out, mut check, mut rest) = (None, false, Vec::new());
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--out" => match it.next() {
                Some(p) => out = Some(PathBuf::from(p)),
                None => bail!("usage: {USAGE}"),
            },
            "--check" => check = true,
            _ => rest.push(a.as_str()),
        }
    }
    let threads = match rest.last().map(|t| t.parse::<usize>()) {
        Some(Ok(n)) => {
            rest.pop();
            n
        }
        _ => std::thread::available_parallelism().map_or(4, |n| n.get()),
    };
    if rest.is_empty() {
        bail!("usage: {USAGE}");
    }
    if threads == 0 {
        bail!("threads must be at least 1");
    }
    let paths = crate::raw_paths(&rest.iter().map(PathBuf::from).collect::<Vec<_>>())?;
    // Build the models outside the timing.
    faces::detect(&[0; 3], 1, 1)?;
    faces::detect_whole(&[0; 3], 1, 1)?;
    faces::detect_whole(&[0; 6], 1, 2)?;
    let probe = Face {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
        score: 1.0,
        left_eye: (0.0, 0.0),
        right_eye: (0.0, 0.0),
        nose: (0.0, 0.0),
        mouth_left: (0.0, 0.0),
        mouth_right: (0.0, 0.0),
    };
    eyes::mesh_of(&[0; 3], 1, 1, 1, &probe);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()?;

    let start = Instant::now();
    let results: Vec<(Result<Frame>, Option<Option<f64>>)> = pool.install(|| {
        use rayon::prelude::*;
        paths
            .par_iter()
            .map(|p| {
                let stored =
                    check.then(|| scan::extract_analysis(p).ok().and_then(|a| a.cue.eye_focus));
                (frame(p), stored)
            })
            .collect()
    });
    let total = start.elapsed();

    let mut lines = format!("{HEADER}\n");
    let (mut errors, mut mismatches, mut crop_faces, mut whole_faces) = (0, 0, 0, 0);
    let mut mesh_ms = Vec::new();
    for (path, (frame, stored)) in paths.iter().zip(results) {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let folder = path
            .parent()
            .and_then(Path::file_name)
            .unwrap_or_default()
            .to_string_lossy();
        let frame = match frame {
            Ok(f) => f,
            Err(e) => {
                errors += 1;
                eprintln!("{name}  error: {e:#}");
                continue;
            }
        };
        for line in frame_lines(&folder, &name, &frame) {
            lines.push_str(&line);
            lines.push('\n');
        }
        crop_faces += frame.crop.len();
        whole_faces += frame.whole.len();
        mesh_ms.extend(
            frame
                .crop
                .iter()
                .chain(&frame.whole)
                .filter_map(|j| j.mesh_ms),
        );
        if let Some(stored) = stored {
            let af = af_rule(&frame);
            let same = match (af, stored) {
                (None, None) => true,
                (Some(a), Some(b)) => (a - b).abs() <= 1e-9,
                _ => false,
            };
            if !same {
                mismatches += 1;
                eprintln!("{name}  mismatch: af {af:?}, scan {stored:?}");
            }
        }
    }
    match &out {
        Some(path) => std::fs::write(path, &lines)?,
        None => print!("{lines}"),
    }
    eprintln!(
        "{} files, {threads} threads, {errors} errors: {:.2}s total; {crop_faces} crop faces, {whole_faces} whole faces",
        paths.len(),
        total.as_secs_f64()
    );
    if !mesh_ms.is_empty() {
        let n = mesh_ms.len();
        let (mean, median, p95, max) = crate::summary(mesh_ms);
        eprintln!(
            "mesh n={n} mean {mean:.1}ms  median {median:.1}ms  p95 {p95:.1}ms  max {max:.1}ms"
        );
    }
    if check {
        eprintln!(
            "check: {mismatches} mismatches of {} files",
            paths.len() - errors
        );
    }
    Ok(())
}

/// Both face sets of the RAW file at `path`, every face scored on one
/// full-size decode of its preview.
fn frame(path: &Path) -> Result<Frame> {
    let (a, jpeg) = reader::read_preview(path)?;
    let focus = sharpness::trusted_focus(&a.shot);
    let (rgb, w, h) = decode_rgb(&jpeg)?;
    let gray = candidate::luma(&rgb, w, h);
    let judge = |face: &Face| {
        let t = Instant::now();
        let meshed = candidate::meshes_face(face);
        let mesh = meshed
            .then(|| eyes::mesh_of(&rgb, w, h, a.orientation, face))
            .flatten();
        let mesh_ms = meshed.then(|| t.elapsed().as_secs_f64() * 1000.0);
        Judged {
            face: *face,
            focus: candidate::eye_focus(&gray, w, h, face, mesh.as_ref()),
            mesh_ms,
        }
    };
    let (point, crop) = match focus {
        Some(_) => {
            let d = faces::detect_around_rgb(&rgb, w, h, a.orientation, focus)?;
            (d.point, d.faces.iter().map(judge).collect())
        }
        None => (None, Vec::new()),
    };
    let whole =
        faces::detect_whole_upright(&faces::decode_whole(&jpeg, a.orientation)?, a.orientation)?
            .faces
            .iter()
            .map(judge)
            .collect();
    Ok(Frame { point, crop, whole })
}

/// The index of the `candidate::nearest_face` of `set` to `point`.
fn nearest(set: &[Judged], point: Option<(usize, usize)>) -> Option<usize> {
    let faces: Vec<Face> = set.iter().map(|j| j.face).collect();
    let n = candidate::nearest_face(&faces, point?)?;
    faces.iter().position(|f| std::ptr::eq(f, n))
}

/// The `eye_focus` of today's rule: with an AF point the crop set's nearest
/// face, without one the whole set's `eyes::judged_face` (the largest at or
/// above `sharpness::FACE_CONFIDENCE`).
fn af_rule(frame: &Frame) -> Option<f64> {
    let (set, i) = match frame.point {
        Some(_) => (&frame.crop, nearest(&frame.crop, frame.point)?),
        None => {
            let faces: Vec<Face> = frame.whole.iter().map(|j| j.face).collect();
            let j = eyes::judged_face(&faces, None)?;
            (&frame.whole, faces.iter().position(|f| std::ptr::eq(f, j))?)
        }
    };
    Some(set[i].focus.probability)
}

/// The rows of one frame: its crop set, then its whole set, one row per face
/// in detection order, or one row with `face` and every face column `-` for
/// an empty set. `nearest` is 1 on the set's `nearest_face` to the crop
/// detection's AF point (0 everywhere without one).
fn frame_lines(folder: &str, file: &str, frame: &Frame) -> Vec<String> {
    let head = match frame.point {
        Some((x, y)) => format!("{folder}\t{file}\t1\t{x}\t{y}"),
        None => format!("{folder}\t{file}\t0\t-\t-"),
    };
    let mut lines = Vec::new();
    for (name, set) in [("crop", &frame.crop), ("whole", &frame.whole)] {
        if set.is_empty() {
            lines.push(format!("{head}\t{name}\t-{}", "\t-".repeat(16)));
            continue;
        }
        let near = nearest(set, frame.point);
        for (i, j) in set.iter().enumerate() {
            lines.push(format!(
                "{head}\t{name}\t{i}\t{}",
                face_columns(j, near == Some(i))
            ));
        }
    }
    lines
}

/// The 16 columns after `face` of one face's row.
fn face_columns(j: &Judged, nearest: bool) -> String {
    let opt = |v: Option<f64>| v.map_or("-".to_string(), |v| v.to_string());
    let f = &j.face;
    let mesh = j.focus.mesh;
    let pose = mesh.and_then(|m| m.pose);
    let scored = match j.focus.scored {
        Scored::Sharper(e) | Scored::Only(e) => match e {
            candidate::Eye::Left => "L",
            candidate::Eye::Right => "R",
        },
        Scored::Window => "window",
    };
    format!(
        "{}\t{}\t{}\t{}\t{}\t{},{}\t{},{}\t{}\t{}\t{:?}\t{scored}\t{}\t{}\t{}\t{}\t{}",
        f.x,
        f.y,
        f.width,
        f.height,
        f.score,
        f.left_eye.0,
        f.left_eye.1,
        f.right_eye.0,
        f.right_eye.1,
        nearest as u8,
        j.focus.probability,
        j.focus.state,
        opt(mesh.and_then(|m| m.ear)),
        opt(pose.map(|p| p.yaw)),
        opt(pose.map(|p| p.pitch)),
        opt(pose.map(|p| p.roll)),
        opt(j.mesh_ms)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use riffle_core::candidate::{EyeMeasures, EyeRegions, FocusCandidate};
    use riffle_core::pose::Pose;
    use riffle_core::sharpness::Window;

    fn face(x: f32, side: f32, score: f32) -> Face {
        Face {
            x,
            y: 10.0,
            width: side,
            height: side,
            score,
            left_eye: (x + side * 0.3, 10.0 + side * 0.4),
            right_eye: (x + side * 0.7, 10.0 + side * 0.4),
            nose: (0.0, 0.0),
            mouth_left: (0.0, 0.0),
            mouth_right: (0.0, 0.0),
        }
    }

    fn judged(face: Face, probability: f64, meshed: bool) -> Judged {
        let regions = EyeRegions {
            contour: None,
            iris: None,
        };
        Judged {
            face,
            focus: EyeFocus {
                window: Window {
                    x: 0,
                    y: 0,
                    width: 24,
                    height: 24,
                },
                lap: 1.0,
                edge_width: None,
                edge_width_rel: None,
                logit: None,
                probability,
                state: FocusCandidate::NotCandidate,
                scored: if meshed {
                    Scored::Sharper(candidate::Eye::Right)
                } else {
                    Scored::Window
                },
                mesh: meshed.then_some(EyeMeasures {
                    left: regions,
                    right: regions,
                    ear: Some(0.25),
                    pose: Some(Pose {
                        yaw: 1.5,
                        pitch: -2.0,
                        roll: 0.125,
                    }),
                    eye_offset: None,
                    edge_gap: None,
                }),
            },
            mesh_ms: meshed.then_some(30.5),
        }
    }

    #[test]
    fn a_face_row_lists_every_column_at_full_precision() {
        let frame = Frame {
            point: Some((100, 50)),
            crop: vec![judged(face(80.0, 100.0, 0.9), 0.123456789012345, true)],
            whole: vec![judged(face(1000.0, 40.0, 0.7), 0.0, false)],
        };
        let lines = frame_lines("2026-06-05", "_DSC0001.ARW", &frame);
        assert_eq!(
            lines,
            [
                "2026-06-05\t_DSC0001.ARW\t1\t100\t50\tcrop\t0\t80\t10\t100\t100\t0.9\t110,50\t150,50\t1\t0.123456789012345\tNotCandidate\tR\t0.25\t1.5\t-2\t0.125\t30.5",
                "2026-06-05\t_DSC0001.ARW\t1\t100\t50\twhole\t0\t1000\t10\t40\t40\t0.7\t1012,26\t1028,26\t1\t0\tNotCandidate\twindow\t-\t-\t-\t-\t-",
            ]
        );
        assert!(lines
            .iter()
            .all(|l| l.split('\t').count() == HEADER.split('\t').count()));
    }

    #[test]
    fn each_set_marks_its_own_nearest_face_and_an_empty_set_gets_one_dash_row() {
        let far = judged(face(1000.0, 100.0, 0.95), 0.2, false);
        let near = judged(face(80.0, 100.0, 0.7), 0.6, false);
        let frame = Frame {
            point: Some((110, 50)),
            crop: Vec::new(),
            whole: vec![far, near],
        };
        let lines = frame_lines("d", "f.ARW", &frame);
        let column = |l: &str, i: usize| l.split('\t').nth(i).unwrap().to_string();
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines[0],
            format!("d\tf.ARW\t1\t110\t50\tcrop\t-{}", "\t-".repeat(16))
        );
        assert_eq!(lines[0].split('\t').count(), HEADER.split('\t').count());
        assert_eq!(
            lines[1..].iter().map(|l| column(l, 14)).collect::<Vec<_>>(),
            ["0", "1"]
        );
        // The crop set is empty with an AF point, so today's rule has no face.
        assert_eq!(af_rule(&frame), None);
    }

    #[test]
    fn without_an_af_point_nothing_is_nearest_and_the_rule_takes_the_largest_confident_face() {
        let small = judged(face(0.0, 50.0, 0.95), 0.2, false);
        let large_low = judged(face(200.0, 300.0, 0.7), 0.9, false);
        let large = judged(face(600.0, 120.0, 0.85), 0.4, false);
        let frame = Frame {
            point: None,
            crop: Vec::new(),
            whole: vec![small, large_low, large],
        };
        let lines = frame_lines("d", "f.ARW", &frame);
        assert!(lines[0].starts_with("d\tf.ARW\t0\t-\t-\tcrop\t-\t"));
        assert!(lines[1..]
            .iter()
            .all(|l| l.split('\t').nth(14) == Some("0")));
        assert_eq!(af_rule(&frame), Some(0.4));
    }

    #[test]
    fn with_an_af_point_the_rule_takes_the_crop_sets_nearest_face() {
        let frame = Frame {
            point: Some((1012, 50)),
            crop: vec![
                judged(face(80.0, 100.0, 0.9), 0.3, false),
                judged(face(1000.0, 40.0, 0.7), 0.8, false),
            ],
            whole: vec![judged(face(80.0, 100.0, 0.9), 0.1, false)],
        };
        assert_eq!(af_rule(&frame), Some(0.8));
    }
}
