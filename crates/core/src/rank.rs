//! The personal best-shot ranking: a pairwise logistic model over each
//! frame's image embedding (projected to `PCA_DIMS` whitened dimensions), the
//! second pass's technical values and its position in the burst, which ranks
//! the frames of one burst against each other. It never changes a judgment.
//!
//! The prior `crates/core/ranking/siglip2.json` is the image-embeddings plan's
//! Step 2 model refit on the app's stored values of 27 culled folders by the
//! plan's offline `rank_prior.py`: the PCA basis, the imputation means, the
//! standardization and the weights. `fit` refits the weights on the user's
//! own judgments in the prior's coordinates, shrunk toward the prior's
//! weights; how far a few folders of viewed frames move the model is not
//! measured.

use std::cmp::Ordering;
use std::sync::OnceLock;

use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::candidate::{candidate, FocusCandidate};
use crate::eyes::closed_probability;
use crate::pose::Pose;
use crate::Flag;

/// The version of the columns below, their order and what each reads. The
/// prior and a stored fit carry it; bump it (and re-run `rank_prior.py`) on
/// any change to them.
pub const RANK_VERSION: i64 = 1;

/// The largest gap between two frames of one burst, `BURST_GAP_MS` in
/// `burst.ts`.
pub const BURST_GAP_MS: i64 = 1000;

const TECH: usize = 10;
const POSITION: usize = 2;
pub const PCA_DIMS: usize = 32;
/// The model's columns: the technical values, the position, the PCA
/// dimensions.
pub const COLUMNS: usize = TECH + POSITION + PCA_DIMS;

/// An explicit reject counts this many times a frame merely not chosen in the
/// pairs of its burst.
pub const REJECT_WEIGHT: f64 = 2.0;

/// The good-photo cuts of `photoTier` in `focus.ts`.
const GOOD_EYE_FOCUS: f64 = 0.9;
const GOOD_EYE_EAR: f64 = 0.25;
const GOOD_MAX_YAW: f64 = 35.0;
const GOOD_MAX_PITCH: f64 = 45.0;
const MAX_EYE_OFFSET: f64 = 0.1;
const MIN_EDGE_GAP: f64 = 0.02;

/// The reasons a score is told by, each a group of columns, with the columns
/// it sums over.
const REASONS: [(&str, std::ops::Range<usize>); 7] = [
    ("Look", TECH + POSITION..COLUMNS),
    ("Burst position", TECH..TECH + POSITION),
    ("Sharpness", 0..1),
    ("AF eye focus", 1..2),
    ("Eyes open", 2..3),
    ("Head pose", 3..6),
    ("Good photo", 6..7),
];

/// The missing-value indicators, which join the reason of the value they
/// stand for: no `eye_focus`, no eyes judgment, no pose.
const MISSING: [(usize, usize); 3] = [(7, 1), (8, 2), (9, 3)];

const FILE: &str = include_str!("../ranking/siglip2.json");

/// `model`, `embed_version` and `rank_version` are checked by a unit test, not
/// at run time.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Prior {
    /// The `embed::ModelSpec` name the basis was fit on.
    #[cfg_attr(not(test), expect(dead_code))]
    model: String,
    #[cfg_attr(not(test), expect(dead_code))]
    embed_version: i64,
    #[cfg_attr(not(test), expect(dead_code))]
    rank_version: i64,
    /// The training mean a missing technical value is set to.
    tech_mean: Vec<f64>,
    pca_mean: Vec<f32>,
    /// `PCA_DIMS` rows, each a component divided by the square root of its
    /// explained variance (whitened).
    pca_basis: Vec<Vec<f32>>,
    /// The standardization of every column.
    mean: Vec<f64>,
    scale: Vec<f64>,
    weights: Vec<f64>,
    /// scikit-learn's `C` the weights were fit at.
    c: f64,
}

fn prior() -> &'static Prior {
    static PRIOR: OnceLock<Prior> = OnceLock::new();
    PRIOR.get_or_init(|| {
        serde_json::from_str(FILE).unwrap_or_else(|e| panic!("ranking/siglip2.json: {e}"))
    })
}

/// The prior's weights, what the ranking uses before any fit.
pub fn prior_weights() -> &'static [f64] {
    &prior().weights
}

/// What the ranking reads of one file: its name and capture time for the
/// bursts, the values the second pass stored for its judged face (all `None`
/// when that pass has not run at the current version) and its embedding.
#[derive(Debug, Clone, Default)]
pub struct Frame {
    pub name: String,
    pub capture_time: Option<String>,
    pub subsec: Option<String>,
    pub sharpness: Option<f64>,
    pub eye_focus: Option<f64>,
    pub eyes_ear: Option<f64>,
    pub pose: Option<Pose>,
    pub eye_offset: Option<f64>,
    pub edge_gap: Option<f64>,
    /// An `embed` vector at `EMBED_VERSION`; a frame without one gets no
    /// score and takes no part in a fit.
    pub vec: Option<Vec<f32>>,
}

fn capture_ms(frame: &Frame) -> Option<i64> {
    let time =
        NaiveDateTime::parse_from_str(frame.capture_time.as_deref()?, "%Y:%m:%d %H:%M:%S").ok()?;
    let subsec: String = frame
        .subsec
        .as_deref()
        .unwrap_or("")
        .chars()
        .take(3)
        .collect();
    let subsec = format!("{subsec:0<3}");
    let ms = if subsec.bytes().all(|b| b.is_ascii_digit()) {
        subsec.parse::<i64>().unwrap_or(0)
    } else {
        0
    };
    Some(time.and_utc().timestamp_millis() + ms)
}

/// The capture order of `orderFiles("capture", ...)` in `sort.ts`.
fn capture_order(a: &Frame, b: &Frame) -> Ordering {
    let by_time = match (&a.capture_time, &b.capture_time) {
        (Some(ta), Some(tb)) => ta.cmp(tb).then_with(|| {
            let (sa, sb) = (
                a.subsec.as_deref().unwrap_or(""),
                b.subsec.as_deref().unwrap_or(""),
            );
            let width = sa.len().max(sb.len());
            format!("{sa:0<width$}").cmp(&format!("{sb:0<width$}"))
        }),
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (None, None) => Ordering::Equal,
    };
    by_time.then_with(|| a.name.cmp(&b.name))
}

/// The bursts of `frames`, as `groupBursts` in `burst.ts` forms them: in
/// capture order, a frame within `BURST_GAP_MS` of the previous one joins its
/// burst, and one without a usable capture time is a burst of its own. Each
/// burst lists indices into `frames`.
pub fn bursts(frames: &[Frame]) -> Vec<Vec<usize>> {
    let mut order: Vec<usize> = (0..frames.len()).collect();
    order.sort_by(|&a, &b| capture_order(&frames[a], &frames[b]));
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut previous: Option<i64> = None;
    for i in order {
        let ms = capture_ms(&frames[i]);
        match (ms, previous, groups.last_mut()) {
            (Some(ms), Some(previous), Some(group)) if ms - previous <= BURST_GAP_MS => {
                group.push(i);
            }
            _ => groups.push(vec![i]),
        }
        previous = ms;
    }
    groups
}

fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-3, 1.0 - 1e-3);
    (p / (1.0 - p)).ln()
}

/// The good-photo rule of `photoTier` in `focus.ts`.
fn good_photo(f: &Frame) -> bool {
    candidate(f.eye_focus) == FocusCandidate::Candidate
        && f.eye_focus.is_some_and(|p| p >= GOOD_EYE_FOCUS)
        && f.eyes_ear.is_some_and(|e| e >= GOOD_EYE_EAR)
        && f.pose
            .is_some_and(|p| p.yaw.abs() <= GOOD_MAX_YAW && p.pitch.abs() <= GOOD_MAX_PITCH)
        && f.eye_offset.is_some_and(|o| o <= MAX_EYE_OFFSET)
        && f.edge_gap.is_some_and(|g| g >= MIN_EDGE_GAP)
}

/// The technical columns of one frame of a burst whose sharpest frame scored
/// `top`, a missing value as NaN.
fn tech(f: &Frame, top: Option<f64>) -> [f64; TECH] {
    let nan = f64::NAN;
    let open = f.eyes_ear.map(|e| 1.0 - closed_probability(e));
    [
        match (f.sharpness, top) {
            (Some(s), Some(top)) if top != 0.0 => (s / top).max(1e-3).ln(),
            _ => nan,
        },
        f.eye_focus.map_or(nan, logit),
        open.unwrap_or(nan),
        f.pose.map_or(nan, |p| p.yaw.abs().min(90.0)),
        f.pose.map_or(nan, |p| p.pitch.abs().min(60.0)),
        f.pose.map_or(nan, |p| p.roll.abs().min(60.0)),
        if good_photo(f) { 1.0 } else { 0.0 },
        if f.eye_focus.is_none() { 1.0 } else { 0.0 },
        if open.is_none() { 1.0 } else { 0.0 },
        if f.pose.is_none() { 1.0 } else { 0.0 },
    ]
}

/// One frame's standardized columns.
pub type Row = [f64; COLUMNS];

/// The standardized columns of every frame with an embedding, burst by burst;
/// `None` for a frame without one.
pub fn rows(frames: &[Frame], bursts: &[Vec<usize>]) -> Vec<Option<Row>> {
    let prior = prior();
    let mut out = vec![None; frames.len()];
    for burst in bursts {
        let top = burst
            .iter()
            .filter_map(|&i| frames[i].sharpness)
            .reduce(f64::max);
        for (position, &i) in burst.iter().enumerate() {
            let f = &frames[i];
            let Some(vec) = &f.vec else {
                continue;
            };
            let mut row = [0.0; COLUMNS];
            for (j, v) in tech(f, top).into_iter().enumerate() {
                row[j] = if v.is_nan() { prior.tech_mean[j] } else { v };
            }
            row[TECH] = if burst.len() > 1 {
                position as f64 / (burst.len() - 1) as f64
            } else {
                0.0
            };
            row[TECH + 1] = if position == 0 { 1.0 } else { 0.0 };
            for (k, basis) in prior.pca_basis.iter().enumerate() {
                row[TECH + POSITION + k] = basis
                    .iter()
                    .zip(vec)
                    .zip(&prior.pca_mean)
                    .map(|((b, v), m)| f64::from(*b) * f64::from(v - m))
                    .sum();
            }
            for (j, x) in row.iter_mut().enumerate() {
                *x = (*x - prior.mean[j]) / prior.scale[j];
            }
            out[i] = Some(row);
        }
    }
    out
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

/// One frame's place in its burst.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Ranked {
    /// The frame's score above its burst's mean, in logits of the pairwise
    /// model: how much more likely to be the pick than an average frame of
    /// the burst.
    pub score: f64,
    /// 1-based, among the burst's frames with a score.
    pub place: usize,
    /// The burst's frames with a score.
    pub of: usize,
    /// What moved the score off the burst's mean, `(reason, logits)`,
    /// largest first by size; the logits sum to `score`.
    pub reasons: Vec<(&'static str, f64)>,
}

/// The rank of every frame with an embedding within its burst under
/// `weights`; `None` for a frame without one.
pub fn rank(frames: &[Frame], weights: &[f64]) -> Vec<Option<Ranked>> {
    let bursts = bursts(frames);
    let rows = rows(frames, &bursts);
    let mut out = vec![None; frames.len()];
    for burst in &bursts {
        let scored: Vec<(usize, &Row)> = burst
            .iter()
            .filter_map(|&i| rows[i].as_ref().map(|r| (i, r)))
            .collect();
        if scored.is_empty() {
            continue;
        }
        let n = scored.len() as f64;
        let mut mean = [0.0; COLUMNS];
        for (_, row) in &scored {
            for (m, x) in mean.iter_mut().zip(row.iter()) {
                *m += x / n;
            }
        }
        let scores: Vec<f64> = scored.iter().map(|(_, r)| dot(&r[..], weights)).collect();
        for (k, &(i, row)) in scored.iter().enumerate() {
            let part = |j: usize| weights[j] * (row[j] - mean[j]);
            let mut reasons: Vec<(&'static str, f64)> = REASONS
                .iter()
                .map(|(name, columns)| {
                    let own: f64 = columns.clone().map(part).sum();
                    let missing: f64 = MISSING
                        .iter()
                        .filter(|(_, value)| columns.contains(value))
                        .map(|&(j, _)| part(j))
                        .sum();
                    (*name, own + missing)
                })
                .collect();
            reasons.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()));
            out[i] = Some(Ranked {
                score: reasons.iter().map(|(_, v)| v).sum(),
                place: 1 + scores.iter().filter(|&&s| s > scores[k]).count(),
                of: scored.len(),
                reasons,
            });
        }
    }
    out
}

/// One folder's frames and judgments for `fit`. `viewed`, parallel to
/// `flags`, says which files the user looked at: only a viewed frame neither
/// picked nor rejected counts as a negative.
pub struct Labeled {
    pub frames: Vec<Frame>,
    pub flags: Vec<Flag>,
    pub viewed: Vec<bool>,
}

/// What `fit` found.
#[derive(Debug, Clone, PartialEq)]
pub struct Fit {
    pub weights: Vec<f64>,
    /// The bursts that gave at least one pair.
    pub bursts: usize,
    /// The folders that gave at least one pair.
    pub folders: usize,
}

/// One weighted pick-minus-other difference.
struct Pair {
    diff: Row,
    weight: f64,
}

fn pairs(folders: &[Labeled]) -> (Vec<Pair>, usize, usize) {
    let mut out = Vec::new();
    let (mut bursts_used, mut folders_used) = (0, 0);
    for folder in folders {
        let bursts = bursts(&folder.frames);
        let rows = rows(&folder.frames, &bursts);
        let mut used = false;
        for burst in &bursts {
            let picks: Vec<&Row> = burst
                .iter()
                .filter(|&&i| folder.flags[i] == Flag::Pick)
                .filter_map(|&i| rows[i].as_ref())
                .collect();
            let others: Vec<(&Row, f64)> = burst
                .iter()
                .filter_map(|&i| {
                    let weight = match folder.flags[i] {
                        Flag::Pick => return None,
                        Flag::Reject => REJECT_WEIGHT,
                        Flag::None if folder.viewed[i] => 1.0,
                        Flag::None => return None,
                    };
                    rows[i].as_ref().map(|r| (r, weight))
                })
                .collect();
            let total: f64 = picks.len() as f64 * others.iter().map(|(_, w)| w).sum::<f64>();
            if total == 0.0 {
                continue;
            }
            for pick in &picks {
                for (other, weight) in &others {
                    let mut diff = [0.0; COLUMNS];
                    for (d, (p, o)) in diff.iter_mut().zip(pick.iter().zip(other.iter())) {
                        *d = p - o;
                    }
                    out.push(Pair {
                        diff,
                        weight: weight / total,
                    });
                }
            }
            bursts_used += 1;
            used = true;
        }
        folders_used += usize::from(used);
    }
    (out, bursts_used, folders_used)
}

/// Fit the weights on `folders`: every pick against every other frame of its
/// burst that counts as a negative (a reject at `REJECT_WEIGHT`, any other
/// frame the user viewed at 1), each burst's pairs weighing 1 in all,
/// by the pairwise logistic loss plus an L2 pull toward the prior's weights
/// as strong as the prior's own regularization (`1 / (2 C)` per burst, the
/// same objective as scikit-learn's at `C`). With no pair it returns the
/// prior's weights.
pub fn fit(folders: &[Labeled]) -> Fit {
    let prior = prior();
    let (pairs, bursts, folders_used) = pairs(folders);
    let tau = 1.0 / (2.0 * prior.c);
    let mut w = prior.weights.clone();
    for _ in 0..50 {
        let mut grad: Vec<f64> = w
            .iter()
            .zip(&prior.weights)
            .map(|(w, p)| tau * (w - p))
            .collect();
        let mut hess = vec![0.0; COLUMNS * COLUMNS];
        for j in 0..COLUMNS {
            hess[j * COLUMNS + j] = tau;
        }
        for pair in &pairs {
            let m = dot(&pair.diff, &w);
            let s = 1.0 / (1.0 + m.exp());
            for (g, d) in grad.iter_mut().zip(&pair.diff) {
                *g -= pair.weight * s * d;
            }
            let h = pair.weight * s * (1.0 - s);
            for j in 0..COLUMNS {
                let hj = h * pair.diff[j];
                for k in j..COLUMNS {
                    hess[j * COLUMNS + k] += hj * pair.diff[k];
                }
            }
        }
        for j in 0..COLUMNS {
            for k in 0..j {
                hess[j * COLUMNS + k] = hess[k * COLUMNS + j];
            }
        }
        let step = solve(hess, grad);
        for (w, s) in w.iter_mut().zip(&step) {
            *w -= s;
        }
        if step.iter().map(|s| s * s).sum::<f64>().sqrt() < 1e-9 {
            break;
        }
    }
    Fit {
        weights: w,
        bursts,
        folders: folders_used,
    }
}

/// `x` with `a x = b` for a symmetric positive definite `a` (row-major,
/// `COLUMNS` square), by Cholesky.
fn solve(mut a: Vec<f64>, mut b: Vec<f64>) -> Vec<f64> {
    let n = COLUMNS;
    for j in 0..n {
        let d = (a[j * n + j] - (0..j).map(|k| a[j * n + k].powi(2)).sum::<f64>()).sqrt();
        a[j * n + j] = d;
        for i in j + 1..n {
            let s: f64 = (0..j).map(|k| a[i * n + k] * a[j * n + k]).sum();
            a[i * n + j] = (a[i * n + j] - s) / d;
        }
    }
    for i in 0..n {
        let s: f64 = (0..i).map(|k| a[i * n + k] * b[k]).sum();
        b[i] = (b[i] - s) / a[i * n + i];
    }
    for i in (0..n).rev() {
        let s: f64 = (i + 1..n).map(|k| a[k * n + i] * b[k]).sum();
        b[i] = (b[i] - s) / a[i * n + i];
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::{EMBED_VERSION, SHIPPED};

    fn frame(name: &str, time: &str, subsec: &str) -> Frame {
        Frame {
            name: name.into(),
            capture_time: Some(time.into()),
            subsec: Some(subsec.into()),
            vec: Some(vec![0.0; 768]),
            ..Frame::default()
        }
    }

    fn names(frames: &[Frame], bursts: &[Vec<usize>]) -> Vec<Vec<String>> {
        bursts
            .iter()
            .map(|b| b.iter().map(|&i| frames[i].name.clone()).collect())
            .collect()
    }

    #[test]
    fn the_prior_is_made_for_the_shipped_embedding_and_these_columns() {
        let prior = prior();
        assert_eq!(prior.model, SHIPPED.name);
        assert_eq!(
            prior.embed_version, EMBED_VERSION,
            "re-run rank_prior.py when EMBED_VERSION moves"
        );
        assert_eq!(prior.rank_version, RANK_VERSION);
        assert_eq!(prior.tech_mean.len(), TECH);
        assert_eq!(prior.pca_mean.len(), 768);
        assert_eq!(prior.pca_basis.len(), PCA_DIMS);
        assert!(prior.pca_basis.iter().all(|b| b.len() == 768));
        for v in [&prior.mean, &prior.scale, &prior.weights] {
            assert_eq!(v.len(), COLUMNS);
        }
        assert!(prior.scale.iter().all(|s| *s > 0.0));
        assert!(prior.c > 0.0);
    }

    #[test]
    fn bursts_group_as_burst_ts_does() {
        let frames = vec![
            frame("c.ARW", "2026:09:19 10:00:01", "500"),
            frame("a.ARW", "2026:09:19 10:00:00", "5"),
            frame("b.ARW", "2026:09:19 10:00:00", "50"),
            frame("d.ARW", "2026:09:19 10:00:02", "51"),
            frame("e.ARW", "2026:09:19 10:00:04", "5"),
            Frame {
                name: "f.ARW".into(),
                ..Frame::default()
            },
            Frame {
                name: "g.ARW".into(),
                capture_time: Some("garbage".into()),
                ..Frame::default()
            },
        ];
        // `5` and `50` both read 10:00:00.500 and go by name; 1000 ms on to
        // 10:00:01.500 stays in the burst, 1010 ms on to 10:00:02.510 does
        // not. A time that does not parse sorts as a string, a missing one
        // last.
        assert_eq!(
            names(&frames, &bursts(&frames)),
            [
                vec!["a.ARW", "b.ARW", "c.ARW"],
                vec!["d.ARW"],
                vec!["e.ARW"],
                vec!["g.ARW"],
                vec!["f.ARW"],
            ]
        );
    }

    #[test]
    fn rows_fill_the_position_and_impute_a_missing_value() {
        let prior = prior();
        let mut frames = vec![
            frame("a.ARW", "2026:09:19 10:00:00", "0"),
            frame("b.ARW", "2026:09:19 10:00:00", "5"),
            frame("c.ARW", "2026:09:19 10:00:01", "0"),
        ];
        frames[0].sharpness = Some(50.0);
        frames[1].sharpness = Some(100.0);
        frames[2].vec = None;
        let bursts = bursts(&frames);
        let rows = rows(&frames, &bursts);
        let raw = |row: &Row, j: usize| row[j] * prior.scale[j] + prior.mean[j];
        let (a, b) = (rows[0].unwrap(), rows[1].unwrap());
        assert!((raw(&a, 0) - 0.5f64.ln()).abs() < 1e-9);
        assert!(raw(&b, 0).abs() < 1e-9, "the sharpest frame is 0");
        assert!((raw(&a, 1) - prior.tech_mean[1]).abs() < 1e-9);
        assert!((raw(&a, 7) - 1.0).abs() < 1e-9, "no eye_focus");
        assert!(raw(&a, TECH).abs() < 1e-9);
        assert!((raw(&b, TECH) - 0.5).abs() < 1e-9);
        assert!((raw(&a, TECH + 1) - 1.0).abs() < 1e-9);
        assert!(raw(&b, TECH + 1).abs() < 1e-9);
        assert!(rows[2].is_none(), "a frame without an embedding");
    }

    #[test]
    fn a_good_photo_clears_every_cut_of_photo_tier() {
        let good = Frame {
            eye_focus: Some(0.95),
            eyes_ear: Some(0.3),
            pose: Some(Pose {
                yaw: 30.0,
                pitch: -40.0,
                roll: 50.0,
            }),
            eye_offset: Some(0.05),
            edge_gap: Some(0.1),
            ..Frame::default()
        };
        assert!(good_photo(&good));
        for bad in [
            Frame {
                eye_focus: Some(0.85),
                ..good.clone()
            },
            Frame {
                eyes_ear: Some(0.2),
                ..good.clone()
            },
            Frame {
                pose: None,
                ..good.clone()
            },
            Frame {
                eye_offset: Some(0.2),
                ..good.clone()
            },
            Frame {
                edge_gap: Some(0.0),
                ..good.clone()
            },
        ] {
            assert!(!good_photo(&bad));
        }
    }

    #[test]
    fn rank_places_each_frame_in_its_burst_and_its_reasons_sum_to_its_score() {
        let mut frames = vec![
            frame("a.ARW", "2026:09:19 10:00:00", "0"),
            frame("b.ARW", "2026:09:19 10:00:00", "5"),
            frame("c.ARW", "2026:09:19 10:00:01", "0"),
            frame("d.ARW", "2026:09:19 10:00:05", "0"),
        ];
        frames[0].sharpness = Some(50.0);
        frames[1].sharpness = Some(100.0);
        frames[2].vec = None;
        let mut weights = vec![0.0; COLUMNS];
        weights[0] = 1.0;
        weights[TECH] = -0.5;
        let ranked = rank(&frames, &weights);
        let (a, b) = (ranked[0].clone().unwrap(), ranked[1].clone().unwrap());
        assert_eq!((b.place, b.of, a.place, a.of), (1, 2, 2, 2));
        assert!(
            (a.score + b.score).abs() < 1e-9,
            "scores sit around the burst mean"
        );
        assert!(b.score > 0.0);
        for r in [&a, &b] {
            let sum: f64 = r.reasons.iter().map(|(_, v)| v).sum();
            assert!((sum - r.score).abs() < 1e-9);
            assert!(r.reasons.windows(2).all(|w| w[0].1.abs() >= w[1].1.abs()));
        }
        assert!(ranked[2].is_none());
        let d = ranked[3].clone().unwrap();
        assert_eq!((d.place, d.of), (1, 1));
        assert!(
            d.score.abs() < 1e-9,
            "a lone frame sits at its burst's mean"
        );
    }

    /// Two frames a second apart in one burst, the `sharper` one at 100.
    fn labeled(first: Flag, second: Flag, viewed: [bool; 2], sharper: usize) -> Labeled {
        let mut frames = vec![
            frame("a.ARW", "2026:09:19 10:00:00", "0"),
            frame("b.ARW", "2026:09:19 10:00:01", "0"),
        ];
        frames[sharper].sharpness = Some(100.0);
        frames[1 - sharper].sharpness = Some(20.0);
        Labeled {
            frames,
            flags: vec![first, second],
            viewed: viewed.to_vec(),
        }
    }

    #[test]
    fn a_fit_without_a_pair_is_the_prior() {
        let fit = fit(&[
            labeled(Flag::Pick, Flag::None, [true, false], 0),
            labeled(Flag::None, Flag::None, [true, true], 0),
            labeled(Flag::Pick, Flag::Pick, [true, true], 0),
        ]);
        assert_eq!(fit.weights, prior_weights());
        assert_eq!((fit.bursts, fit.folders), (0, 0));
    }

    #[test]
    fn a_fit_counts_rejects_viewed_or_not_and_the_rest_only_when_viewed() {
        let fit = fit(&[
            labeled(Flag::Pick, Flag::Reject, [false, false], 0),
            labeled(Flag::Pick, Flag::Reject, [true, true], 0),
            labeled(Flag::Pick, Flag::None, [true, true], 0),
            labeled(Flag::Pick, Flag::None, [true, false], 0),
        ]);
        assert_eq!((fit.bursts, fit.folders), (3, 3));
    }

    #[test]
    fn an_unflagged_frame_is_a_negative_only_when_viewed() {
        let viewed = fit(&[labeled(Flag::Pick, Flag::None, [false, true], 0)]);
        assert_eq!((viewed.bursts, viewed.folders), (1, 1));
        let unviewed = fit(&[labeled(Flag::Pick, Flag::None, [true, false], 0)]);
        assert_eq!((unviewed.bursts, unviewed.folders), (0, 0));
        assert_eq!(unviewed.weights, prior_weights());
    }

    #[test]
    fn a_fit_leans_toward_what_the_picks_have_and_more_with_more_data() {
        let few: Vec<Labeled> = (0..5)
            .map(|_| labeled(Flag::Pick, Flag::None, [true, true], 0))
            .collect();
        let many: Vec<Labeled> = (0..500)
            .map(|_| labeled(Flag::Pick, Flag::None, [true, true], 0))
            .collect();
        let (few, many) = (fit(&few), fit(&many));
        let frames = labeled(Flag::Pick, Flag::None, [true, true], 0).frames;
        let rows = rows(&frames, &bursts(&frames));
        let (pick, other) = (rows[0].unwrap(), rows[1].unwrap());
        let margin = |w: &[f64]| dot(&pick, w) - dot(&other, w);
        assert!(margin(&few.weights) > margin(prior_weights()));
        assert!(margin(&many.weights) > margin(&few.weights));
        assert_eq!((many.bursts, many.folders), (500, 500));
    }

    #[test]
    fn solve_inverts_a_symmetric_positive_definite_matrix() {
        let n = COLUMNS;
        let mut a = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                a[i * n + j] = if i == j {
                    4.0
                } else {
                    1.0 / (1 + i + j) as f64
                };
            }
        }
        let x: Vec<f64> = (0..n).map(|i| i as f64 - 20.0).collect();
        let b: Vec<f64> = (0..n)
            .map(|i| (0..n).map(|j| a[i * n + j] * x[j]).sum())
            .collect();
        let solved = solve(a, b);
        assert!(solved.iter().zip(&x).all(|(s, x)| (s - x).abs() < 1e-9));
    }
}
