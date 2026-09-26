use anyhow::{anyhow, bail, Result};
use riffle_core::candidate;
use riffle_core::decode::{apply_orientation, decode_rgb};
use riffle_core::faces;
use riffle_core::partial;
use riffle_core::reader;
use riffle_core::scan;
use riffle_core::sharpness;
use riffle_core::xmp;
use riffle_core::Flag;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Instant;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("info") => info(Path::new(&args[1])),
        Some("focusbox") => focusbox(Path::new(&args[1]), Path::new(&args[2])),
        Some("faces") => faces(Path::new(&args[1]), Path::new(&args[2])),
        Some("bench") => bench(&args[1..]),
        Some("crop") => crop(
            Path::new(&args[1]),
            Path::new(&args[2]),
            args.get(3).map(|s| s.parse()).transpose()?,
        ),
        Some("scan") => scan_dir(Path::new(&args[1]), args.get(2).map(|t| t.parse()).transpose()?),
        Some("candidates") => {
            let mut dirs = &args[1..];
            let threads = match dirs.last().map(|t| t.parse::<usize>()) {
                Some(Ok(n)) => {
                    dirs = &dirs[..dirs.len() - 1];
                    Some(n)
                }
                _ => None,
            };
            if dirs.is_empty() {
                bail!("usage: riffle-cli candidates <dir>... [threads]");
            }
            candidates(&dirs.iter().map(PathBuf::from).collect::<Vec<_>>(), threads)
        }
        _ => bail!(
            "usage: riffle-cli <info|focusbox|faces|bench> <file.ARW> [out.png]\n       riffle-cli crop <file.ARW> <out.png> [size]\n       riffle-cli scan <dir> [threads]\n       riffle-cli candidates <dir>... [threads]"
        ),
    }
}

fn info(path: &Path) -> Result<()> {
    let a = reader::read_metadata(path)?;
    println!("orientation: {}", a.orientation);
    println!("preview: {:?}", a.preview);
    println!("full:    {:?}", a.full);
    println!(
        "capture_time: {}",
        a.shot.capture_time.as_deref().unwrap_or("-")
    );
    println!("subsec: {}", a.shot.subsec.as_deref().unwrap_or("-"));
    match a.shot.focus {
        Some(f) => println!("focus: {} {} {} {}", f.sensor_w, f.sensor_h, f.x, f.y),
        None => println!("focus: -"),
    }
    match a.shot.focus_mode {
        Some(m) => println!("focus mode: {m}"),
        None => println!("focus mode: -"),
    }
    Ok(())
}

fn draw_rect(rgb: &mut [u8], w: usize, h: usize, x0: i64, y0: i64, bw: i64, bh: i64) {
    let mut put = |x: i64, y: i64| {
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let i = (y as usize * w + x as usize) * 3;
            rgb[i] = 255;
            rgb[i + 1] = 0;
            rgb[i + 2] = 0;
        }
    };
    for t in 0..3 {
        for x in x0..x0 + bw {
            put(x, y0 + t);
            put(x, y0 + bh - t);
        }
        for y in y0..y0 + bh {
            put(x0 + t, y);
            put(x0 + bw - t, y);
        }
    }
}

fn focusbox(path: &Path, out: &Path) -> Result<()> {
    let (a, jpeg) = reader::read_preview(path)?;

    let t = Instant::now();
    let (rgb, w, h) = decode_rgb(&jpeg)?;
    println!("preview {w}x{h} decoded in {:?}", t.elapsed());

    let f = a
        .shot
        .focus
        .ok_or_else(|| anyhow!("no FocusLocation in {path:?}"))?;
    let (fw, fh, fx, fy) = (f.sensor_w, f.sensor_h, f.x, f.y);
    let (frame_w, frame_h) = a
        .shot
        .focus_frame
        .map_or((219, 219), |f| (f.width, f.height));
    println!("focus: sensor {fw}x{fh} at ({fx},{fy}) frame {frame_w}x{frame_h}");

    // FocusLocation is in unrotated sensor coordinates, so draw on the unrotated preview first.
    let sx = w as f64 / fw as f64;
    let sy = h as f64 / fh as f64;
    let bw = (frame_w as f64 * sx) as i64;
    let bh = (frame_h as f64 * sy) as i64;
    let mut rgb = rgb;
    draw_rect(
        &mut rgb,
        w,
        h,
        (fx as f64 * sx) as i64 - bw / 2,
        (fy as f64 * sy) as i64 - bh / 2,
        bw,
        bh,
    );

    // Rotate to display orientation, then write it out.
    let (rgb, w, h) = apply_orientation(&rgb, w, h, a.orientation);
    image::save_buffer(out, &rgb, w as u32, h as u32, image::ColorType::Rgb8)?;
    println!("wrote {out:?} ({w}x{h}, orientation {})", a.orientation);
    Ok(())
}

/// Print the focus candidate cue, and draw the searched region (the crop
/// around the AF point, or nothing for the whole image), the AF point, the
/// faces found with their eyes and the eye window of the nearest face on the
/// upright preview.
fn faces(path: &Path, out: &Path) -> Result<()> {
    let (a, jpeg) = reader::read_preview(path)?;
    let focus = sharpness::trusted_focus(&a.shot);
    // The first call builds the model; time a second one too.
    let t = Instant::now();
    let d = faces::detect_around(&jpeg, a.orientation, focus)?;
    let first = t.elapsed();
    let t = Instant::now();
    faces::detect_around(&jpeg, a.orientation, focus)?;
    let (w, h) = (d.width, d.height);
    println!(
        "preview {w}x{h}: {} face(s), decode + detection {:?} (first call {first:?})",
        d.faces.len(),
        t.elapsed()
    );
    let cue = candidate::focus_cue(&jpeg, a.orientation, focus)?;
    println!(
        "candidate: {:?}, in focus {}",
        cue.state,
        cue.eye_focus
            .map_or("-".to_string(), |p| format!("{:.0}%", p * 100.0))
    );
    let (mut rgb, _, _) = decode_rgb(&jpeg)?;
    if let Some(f) = cue.face {
        println!(
            "nearest face ({:.0},{:.0}) {:.0}x{:.0}",
            f.x, f.y, f.width, f.height
        );
        let m = candidate::eye_focus(&candidate::luma(&rgb, w, h), w, h, &f);
        println!(
            "lap {:.1}, edge width {}, logit {}",
            m.lap,
            m.edge_width.map_or("-".to_string(), |e| format!("{e:.2}")),
            m.logit.map_or("-".to_string(), |l| format!("{l:.3}"))
        );
        let r = candidate::eye_window(w, h, &f);
        draw_rect(
            &mut rgb,
            w,
            h,
            r.x as i64,
            r.y as i64,
            r.width as i64 - 1,
            r.height as i64 - 1,
        );
    }
    if let Some((px, py)) = d.point {
        println!("AF point ({px},{py})");
        let r = sharpness::window_at(w, h, px, py, faces::CATCH_CROP);
        draw_rect(
            &mut rgb,
            w,
            h,
            r.x as i64,
            r.y as i64,
            r.width as i64 - 1,
            r.height as i64 - 1,
        );
        draw_rect(&mut rgb, w, h, px as i64 - 10, py as i64 - 10, 20, 20);
    }
    for f in &d.faces {
        println!(
            "face ({:.0},{:.0}) {:.0}x{:.0} score {:.2} eyes ({:.0},{:.0}) ({:.0},{:.0})",
            f.x,
            f.y,
            f.width,
            f.height,
            f.score,
            f.left_eye.0,
            f.left_eye.1,
            f.right_eye.0,
            f.right_eye.1
        );
        draw_rect(
            &mut rgb,
            w,
            h,
            f.x as i64,
            f.y as i64,
            f.width as i64,
            f.height as i64,
        );
        for (x, y) in [f.left_eye, f.right_eye] {
            draw_rect(&mut rgb, w, h, x as i64 - 4, y as i64 - 4, 8, 8);
        }
    }
    let (rgb, w, h) = faces::upright_rgb(&rgb, w, h, a.orientation);
    image::save_buffer(out, &rgb, w as u32, h as u32, image::ColorType::Rgb8)?;
    println!("wrote {out:?} ({w}x{h}, orientation {})", a.orientation);
    Ok(())
}

const CROP_SIZE: usize = 512;

fn stats(label: &str, mut ms: Vec<f64>) {
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = ms.len();
    let mean = ms.iter().sum::<f64>() / n as f64;
    println!(
        "{label:<28} n={n:<4} mean {mean:6.1}ms  median {:6.1}ms  p95 {:6.1}ms  max {:6.1}ms",
        ms[n / 2],
        ms[((n as f64 * 0.95) as usize).min(n - 1)],
        ms[n - 1]
    );
}

fn bench(paths: &[String]) -> Result<()> {
    let mut t_preview = Vec::new();
    let mut t_full = Vec::new();
    let mut t_crop = Vec::new();
    let mut t_faces = Vec::new();

    for p in paths {
        let path = Path::new(p);
        let a = reader::read_metadata(path)?;

        if a.preview.is_some() {
            let (_, jpeg) = reader::read_preview(path)?;
            let t = Instant::now();
            let (rgb, w, h) = decode_rgb(&jpeg)?;
            t_preview.push(t.elapsed().as_secs_f64() * 1000.0);

            // YuNet is trained on upright faces, so orient before detecting.
            let (rgb, w, h) = apply_orientation(&rgb, w, h, a.orientation);

            if t_faces.is_empty() {
                // Build the model outside the timing.
                faces::detect(&rgb, w, h)?;
            }
            let t = Instant::now();
            faces::detect(&rgb, w, h)?;
            t_faces.push(t.elapsed().as_secs_f64() * 1000.0);
        }

        if a.full.is_some() {
            let (a, jpeg) = reader::read_full(path)?;
            let t = Instant::now();
            decode_rgb(&jpeg)?;
            t_full.push(t.elapsed().as_secs_f64() * 1000.0);

            // The center fallback the app uses when there is no FocusLocation.
            let t = Instant::now();
            partial::decode_focus_crop(&jpeg, a.shot.focus, CROP_SIZE, CROP_SIZE)?;
            t_crop.push(t.elapsed().as_secs_f64() * 1000.0);
        }
    }

    println!();
    if !t_preview.is_empty() {
        stats("1. preview decode", t_preview);
    }
    if !t_full.is_empty() {
        stats("2. JpgFromRaw full decode", t_full);
    }
    if !t_crop.is_empty() {
        stats("3. partial decode 512px", t_crop);
    }
    if !t_faces.is_empty() {
        stats("4. face detection", t_faces);
    }
    Ok(())
}

/// Extract a whole folder in parallel, with no database: the folder scan
/// throughput benchmark.
fn scan_dir(dir: &Path, threads: Option<usize>) -> Result<()> {
    let threads =
        threads.unwrap_or_else(|| std::thread::available_parallelism().map_or(4, |n| n.get()));
    if threads == 0 {
        bail!("threads must be at least 1");
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| riffle_core::scan::is_raw_file(p))
        .collect();
    paths.sort();
    if paths.is_empty() {
        bail!("no RAW (ARW/DNG) files in {dir:?}");
    }

    let start = Instant::now();
    // Per-file cost is the gap between two completions on the same worker, so
    // each worker's previous completion is kept alongside the totals.
    let last = Mutex::new(vec![start; threads]);
    let acc = Mutex::new((Vec::<f64>::new(), 0usize, 0usize));
    scan::extract_all(
        &paths,
        threads,
        |_, r| {
            let now = Instant::now();
            let ms = {
                let mut last = last.lock().unwrap();
                let w = rayon::current_thread_index().unwrap_or(0);
                let ms = now.duration_since(last[w]).as_secs_f64() * 1000.0;
                last[w] = now;
                ms
            };
            let mut acc = acc.lock().unwrap();
            acc.0.push(ms);
            match r {
                Ok(e) => acc.1 += e.thumbnail.len(),
                Err(_) => acc.2 += 1,
            }
        },
        &AtomicBool::new(false),
    )
    .map_err(|e| anyhow!(e))?;
    let total = start.elapsed();

    let (mut ms, bytes, errors) = acc.into_inner().unwrap();
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = paths.len();
    println!(
        "{n} files, {threads} threads, {errors} errors: {:.2}s total, {:.0} files/s",
        total.as_secs_f64(),
        n as f64 / total.as_secs_f64()
    );
    println!(
        "per file on a worker: mean {:.1}ms  p95 {:.1}ms",
        ms.iter().sum::<f64>() / ms.len() as f64,
        ms[((ms.len() as f64 * 0.95) as usize).min(ms.len() - 1)]
    );
    println!(
        "thumbnails: {bytes} bytes total, {} bytes mean",
        bytes / n.max(1)
    );
    Ok(())
}

/// Compute the focus candidate cue of every RAW file in one or more folders
/// in parallel and, over the files with a face whose XMP sidecar holds a
/// pick / reject flag (a pick is in focus), pooled across the folders, print
/// the AUC of the Laplacian alone and of the combined logit, and the precision
/// and coverage of the candidates.
fn candidates(dirs: &[PathBuf], threads: Option<usize>) -> Result<()> {
    let threads =
        threads.unwrap_or_else(|| std::thread::available_parallelism().map_or(4, |n| n.get()));
    if threads == 0 {
        bail!("threads must be at least 1");
    }
    let mut paths: Vec<PathBuf> = Vec::new();
    for dir in dirs {
        let mut found: Vec<PathBuf> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| riffle_core::scan::is_raw_file(p))
            .collect();
        if found.is_empty() {
            bail!("no RAW (ARW/DNG) files in {dir:?}");
        }
        found.sort();
        paths.extend(found);
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()?;

    let start = Instant::now();
    let cues: Vec<_> = pool.install(|| {
        use rayon::prelude::*;
        paths.par_iter().map(|p| scan::extract_faces(p)).collect()
    });
    let total = start.elapsed();
    // The measures behind each face's probability, for the report only.
    let measures: Vec<Option<candidate::EyeFocus>> = pool.install(|| {
        use rayon::prelude::*;
        paths
            .par_iter()
            .zip(&cues)
            .map(|(p, cue)| {
                let face = cue.as_ref().ok()?.face?;
                let (_, jpeg) = reader::read_preview(p).ok()?;
                let (rgb, w, h) = decode_rgb(&jpeg).ok()?;
                let gray = candidate::luma(&rgb, w, h);
                Some(candidate::eye_focus(&gray, w, h, &face))
            })
            .collect()
    });

    // `hits` are candidates in focus: the numerator of both rates.
    let (mut labeled, mut cands, mut in_focus, mut hits) = (0, 0, 0, 0);
    let (mut errors, mut faced, mut no_edge) = (0, 0, 0);
    // (in focus, lap, logit) of each labeled faced frame.
    let mut scored: Vec<(bool, f64, Option<f64>)> = Vec::new();
    for ((path, cue), m) in paths.iter().zip(&cues).zip(&measures) {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let flag = std::fs::read(xmp::sidecar_path(path))
            .ok()
            .and_then(|b| xmp::read_flag(&b).ok())
            .filter(|f| *f != Flag::None);
        let cue = match cue {
            Ok(c) => c,
            Err(e) => {
                errors += 1;
                println!("{name}  error: {e}");
                continue;
            }
        };
        let face = cue.face.map_or("-".to_string(), |f| {
            format!("({:.0},{:.0}) {:.0}x{:.0}", f.x, f.y, f.width, f.height)
        });
        let opt =
            |v: Option<f64>, digits: usize| v.map_or("-".to_string(), |v| format!("{v:.digits$}"));
        println!(
            "{name}  {:?}  p {}  lap {}  edge {}  {face}  {}",
            cue.state,
            opt(cue.eye_focus, 3),
            opt(m.map(|m| m.lap), 1),
            opt(m.and_then(|m| m.edge_width), 2),
            flag.map_or("-".to_string(), |f| format!("{f:?}"))
        );
        if cue.face.is_some() {
            faced += 1;
            no_edge += m.is_some_and(|m| m.edge_width.is_none()) as usize;
        }
        let (Some(flag), Some(m)) = (flag, m) else {
            continue;
        };
        labeled += 1;
        let cand = cue.state == candidate::FocusCandidate::Candidate;
        let pick = flag == Flag::Pick;
        cands += cand as usize;
        in_focus += pick as usize;
        hits += (cand && pick) as usize;
        scored.push((pick, m.lap, m.logit));
    }
    println!(
        "{} files in {} folder(s), {threads} threads, {errors} errors: {:.2}s total",
        paths.len(),
        dirs.len(),
        total.as_secs_f64()
    );
    println!("{faced} with a face, {no_edge} of them with no edge width (probability 0)");
    if labeled > 0 {
        let pct = |a: usize, b: usize| 100.0 * a as f64 / b.max(1) as f64;
        println!(
            "{labeled} labeled with a face ({in_focus} in focus, {} off): candidates {cands}, in focus {hits} ({:.1}%); in-focus frames {in_focus}, candidates {hits} ({:.1}%)",
            labeled - in_focus,
            pct(hits, cands),
            pct(hits, in_focus)
        );
        let with_edge: Vec<(bool, f64)> = scored
            .iter()
            .filter_map(|&(pick, _, logit)| logit.map(|l| (pick, l)))
            .collect();
        println!(
            "AUC: lap {:.3}, combined {:.3} ({} frames with an edge width)",
            auc(scored.iter().map(|&(pick, lap, _)| (pick, lap))),
            auc(with_edge.iter().copied()),
            with_edge.len()
        );
    }
    Ok(())
}

/// The area under the ROC curve of `(in focus, score)` pairs: the share of
/// (in focus, off) pairs where the in-focus frame scores higher, ties 0.5.
fn auc(frames: impl Iterator<Item = (bool, f64)>) -> f64 {
    let (pos, neg): (Vec<_>, Vec<_>) = frames.partition(|(pick, _)| *pick);
    let mut s = 0.0;
    for (_, a) in &pos {
        for (_, b) in &neg {
            s += if a > b {
                1.0
            } else if a == b {
                0.5
            } else {
                0.0
            };
        }
    }
    s / (pos.len() * neg.len()) as f64
}

/// Write out the partial decode so it can be checked by eye.
fn crop(path: &Path, out: &Path, size: Option<usize>) -> Result<()> {
    let size = size.unwrap_or(CROP_SIZE);
    let t = Instant::now();
    let (a, jpeg) = reader::read_full(path)?;
    println!(
        "read JpgFromRaw ({} bytes) in {:?}",
        jpeg.len(),
        t.elapsed()
    );

    let t = Instant::now();
    let c = partial::decode_focus_crop(&jpeg, a.shot.focus, size, size)?;
    println!(
        "crop {}x{} at ({},{}) point ({},{}) in {:?}",
        c.crop.width,
        c.crop.height,
        c.crop.x,
        c.crop.y,
        c.point_x,
        c.point_y,
        t.elapsed()
    );

    let rgb: Vec<u8> = c
        .crop
        .pixels
        .chunks_exact(4)
        .flat_map(|p| p[..3].to_vec())
        .collect();
    let (rgb, w, h) = apply_orientation(&rgb, c.crop.width, c.crop.height, a.orientation);
    image::save_buffer(out, &rgb, w as u32, h as u32, image::ColorType::Rgb8)?;
    println!("wrote {out:?} ({w}x{h})");
    Ok(())
}
