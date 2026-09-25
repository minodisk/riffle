//! The focus candidate cue: the Laplacian variance of the luma over a window
//! between the eyes of the face nearest the AF point ("eye sharpness"), and
//! whether it clears `CANDIDATE_THRESHOLD`.
//!
//! Faces come from `faces::detect_around_rgb` in a `CATCH_CROP` square around
//! the trusted AF point. A Sony eye-AF frame gets the same detection: the
//! camera tracking a face does not say whether that face is sharp.
//! Everything is in the preview's stored (unrotated) coordinates.

use anyhow::Result;

use crate::arw::FocusLocation;
use crate::decode::decode_rgb;
use crate::faces::{detect_around_rgb, Detection, Face};
use crate::sharpness::{laplacian_variance, window_at, Window};

/// Eye sharpness at or above which a frame is a focus candidate. On 500
/// hand-labeled ILCE-7M5 frames, the frames at or above 80 were in focus 93%
/// of the time (310/335) and covered 80% of the in-focus frames; those below
/// it were in focus 48% of the time.
pub const CANDIDATE_THRESHOLD: f64 = 80.0;
/// Smallest side of the eye window, in preview pixels. The window of the
/// 500-frame validation behind `CANDIDATE_THRESHOLD` (AUC 0.82 against 0.67
/// for the sharpness score) was the face box's long side, at least 24 px.
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
    pub eye_sharpness: Option<f64>,
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
/// `width` x `height` RGB image, the formula `CANDIDATE_THRESHOLD` was
/// validated with.
pub fn luma(rgb: &[u8], width: usize, height: usize) -> Vec<u8> {
    rgb.chunks_exact(3)
        .take(width * height)
        .map(|p| ((299 * p[0] as u32 + 587 * p[1] as u32 + 114 * p[2] as u32) / 1000) as u8)
        .collect()
}

/// The Laplacian variance of `gray` over the `eye_window` of `face`.
pub fn eye_sharpness(gray: &[u8], width: usize, height: usize, face: &Face) -> f64 {
    laplacian_variance(gray, width, eye_window(width, height, face))
}

/// The state an eye sharpness gives; `None` (no face near the AF point) is
/// `Unknown`.
pub fn candidate(eye_sharpness: Option<f64>) -> FocusCandidate {
    match eye_sharpness {
        None => FocusCandidate::Unknown,
        Some(s) if s >= CANDIDATE_THRESHOLD => FocusCandidate::Candidate,
        Some(_) => FocusCandidate::NotCandidate,
    }
}

/// The cue of `preview` for the AF point of `focus` (pass
/// `sharpness::trusted_focus`). Without one it is `Unknown`, with no decode
/// and no detection; otherwise the preview is decoded once, faces are
/// detected around the point and the nearest one's eyes are scored.
pub fn focus_cue(preview: &[u8], orientation: u16, focus: Option<FocusLocation>) -> Result<Cue> {
    if focus.is_none() {
        return Ok(Cue::unknown());
    }
    let (rgb, width, height) = decode_rgb(preview)?;
    let gray = luma(&rgb, width, height);
    let detection = detect_around_rgb(&rgb, width, height, orientation, focus)?;
    let face = detection
        .point
        .and_then(|p| nearest_face(&detection.faces, p))
        .copied();
    let eye_sharpness = face.map(|f| eye_sharpness(&gray, width, height, &f));
    Ok(Cue {
        state: candidate(eye_sharpness),
        eye_sharpness,
        face,
        detection: Some(detection),
    })
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

    #[test]
    fn the_threshold_splits_the_states() {
        assert_eq!(candidate(None), FocusCandidate::Unknown);
        assert_eq!(candidate(Some(79.99)), FocusCandidate::NotCandidate);
        assert_eq!(candidate(Some(80.0)), FocusCandidate::Candidate);
        assert_eq!(candidate(Some(300.0)), FocusCandidate::Candidate);
    }

    #[test]
    fn no_af_point_is_unknown_without_decoding() {
        assert_eq!(focus_cue(&[0u8; 16], 1, None).unwrap(), Cue::unknown());
    }
}
