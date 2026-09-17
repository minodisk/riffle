//! The commands the frontend invokes: folder picking, ARW enumeration and
//! preview extraction.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::ipc::Response;
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::index::{self, FileStat, Index, IndexedFile};

/// Size of the header that precedes the JPEG bytes in a `preview` payload.
pub const PREVIEW_HEADER_LEN: usize = 8;

/// Tag identifying the payload kind and version: an IFD0 preview JPEG, v1.
pub const PREVIEW_KIND_JPEG_V1: u16 = 1;

/// Tag for the other payload kind: a cached thumbnail JPEG, v1.
pub const THUMBNAIL_KIND_JPEG_V1: u16 = 2;

/// Build a `preview` payload: a fixed-size little-endian header followed by the
/// embedded JPEG bytes, copied verbatim.
///
/// Header layout (8 bytes, little-endian):
///
/// | offset | size | field                                       |
/// |--------|------|---------------------------------------------|
/// | 0      | 2    | kind/version tag (`PREVIEW_KIND_JPEG_V1` or |
/// |        |      | `THUMBNAIL_KIND_JPEG_V1`)                   |
/// | 2      | 2    | EXIF Orientation (1..8)                     |
/// | 4      | 4    | reserved, zero                              |
///
/// Width and height are not carried: the JPEG itself has them and
/// `createImageBitmap` reports them.
fn payload(kind: u16, orientation: u16, jpeg: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(PREVIEW_HEADER_LEN + jpeg.len());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(&orientation.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(jpeg);
    out
}

fn is_arw(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("arw"))
}

/// List the ARW files directly in `dir`, sorted by file name. Entries that
/// cannot be read are skipped; a directory that cannot be read is an error.
fn list_arw_in(dir: &Path) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_arw(p))
        .collect();
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(files
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

/// Extract a file's IFD0 preview JPEG along with the Orientation, reading only
/// the bounded prefix the metadata and the preview need rather than the whole
/// 48 MB file.
fn read_preview(path: &Path) -> Result<(u16, Vec<u8>), String> {
    let (arw, jpeg) =
        riffle_core::reader::read_preview(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((arw.orientation, jpeg))
}

/// Open the native folder picker and resolve once the user answers, or `None`
/// if they cancel.
///
/// The dialog is driven by the main thread's run loop, so the blocking variant
/// must not be used: a synchronous command runs on the main thread, and waiting
/// there freezes the very loop that delivers the dialog's button events. The
/// callback form hands the answer back over a channel this async command awaits
/// instead.
#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.try_send(path);
    });
    rx.recv().await.flatten().map(|p| p.to_string())
}

#[tauri::command]
pub fn list_arw(dir: String) -> Result<Vec<String>, String> {
    list_arw_in(Path::new(&canonicalize(&dir)))
}

/// The folder a dropped path stands for: a directory is taken as it is, a
/// file is taken by its parent directory (dragging one ARW is the obvious
/// gesture), and anything else — a path that is gone by the time it lands, or
/// a file at a filesystem root — is `None`.
fn dropped_dir(path: &Path) -> Option<PathBuf> {
    if path.is_dir() {
        return Some(path.to_path_buf());
    }
    if path.is_file() {
        return path.parent().map(Path::to_path_buf);
    }
    None
}

/// Resolve a path dropped on the window to the folder to open. Whether the
/// path is a directory can only be answered by a `stat`, so the frontend asks
/// instead of guessing from the string.
#[tauri::command]
pub async fn dropped_folder(path: String) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        dropped_dir(Path::new(&path)).map(|p| p.to_string_lossy().into_owned())
    })
    .await
    .ok()
    .flatten()
}

#[tauri::command]
pub async fn preview(path: String) -> Result<Response, String> {
    let bytes = tauri::async_runtime::spawn_blocking(move || read_preview(Path::new(&path)))
        .await
        .map_err(|e| e.to_string())??;
    Ok(Response::new(payload(
        PREVIEW_KIND_JPEG_V1,
        bytes.0,
        &bytes.1,
    )))
}

/// The id handed out to each `scan_folder` call, the cancel flag and join
/// handle of the one scan that may be running, and the work queued for
/// `start_scan` to actually spawn — all behind one lock so that minting an
/// id, taking `running`, checking the latest id and storing into `running`
/// or `pending` are never interleaved between two concurrent `scan_folder`/
/// `start_scan` calls (the two `scan_folder` futures are independent IPC
/// tasks with no ordering guarantee between them, and likewise for
/// `start_scan`).
#[derive(Default)]
pub struct Scans(Mutex<ScansState>);

#[derive(Default)]
struct ScansState {
    next_id: u64,
    /// The id `scan_folder` most recently handed out. Both `scan_folder`
    /// (before inserting into `pending`) and `start_scan` (before storing
    /// into `running`) re-check against this so an id that has since been
    /// superseded is refused rather than acted on late.
    latest_id: u64,
    running: Option<(Arc<AtomicBool>, tauri::async_runtime::JoinHandle<()>)>,
    pending: HashMap<u64, PendingScan>,
}

impl ScansState {
    fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
}

struct PendingScan {
    dir: String,
    todo: Vec<FileStat>,
    cancel: Arc<AtomicBool>,
}

/// The index, shared between the commands and the scan thread. It is a cache:
/// `None` when the cache directory could not be opened (see `main.rs`), in
/// which case the commands below degrade to "no thumbnails" instead of the
/// app failing to launch.
pub struct AppIndex(pub Option<Arc<Mutex<Index>>>);

/// Threads the scan runs on: two fewer than the cores. Step 3 measured that
/// this costs ~10% of scan throughput against using every core, and leaves two
/// cores for the paging path so the app stays responsive while scanning; more
/// threads than cores did not help and doubled the per-file p95.
fn scan_threads() -> usize {
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    cores.saturating_sub(2).max(1)
}

/// What `scan_folder` returns: the number of files that need scanning and the
/// id the caller must match against `scan-progress`/`scan-done` events to tell
/// this scan's events apart from an older, still-draining one for the same
/// folder.
#[derive(serde::Serialize)]
pub struct ScanStarted {
    total: usize,
    scan_id: u64,
}

/// Bring the index of `dir` up to date: wait for a previous scan's last write
/// to land, then drop the rows of files that are gone or changed and report
/// how many have no valid row. The actual scan does not start until the
/// frontend calls `start_scan` with the returned `scan_id`, so this only
/// prepares the work; call `start_scan` right after storing the id. A no-op
/// (nothing to scan, no `scan-done` either) when the index cache is
/// unavailable.
#[tauri::command]
pub async fn scan_folder(app: tauri::AppHandle, dir: String) -> Result<ScanStarted, String> {
    let scans = app.state::<Scans>();
    let (scan_id, previous) = {
        let mut state = index::lock(&scans.0);
        let scan_id = state.next_id();
        state.latest_id = scan_id;
        (scan_id, state.running.take())
    };
    let dir = canonicalize(&dir);
    let cancel = Arc::new(AtomicBool::new(false));

    if let Some((previous_cancel, previous_handle)) = previous {
        previous_cancel.store(true, Ordering::Relaxed);
        let _ = previous_handle.await;
    }

    let Some(index) = app.state::<AppIndex>().0.clone() else {
        return Ok(ScanStarted { total: 0, scan_id });
    };

    let listed = {
        let dir = dir.clone();
        tauri::async_runtime::spawn_blocking(move || list_arw_in(Path::new(&dir)))
            .await
            .map_err(|e| e.to_string())??
    };

    let todo = {
        let (dir, index) = (dir.clone(), index);
        tauri::async_runtime::spawn_blocking(move || {
            let files: Vec<_> = listed
                .iter()
                .filter_map(|p| index::stat(Path::new(p)).ok())
                .collect();
            index::lock(&index).reconcile(&dir, &files)
        })
        .await
        .map_err(|e| e.to_string())??
    };

    let total = todo.len();
    let mut state = index::lock(&scans.0);
    if state.latest_id != scan_id {
        // A later `scan_folder` has superseded this one while it was
        // listing/reconciling; its own `start_scan` (or none at all) owns
        // `pending`/`running` now, so queuing this work would either be
        // overwritten or, worse, race the id check below in `start_scan`.
        return Ok(ScanStarted { total, scan_id });
    }
    // Any entry still here belongs to a scan this one has already superseded
    // (only one `scan_folder`/`start_scan` pair is ever live at a time); it
    // never started, so there is nothing to cancel or join, just drop it.
    state.pending.clear();
    state
        .pending
        .insert(scan_id, PendingScan { dir, todo, cancel });
    drop(state);
    Ok(ScanStarted { total, scan_id })
}

/// Start the scan `scan_folder` prepared for `scan_id`, in the background.
/// A no-op if there is no pending work under that id (the index cache was
/// unavailable, or this scan has since been superseded).
///
/// The latest-id check and the store into `running` happen under the same
/// lock as `scan_folder`'s own id-minting and `running`-taking, so a
/// `scan_folder` that supersedes this id can never interleave between the
/// check and the store: either it runs first, in which case this call sees
/// its own id is stale and does not spawn at all, or it runs after, in which
/// case it takes the handle this call just stored and joins it before
/// reconciling.
#[tauri::command]
pub fn start_scan(app: tauri::AppHandle, scan_id: u64) -> Result<(), String> {
    let scans = app.state::<Scans>();
    let mut state = index::lock(&scans.0);
    if state.latest_id != scan_id {
        state.pending.remove(&scan_id);
        return Ok(());
    }
    let Some(pending) = state.pending.remove(&scan_id) else {
        return Ok(());
    };
    let Some(index) = app.state::<AppIndex>().0.clone() else {
        return Ok(());
    };
    let PendingScan { dir, todo, cancel } = pending;

    let handle = tauri::async_runtime::spawn_blocking({
        let cancel = cancel.clone();
        let app = app.clone();
        move || {
            let summary = index::run_scan(
                &index,
                &dir,
                &todo,
                scan_threads(),
                &cancel,
                |done, total| {
                    let _ = app.emit(
                        "scan-progress",
                        Progress {
                            dir: &dir,
                            scan_id,
                            done,
                            total,
                        },
                    );
                },
            );
            let _ = app.emit(
                "scan-done",
                Done {
                    dir: &dir,
                    scan_id,
                    total: summary.total,
                    errors: summary.errors,
                },
            );
        }
    });
    state.running = Some((cancel, handle));
    Ok(())
}

#[derive(serde::Serialize, Clone)]
struct Progress<'a> {
    dir: &'a str,
    scan_id: u64,
    done: usize,
    total: usize,
}

#[derive(serde::Serialize, Clone)]
struct Done<'a> {
    dir: &'a str,
    scan_id: u64,
    total: usize,
    errors: usize,
}

/// The indexed rows of `dir`, in the same order as `list_arw`. Empty when the
/// index cache is unavailable.
#[tauri::command]
pub async fn folder_entries(
    app: tauri::AppHandle,
    dir: String,
) -> Result<Vec<IndexedFile>, String> {
    let dir = canonicalize(&dir);
    let Some(index) = app.state::<AppIndex>().0.clone() else {
        return Ok(Vec::new());
    };
    tauri::async_runtime::spawn_blocking(move || index::lock(&index).entries(&dir))
        .await
        .map_err(|e| e.to_string())?
}

/// The cached thumbnail of one file, in the same envelope as `preview`.
#[tauri::command]
pub async fn thumbnail(app: tauri::AppHandle, path: String) -> Result<Response, String> {
    let Some(index) = app.state::<AppIndex>().0.clone() else {
        return Err("no index cache available".to_string());
    };
    let (orientation, jpeg) =
        tauri::async_runtime::spawn_blocking(move || index::lock(&index).thumbnail(&path))
            .await
            .map_err(|e| e.to_string())??;
    Ok(Response::new(payload(
        THUMBNAIL_KIND_JPEG_V1,
        orientation,
        &jpeg,
    )))
}

/// Resolve symlinks and normalize a folder path so the same folder reached
/// through different spellings (a trailing separator, a symlinked parent,
/// `/tmp` vs `/private/tmp` on macOS) shares one row set in the index. Falls
/// back to the original string when canonicalization fails (e.g. the folder
/// was removed between picking and scanning).
fn canonicalize(dir: &str) -> String {
    std::fs::canonicalize(dir)
        .map_or_else(|_| dir.to_string(), |p| p.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("riffle-app-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn lists_only_arw_files_sorted_by_name() {
        let dir = temp_dir("list");
        for name in ["b.ARW", "a.arw", "c.Arw", "d.jpg", "e.arw.txt"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        std::fs::create_dir(dir.join("sub.arw")).unwrap();

        let files = list_arw_in(&dir).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|p| {
                Path::new(p)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(names, ["a.arw", "b.ARW", "c.Arw"]);
        assert!(files.iter().all(|p| Path::new(p).is_absolute()));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unreadable_directory_is_an_error() {
        let missing = std::env::temp_dir().join("riffle-app-does-not-exist");
        assert!(list_arw_in(&missing).is_err());
    }

    #[test]
    fn payload_header_encodes_kind_and_orientation() {
        let jpeg = [0xff, 0xd8, 0xff, 0xd9];
        let out = payload(PREVIEW_KIND_JPEG_V1, 8, &jpeg);
        assert_eq!(out.len(), PREVIEW_HEADER_LEN + jpeg.len());
        assert_eq!(u16::from_le_bytes([out[0], out[1]]), PREVIEW_KIND_JPEG_V1);
        assert_eq!(u16::from_le_bytes([out[2], out[3]]), 8);
        assert_eq!(u32::from_le_bytes([out[4], out[5], out[6], out[7]]), 0);
        assert_eq!(&out[PREVIEW_HEADER_LEN..], &jpeg);
    }

    #[test]
    fn thumbnail_payload_header_encodes_kind_two_and_orientation() {
        let jpeg = [0xff, 0xd8, 0xff, 0xd9];
        let out = payload(THUMBNAIL_KIND_JPEG_V1, 6, &jpeg);
        assert_eq!(u16::from_le_bytes([out[0], out[1]]), THUMBNAIL_KIND_JPEG_V1);
        assert_eq!(u16::from_le_bytes([out[2], out[3]]), 6);
        assert_eq!(&out[PREVIEW_HEADER_LEN..], &jpeg);
    }

    #[test]
    fn dropped_directory_is_taken_as_is_and_a_file_by_its_parent() {
        let dir = temp_dir("dropped");
        let file = dir.join("a.arw");
        std::fs::write(&file, b"x").unwrap();

        assert_eq!(dropped_dir(&dir), Some(dir.clone()));
        assert_eq!(dropped_dir(&file), Some(dir.clone()));
        assert_eq!(dropped_dir(&dir.join("gone.arw")), None);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_preview_is_an_error() {
        let dir = temp_dir("preview");
        let path = dir.join("broken.arw");
        std::fs::write(&path, b"II\x2a\x00\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00").unwrap();
        assert!(read_preview(&path).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
