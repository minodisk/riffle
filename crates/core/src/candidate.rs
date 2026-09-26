//! The focus candidate cue: how likely the eyes of the face nearest the AF
//! point are in focus, and whether that clears `CANDIDATE_LOGIT`.
//!
//! The score combines two measures over a window between the eyes: the
//! Laplacian variance of the luma (`lap`) and the mean edge width
//! (`edge_width`) relative to the window side, in a logistic regression
//! fitted on hand-labeled frames (`LOGIT_INTERCEPT`, `LOGIT_LAP`,
//! `LOGIT_EDGE_WIDTH`). Its sigmoid is the in-focus probability.
//!
//! Faces come from `faces::detect_around_rgb` in a `CATCH_CROP` square around
//! the trusted AF point. A Sony eye-AF frame gets the same detection: the
//! camera tracking a face does not say whether that face is sharp.
//! Everything is in the preview's stored (unrotated) coordinates.

use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;

use crate::arw::FocusLocation;
use crate::decode::decode_rgb;
use crate::faces::{detect_around_rgb, Detection, Face};
use crate::sharpness::{laplacian_variance, window_at, Window};

/// The intercept of the in-focus logit
/// `LOGIT_INTERCEPT + LOGIT_LAP * ln(lap + 1) + LOGIT_EDGE_WIDTH * ln(edge_width / window side)`,
/// fitted on 406 hand-labeled faced frames from 5 folders. A coefficient
/// change is a new model: re-validate it with `riffle-cli candidates`.
pub const LOGIT_INTERCEPT: f64 = -4.725633355883976;
/// The weight of `ln(lap + 1)` in the in-focus logit.
pub const LOGIT_LAP: f64 = 0.6826458385175557;
/// The weight of `ln(edge_width / window side)` in the in-focus logit.
pub const LOGIT_EDGE_WIDTH: f64 = -1.1949836425055467;
/// The logit at or above which a frame is a focus candidate (an in-focus
/// probability of about 77%), chosen to keep the in-focus coverage of the
/// earlier Laplacian-only threshold. On the 406 training frames the combined
/// score reaches AUC 0.852 (0.816 for the Laplacian alone), and the frames at
/// or above it are in focus 93.1% of the time and cover 91.4% of the
/// in-focus frames. On 400 held-out frames from 4 other folders it reaches
/// AUC 0.754 (0.635), precision 89.1% and coverage 95.3%.
pub const CANDIDATE_LOGIT: f64 = 1.2194865955352432;
/// Smallest side of the eye window, in preview pixels. The combined score was
/// fitted and validated on this window, the face box's long side, at least
/// 24 px (the earlier Laplacian-only validation on 500 frames also used it).
pub const CANDIDATE_WINDOW_MIN: usize = 24;

/// Whether a frame is a focus candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusCandidate {
    /// The eyes of the face nearest the AF point are sharp.
    Candidate,
    /// A face lies near the AF point, and its eyes are not sharp.
    NotCandidate,
    /// No trusted AF point, no face near it, or the preview could not be
    /// decoded or searched.
    #[default]
    Unknown,
}

/// The cue of one preview.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cue {
    pub state: FocusCandidate,
    /// The in-focus probability of the eyes of `face`
    /// (`EyeFocus::probability`); `None` when `state` is `Unknown`.
    pub eye_focus: Option<f64>,
    /// The face nearest the AF point, in stored coordinates.
    pub face: Option<Face>,
    /// The detection the face was picked from; `None` without an AF point,
    /// and in the `Cue::unknown` a caller falls back to after a failure.
    pub detection: Option<Detection>,
}

impl Cue {
    pub fn unknown() -> Self {
        Self::default()
    }
}

/// The face in `faces` whose closer eye is nearest `point`, the distance
/// divided by the face box's long side so a large face does not win on size
/// alone. Faces whose long side is not positive are skipped.
pub fn nearest_face(faces: &[Face], point: (usize, usize)) -> Option<&Face> {
    let (px, py) = (point.0 as f32, point.1 as f32);
    faces
        .iter()
        .filter(|f| f.width.max(f.height) > 0.0)
        .map(|f| {
            let eye = |(x, y): (f32, f32)| (x - px).hypot(y - py);
            let d = eye(f.left_eye).min(eye(f.right_eye)) / f.width.max(f.height);
            (f, d)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(f, _)| f)
}

/// The square window centered between the eyes of `face` (clamped into the
/// `width` x `height` image), its side the face box's long side, at least
/// `CANDIDATE_WINDOW_MIN`.
pub fn eye_window(width: usize, height: usize, face: &Face) -> Window {
    let cx = (face.left_eye.0 + face.right_eye.0) / 2.0;
    let cy = (face.left_eye.1 + face.right_eye.1) / 2.0;
    let side = (face.width.max(face.height).max(0.0) as usize).max(CANDIDATE_WINDOW_MIN);
    window_at(
        width,
        height,
        (cx.max(0.0) as usize).min(width - 1),
        (cy.max(0.0) as usize).min(height - 1),
        side,
    )
}

/// The luma `(299 R + 587 G + 114 B) / 1000` of each pixel of a
/// `width` x `height` RGB image, the formula the combined score was
/// validated with.
pub fn luma(rgb: &[u8], width: usize, height: usize) -> Vec<u8> {
    rgb.chunks_exact(3)
        .take(width * height)
        .map(|p| ((299 * p[0] as u32 + 587 * p[1] as u32 + 114 * p[2] as u32) / 1000) as u8)
        .collect()
}

/// The Marziliano-style mean edge width of `gray` (`width` pixels per row)
/// over `window`, in pixels; lower is sharper. The threshold is the 90th
/// percentile of `max(|gx|, |gy|)` of the Sobel gradient over the window
/// interior, at least 16. At each local maximum of `|gx|` along a row (and of
/// `|gy|` along a column) at or above it, the walk goes both ways while the
/// intensity stays strictly monotonic in the edge's direction, bounded by the
/// window; the width is the distance between the two extrema. Rows and
/// columns are pooled into one mean. `None` when no edge qualifies, or the
/// window is too narrow for the interior loops.
pub fn edge_width(gray: &[u8], width: usize, window: Window) -> Option<f64> {
    if window.width < 3 || window.height < 3 {
        return None;
    }
    let p = |x: usize, y: usize| gray[y * width + x] as i32;
    let (x0, y0) = (window.x + 1, window.y + 1);
    let (x1, y1) = (window.x + window.width - 1, window.y + window.height - 1);
    let gx = |x: usize, y: usize| {
        (p(x + 1, y - 1) + 2 * p(x + 1, y) + p(x + 1, y + 1))
            - (p(x - 1, y - 1) + 2 * p(x - 1, y) + p(x - 1, y + 1))
    };
    let gy = |x: usize, y: usize| {
        (p(x - 1, y + 1) + 2 * p(x, y + 1) + p(x + 1, y + 1))
            - (p(x - 1, y - 1) + 2 * p(x, y - 1) + p(x + 1, y - 1))
    };
    let mut mags = Vec::with_capacity((x1 - x0) * (y1 - y0));
    for y in y0..y1 {
        for x in x0..x1 {
            mags.push(gx(x, y).abs().max(gy(x, y).abs()));
        }
    }
    mags.sort_unstable();
    let t = mags[mags.len() * 9 / 10].max(16);
    let (mut total, mut count) = (0usize, 0usize);
    for y in y0..y1 {
        for x in x0 + 1..x1 - 1 {
            let m = gx(x, y).abs();
            if m < t || m < gx(x - 1, y).abs() || m <= gx(x + 1, y).abs() {
                continue;
            }
            let up = gx(x, y) > 0;
            let mut l = x;
            while l > window.x && (p(l - 1, y) < p(l, y)) == up && p(l - 1, y) != p(l, y) {
                l -= 1;
            }
            let mut r = x;
            while r + 1 < window.x + window.width
                && (p(r + 1, y) > p(r, y)) == up
                && p(r + 1, y) != p(r, y)
            {
                r += 1;
            }
            total += r - l;
            count += 1;
        }
    }
    for x in x0..x1 {
        for y in y0 + 1..y1 - 1 {
            let m = gy(x, y).abs();
            if m < t || m < gy(x, y - 1).abs() || m <= gy(x, y + 1).abs() {
                continue;
            }
            let up = gy(x, y) > 0;
            let mut u = y;
            while u > window.y && (p(x, u - 1) < p(x, u)) == up && p(x, u - 1) != p(x, u) {
                u -= 1;
            }
            let mut d = y;
            while d + 1 < window.y + window.height
                && (p(x, d + 1) > p(x, d)) == up
                && p(x, d + 1) != p(x, d)
            {
                d += 1;
            }
            total += d - u;
            count += 1;
        }
    }
    (count > 0).then(|| total as f64 / count as f64)
}

fn sigmoid(logit: f64) -> f64 {
    1.0 / (1.0 + (-logit).exp())
}

/// The in-focus probability at `CANDIDATE_LOGIT`: a probability at or above
/// it reads back as `Candidate` in `candidate`.
pub fn candidate_probability() -> f64 {
    sigmoid(CANDIDATE_LOGIT)
}

/// The measures behind the cue of one face.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EyeFocus {
    /// The Laplacian variance of the luma over the eye window.
    pub lap: f64,
    /// `edge_width` over the eye window; `None` when no edge qualifies.
    pub edge_width: Option<f64>,
    /// `edge_width` divided by the eye window's side.
    pub edge_width_rel: Option<f64>,
    /// The in-focus logit; `None` without an edge width.
    pub logit: Option<f64>,
    /// The in-focus probability, the sigmoid of `logit`. Without an edge width
    /// it is 0 and the state `NotCandidate`: a window with no gradient above
    /// the noise floor has no sharp edge, and a face the detector found is
    /// never left `Unknown`.
    pub probability: f64,
    pub state: FocusCandidate,
}

/// The `EyeFocus` of `gray` over the `eye_window` of `face`.
pub fn eye_focus(gray: &[u8], width: usize, height: usize, face: &Face) -> EyeFocus {
    let window = eye_window(width, height, face);
    let lap = laplacian_variance(gray, width, window);
    let edge_width = edge_width(gray, width, window);
    let edge_width_rel = edge_width.map(|e| e / window.width as f64);
    let logit = edge_width_rel
        .map(|rel| LOGIT_INTERCEPT + LOGIT_LAP * (lap + 1.0).ln() + LOGIT_EDGE_WIDTH * rel.ln());
    let (probability, state) = logit.map_or((0.0, FocusCandidate::NotCandidate), scored);
    EyeFocus {
        lap,
        edge_width,
        edge_width_rel,
        logit,
        probability,
        state,
    }
}

/// The probability and state of `logit`. The state compares the logit with
/// `CANDIDATE_LOGIT`; below it, a sigmoid that rounds up to
/// `candidate_probability()` is lowered by one step so `candidate` reads the
/// probability back into the same state.
fn scored(logit: f64) -> (f64, FocusCandidate) {
    let p = sigmoid(logit);
    if logit >= CANDIDATE_LOGIT {
        (p, FocusCandidate::Candidate)
    } else {
        (
            p.min(candidate_probability().next_down()),
            FocusCandidate::NotCandidate,
        )
    }
}

/// The state an in-focus probability gives, against
/// `candidate_probability()`; `None` (no face near the AF point) is
/// `Unknown`.
pub fn candidate(eye_focus: Option<f64>) -> FocusCandidate {
    match eye_focus {
        None => FocusCandidate::Unknown,
        Some(p) if p >= candidate_probability() => FocusCandidate::Candidate,
        Some(_) => FocusCandidate::NotCandidate,
    }
}

/// The cue of `preview` for the AF point of `focus` (pass
/// `sharpness::trusted_focus`). Without one it is `Unknown`, with no decode
/// and no detection; otherwise the preview is decoded once, faces are
/// detected around the point and the nearest one's eyes are scored.
pub fn focus_cue(preview: &[u8], orientation: u16, focus: Option<FocusLocation>) -> Result<Cue> {
    focus_cue_unless(preview, orientation, focus, &AtomicBool::new(false))
        .expect("a never-set flag never abandons")
}

/// `focus_cue`, abandoned (`None`) once `cancel` is set between the decode,
/// the detection and the eye scoring.
pub fn focus_cue_unless(
    preview: &[u8],
    orientation: u16,
    focus: Option<FocusLocation>,
    cancel: &AtomicBool,
) -> Option<Result<Cue>> {
    if focus.is_none() {
        return Some(Ok(Cue::unknown()));
    }
    let canceled = || cancel.load(Ordering::Relaxed);
    let (rgb, width, height) = match decode_rgb(preview) {
        Ok(decoded) => decoded,
        Err(e) => return Some(Err(e)),
    };
    let gray = luma(&rgb, width, height);
    if canceled() {
        return None;
    }
    let detection = match detect_around_rgb(&rgb, width, height, orientation, focus) {
        Ok(detection) => detection,
        Err(e) => return Some(Err(e)),
    };
    let face = detection
        .point
        .and_then(|p| nearest_face(&detection.faces, p))
        .copied();
    if canceled() {
        return None;
    }
    let focus = face.map(|f| eye_focus(&gray, width, height, &f));
    Some(Ok(Cue {
        state: focus.map_or(FocusCandidate::Unknown, |f| f.state),
        eye_focus: focus.map(|f| f.probability),
        face,
        detection: Some(detection),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face(x: f32, y: f32, side: f32, left_eye: (f32, f32), right_eye: (f32, f32)) -> Face {
        Face {
            x,
            y,
            width: side,
            height: side,
            score: 0.9,
            left_eye,
            right_eye,
        }
    }

    #[test]
    fn the_nearest_face_is_chosen_by_eye_distance_not_box_center() {
        // The point sits on the eye of `a` but closer to the box center of `b`.
        let a = face(0.0, 0.0, 100.0, (20.0, 20.0), (80.0, 20.0));
        let b = face(40.0, -30.0, 100.0, (60.0, 60.0), (120.0, 60.0));
        let faces = [b, a];
        assert_eq!(nearest_face(&faces, (80, 22)), Some(&a));
    }

    #[test]
    fn the_eye_distance_is_relative_to_the_face_size() {
        // Large: eye 50 px away on a 200 px face (0.25). Small: eye 10 px away
        // on a 20 px face (0.5).
        let large = face(0.0, 0.0, 200.0, (150.0, 100.0), (170.0, 100.0));
        let small = face(0.0, 0.0, 20.0, (110.0, 100.0), (115.0, 100.0));
        assert_eq!(nearest_face(&[small, large], (100, 100)), Some(&large));
        // Small eye 4 px away (0.2) now beats the large one.
        let small = face(0.0, 0.0, 20.0, (104.0, 100.0), (115.0, 100.0));
        assert_eq!(nearest_face(&[large, small], (100, 100)), Some(&small));
    }

    #[test]
    fn no_faces_or_empty_boxes_give_none() {
        assert_eq!(nearest_face(&[], (0, 0)), None);
        let empty = face(0.0, 0.0, 0.0, (0.0, 0.0), (0.0, 0.0));
        assert_eq!(nearest_face(&[empty], (0, 0)), None);
    }

    #[test]
    fn the_eye_window_is_centered_between_the_eyes() {
        let f = Face {
            height: 80.0,
            ..face(100.0, 100.0, 60.0, (110.0, 130.0), (150.0, 134.0))
        };
        assert_eq!(
            eye_window(1000, 700, &f),
            Window {
                x: 90,
                y: 92,
                width: 80,
                height: 80
            }
        );
    }

    #[test]
    fn a_tiny_face_gets_the_minimum_window() {
        let f = face(100.0, 100.0, 10.0, (103.0, 104.0), (107.0, 104.0));
        let w = eye_window(1000, 700, &f);
        assert_eq!(
            (w.width, w.height),
            (CANDIDATE_WINDOW_MIN, CANDIDATE_WINDOW_MIN)
        );
        assert_eq!((w.x, w.y), (105 - 12, 104 - 12));
    }

    #[test]
    fn a_face_at_a_corner_is_clamped_into_the_image() {
        let f = face(-20.0, -20.0, 60.0, (-5.0, -5.0), (5.0, -5.0));
        assert_eq!(
            eye_window(1000, 700, &f),
            Window {
                x: 0,
                y: 0,
                width: 60,
                height: 60
            }
        );
        let f = face(980.0, 680.0, 60.0, (1005.0, 705.0), (1015.0, 705.0));
        assert_eq!(
            eye_window(1000, 700, &f),
            Window {
                x: 940,
                y: 640,
                width: 60,
                height: 60
            }
        );
    }

    #[test]
    fn luma_uses_the_bt601_integer_weights() {
        let rgb = [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
        assert_eq!(luma(&rgb, 4, 1), vec![76, 149, 29, 255]);
    }

    /// A 40 x 40 image dark left of column 15 and bright from column
    /// `15 + blur`, ramping linearly in between.
    fn ramp(blur: usize) -> Vec<u8> {
        let row: Vec<u8> = (0..40)
            .map(|x: usize| (x.saturating_sub(15).min(blur) * 200 / blur) as u8)
            .collect();
        row.repeat(40)
    }

    const WHOLE: Window = Window {
        x: 0,
        y: 0,
        width: 40,
        height: 40,
    };

    #[test]
    fn a_blurred_step_edge_measures_its_blur_width() {
        assert_eq!(edge_width(&ramp(4), 40, WHOLE), Some(4.0));
        assert_eq!(edge_width(&ramp(8), 40, WHOLE), Some(8.0));
    }

    #[test]
    fn a_flat_window_has_no_edge_and_is_not_a_candidate() {
        let flat = vec![128u8; 40 * 40];
        assert_eq!(edge_width(&flat, 40, WHOLE), None);
        let narrow = Window { width: 2, ..WHOLE };
        assert_eq!(edge_width(&ramp(4), 40, narrow), None);
        let f = face(0.0, 0.0, 40.0, (10.0, 20.0), (30.0, 20.0));
        let focus = eye_focus(&flat, 40, 40, &f);
        assert_eq!(focus.edge_width, None);
        assert_eq!(focus.logit, None);
        assert_eq!(focus.probability, 0.0);
        assert_eq!(focus.state, FocusCandidate::NotCandidate);
        assert_eq!(
            candidate(Some(focus.probability)),
            FocusCandidate::NotCandidate
        );
    }

    #[test]
    fn the_combined_score_follows_the_frozen_coefficients() {
        let f = face(0.0, 0.0, 40.0, (10.0, 20.0), (30.0, 20.0));
        let gray = ramp(4);
        let focus = eye_focus(&gray, 40, 40, &f);
        let lap = laplacian_variance(&gray, 40, WHOLE);
        let logit = LOGIT_INTERCEPT + LOGIT_LAP * (lap + 1.0).ln() + LOGIT_EDGE_WIDTH * 0.1f64.ln();
        assert_eq!(focus.lap, lap);
        assert_eq!(focus.edge_width_rel, Some(0.1));
        assert_eq!(focus.logit, Some(logit));
        assert_eq!(focus.probability, 1.0 / (1.0 + (-logit).exp()));
    }

    #[test]
    fn the_logit_threshold_splits_the_states_and_the_probability_reads_back() {
        let below = CANDIDATE_LOGIT.next_down();
        assert_eq!(scored(below).1, FocusCandidate::NotCandidate);
        assert_eq!(scored(CANDIDATE_LOGIT).1, FocusCandidate::Candidate);
        assert_eq!(
            scored(CANDIDATE_LOGIT.next_up()).1,
            FocusCandidate::Candidate
        );
        assert_eq!(scored(CANDIDATE_LOGIT).0, candidate_probability());
        assert!((candidate_probability() - 0.772).abs() < 0.001);
        let mut logit = CANDIDATE_LOGIT;
        let mut ulp_steps = 0;
        for _ in 0..64 {
            logit = logit.next_down();
            ulp_steps += 1;
        }
        for _ in 0..128 {
            let (p, state) = scored(logit);
            assert_eq!(
                candidate(Some(p)),
                state,
                "logit {logit} ({ulp_steps} ulps)"
            );
            logit = logit.next_up();
            ulp_steps -= 1;
        }
        assert_eq!(candidate(None), FocusCandidate::Unknown);
        assert_eq!(candidate(Some(0.0)), FocusCandidate::NotCandidate);
        assert_eq!(candidate(Some(1.0)), FocusCandidate::Candidate);
    }

    #[test]
    fn no_af_point_is_unknown_without_decoding() {
        assert_eq!(focus_cue(&[0u8; 16], 1, None).unwrap(), Cue::unknown());
    }

    #[test]
    fn a_set_flag_abandons_the_cue_after_the_decode() {
        let rgb = vec![128u8; 64 * 48 * 3];
        let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        c.set_size(64, 48);
        c.set_quality(80.0);
        let mut c = c.start_compress(Vec::new()).unwrap();
        c.write_scanlines(&rgb).unwrap();
        let jpeg = c.finish().unwrap();
        let focus = Some(FocusLocation {
            sensor_w: 6000,
            sensor_h: 4000,
            x: 3000,
            y: 2000,
        });
        let set = AtomicBool::new(true);
        assert!(focus_cue_unless(&jpeg, 1, focus, &set).is_none());
        assert!(
            focus_cue_unless(&[0u8; 16], 1, focus, &set).is_some_and(|r| r.is_err()),
            "a decode failure is still an error"
        );
        assert!(focus_cue_unless(&jpeg, 1, focus, &AtomicBool::new(false)).is_some());
    }
}
