//! Per-language values, one JSON file per language in `crates/core/i18n/`
//! named by its language code (`en.json`, `ja.json`) and embedded at build
//! time by `build.rs`, so adding a language is adding a file.
//!
//! Each file is namespaced: `meta.name` is the language's name in that
//! language, and `lightroom.colorLabels` the names of Lightroom's default
//! color label set in that UI language, with `verified` naming the Lightroom
//! version, OS and UI language they were checked against. `en.json` is the
//! reference: every key of another file must exist in it, and a missing key
//! reads as `en.json`'s value. With only `colorLabels` populated this reduces
//! to "a language without `colorLabels` offers no preset".

use std::sync::OnceLock;

use serde::Deserialize;

use crate::xmp::LabelNames;

/// `(language code, file contents)`, sorted by code.
const FILES: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/i18n.rs"));

/// A language's Lightroom default color label set.
#[derive(Clone, Debug, PartialEq)]
pub struct Preset {
    pub code: String,
    /// The language's name in that language.
    pub name: String,
    /// The Lightroom version, OS and UI language the names were checked
    /// against.
    pub verified: String,
    pub names: LabelNames,
}

#[derive(Deserialize)]
struct File {
    meta: Meta,
    #[serde(default)]
    lightroom: Lightroom,
}

#[derive(Deserialize)]
struct Meta {
    name: String,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Lightroom {
    color_labels: Option<ColorLabels>,
}

#[derive(Deserialize)]
struct ColorLabels {
    verified: String,
    red: String,
    yellow: String,
    green: String,
    blue: String,
    purple: String,
}

/// The Lightroom color label presets, English first and then the rest by
/// language code, one per language whose file has `lightroom.colorLabels`.
pub fn presets() -> &'static [Preset] {
    static PRESETS: OnceLock<Vec<Preset>> = OnceLock::new();
    PRESETS.get_or_init(|| {
        let mut presets: Vec<Preset> = FILES
            .iter()
            .filter_map(|(code, json)| {
                let file: File =
                    serde_json::from_str(json).unwrap_or_else(|e| panic!("i18n/{code}.json: {e}"));
                let labels = file.lightroom.color_labels?;
                Some(Preset {
                    code: code.to_string(),
                    name: file.meta.name,
                    verified: labels.verified,
                    names: LabelNames {
                        red: labels.red,
                        yellow: labels.yellow,
                        green: labels.green,
                        blue: labels.blue,
                        purple: labels.purple,
                    },
                })
            })
            .collect();
        presets.sort_by_key(|preset| preset.code != "en");
        presets
    })
}

/// The preset of the language `code`, if its file has one.
pub fn preset(code: &str) -> Option<&'static Preset> {
    presets().iter().find(|preset| preset.code == code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn parse(code: &str, json: &str) -> Value {
        serde_json::from_str(json).unwrap_or_else(|e| panic!("i18n/{code}.json: {e}"))
    }

    fn reference() -> Value {
        let (_, json) = FILES.iter().find(|(code, _)| *code == "en").unwrap();
        parse("en", json)
    }

    fn assert_paths_in(value: &Value, reference: &Value, path: &str, code: &str) {
        let Value::Object(map) = value else {
            return;
        };
        for (key, child) in map {
            let path = if path.is_empty() {
                key.clone()
            } else {
                format!("{path}.{key}")
            };
            let Some(expected) = reference.get(key) else {
                panic!("i18n/{code}.json: {path} is not in en.json");
            };
            assert_paths_in(child, expected, &path, code);
        }
    }

    #[test]
    fn every_key_exists_in_en_json() {
        let reference = reference();
        for (code, json) in FILES {
            assert_paths_in(&parse(code, json), &reference, "", code);
        }
    }

    #[test]
    fn every_file_is_complete() {
        for (code, json) in FILES {
            let file: File =
                serde_json::from_str(json).unwrap_or_else(|e| panic!("i18n/{code}.json: {e}"));
            assert!(
                !file.meta.name.trim().is_empty(),
                "i18n/{code}.json: meta.name"
            );
            if let Some(labels) = file.lightroom.color_labels {
                for (key, value) in [
                    ("verified", &labels.verified),
                    ("red", &labels.red),
                    ("yellow", &labels.yellow),
                    ("green", &labels.green),
                    ("blue", &labels.blue),
                    ("purple", &labels.purple),
                ] {
                    assert!(
                        !value.trim().is_empty(),
                        "i18n/{code}.json: lightroom.colorLabels.{key}"
                    );
                }
            }
        }
    }

    #[test]
    fn every_file_is_named_by_a_language_code() {
        for (code, _) in FILES {
            let mut parts = code.split('-');
            let language = parts.next().unwrap();
            assert!(
                (2..=3).contains(&language.len())
                    && language.chars().all(|c| c.is_ascii_lowercase()),
                "i18n/{code}.json"
            );
            for part in parts {
                assert!(
                    (2..=8).contains(&part.len())
                        && part.chars().all(|c| c.is_ascii_alphanumeric()),
                    "i18n/{code}.json"
                );
            }
        }
    }

    #[test]
    fn english_comes_first_and_matches_the_default_names() {
        let presets = presets();
        assert_eq!(presets[0].code, "en");
        assert_eq!(presets[0].name, "English");
        assert_eq!(presets[0].names, LabelNames::default());
        let rest: Vec<&str> = presets[1..].iter().map(|p| p.code.as_str()).collect();
        let mut sorted = rest.clone();
        sorted.sort();
        assert_eq!(rest, sorted);
    }

    #[test]
    fn japanese_carries_the_lightroom_names() {
        let ja = preset("ja").unwrap();
        assert_eq!(ja.name, "日本語");
        assert_eq!(ja.names.red, "レッド");
        assert_eq!(ja.names.purple, "パープル");
    }
}
