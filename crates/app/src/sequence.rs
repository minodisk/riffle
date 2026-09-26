//! `File > Sequence JPEG Timestamps…`: the commands that preview and run
//! `riffle_core::sequence` on a folder of exported JPEGs, and the state of the
//! one run that may be in progress.
//!
//! JPEGs are not indexed, so nothing here touches the index, and a run is not
//! refused while a scan is in progress. The output is written to a sibling of
//! the picked folder, so even when that folder is the open RAW folder the
//! watcher, which watches it non-recursively, sees no `folder-changed` for it.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::index;

/// The id of the last run handed out and the cancel flag and join handle of
/// the one run that may be in progress, behind one lock so that the "already
/// running" check and the store are never interleaved between two
/// `sequence_run` calls.
#[derive(Default)]
pub struct Sequences(Mutex<SequenceState>);

#[derive(Default)]
struct SequenceState {
    next_id: u64,
    /// The run in progress, cleared by its own task when it ends.
    running: Option<(u64, Arc<AtomicBool>, tauri::async_runtime::JoinHandle<()>)>,
}

impl SequenceState {
    fn finish(&mut self, run_id: u64) {
        if self
            .running
            .as_ref()
            .is_some_and(|(id, _, _)| *id == run_id)
        {
            self.running = None;
        }
    }
}

const RUN_IN_PROGRESS: &str = "a sequence run is already in progress";

/// Mint a run id and store the task `spawn` starts for it, or refuse while
/// another run is in progress. The lock is held across `spawn`, so the task
/// cannot clear its entry before it is stored.
fn begin(
    sequences: &Sequences,
    spawn: impl FnOnce(u64, Arc<AtomicBool>) -> tauri::async_runtime::JoinHandle<()>,
) -> Result<u64, String> {
    let mut state = index::lock(&sequences.0);
    if state.running.is_some() {
        return Err(RUN_IN_PROGRESS.to_string());
    }
    state.next_id += 1;
    let run_id = state.next_id;
    let cancel = Arc::new(AtomicBool::new(false));
    let handle = spawn(run_id, cancel.clone());
    state.running = Some((run_id, cancel, handle));
    Ok(run_id)
}

/// One file that could not be read or written.
#[derive(Debug, Clone, Serialize)]
pub struct Failure {
    pub path: String,
    pub message: String,
}

/// One file of a preview, in the computed order.
#[derive(Debug, Serialize)]
pub struct Row {
    pub path: String,
    pub old: String,
    pub new: String,
    pub changed: bool,
}

/// What `sequence_preview` returns.
#[derive(Debug, Serialize)]
pub struct Preview {
    pub output_dir: String,
    pub output_exists: bool,
    pub rows: Vec<Row>,
    pub failed: Vec<Failure>,
}

fn failure(path: &Path, message: &str) -> Failure {
    Failure {
        path: path.to_string_lossy().into_owned(),
        message: message.to_string(),
    }
}

fn preview(dir: &Path) -> Result<Preview, String> {
    let out = riffle_core::sequence::output_dir(dir)?;
    let summary = riffle_core::sequence::run(dir, true, &AtomicBool::new(false), |_, _, _, _| {})?;
    let mut rows = Vec::new();
    let mut failed = Vec::new();
    for (path, result) in &summary.results {
        match result {
            Ok(outcome) => rows.push(Row {
                path: path.to_string_lossy().into_owned(),
                old: outcome.old.to_string(),
                new: outcome.new.to_string(),
                changed: outcome.changed,
            }),
            Err(message) => failed.push(failure(path, message)),
        }
    }
    Ok(Preview {
        output_dir: out.to_string_lossy().into_owned(),
        output_exists: out.exists(),
        rows,
        failed,
    })
}

/// The times a run on `dir` would write, without touching the disk.
#[tauri::command]
pub async fn sequence_preview(dir: String) -> Result<Preview, String> {
    tauri::async_runtime::spawn_blocking(move || preview(Path::new(&dir)))
        .await
        .map_err(|e| e.to_string())?
}

#[derive(Serialize, Clone)]
struct Progress<'a> {
    run_id: u64,
    dir: &'a str,
    done: usize,
    total: usize,
}

#[derive(Serialize, Clone)]
struct Done<'a> {
    run_id: u64,
    dir: &'a str,
    output_dir: String,
    written: usize,
    total: usize,
    failed: Vec<Failure>,
    canceled: bool,
}

/// Write the sequenced copy of `dir` in the background and return the run's
/// id. Streams `sequence-progress` and ends with one `sequence-done`, which
/// follows every progress event of the run because `run` only returns after
/// its last progress callback. A folder-level error (no JPEGs, an output
/// folder that cannot be rebuilt) arrives as `sequence-done`'s one failure,
/// keyed by `dir`.
#[tauri::command]
pub async fn sequence_run(app: tauri::AppHandle, dir: String) -> Result<u64, String> {
    let sequences = app.state::<Sequences>();
    begin(&sequences, |run_id, cancel| {
        let app = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let path = Path::new(&dir);
            let output_dir = riffle_core::sequence::output_dir(path)
                .map(|out| out.to_string_lossy().into_owned())
                .unwrap_or_default();
            let summary = riffle_core::sequence::run(path, false, &cancel, |done, total, _, _| {
                let _ = app.emit(
                    "sequence-progress",
                    Progress {
                        run_id,
                        dir: &dir,
                        done,
                        total,
                    },
                );
            });
            let done = match summary {
                Ok(summary) => {
                    let failed: Vec<Failure> = summary
                        .results
                        .iter()
                        .filter_map(|(path, result)| {
                            result.as_ref().err().map(|message| failure(path, message))
                        })
                        .collect();
                    Done {
                        run_id,
                        dir: &dir,
                        output_dir,
                        written: summary.results.len() - failed.len(),
                        total: summary.total,
                        failed,
                        canceled: summary.canceled,
                    }
                }
                Err(message) => Done {
                    run_id,
                    dir: &dir,
                    output_dir,
                    written: 0,
                    total: 0,
                    failed: vec![failure(path, &message)],
                    canceled: false,
                },
            };
            // Cleared before `sequence-done`, so a run started in answer to
            // it is not refused.
            index::lock(&app.state::<Sequences>().0).finish(run_id);
            let _ = app.emit("sequence-done", done);
        })
    })
}

/// Ask the run `run_id` to stop before its next file; a finished or unknown
/// run is ignored.
#[tauri::command]
pub async fn sequence_cancel(app: tauri::AppHandle, run_id: u64) {
    if let Some((id, cancel, _)) = &index::lock(&app.state::<Sequences>().0).running {
        if *id == run_id {
            cancel.store(true, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{begin, preview, Sequences, RUN_IN_PROGRESS};
    use crate::index;
    use std::path::{Path, PathBuf};
    use std::sync::{mpsc, Arc};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("riffle-sequence-{name}-{}", std::process::id()))
            .join("export");
        let _ = std::fs::remove_dir_all(dir.parent().unwrap());
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A JPEG header holding only an Exif APP1 with `DateTimeOriginal`, which
    /// is all a dry run reads.
    fn jpeg(datetime: &str) -> Vec<u8> {
        let mut tiff = b"II".to_vec();
        tiff.extend_from_slice(&42u16.to_le_bytes());
        tiff.extend_from_slice(&8u32.to_le_bytes());
        // IFD0 at 8: the Exif IFD pointer, pointing at 26.
        tiff.extend_from_slice(&1u16.to_le_bytes());
        tiff.extend_from_slice(&0x8769u16.to_le_bytes());
        tiff.extend_from_slice(&4u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&26u32.to_le_bytes());
        tiff.extend_from_slice(&0u32.to_le_bytes());
        // Exif IFD at 26: DateTimeOriginal, its value at 44.
        tiff.extend_from_slice(&1u16.to_le_bytes());
        tiff.extend_from_slice(&0x9003u16.to_le_bytes());
        tiff.extend_from_slice(&2u16.to_le_bytes());
        tiff.extend_from_slice(&20u32.to_le_bytes());
        tiff.extend_from_slice(&44u32.to_le_bytes());
        tiff.extend_from_slice(&0u32.to_le_bytes());
        tiff.extend_from_slice(datetime.as_bytes());
        tiff.push(0);
        let mut payload = b"Exif\0\0".to_vec();
        payload.extend_from_slice(&tiff);
        let mut out = vec![0xFF, 0xD8, 0xFF, 0xE1];
        out.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(&payload);
        out.extend_from_slice(&[0xFF, 0xD9]);
        out
    }

    fn name(path: &str) -> String {
        Path::new(path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn a_preview_lists_the_files_in_time_order() {
        let dir = temp_dir("preview");
        std::fs::write(dir.join("DSC00002.JPG"), jpeg("2024:01:02 03:04:59")).unwrap();
        std::fs::write(dir.join("L1000001.jpg"), jpeg("2024:01:02 03:04:58")).unwrap();
        std::fs::write(dir.join("DSC00001.jpg"), jpeg("2024:01:02 03:04:58")).unwrap();
        std::fs::write(dir.join("broken.jpg"), b"not a jpeg").unwrap();

        let preview = preview(&dir).unwrap();

        let rows: Vec<(String, &str, &str, bool)> = preview
            .rows
            .iter()
            .map(|r| (name(&r.path), r.old.as_str(), r.new.as_str(), r.changed))
            .collect();
        assert_eq!(
            rows,
            [
                (
                    "DSC00001.jpg".to_string(),
                    "2024:01:02 03:04:58",
                    "2024:01:02 03:04:58",
                    false
                ),
                (
                    "L1000001.jpg".to_string(),
                    "2024:01:02 03:04:58",
                    "2024:01:02 03:04:59",
                    true
                ),
                (
                    "DSC00002.JPG".to_string(),
                    "2024:01:02 03:04:59",
                    "2024:01:02 03:05:00",
                    true
                ),
            ]
        );
        assert_eq!(preview.failed.len(), 1);
        assert_eq!(name(&preview.failed[0].path), "broken.jpg");
        let out = dir.with_file_name("export-sequenced");
        assert_eq!(Path::new(&preview.output_dir), out);
        assert!(!preview.output_exists);
        assert!(!out.exists());

        std::fs::create_dir(&out).unwrap();
        assert!(super::preview(&dir).unwrap().output_exists);
    }

    #[test]
    fn a_preview_of_a_folder_without_jpegs_is_an_error() {
        let dir = temp_dir("empty");
        assert!(preview(&dir).is_err());
    }

    #[test]
    fn a_second_run_is_refused_until_the_first_one_ends() {
        let sequences = Arc::new(Sequences::default());
        let (go, rx) = mpsc::channel::<()>();
        let first = begin(&sequences, |run_id, _| {
            let sequences = sequences.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let _ = rx.recv();
                index::lock(&sequences.0).finish(run_id);
            })
        })
        .unwrap();

        let refused = begin(&sequences, |_, _| unreachable!());
        assert_eq!(refused, Err(RUN_IN_PROGRESS.to_string()));

        let _ = go.send(());
        let waited = tauri::async_runtime::spawn_blocking({
            let sequences = sequences.clone();
            move || {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
                while index::lock(&sequences.0).running.is_some()
                    && std::time::Instant::now() < deadline
                {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        });
        tauri::async_runtime::block_on(waited).unwrap();
        assert!(index::lock(&sequences.0).running.is_none());

        let second = begin(&sequences, |_, _| {
            tauri::async_runtime::spawn_blocking(|| {})
        });
        assert_eq!(second, Ok(first + 1));
    }
}
