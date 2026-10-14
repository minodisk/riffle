//! The genre labels of an image embedding, from the genre data file
//! `crates/core/genres/siglip2.json`, written by the plan's offline
//! `genre_vectors.py` (text vectors of each label's prompt ensemble from the
//! SigLIP 2 text tower, which is not shipped) and embedded at build time.
//!
//! A label is on when the cosine of the embedding with its vector (the mean
//! of its L2-normalized prompt vectors, normalized again) reaches its
//! threshold, calibrated per label on hand-labeled frames (SigLIP's sigmoid
//! space gives no shared cut), or when one of its sub-genres is on. A frame
//! with no label on is `UNKNOWN`. A burst's genre is the top-level label
//! on for the most of its frames (`of_burst`), which the personal ranking
//! keeps one offset per.

use std::sync::OnceLock;

use serde::Deserialize;

/// The label of a frame no genre clears.
pub const UNKNOWN: &str = "Unknown";

const FILE: &str = include_str!("../genres/siglip2.json");

/// `model` and `embed_version` are checked by a unit test, not at run time.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct File {
    /// The `embed::ModelSpec` name the vectors were made for.
    #[cfg_attr(not(test), expect(dead_code))]
    model: String,
    /// The `embed::EMBED_VERSION` the vectors were made for.
    #[cfg_attr(not(test), expect(dead_code))]
    embed_version: i64,
    labels: Vec<Label>,
}

#[derive(Deserialize)]
struct Label {
    label: String,
    parent: Option<String>,
    /// On the cosine with `vector`.
    threshold: f32,
    vector: Vec<f32>,
}

fn file() -> &'static File {
    static FILE_DATA: OnceLock<File> = OnceLock::new();
    FILE_DATA.get_or_init(|| {
        serde_json::from_str(FILE).unwrap_or_else(|e| panic!("genres/siglip2.json: {e}"))
    })
}

/// The labels `vec` (an `embed` vector at `EMBED_VERSION`) is on, in the
/// data file's order, a parent on with any of its sub-genres; `[UNKNOWN]`
/// when none is.
pub fn genres_of(vec: &[f32]) -> Vec<&'static str> {
    let labels = &file().labels;
    let mut on: Vec<bool> = labels
        .iter()
        .map(|l| l.vector.iter().zip(vec).map(|(a, b)| a * b).sum::<f32>() >= l.threshold)
        .collect();
    for (i, label) in labels.iter().enumerate() {
        if on[i] {
            if let Some(parent) = &label.parent {
                if let Some(p) = labels.iter().position(|l| &l.label == parent) {
                    on[p] = true;
                }
            }
        }
    }
    let found: Vec<&'static str> = labels
        .iter()
        .zip(&on)
        .filter(|(_, on)| **on)
        .map(|(l, _)| l.label.as_str())
        .collect();
    if found.is_empty() {
        vec![UNKNOWN]
    } else {
        found
    }
}

/// The labels with no parent, in the data file's order.
pub fn top_level() -> Vec<&'static str> {
    file()
        .labels
        .iter()
        .filter(|l| l.parent.is_none())
        .map(|l| l.label.as_str())
        .collect()
}

/// The genre of a burst whose frames have the embeddings `vecs`: the
/// top-level label `genres_of` turns on for the most of them (a frame
/// counting toward every top-level label it turns on), a tie going to the
/// label earlier in the data file; `UNKNOWN` when no frame turns one on.
pub fn of_burst<'a>(vecs: impl IntoIterator<Item = &'a [f32]>) -> &'static str {
    let top = top_level();
    let mut counts = vec![0usize; top.len()];
    for vec in vecs {
        for label in genres_of(vec) {
            if let Some(k) = top.iter().position(|t| *t == label) {
                counts[k] += 1;
            }
        }
    }
    let mut best: Option<usize> = None;
    for (k, &count) in counts.iter().enumerate() {
        if count > 0 && best.is_none_or(|b| count > counts[b]) {
            best = Some(k);
        }
    }
    best.map_or(UNKNOWN, |k| top[k])
}

/// A vector `genres_of` turns `label` on for (and its parent), and no other
/// label: `label`'s vector with its projection onto every other label's
/// removed, scaled so its dot with `label`'s vector is 1.
#[cfg(test)]
pub(crate) fn vector_of(label: &str) -> Vec<f32> {
    let labels = &file().labels;
    let to_f64 = |v: &[f32]| v.iter().map(|x| f64::from(*x)).collect::<Vec<f64>>();
    let dot = |a: &[f64], b: &[f64]| a.iter().zip(b).map(|(a, b)| a * b).sum::<f64>();
    let target = labels.iter().find(|l| l.label == label).unwrap();
    let mut basis: Vec<Vec<f64>> = Vec::new();
    for other in labels.iter().filter(|l| l.label != label) {
        let mut v = to_f64(&other.vector);
        for b in &basis {
            let p = dot(&v, b);
            v.iter_mut().zip(b).for_each(|(x, b)| *x -= p * b);
        }
        let norm = dot(&v, &v).sqrt();
        basis.push(v.iter().map(|x| x / norm).collect());
    }
    let t = to_f64(&target.vector);
    let mut u = t.clone();
    for b in &basis {
        let p = dot(&u, b);
        u.iter_mut().zip(b).for_each(|(x, b)| *x -= p * b);
    }
    let scale = dot(&u, &t);
    u.iter().map(|x| (x / scale) as f32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::{EMBED_VERSION, SHIPPED};

    #[test]
    fn the_data_file_is_made_for_the_shipped_embedding() {
        let file = file();
        assert_eq!(file.model, SHIPPED.name);
        assert_eq!(
            file.embed_version, EMBED_VERSION,
            "re-run genre_vectors.py (and recalibrate) when EMBED_VERSION moves"
        );
    }

    #[test]
    fn every_label_has_a_unit_vector_and_a_top_level_parent() {
        let labels = &file().labels;
        assert!(!labels.is_empty());
        for label in labels {
            assert_eq!(label.vector.len(), 768, "{}", label.label);
            let norm = label.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 1e-4, "{}: {norm}", label.label);
            assert!(label.threshold > 0.0 && label.threshold < 1.0);
            assert_ne!(label.label, UNKNOWN);
            if let Some(parent) = &label.parent {
                let parent = labels.iter().find(|l| &l.label == parent);
                assert!(
                    parent.is_some_and(|p| p.parent.is_none()),
                    "{}",
                    label.label
                );
            }
        }
    }

    #[test]
    fn a_sub_genre_turns_its_parent_on() {
        let labels = &file().labels;
        let sub = labels.iter().find(|l| l.parent.is_some()).unwrap();
        let found = genres_of(&sub.vector);
        assert!(found.contains(&sub.label.as_str()));
        assert!(found.contains(&sub.parent.as_deref().unwrap()));
    }

    #[test]
    fn a_vector_no_label_clears_is_unknown() {
        assert_eq!(genres_of(&[0.0; 768]), [UNKNOWN]);
    }

    #[test]
    fn the_top_level_labels_are_those_without_a_parent_in_file_order() {
        let top = top_level();
        let expected: Vec<&str> = file()
            .labels
            .iter()
            .filter(|l| l.parent.is_none())
            .map(|l| l.label.as_str())
            .collect();
        assert_eq!(top, expected);
        assert!(top.len() >= 2);
        for label in &top {
            assert_eq!(genres_of(&vector_of(label)), [*label]);
        }
    }

    #[test]
    fn a_burst_goes_to_the_label_most_of_its_frames_turn_on() {
        let top = top_level();
        let (a, b) = (vector_of(top[0]), vector_of(top[1]));
        assert_eq!(of_burst([&b[..], &a[..], &b[..]]), top[1]);
    }

    #[test]
    fn a_tie_goes_to_the_label_earlier_in_the_file() {
        let top = top_level();
        let (a, b) = (vector_of(top[0]), vector_of(top[1]));
        assert_eq!(of_burst([&b[..], &a[..]]), top[0]);
        assert_eq!(of_burst([&a[..], &b[..]]), top[0]);
    }

    #[test]
    fn a_frame_counts_toward_every_top_level_label_it_turns_on() {
        let top = top_level();
        let (a, b) = (vector_of(top[0]), vector_of(top[1]));
        let both: Vec<f32> = a.iter().zip(&b).map(|(a, b)| a + b).collect();
        assert_eq!(genres_of(&both), [top[0], top[1]]);
        // Were `both` counted for one label only, one of these would go the
        // other way.
        assert_eq!(of_burst([&a[..], &b[..], &both[..]]), top[0]);
        assert_eq!(of_burst([&a[..], &b[..], &b[..], &both[..]]), top[1]);
        let sub = file().labels.iter().find(|l| l.parent.is_some()).unwrap();
        let parent = sub.parent.as_deref().unwrap();
        assert_eq!(of_burst([&vector_of(&sub.label)[..]]), parent);
    }

    #[test]
    fn no_vector_or_no_label_on_is_unknown() {
        assert_eq!(of_burst(std::iter::empty::<&[f32]>()), UNKNOWN);
        assert_eq!(of_burst([&[0.0f32; 768][..], &[0.0; 768][..]]), UNKNOWN);
    }
}
