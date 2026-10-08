//! The focus candidate cue: how likely the eyes of the face nearest the AF
//! point are in focus, and whether that clears `CANDIDATE_LOGIT`.
//!
//! The score combines two measures over each eye's region of the face mesh
//! (`eyes::mesh_of`): the Laplacian variance of the luma (`lap`) and the
//! mean edge width (`edge_width`) relative to the region's longer side, in a
//! logistic regression fitted on hand-labeled frames (`MESH_LOGIT_*`); the
//! sharper eye's logit is the frame's. When no eye region counts (no mesh,
//! or regions under `EYE_REGION_MIN` or without an edge width) the same two
//! measures over a window between the eyes are scored with the window model
//! (`LOGIT_*`). Its sigmoid is the in-focus probability. The mesh runs only
//! on a face whose box long side is at least `eyes::EYES_MIN_FACE`
//! (`meshes_face`): a smaller face goes straight to the window, as no eye
//! region of one under 58 px counted on the labeled frames.
//!
//! Faces come from `faces::detect_around_rgb` in a `CATCH_CROP` square around
//! the trusted AF point. A Sony eye-AF frame gets the same detection: the
//! camera tracking a face does not say whether that face is sharp.
//! Everything is in the preview's stored (unrotated) coordinates.

use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;

use crate::arw::FocusLocation;
use crate::decode::decode_rgb;
use crate::eyes::{
    mesh_of, Mesh, EYES_MIN_FACE, LEFT_EYE_CONTOUR, LEFT_IRIS, RIGHT_EYE_CONTOUR, RIGHT_IRIS,
};
use crate::faces::{detect_around_rgb, Detection, Face};
use crate::pose::Pose;
use crate::sharpness::{laplacian_variance, window_at, Window};

/// The intercept of the eye window's in-focus logit
/// `LOGIT_INTERCEPT + LOGIT_LAP * ln(lap + 1) + LOGIT_EDGE_WIDTH * ln(edge_width / window side)`,
/// fitted on 406 hand-labeled faced frames from 5 folders, now the fallback
/// for frames with no mesh eye region that counts. A coefficient change is a
/// new model: re-validate it with `riffle-cli candidates`.
pub const LOGIT_INTERCEPT: f64 = -4.725633355883976;
/// The weight of `ln(lap + 1)` in the in-focus logit.
pub const LOGIT_LAP: f64 = 0.6826458385175557;
/// The weight of `ln(edge_width / window side)` in the in-focus logit.
pub const LOGIT_EDGE_WIDTH: f64 = -1.1949836425055467;
/// The logit at or above which a frame is a focus candidate (an in-focus
/// probability of about 77%), for both the mesh eye model (whose intercept is
/// shifted onto it) and the window model. It was chosen for the window model
/// to keep the in-focus coverage of the
/// earlier Laplacian-only threshold. On the 406 training frames the combined
/// score reaches AUC 0.852 (0.816 for the Laplacian alone), and the frames at
/// or above it are in focus 93.1% of the time and cover 91.4% of the
/// in-focus frames. On 400 held-out frames from 4 other folders it reaches
/// AUC 0.754 (0.635), precision 89.1% and coverage 95.3%.
///
/// The fit put the threshold at 1.2194865955352432, the logit of the boundary
/// training frame computed from lap and edge width rounded to 2 and 5
/// decimals. At full precision that frame's logit is 1.2194442..., just
/// below it, so the threshold is lowered to 1.2194 to keep the frame and the
/// 91.4% coverage. No other training or held-out frame's logit lies in
/// between.
pub const CANDIDATE_LOGIT: f64 = 1.2194;
/// Smallest side of the eye window, in preview pixels. The combined score was
/// fitted and validated on this window, the face box's long side, at least
/// 24 px (the earlier Laplacian-only validation on 500 frames also used it).
pub const CANDIDATE_WINDOW_MIN: usize = 24;
/// The margin `eye_region` grows a mesh eye's bounding box by on each side,
/// as a fraction of the box's longer side (the eye's width, upright or on a
/// quarter-turned preview), so the lid edges and lashes lie inside. Chosen
/// with `EYE_REGION_MIN` as the highest training AUC of 4 margins x 7 floors
/// (`docs/plans/20261008-mesh-eye-focus/fit.md`).
pub const EYE_REGION_MARGIN: f32 = 0.5;
/// The smallest longer side of an eye region that is scored, in preview
/// pixels; a smaller eye does not count, and a frame with no eye that counts
/// falls back to the eye window (61% of the training frames, 57% held-out).
pub const EYE_REGION_MIN: usize = 24;
/// The intercept of the mesh eye logit
/// `MESH_LOGIT_INTERCEPT + MESH_LOGIT_LAP * ln(lap + 1) + MESH_LOGIT_EDGE_WIDTH * ln(edge_width / region's longer side)`,
/// fitted on both eyes of the 406 training frames (each eye carrying its
/// frame's label). Together with the window fallback, taking the sharper eye
/// reaches AUC 0.882 on the training frames, precision 93.9% and coverage
/// 91.4%, and on the 400 held-out frames AUC 0.800, precision 88.6% and
/// coverage 95.9%.
///
/// The fit's own intercept is -8.158737592019197 with its threshold at
/// 0.8343419969086643 (midway between the boundary training pick and the
/// next lower logit, keeping the 91.4% coverage). Both models share one
/// threshold, `CANDIDATE_LOGIT`, because the index derives the state from
/// the stored probability alone: the intercept here is shifted by
/// `CANDIDATE_LOGIT - 0.8343419969086643`, which moves the mesh model's
/// probabilities but not its states, and leaves the window model as fitted.
pub const MESH_LOGIT_INTERCEPT: f64 = -7.773679588927861;
/// The weight of `ln(lap + 1)` in the mesh eye logit.
pub const MESH_LOGIT_LAP: f64 = 1.9417143647766388;
/// The weight of `ln(edge_width / region's longer side)` in the mesh eye
/// logit.
pub const MESH_LOGIT_EDGE_WIDTH: f64 = -1.3161251379398846;

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

/// An eye of a face mesh, by the side of the image it is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eye {
    Left,
    Right,
}

/// What `eye_focus` scored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scored {
    /// Both eyes counted; this one had the higher logit (the left on a tie).
    Sharper(Eye),
    /// Only this eye counted.
    Only(Eye),
    /// No eye counted: the eye window, with the window model.
    Window,
}

/// The measures behind the cue of one face.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EyeFocus {
    /// The region scored: the chosen eye's contour region, or the eye window.
    pub window: Window,
    /// The Laplacian variance of the luma over `window`.
    pub lap: f64,
    /// `edge_width` over `window`; `None` when no edge qualifies.
    pub edge_width: Option<f64>,
    /// `edge_width` divided by `window`'s longer side (an eye region) or its
    /// width (the eye window).
    pub edge_width_rel: Option<f64>,
    /// The in-focus logit; `None` without an edge width.
    pub logit: Option<f64>,
    /// The in-focus probability, the sigmoid of `logit`. Without an edge width
    /// it is 0 and the state `NotCandidate`: a window with no gradient above
    /// the noise floor has no sharp edge, and a face the detector found is
    /// never left `Unknown`.
    pub probability: f64,
    pub state: FocusCandidate,
    pub scored: Scored,
    /// Both eyes' measures and the pose; `None` without a mesh.
    pub mesh: Option<EyeMeasures>,
}

/// The mesh eye logit of a contour region, `None` when the eye does not
/// count: under `EYE_REGION_MIN` on its longer side or without an edge width.
fn mesh_logit(region: Option<RegionMeasures>) -> Option<f64> {
    let r = region?;
    if r.window.width.max(r.window.height) < EYE_REGION_MIN {
        return None;
    }
    let rel = r.edge_width_rel?;
    Some(
        MESH_LOGIT_INTERCEPT
            + MESH_LOGIT_LAP * (r.lap + 1.0).ln()
            + MESH_LOGIT_EDGE_WIDTH * rel.ln(),
    )
}

/// The `EyeFocus` of `gray` for `face` and its `mesh`: the higher logit of
/// the mesh eyes that count, else the `eye_window` of `face` scored with the
/// window model.
pub fn eye_focus(
    gray: &[u8],
    width: usize,
    height: usize,
    face: &Face,
    mesh: Option<&Mesh>,
) -> EyeFocus {
    let measures = mesh.map(|m| mesh_eye_measures(gray, width, height, m));
    let left = measures.and_then(|m| Some((m.left.contour?, mesh_logit(m.left.contour)?)));
    let right = measures.and_then(|m| Some((m.right.contour?, mesh_logit(m.right.contour)?)));
    let chosen = match (left, right) {
        (Some(l), Some(r)) if r.1 > l.1 => Some((r, Scored::Sharper(Eye::Right))),
        (Some(l), Some(_)) => Some((l, Scored::Sharper(Eye::Left))),
        (Some(l), None) => Some((l, Scored::Only(Eye::Left))),
        (None, Some(r)) => Some((r, Scored::Only(Eye::Right))),
        (None, None) => None,
    };
    if let Some(((region, logit), scored_eye)) = chosen {
        let (probability, state) = scored(logit);
        return EyeFocus {
            window: region.window,
            lap: region.lap,
            edge_width: region.edge_width,
            edge_width_rel: region.edge_width_rel,
            logit: Some(logit),
            probability,
            state,
            scored: scored_eye,
            mesh: measures,
        };
    }
    let window = eye_window(width, height, face);
    let lap = laplacian_variance(gray, width, window);
    let edge_width = edge_width(gray, width, window);
    let edge_width_rel = edge_width.map(|e| e / window.width as f64);
    let logit = edge_width_rel
        .map(|rel| LOGIT_INTERCEPT + LOGIT_LAP * (lap + 1.0).ln() + LOGIT_EDGE_WIDTH * rel.ln());
    let (probability, state) = logit.map_or((0.0, FocusCandidate::NotCandidate), scored);
    EyeFocus {
        window,
        lap,
        edge_width,
        edge_width_rel,
        logit,
        probability,
        state,
        scored: Scored::Window,
        mesh: measures,
    }
}

/// The bounding box of the mesh `points` named by `indices` (in the
/// `width` x `height` image's pixels), grown by `EYE_REGION_MARGIN` times its
/// longer side on each side and clamped into the image. `None` when an index is out
/// of range, a point is not finite, or the clamped box is under 3 px on a
/// side.
pub fn eye_region(
    points: &[[f32; 2]],
    indices: &[usize],
    width: usize,
    height: usize,
) -> Option<Window> {
    let (mut x0, mut y0) = (f32::INFINITY, f32::INFINITY);
    let (mut x1, mut y1) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
    for &i in indices {
        let [x, y] = *points.get(i)?;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
    }
    let margin = (x1 - x0).max(y1 - y0) * EYE_REGION_MARGIN;
    let lo = |v: f32| (v - margin).floor().max(0.0) as usize;
    let hi = |v: f32, limit: usize| ((v + margin).ceil().max(0.0) as usize).min(limit);
    let (left, top) = (lo(x0), lo(y0));
    let (right, bottom) = (hi(x1, width), hi(y1, height));
    let window = Window {
        x: left,
        y: top,
        width: right.saturating_sub(left),
        height: bottom.saturating_sub(top),
    };
    (window.width >= 3 && window.height >= 3).then_some(window)
}

/// The Laplacian variance, the `edge_width` and the edge width over the
/// window's longer side (the eye's width, not the face's, whichever way the
/// stored preview is turned) of `gray` (`width` pixels per row) over
/// `window`.
pub fn eye_measures(gray: &[u8], width: usize, window: Window) -> (f64, Option<f64>, Option<f64>) {
    let lap = laplacian_variance(gray, width, window);
    let edge = edge_width(gray, width, window);
    (
        lap,
        edge,
        edge.map(|e| e / window.width.max(window.height) as f64),
    )
}

/// The measures of one mesh region.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionMeasures {
    pub window: Window,
    pub lap: f64,
    pub edge_width: Option<f64>,
    pub edge_width_rel: Option<f64>,
}

/// The regions of one eye; `None` where `eye_region` gives none.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EyeRegions {
    pub contour: Option<RegionMeasures>,
    pub iris: Option<RegionMeasures>,
}

/// The per-eye measures over a face mesh.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EyeMeasures {
    /// The eye on the left of the image.
    pub left: EyeRegions,
    /// The eye on the right of the image.
    pub right: EyeRegions,
    pub pose: Option<Pose>,
}

/// The `EyeMeasures` of `gray` (`width` x `height`, stored coordinates) over
/// the eyelid contour and iris regions of `mesh`.
pub fn mesh_eye_measures(gray: &[u8], width: usize, height: usize, mesh: &Mesh) -> EyeMeasures {
    let region = |indices: &[usize]| {
        eye_region(&mesh.points, indices, width, height).map(|window| {
            let (lap, edge_width, edge_width_rel) = eye_measures(gray, width, window);
            RegionMeasures {
                window,
                lap,
                edge_width,
                edge_width_rel,
            }
        })
    };
    EyeMeasures {
        left: EyeRegions {
            contour: region(&LEFT_EYE_CONTOUR),
            iris: region(&LEFT_IRIS),
        },
        right: EyeRegions {
            contour: region(&RIGHT_EYE_CONTOUR),
            iris: region(&RIGHT_IRIS),
        },
        pose: mesh.pose,
    }
}

/// The fewest pixels an `eye_mask` holds; a smaller one is `None`.
pub const EYE_MASK_MIN: usize = 3;

/// The pixels inside an eye's contour polygon.
#[derive(Debug, Clone, PartialEq)]
pub struct EyeMask {
    /// The bounding box of the masked pixels.
    pub window: Window,
    /// Whether each pixel of `window` is masked, row-major.
    pub inside: Vec<bool>,
}

impl EyeMask {
    /// Whether the image pixel `(x, y)` is masked.
    pub fn contains(&self, x: usize, y: usize) -> bool {
        let w = self.window;
        x >= w.x
            && y >= w.y
            && x < w.x + w.width
            && y < w.y + w.height
            && self.inside[(y - w.y) * w.width + x - w.x]
    }

    /// The number of masked pixels.
    pub fn pixels(&self) -> usize {
        self.inside.iter().filter(|&&b| b).count()
    }
}

/// The pixels of the `width` x `height` image inside the closed polygon of the
/// mesh `points` named by `indices`, in order: a pixel is inside when its
/// center lies inside by the even-odd rule along its row (the scanline fill).
/// `dilation` > 0 dilates the raster morphologically by a disk of radius
/// `dilation` times the polygon bounding box's longer side: a pixel whose
/// center lies within that radius of the polygon's boundary is inside too.
/// Clamped into the image. `None` when an index is out of range, a point is
/// not finite, or fewer than `EYE_MASK_MIN` pixels are inside.
pub fn eye_mask(
    points: &[[f32; 2]],
    indices: &[usize],
    width: usize,
    height: usize,
    dilation: f32,
) -> Option<EyeMask> {
    let mut poly = Vec::with_capacity(indices.len());
    let (mut x0, mut y0) = (f32::INFINITY, f32::INFINITY);
    let (mut x1, mut y1) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
    for &i in indices {
        let [x, y] = *points.get(i)?;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        poly.push((x, y));
    }
    let radius = (x1 - x0).max(y1 - y0) * dilation;
    let lo = |v: f32, limit: usize| ((v - radius).floor().max(0.0) as usize).min(limit);
    let hi = |v: f32, limit: usize| ((v + radius).ceil().max(0.0) as usize).min(limit);
    let (left, right) = (lo(x0, width), hi(x1, width));
    let (top, bottom) = (lo(y0, height), hi(y1, height));
    let edges: Vec<((f32, f32), (f32, f32))> = (0..poly.len())
        .map(|k| (poly[k], poly[(k + 1) % poly.len()]))
        .collect();
    let near = |cx: f32, cy: f32| {
        edges.iter().any(|&((ax, ay), (bx, by))| {
            let (dx, dy) = (bx - ax, by - ay);
            let len = dx * dx + dy * dy;
            let t = if len > 0.0 {
                (((cx - ax) * dx + (cy - ay) * dy) / len).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (cx - ax - t * dx).hypot(cy - ay - t * dy) <= radius
        })
    };
    let mut set = Vec::new();
    let mut crossings = Vec::with_capacity(edges.len());
    for y in top..bottom {
        let cy = y as f32 + 0.5;
        crossings.clear();
        for &((ax, ay), (bx, by)) in &edges {
            if (ay > cy) != (by > cy) {
                crossings.push(ax + (cy - ay) * (bx - ax) / (by - ay));
            }
        }
        for x in left..right {
            let cx = x as f32 + 0.5;
            let odd = crossings.iter().filter(|&&c| c > cx).count() % 2 == 1;
            if odd || (radius > 0.0 && near(cx, cy)) {
                set.push((x, y));
            }
        }
    }
    if set.len() < EYE_MASK_MIN {
        return None;
    }
    let mx0 = set.iter().map(|p| p.0).min()?;
    let mx1 = set.iter().map(|p| p.0).max()?;
    let my0 = set.iter().map(|p| p.1).min()?;
    let my1 = set.iter().map(|p| p.1).max()?;
    let window = Window {
        x: mx0,
        y: my0,
        width: mx1 - mx0 + 1,
        height: my1 - my0 + 1,
    };
    let mut inside = vec![false; window.width * window.height];
    for (x, y) in set {
        inside[(y - my0) * window.width + x - mx0] = true;
    }
    Some(EyeMask { window, inside })
}

/// The Laplacian variance of `gray` (`width` pixels per row) over the pixels
/// of `mask` whose four neighbors lie in the image, with the integer formula
/// of `sharpness::laplacian_variance`; 0.0 when no pixel qualifies.
pub fn masked_laplacian_variance(gray: &[u8], width: usize, mask: &EyeMask) -> f64 {
    let height = gray.len() / width;
    let w = mask.window;
    let (mut sum, mut sum_sq, mut n) = (0i64, 0i64, 0i64);
    for y in w.y.max(1)..(w.y + w.height).min(height.saturating_sub(1)) {
        for x in w.x.max(1)..(w.x + w.width).min(width.saturating_sub(1)) {
            if !mask.contains(x, y) {
                continue;
            }
            let i = y * width + x;
            let l = gray[i - width] as i32
                + gray[i + width] as i32
                + gray[i - 1] as i32
                + gray[i + 1] as i32
                - 4 * gray[i] as i32;
            sum += l as i64;
            sum_sq += (l * l) as i64;
            n += 1;
        }
    }
    if n == 0 {
        return 0.0;
    }
    let mean = sum as f64 / n as f64;
    sum_sq as f64 / n as f64 - mean * mean
}

/// `edge_width` over `mask`: the Sobel threshold is the 90th percentile over
/// the masked pixels whose eight neighbors lie in the image, at least 16, and
/// the seeds (local maxima) are those masked pixels whose neighbor along the
/// row (column) has a gradient too. The walk stays bounded by the mask's
/// window, not the mask, so a width means what it means in `edge_width`.
/// `None` when no edge qualifies.
pub fn masked_edge_width(gray: &[u8], width: usize, mask: &EyeMask) -> Option<f64> {
    let height = gray.len() / width;
    let window = mask.window;
    let p = |x: usize, y: usize| gray[y * width + x] as i32;
    let graded = |x: usize, y: usize| x >= 1 && y >= 1 && x + 1 < width && y + 1 < height;
    let gx = |x: usize, y: usize| {
        (p(x + 1, y - 1) + 2 * p(x + 1, y) + p(x + 1, y + 1))
            - (p(x - 1, y - 1) + 2 * p(x - 1, y) + p(x - 1, y + 1))
    };
    let gy = |x: usize, y: usize| {
        (p(x - 1, y + 1) + 2 * p(x, y + 1) + p(x + 1, y + 1))
            - (p(x - 1, y - 1) + 2 * p(x, y - 1) + p(x + 1, y - 1))
    };
    let (xs, ys) = (
        window.x..window.x + window.width,
        window.y..window.y + window.height,
    );
    let masked = |x: usize, y: usize| mask.contains(x, y) && graded(x, y);
    let mut mags = Vec::new();
    for y in ys.clone() {
        for x in xs.clone() {
            if masked(x, y) {
                mags.push(gx(x, y).abs().max(gy(x, y).abs()));
            }
        }
    }
    if mags.is_empty() {
        return None;
    }
    mags.sort_unstable();
    let t = mags[mags.len() * 9 / 10].max(16);
    let (mut total, mut count) = (0usize, 0usize);
    for y in ys.clone() {
        for x in xs.clone() {
            if !masked(x, y) || !graded(x - 1, y) || !graded(x + 1, y) {
                continue;
            }
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
    for x in xs.clone() {
        for y in ys.clone() {
            if !masked(x, y) || !graded(x, y - 1) || !graded(x, y + 1) {
                continue;
            }
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

/// `eye_measures` over `mask`: the masked Laplacian variance, the masked edge
/// width and that width over the mask window's longer side.
pub fn masked_eye_measures(
    gray: &[u8],
    width: usize,
    mask: &EyeMask,
) -> (f64, Option<f64>, Option<f64>) {
    let lap = masked_laplacian_variance(gray, width, mask);
    let edge = masked_edge_width(gray, width, mask);
    let side = mask.window.width.max(mask.window.height) as f64;
    (lap, edge, edge.map(|e| e / side))
}

/// Whether the cue runs the face mesh on `face`: its box long side is at
/// least `EYES_MIN_FACE`, the floor the closed-eyes judgment uses.
pub fn meshes_face(face: &Face) -> bool {
    face.width.max(face.height) >= EYES_MIN_FACE
}

/// The `eye_focus` of `face` on its mesh, `None` once `cancel` is set
/// while the mesh runs. A face `meshes_face` turns down gets no mesh, so the
/// eye window scores it.
fn scored_face(
    rgb: &[u8],
    gray: &[u8],
    width: usize,
    height: usize,
    orientation: u16,
    face: &Face,
    cancel: &AtomicBool,
) -> Option<EyeFocus> {
    let mesh = meshes_face(face)
        .then(|| mesh_of(rgb, width, height, orientation, face))
        .flatten();
    if cancel.load(Ordering::Relaxed) {
        return None;
    }
    Some(eye_focus(gray, width, height, face, mesh.as_ref()))
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
/// the detection, the face mesh and the eye scoring. The mesh runs only on a
/// face `meshes_face` accepts; a smaller one is scored on the eye window
/// without it. The mesh model's plan is built once per process (`eyes`'
/// `OnceLock`), so the first meshed file of a scan pays about 180 ms more.
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
    let focus = match face {
        Some(f) => Some(scored_face(
            &rgb,
            &gray,
            width,
            height,
            orientation,
            &f,
            cancel,
        )?),
        None => None,
    };
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
        let focus = eye_focus(&flat, 40, 40, &f, None);
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
        let focus = eye_focus(&gray, 40, 40, &f, None);
        let lap = laplacian_variance(&gray, 40, WHOLE);
        let logit = LOGIT_INTERCEPT + LOGIT_LAP * (lap + 1.0).ln() + LOGIT_EDGE_WIDTH * 0.1f64.ln();
        assert_eq!(
            (focus.scored, focus.window, focus.mesh),
            (Scored::Window, WHOLE, None)
        );
        assert_eq!(focus.lap, lap);
        assert_eq!(focus.edge_width_rel, Some(0.1));
        assert_eq!(focus.logit, Some(logit));
        assert_eq!(focus.probability, 1.0 / (1.0 + (-logit).exp()));
    }

    /// Four points of an eye 40 px wide and 10 px high whose box starts at
    /// `(x, y)`.
    fn eye_points(x: f32, y: f32) -> Vec<[f32; 2]> {
        vec![
            [x, y + 5.0],
            [x + 20.0, y],
            [x + 40.0, y + 5.0],
            [x + 20.0, y + 10.0],
        ]
    }

    #[test]
    fn an_eye_region_is_the_box_grown_by_the_margin() {
        assert_eq!(EYE_REGION_MARGIN, 0.5);
        assert_eq!(
            eye_region(&eye_points(100.0, 200.0), &[0, 1, 2, 3], 1000, 700),
            Some(Window {
                x: 80,
                y: 180,
                width: 80,
                height: 50
            })
        );
        assert_eq!(
            eye_region(&eye_points(100.5, 200.5), &[0, 1, 2, 3], 1000, 700),
            Some(Window {
                x: 80,
                y: 180,
                width: 81,
                height: 51
            }),
            "fractional edges round outward"
        );
    }

    #[test]
    fn a_quarter_turned_eye_takes_its_margin_from_the_longer_side() {
        let turned: Vec<[f32; 2]> = eye_points(100.0, 200.0)
            .iter()
            .map(|&[x, y]| [y, x])
            .collect();
        assert_eq!(
            eye_region(&turned, &[0, 1, 2, 3], 1000, 700),
            Some(Window {
                x: 180,
                y: 80,
                width: 50,
                height: 80
            })
        );
    }

    #[test]
    fn an_eye_region_is_clamped_at_the_image_edge() {
        assert_eq!(
            eye_region(&eye_points(5.0, 2.0), &[0, 1, 2, 3], 50, 14),
            Some(Window {
                x: 0,
                y: 0,
                width: 50,
                height: 14
            })
        );
        assert_eq!(
            eye_region(&eye_points(-80.0, 20.0), &[0, 1, 2, 3], 100, 100),
            None,
            "a box wholly outside the image"
        );
    }

    #[test]
    fn an_eye_region_is_none_for_a_bad_point_or_a_tiny_box() {
        let mut points = eye_points(100.0, 200.0);
        points[2][1] = f32::NAN;
        assert_eq!(eye_region(&points, &[0, 1, 2, 3], 1000, 700), None);
        points[2] = [f32::INFINITY, 205.0];
        assert_eq!(eye_region(&points, &[0, 1, 2, 3], 1000, 700), None);
        let points = eye_points(100.0, 200.0);
        assert_eq!(
            eye_region(&points, &[0, 1, 2, 4], 1000, 700),
            None,
            "an index past the points"
        );
        let flat = [[10.0, 10.0], [11.0, 10.0], [12.0, 10.0]];
        assert_eq!(eye_region(&flat, &[0, 1, 2], 100, 100), None);
        let small = [[10.0, 10.0], [10.5, 10.5]];
        assert_eq!(
            eye_region(&small, &[0, 1], 100, 100),
            None,
            "half a pixel plus its margin rounds out to 2 px"
        );
    }

    #[test]
    fn eye_measures_are_relative_to_the_longer_side() {
        let gray = ramp(4);
        for (w, h) in [(20, 16), (20, 30)] {
            let half = Window {
                x: 10,
                y: 5,
                width: w,
                height: h,
            };
            assert_eq!(
                eye_measures(&gray, 40, half),
                (
                    laplacian_variance(&gray, 40, half),
                    Some(4.0),
                    Some(4.0 / w.max(h) as f64)
                )
            );
        }
        let (_, edge, rel) = eye_measures(&ramp(8), 40, WHOLE);
        assert_eq!((edge, rel), (Some(8.0), Some(0.2)));
        let flat = vec![128u8; 40 * 40];
        assert_eq!(eye_measures(&flat, 40, WHOLE), (0.0, None, None));
    }

    #[test]
    fn mesh_eye_measures_take_each_eye_and_iris_from_the_mesh() {
        let mut points = vec![[f32::NAN; 2]; crate::eyes::LANDMARKS];
        for (k, &i) in LEFT_EYE_CONTOUR.iter().enumerate() {
            points[i] = [4.0 + (k % 2) as f32 * 8.0, 4.0 + (k % 3) as f32 * 4.0];
        }
        for (k, &i) in RIGHT_IRIS.iter().enumerate() {
            points[i] = [24.0 + (k % 2) as f32 * 8.0, 20.0 + (k % 3) as f32 * 4.0];
        }
        let pose = Pose {
            yaw: 10.0,
            pitch: 0.0,
            roll: 0.0,
        };
        let mesh = Mesh {
            points,
            pose: Some(pose),
        };
        let gray = ramp(4);
        let m = mesh_eye_measures(&gray, 40, 40, &mesh);
        let left = Window {
            x: 0,
            y: 0,
            width: 16,
            height: 16,
        };
        let (lap, edge_width, edge_width_rel) = eye_measures(&gray, 40, left);
        assert_eq!(
            m.left.contour,
            Some(RegionMeasures {
                window: left,
                lap,
                edge_width,
                edge_width_rel
            })
        );
        assert_eq!(m.right.iris.map(|r| r.window.x), Some(20));
        assert_eq!((m.left.iris, m.right.contour), (None, None));
        assert_eq!(m.pose, Some(pose));
    }

    /// A 120 x 60 image with a vertical step edge at column 20 blurred over
    /// `left` px and one at column 80 blurred over `right` px.
    fn two_steps(left: usize, right: usize) -> Vec<u8> {
        let step =
            |x: usize, at: usize, blur: usize| (x.saturating_sub(at).min(blur) * 200 / blur) as u8;
        let row: Vec<u8> = (0..120)
            .map(|x| {
                if x < 60 {
                    step(x, 20, left)
                } else {
                    step(x, 80, right)
                }
            })
            .collect();
        row.repeat(60)
    }

    /// A mesh whose eye contours span boxes `(x, y, width, height)` (`None`
    /// leaves that eye's points NaN), with `pose`.
    fn boxed_eyes(
        left: Option<(f32, f32, f32, f32)>,
        right: Option<(f32, f32, f32, f32)>,
        pose: Option<Pose>,
    ) -> Mesh {
        let mut points = vec![[f32::NAN; 2]; crate::eyes::LANDMARKS];
        for (contour, eye) in [(&LEFT_EYE_CONTOUR, left), (&RIGHT_EYE_CONTOUR, right)] {
            let Some((x, y, w, h)) = eye else { continue };
            for (k, &i) in contour.iter().enumerate() {
                points[i] = [x + (k % 2) as f32 * w, y + (k / 2 % 2) as f32 * h];
            }
        }
        Mesh { points, pose }
    }

    /// Both eyes 20 x 10, their regions 40 x 30 around the two step edges.
    fn both_eyes(pose: Option<Pose>) -> Mesh {
        boxed_eyes(
            Some((10.0, 20.0, 20.0, 10.0)),
            Some((70.0, 20.0, 20.0, 10.0)),
            pose,
        )
    }

    fn two_eyed() -> Face {
        face(0.0, 0.0, 60.0, (20.0, 25.0), (80.0, 25.0))
    }

    #[test]
    fn the_sharper_eye_scores_the_frame_whatever_the_pose() {
        let left = Window {
            x: 0,
            y: 10,
            width: 40,
            height: 30,
        };
        let right = Window { x: 60, ..left };
        // A face turned far toward the blurred eye: the pose does not choose.
        let turned = |yaw| Pose {
            yaw,
            pitch: 0.0,
            roll: 0.0,
        };
        for (blurs, yaw, eye, window) in [
            ((2, 8), 70.0, Eye::Left, left),
            ((8, 2), -70.0, Eye::Right, right),
        ] {
            let gray = two_steps(blurs.0, blurs.1);
            for pose in [None, Some(turned(yaw))] {
                let mesh = both_eyes(pose);
                let focus = eye_focus(&gray, 120, 60, &two_eyed(), Some(&mesh));
                let m = mesh_eye_measures(&gray, 120, 60, &mesh);
                let (l, r) = (mesh_logit(m.left.contour), mesh_logit(m.right.contour));
                assert!(l.is_some() && r.is_some());
                assert_eq!(focus.scored, Scored::Sharper(eye));
                assert_eq!(focus.window, window);
                assert_eq!(focus.logit, Some(l.unwrap().max(r.unwrap())));
                assert_eq!(focus.mesh, Some(m));
            }
        }
    }

    #[test]
    fn an_eye_counts_from_the_minimum_region_side() {
        let gray = two_steps(2, 2);
        // A 12 px wide eye grows to a 24 px region, an 11 px one to 23 px.
        for (w, side, scored) in [
            (12.0, 24, Scored::Only(Eye::Left)),
            (11.0, 23, Scored::Window),
        ] {
            let mesh = boxed_eyes(Some((20.0, 20.0, w, 4.0)), None, None);
            let m = mesh_eye_measures(&gray, 120, 60, &mesh);
            assert_eq!(m.left.contour.map(|r| r.window.width), Some(side));
            assert!(m.left.contour.and_then(|r| r.edge_width).is_some());
            let focus = eye_focus(&gray, 120, 60, &two_eyed(), Some(&mesh));
            assert_eq!(focus.scored, scored, "{side} px");
        }
        assert_eq!(EYE_REGION_MIN, 24);
    }

    #[test]
    fn an_eye_that_does_not_count_leaves_the_other() {
        let gray = two_steps(2, 8);
        let small = Some((75.0, 20.0, 4.0, 4.0));
        let mesh = boxed_eyes(Some((10.0, 20.0, 20.0, 10.0)), small, None);
        let focus = eye_focus(&gray, 120, 60, &two_eyed(), Some(&mesh));
        assert_eq!(focus.scored, Scored::Only(Eye::Left));
        let mut mesh = both_eyes(None);
        mesh.points[LEFT_EYE_CONTOUR[3]] = [f32::INFINITY, 20.0];
        let focus = eye_focus(&gray, 120, 60, &two_eyed(), Some(&mesh));
        assert_eq!(focus.scored, Scored::Only(Eye::Right));
        assert_eq!(focus.mesh.map(|m| m.left.contour), Some(None));
    }

    #[test]
    fn the_window_scores_a_face_with_no_eye_that_counts() {
        let gray = two_steps(2, 8);
        let window = eye_focus(&gray, 120, 60, &two_eyed(), None);
        assert_eq!(window.scored, Scored::Window);
        assert_eq!(window.window, eye_window(120, 60, &two_eyed()));
        let tiny = boxed_eyes(
            Some((20.0, 20.0, 4.0, 4.0)),
            Some((80.0, 20.0, 4.0, 4.0)),
            None,
        );
        let mut bad = both_eyes(None);
        bad.points[LEFT_EYE_CONTOUR[0]] = [f32::NAN, 20.0];
        bad.points[RIGHT_EYE_CONTOUR[0]] = [80.0, f32::NAN];
        for mesh in [&tiny, &bad] {
            let focus = eye_focus(&gray, 120, 60, &two_eyed(), Some(mesh));
            assert_eq!(focus.scored, Scored::Window);
            assert_eq!((focus.window, focus.logit), (window.window, window.logit));
            assert!(focus.mesh.is_some());
        }
        let flat = vec![128u8; 120 * 60];
        let focus = eye_focus(&flat, 120, 60, &two_eyed(), Some(&both_eyes(None)));
        assert_eq!(focus.scored, Scored::Window);
        assert_eq!(
            focus
                .mesh
                .and_then(|m| m.left.contour)
                .map(|r| r.edge_width),
            Some(None)
        );
    }

    fn mask_of(
        polygon: &[[f32; 2]],
        width: usize,
        height: usize,
        dilation: f32,
    ) -> Option<EyeMask> {
        let indices: Vec<usize> = (0..polygon.len()).collect();
        eye_mask(polygon, &indices, width, height, dilation)
    }

    const DIAMOND: [[f32; 2]; 4] = [[10.0, 0.0], [20.0, 10.0], [10.0, 20.0], [0.0, 10.0]];
    /// A 30 x 20 U open at the bottom, its notch 10 x 10.
    const NOTCHED: [[f32; 2]; 8] = [
        [0.0, 0.0],
        [30.0, 0.0],
        [30.0, 20.0],
        [20.0, 20.0],
        [20.0, 10.0],
        [10.0, 10.0],
        [10.0, 20.0],
        [0.0, 20.0],
    ];

    #[test]
    fn an_eye_mask_fills_the_polygon_at_pixel_centers() {
        let m = mask_of(&DIAMOND, 100, 100, 0.0).unwrap();
        assert_eq!(m.pixels(), 200, "the diamond's area");
        assert_eq!(
            m.window,
            Window {
                x: 0,
                y: 0,
                width: 19,
                height: 20
            }
        );
        assert!(m.contains(10, 10) && m.contains(9, 0) && m.contains(0, 9));
        assert!(!m.contains(1, 1) && !m.contains(10, 0) && !m.contains(50, 50));
        let m = mask_of(&NOTCHED, 100, 100, 0.0).unwrap();
        assert_eq!(m.pixels(), 500);
        assert_eq!(
            m.window,
            Window {
                x: 0,
                y: 0,
                width: 30,
                height: 20
            }
        );
        assert!(m.contains(15, 5) && m.contains(5, 15) && m.contains(25, 19));
        assert!(!m.contains(15, 15) && !m.contains(10, 10));
    }

    #[test]
    fn dilation_grows_an_eye_mask_and_never_shrinks_it() {
        for polygon in [&DIAMOND[..], &NOTCHED[..]] {
            // Away from the image edges, so no growth is clamped off.
            let shifted: Vec<[f32; 2]> =
                polygon.iter().map(|&[x, y]| [x + 30.0, y + 30.0]).collect();
            let masks: Vec<EyeMask> = [0.0, 0.1, 0.25, 0.5]
                .iter()
                .map(|&d| mask_of(&shifted, 100, 100, d).unwrap())
                .collect();
            for pair in masks.windows(2) {
                assert!(pair[1].pixels() > pair[0].pixels());
                for y in 0..100 {
                    for x in 0..100 {
                        assert!(!pair[0].contains(x, y) || pair[1].contains(x, y));
                    }
                }
            }
        }
        let notch = mask_of(&NOTCHED, 100, 100, 0.1).unwrap();
        assert!(notch.contains(10, 15) && !notch.contains(15, 15));
    }

    #[test]
    fn an_eye_mask_is_clamped_at_the_image_edge() {
        let square = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let m = mask_of(&square, 30, 30, 0.5).unwrap();
        assert_eq!(
            m.window,
            Window {
                x: 0,
                y: 0,
                width: 15,
                height: 15
            }
        );
        assert!(m.contains(13, 10) && !m.contains(14, 14));
        let m = mask_of(&square, 12, 8, 0.5).unwrap();
        assert_eq!(
            m.window,
            Window {
                x: 0,
                y: 0,
                width: 12,
                height: 8
            }
        );
        assert_eq!(m.pixels(), 96);
        let away: Vec<[f32; 2]> = square.iter().map(|&[x, y]| [x - 80.0, y]).collect();
        assert_eq!(
            mask_of(&away, 30, 30, 0.5),
            None,
            "wholly outside the image"
        );
    }

    #[test]
    fn an_eye_mask_is_none_for_a_bad_point_or_a_tiny_polygon() {
        let mut bad = DIAMOND;
        bad[2][1] = f32::NAN;
        assert_eq!(mask_of(&bad, 100, 100, 0.5), None);
        bad[2] = [f32::INFINITY, 20.0];
        assert_eq!(mask_of(&bad, 100, 100, 0.5), None);
        assert_eq!(
            eye_mask(&DIAMOND, &[0, 1, 2, 4], 100, 100, 0.5),
            None,
            "an index past the points"
        );
        let two = [[10.0, 10.0], [12.0, 10.0], [12.0, 11.0], [10.0, 11.0]];
        assert_eq!(mask_of(&two, 100, 100, 0.0), None);
        let three = [[10.0, 10.0], [13.0, 10.0], [13.0, 11.0], [10.0, 11.0]];
        assert_eq!(
            mask_of(&three, 100, 100, 0.0).map(|m| m.pixels()),
            Some(EYE_MASK_MIN)
        );
    }

    #[test]
    fn the_masked_laplacian_matches_the_window_one_on_its_interior() {
        for gray in [ramp(4), ramp(8), two_steps(2, 8)[..1600].to_vec()] {
            let window = Window {
                x: 5,
                y: 5,
                width: 20,
                height: 16,
            };
            let interior = [[6.0, 6.0], [24.0, 6.0], [24.0, 20.0], [6.0, 20.0]];
            let m = mask_of(&interior, 40, 40, 0.0).unwrap();
            assert_eq!(
                masked_laplacian_variance(&gray, 40, &m),
                laplacian_variance(&gray, 40, window)
            );
            let whole = [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]];
            let m = mask_of(&whole, 40, 40, 0.0).unwrap();
            assert_eq!(m.window, WHOLE);
            assert_eq!(
                masked_laplacian_variance(&gray, 40, &m),
                laplacian_variance(&gray, 40, WHOLE)
            );
            assert_eq!(
                masked_edge_width(&gray, 40, &m),
                edge_width(&gray, 40, WHOLE)
            );
        }
        let corner = EyeMask {
            window: Window {
                x: 0,
                y: 0,
                width: 1,
                height: 3,
            },
            inside: vec![true; 3],
        };
        assert_eq!(masked_laplacian_variance(&ramp(4), 40, &corner), 0.0);
    }

    #[test]
    fn the_masked_edge_width_measures_only_the_masked_edges() {
        let gray = two_steps(2, 8);
        let window = Window {
            x: 0,
            y: 0,
            width: 120,
            height: 60,
        };
        let half = |keep: fn(usize) -> bool| EyeMask {
            window,
            inside: (0..120 * 60).map(|i| keep(i % 120)).collect(),
        };
        let left = half(|x| x < 50);
        let right = half(|x| x >= 70);
        assert_eq!(masked_edge_width(&gray, 120, &left), Some(2.0));
        assert_eq!(masked_edge_width(&gray, 120, &right), Some(8.0));
        let (lap, edge, rel) = masked_eye_measures(&gray, 120, &right);
        assert_eq!(lap, masked_laplacian_variance(&gray, 120, &right));
        assert_eq!((edge, rel), (Some(8.0), Some(8.0 / 120.0)));
        let flat = vec![128u8; 120 * 60];
        assert_eq!(masked_eye_measures(&flat, 120, &left), (0.0, None, None));
    }

    #[test]
    fn the_mesh_score_follows_the_frozen_coefficients_on_the_shared_threshold() {
        assert_eq!(MESH_LOGIT_LAP, 1.9417143647766388);
        assert_eq!(MESH_LOGIT_EDGE_WIDTH, -1.3161251379398846);
        assert_eq!(
            MESH_LOGIT_INTERCEPT,
            -8.158737592019197 + (CANDIDATE_LOGIT - 0.8343419969086643)
        );
        let gray = two_steps(2, 8);
        let mesh = boxed_eyes(Some((10.0, 20.0, 20.0, 10.0)), None, None);
        let focus = eye_focus(&gray, 120, 60, &two_eyed(), Some(&mesh));
        let window = Window {
            x: 0,
            y: 10,
            width: 40,
            height: 30,
        };
        let (lap, edge, rel) = eye_measures(&gray, 120, window);
        let logit = MESH_LOGIT_INTERCEPT
            + MESH_LOGIT_LAP * (lap + 1.0).ln()
            + MESH_LOGIT_EDGE_WIDTH * (edge.unwrap() / 40.0).ln();
        assert_eq!(rel, edge.map(|e| e / 40.0));
        assert_eq!(focus.scored, Scored::Only(Eye::Left));
        assert_eq!(focus.logit, Some(logit));
        assert_eq!((focus.probability, focus.state), scored(logit));
        assert_eq!(candidate(Some(focus.probability)), focus.state);
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

    #[test]
    fn a_set_flag_abandons_the_cue_after_the_mesh() {
        let gray = two_steps(2, 8);
        let rgb: Vec<u8> = gray.iter().flat_map(|&g| [g, g, g]).collect();
        // Outside the image: no mesh, no model run.
        let away = face(500.0, 500.0, 60.0, (520.0, 525.0), (580.0, 525.0));
        let set = AtomicBool::new(true);
        assert_eq!(scored_face(&rgb, &gray, 120, 60, 1, &away, &set), None);
        let focus = scored_face(&rgb, &gray, 120, 60, 1, &away, &AtomicBool::new(false));
        assert_eq!(
            focus.map(|f| (f.scored, f.mesh)),
            Some((Scored::Window, None))
        );
    }

    #[test]
    fn a_face_under_the_floor_is_scored_on_the_window_without_the_mesh() {
        let gray = two_steps(2, 8);
        let rgb: Vec<u8> = gray.iter().flat_map(|&g| [g, g, g]).collect();
        // Inside the image, so only the gate keeps the model from running.
        let small = face(20.0, 0.0, 59.0, (35.0, 25.0), (65.0, 25.0));
        assert!(!meshes_face(&small));
        let focus = scored_face(&rgb, &gray, 120, 60, 1, &small, &AtomicBool::new(false));
        assert_eq!(
            focus.map(|f| (f.scored, f.mesh)),
            Some((Scored::Window, None))
        );
        let side = |s: f32| face(0.0, 0.0, s, (20.0, 25.0), (40.0, 25.0));
        assert!(!meshes_face(&side(EYES_MIN_FACE.next_down())));
        assert!(meshes_face(&side(EYES_MIN_FACE)));
        let wide = Face {
            height: 10.0,
            ..side(EYES_MIN_FACE)
        };
        assert!(meshes_face(&wide));
    }
}
