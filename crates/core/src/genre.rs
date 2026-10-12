//! The genre labels of an image embedding, from the genre data file
//! `crates/core/genres/siglip2.json`, written by the plan's offline
//! `genre_vectors.py` (text vectors of each label's prompt ensemble from the
//! SigLIP 2 text tower, which is not shipped) and embedded at build time.
//!
//! A label is on when the cosine of the embedding with its vector (the mean
//! of its L2-normalized prompt vectors, normalized again) reaches its
//! threshold, calibrated per label on hand-labeled frames (SigLIP's sigmoid
//! space gives no shared cut), or when one of its sub-genres is on. A frame
//! with no label on is `UNKNOWN`.

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
}
