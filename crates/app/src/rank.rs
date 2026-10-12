//! The personal ranking in the app (`riffle_core::rank`): the folders marked
//! culling finished, the weights fit on the user's judgments, stored as
//! `ranking.json` in the app's local data dir, and the per-file rank the meta
//! pane and the `recommended` sort read. It never changes a judgment.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use riffle_core::embed::EMBED_VERSION;
use riffle_core::rank::{self, Labeled, Ranked, COLUMNS, RANK_VERSION};
use tauri::{AppHandle, Emitter, Manager};

use crate::commands::{canonicalize, AppIndex, AppIndexReader};
use crate::index::lock;

/// The fit's file under the app's local data dir.
const FILE: &str = "ranking.json";

/// The weights in use, read from `FILE` on first use; `fit` serializes the
/// refits.
#[derive(Default)]
pub struct AppRank {
    stored: Mutex<Option<Stored>>,
    fit: Mutex<()>,
}

/// What `FILE` holds: the weights with the versions they were fit at and the
/// size of what they were fit on. Zero bursts means the prior's weights.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stored {
    rank_version: i64,
    embed_version: i64,
    weights: Vec<f64>,
    bursts: usize,
    folders: usize,
}

impl Stored {
    fn prior() -> Self {
        Stored {
            rank_version: RANK_VERSION,
            embed_version: EMBED_VERSION,
            weights: rank::prior_weights().to_vec(),
            bursts: 0,
            folders: 0,
        }
    }

    /// `json`'s fit when it was made at the current versions, else the prior.
    fn parse(json: &str) -> Self {
        serde_json::from_str::<Stored>(json)
            .ok()
            .filter(|s| {
                s.rank_version == RANK_VERSION
                    && s.embed_version == EMBED_VERSION
                    && s.weights.len() == COLUMNS
            })
            .unwrap_or_else(Stored::prior)
    }
}

/// How much of the model is the user's own, for the meta pane and the
/// settings: the bursts and folders it was fit on and a level read from the
/// bursts.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ModelInfo {
    bursts: usize,
    folders: usize,
    confidence: &'static str,
}

/// The level of a fit on `bursts` bursts with at least one pair: `none` for
/// the prior alone, then `low`, `medium` and `high`. Step 2's learning curve
/// on 27 folders (about 70 scorable bursts each) still rose from 0.699 at
/// about 700 bursts to 0.708 at 1800.
fn confidence(bursts: usize) -> &'static str {
    match bursts {
        0 => "none",
        1..300 => "low",
        300..1000 => "medium",
        _ => "high",
    }
}

fn info(stored: &Stored) -> ModelInfo {
    ModelInfo {
        bursts: stored.bursts,
        folders: stored.folders,
        confidence: confidence(stored.bursts),
    }
}

fn file(app: &AppHandle) -> Option<PathBuf> {
    match app.path().app_local_data_dir() {
        Ok(dir) => Some(dir.join(FILE)),
        Err(e) => {
            log::error!("no local data dir for the ranking: {e}");
            None
        }
    }
}

fn stored(app: &AppHandle) -> Stored {
    let state = app.state::<AppRank>();
    let mut stored = lock(&state.stored);
    stored
        .get_or_insert_with(|| {
            file(app)
                .and_then(|path| std::fs::read_to_string(path).ok())
                .map_or_else(Stored::prior, |json| Stored::parse(&json))
        })
        .clone()
}

/// The rank of `dir`'s files in their bursts, keyed by path (a file without
/// an embedding is absent), with the model's size.
#[derive(Debug, serde::Serialize)]
pub struct RankingOf {
    model: ModelInfo,
    files: HashMap<String, Ranked>,
}

#[tauri::command]
pub async fn ranking_of(app: AppHandle, dir: String) -> Result<RankingOf, String> {
    let dir = canonicalize(&dir);
    let stored = stored(&app);
    let model = info(&stored);
    let Some(index) = app.state::<AppIndexReader>().0.clone() else {
        return Ok(RankingOf {
            model,
            files: HashMap::new(),
        });
    };
    tauri::async_runtime::spawn_blocking(move || {
        let rows = lock(&index).rank_rows(&dir)?;
        let frames: Vec<rank::Frame> = rows.iter().map(|r| r.frame.clone()).collect();
        let files = rows
            .into_iter()
            .zip(rank::rank(&frames, &stored.weights))
            .filter_map(|(row, ranked)| ranked.map(|r| (row.path, r)))
            .collect();
        Ok(RankingOf { model, files })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The size of the model in use, for the settings.
#[tauri::command]
pub fn ranking_model(app: AppHandle) -> ModelInfo {
    info(&stored(&app))
}

/// Refit on the folders `rank_dirs` lists, store the weights and announce
/// them with `ranking-updated`.
fn refit(app: &AppHandle) -> Result<ModelInfo, String> {
    let state = app.state::<AppRank>();
    let _fit = lock(&state.fit);
    let Some(index) = app.state::<AppIndexReader>().0.clone() else {
        return Err("no index cache available".to_string());
    };
    let started = std::time::Instant::now();
    // One folder per lock, so the strip's reads on the same connection wait
    // at most one folder's read.
    let dirs = lock(&index).rank_dirs()?;
    let folders: Vec<Labeled> = dirs
        .into_iter()
        .map(|(dir, finished)| {
            let rows = lock(&index).rank_rows(&dir)?;
            Ok(Labeled {
                flags: rows.iter().map(|r| r.flag).collect(),
                frames: rows.into_iter().map(|r| r.frame).collect(),
                finished,
            })
        })
        .collect::<Result<_, String>>()?;
    let fit = rank::fit(&folders);
    let stored = Stored {
        weights: fit.weights,
        bursts: fit.bursts,
        folders: fit.folders,
        ..Stored::prior()
    };
    if let Some(path) = file(app) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string(&stored).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    log::info!(
        "ranking fit: {} bursts from {} folders ({} read) in {}ms",
        stored.bursts,
        stored.folders,
        folders.len(),
        started.elapsed().as_millis()
    );
    let info = info(&stored);
    *lock(&state.stored) = Some(stored);
    let _ = app.emit("ranking-updated", &info);
    Ok(info)
}

/// The settings' refit button.
#[tauri::command]
pub async fn refit_ranking(app: AppHandle) -> Result<ModelInfo, String> {
    tauri::async_runtime::spawn_blocking(move || refit(&app))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn culling_finished(app: AppHandle, dir: String) -> Result<bool, String> {
    let dir = canonicalize(&dir);
    let Some(index) = app.state::<AppIndexReader>().0.clone() else {
        return Ok(false);
    };
    tauri::async_runtime::spawn_blocking(move || lock(&index).culling_finished(&dir))
        .await
        .map_err(|e| e.to_string())?
}

/// Mark `dir`'s culling finished or not from the folder tree, then refit in
/// the background while AI analysis is on.
#[tauri::command]
pub async fn set_culling_finished(
    app: AppHandle,
    dir: String,
    finished: bool,
) -> Result<(), String> {
    let dir = canonicalize(&dir);
    let Some(index) = app.state::<AppIndex>().0.clone() else {
        return Err("no index cache available".to_string());
    };
    tauri::async_runtime::spawn_blocking(move || {
        lock(&index).set_culling_finished(&dir, finished)?;
        if app.state::<crate::ai::AppAi>().enabled() {
            std::thread::spawn(move || {
                if let Err(e) = refit(&app) {
                    log::error!("ranking fit failed: {e}");
                }
            });
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fit_at_other_versions_or_of_another_size_falls_back_to_the_prior() {
        let fit = Stored {
            weights: vec![0.5; COLUMNS],
            bursts: 120,
            folders: 3,
            ..Stored::prior()
        };
        let json = serde_json::to_string(&fit).unwrap();
        assert_eq!(Stored::parse(&json), fit);
        for stale in [
            Stored {
                rank_version: RANK_VERSION - 1,
                ..fit.clone()
            },
            Stored {
                embed_version: EMBED_VERSION + 1,
                ..fit.clone()
            },
            Stored {
                weights: vec![0.5; COLUMNS - 1],
                ..fit.clone()
            },
        ] {
            let json = serde_json::to_string(&stale).unwrap();
            assert_eq!(Stored::parse(&json), Stored::prior());
        }
        assert_eq!(Stored::parse("not json"), Stored::prior());
    }

    /// The cross-check of `riffle_core::rank` against the plan's
    /// `rank_prior.py`: every folder of `RIFFLE_RANK_OUT/index.sqlite` (the
    /// index `rank_dump_of_real_folders` wrote) scored with the prior's
    /// weights from `rank_rows`, against `RIFFLE_RANK_OUT/prior_scores.tsv`;
    /// then the fit on those folders as if all were marked finished, with
    /// the app's own flags. Run it with `cargo test --release -p riffle-app
    /// the_scores_match_rank_prior_py -- --ignored --nocapture`.
    #[test]
    #[ignore = "needs RIFFLE_RANK_OUT"]
    fn the_scores_match_rank_prior_py() {
        let out = PathBuf::from(std::env::var("RIFFLE_RANK_OUT").expect("set RIFFLE_RANK_OUT"));
        let expected: HashMap<(String, String), f64> =
            std::fs::read_to_string(out.join("prior_scores.tsv"))
                .unwrap()
                .lines()
                .skip(1)
                .map(|line| {
                    let cols: Vec<&str> = line.split('\t').collect();
                    ((cols[0].into(), cols[1].into()), cols[2].parse().unwrap())
                })
                .collect();
        let index = crate::index::Index::open(&out.join("index.sqlite")).unwrap();
        let dirs: Vec<String> = index
            .conn
            .prepare("SELECT dir FROM folders ORDER BY dir")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let (mut compared, mut worst) = (0, 0.0f64);
        let mut folders = Vec::new();
        for dir in &dirs {
            let folder = std::path::Path::new(dir)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let rows = index.rank_rows(dir).unwrap();
            let frames: Vec<rank::Frame> = rows.iter().map(|r| r.frame.clone()).collect();
            let bursts = rank::bursts(&frames);
            for (row, x) in rows.iter().zip(rank::rows(&frames, &bursts)) {
                let (Some(x), Some(want)) =
                    (x, expected.get(&(folder.clone(), row.frame.name.clone())))
                else {
                    continue;
                };
                let got: f64 = x
                    .iter()
                    .zip(rank::prior_weights())
                    .map(|(a, b)| a * b)
                    .sum();
                worst = worst.max((got - want).abs());
                compared += 1;
            }
            folders.push(Labeled {
                flags: rows.iter().map(|r| r.flag).collect(),
                frames,
                finished: true,
            });
        }
        eprintln!(
            "{compared} of {} scores compared, largest difference {worst:.2e}",
            expected.len()
        );
        let fit = rank::fit(&folders);
        let moved = fit
            .weights
            .iter()
            .zip(rank::prior_weights())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();
        let norm = rank::prior_weights()
            .iter()
            .map(|w| w * w)
            .sum::<f64>()
            .sqrt();
        eprintln!(
            "fit on {} bursts from {} folders: weights moved {moved:.3} from the prior's (norm {norm:.3})",
            fit.bursts, fit.folders
        );
        assert_eq!(compared, expected.len());
        assert!(worst < 1e-4, "{worst}");
    }

    #[test]
    fn the_confidence_rises_with_the_bursts_fit_on() {
        assert_eq!(confidence(0), "none");
        assert_eq!(confidence(1), "low");
        assert_eq!(confidence(299), "low");
        assert_eq!(confidence(300), "medium");
        assert_eq!(confidence(1000), "high");
    }
}
