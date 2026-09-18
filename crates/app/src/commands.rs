//! The commands the frontend invokes: folder picking, ARW enumeration and
//! preview extraction.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::ipc::Response;
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;

use crate::index::{self, FileStat, Index, IndexedFile, SidecarStat};
use crate::sidecar::{SidecarFormat, Writer};

/// Size of the header that precedes the JPEG bytes in a `preview` payload.
pub const PREVIEW_HEADER_LEN: usize = 8;

/// Tag identifying the payload kind and version: an IFD0 preview JPEG, v1.
pub const PREVIEW_KIND_JPEG_V1: u16 = 1;

/// Tag for the other payload kind: a cached thumbnail JPEG, v1.
pub const THUMBNAIL_KIND_JPEG_V1: u16 = 2;

/// Size of the header that precedes the pixels in a `focus_crop` payload.
pub const CROP_HEADER_LEN: usize = 32;

/// Tag for the third payload kind: a raw RGBA focus crop, v2. v1 was the same
/// pixels behind a 24-byte header without the timings.
pub const CROP_KIND_RGBA_V2: u16 = 4;

/// The largest crop produced per axis, in JPEG pixels. The payload is raw
/// RGBA, so this caps it at 4 MB; the decode itself barely depends on the
/// size.
pub const CROP_MAX: usize = 1024;

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

/// Build a `focus_crop` payload: a fixed-size little-endian header followed by
/// the crop's RGBA bytes.
///
/// Header layout (32 bytes, little-endian):
///
/// | offset | size | field                                     |
/// |--------|------|-------------------------------------------|
/// | 0      | 2    | kind/version tag (`CROP_KIND_RGBA_V2`)    |
/// | 2      | 2    | EXIF Orientation (1..8)                   |
/// | 4      | 4    | crop width in pixels                      |
/// | 8      | 4    | crop height in pixels                     |
/// | 12     | 4    | point of interest x, in crop pixels       |
/// | 16     | 4    | point of interest y, in crop pixels       |
/// | 20     | 4    | ranged read of the JpgFromRaw, in us      |
/// | 24     | 4    | partial decode, in us                     |
/// | 28     | 4    | reserved, zero                            |
///
/// The crop is cut in unrotated JPEG coordinates; the Orientation is the one
/// the frontend already applies to the preview.
fn crop_payload(
    orientation: u16,
    crop: &riffle_core::partial::FocusCrop,
    timing: CropTiming,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(CROP_HEADER_LEN + crop.crop.pixels.len());
    out.extend_from_slice(&CROP_KIND_RGBA_V2.to_le_bytes());
    out.extend_from_slice(&orientation.to_le_bytes());
    for value in [
        crop.crop.width,
        crop.crop.height,
        crop.point_x,
        crop.point_y,
    ] {
        out.extend_from_slice(&(value as u32).to_le_bytes());
    }
    out.extend_from_slice(&timing.read_us.to_le_bytes());
    out.extend_from_slice(&timing.decode_us.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&crop.crop.pixels);
    out
}

/// The crop size in unrotated JPEG pixels for a viewport of `width` x `height`
/// device pixels: each axis is capped, and the two are swapped for the
/// quarter-turn orientations, where the viewport's width runs along the
/// unrotated JPEG's height.
fn crop_size(orientation: u16, width: usize, height: usize) -> (usize, usize) {
    let w = width.clamp(1, CROP_MAX);
    let h = height.clamp(1, CROP_MAX);
    if matches!(orientation, 6 | 8) {
        (h, w)
    } else {
        (w, h)
    }
}

/// How long the two phases of one `focus_crop` took, in microseconds.
#[derive(Clone, Copy)]
pub struct CropTiming {
    read_us: u32,
    decode_us: u32,
}

/// Cut the 1:1 crop around the focus point out of a file's full-resolution
/// JpgFromRaw, reading it by range rather than loading the whole ARW. The
/// ranged read and the partial decode are timed separately, so that an
/// end-to-end measurement can be split into its parts.
fn read_focus_crop(
    path: &Path,
    width: usize,
    height: usize,
) -> Result<(u16, riffle_core::partial::FocusCrop, CropTiming), String> {
    let started = std::time::Instant::now();
    let (arw, jpeg) =
        riffle_core::reader::read_full(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let read = started.elapsed();
    let (w, h) = crop_size(arw.orientation, width, height);
    let decode_started = std::time::Instant::now();
    let crop = riffle_core::partial::decode_focus_crop(&jpeg, arw.shot.focus, w, h)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let timing = CropTiming {
        read_us: read.as_micros() as u32,
        decode_us: decode_started.elapsed().as_micros() as u32,
    };
    Ok((arw.orientation, crop, timing))
}

/// List the RAW (ARW and DNG) files directly in `dir`, sorted by file name. Entries that
/// cannot be read are skipped; a directory that cannot be read is an error.
fn list_arw_in(dir: &Path) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && riffle_core::scan::is_raw_file(p))
        .collect();
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(files
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

/// Largest sidecar the folder-open pass reads. A Lightroom sidecar is tens of
/// KB; anything past this is not a sidecar this app should be parsing, and is
/// left alone rather than failing the open.
const MAX_SIDECAR_BYTES: i64 = 4 * 1024 * 1024;

/// The sidecars of `format` directly in `dir`, keyed by lower-cased file name
/// so that a `FOO.XMP` written by another tool is found for `FOO.ARW`.
///
/// One listing of the directory, rather than a `stat` of 5000 guessed names:
/// the second open of a folder is meant to cost no more than the listing the
/// ARWs already pay for.
fn list_sidecars_in(dir: &Path, format: SidecarFormat) -> HashMap<String, SidecarStat> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return HashMap::new();
    };
    entries
        .flatten()
        .filter(|e| format.matches(&e.file_name().to_string_lossy()))
        .filter_map(|e| {
            let stat = index::stat(&e.path()).ok()?;
            let name = e.file_name().to_string_lossy().to_lowercase();
            Some((name, (stat.path, stat.size, stat.mtime_ns)))
        })
        .collect()
}

/// Bring the `ratings` rows of `dir` in line with the sidecars on disk and
/// return the judgements that still have to be written.
///
/// The sidecar is the source of truth, so anything whose stat changed since
/// the app last saw it is read back here; a sidecar that cannot be read or
/// parsed leaves its row as it was rather than erroring the open.
fn reconcile_sidecars_of(
    dir: &str,
    listed: &[String],
    index: &Arc<Mutex<Index>>,
    format: SidecarFormat,
) -> Result<Vec<(String, Option<i8>)>, String> {
    let sidecars = list_sidecars_in(Path::new(dir), format);
    let pairs: Vec<(String, Option<SidecarStat>)> = listed
        .iter()
        .map(|path| {
            let name = format
                .sidecar_path(Path::new(path))
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase());
            let stat = name.and_then(|n| sidecars.get(&n)).cloned();
            (path.clone(), stat)
        })
        .collect();

    let to_parse = index::lock(index).reconcile_sidecars(dir, &pairs)?;
    // An oversize sidecar is rejected outright (see `MAX_SIDECAR_BYTES`), so
    // its row cannot be brought in line with it here. Its path is tracked
    // separately so it can be kept out of what the writer is handed below:
    // otherwise a dirty row for it would have the writer patch a sidecar
    // whose external edit was never read, overwriting it unread.
    let oversize: std::collections::HashSet<&str> = to_parse
        .iter()
        .filter(|(_, (_, size, _), _)| *size > MAX_SIDECAR_BYTES)
        .map(|(path, _, _)| path.as_str())
        .collect();
    let parsed: Vec<(String, Option<i8>, i64, i64, bool)> = to_parse
        .iter()
        .filter(|(path, _, _)| !oversize.contains(path.as_str()))
        .filter_map(|(path, (sidecar, size, mtime_ns), dirty)| {
            let bytes = std::fs::read(sidecar).ok()?;
            let rating = format.read_rating(&bytes).ok()?;
            Some((path.clone(), rating, *size, *mtime_ns, *dirty))
        })
        .collect();
    let mut index = index::lock(index);
    index.store_sidecar_ratings(dir, &parsed)?;
    // After storing, so a row the sidecar just won stays out of this: only
    // what still has nowhere to be read back from is written out. An
    // oversize sidecar's row is excluded too, even if still dirty, since the
    // writer must not patch a sidecar whose contents were never read.
    let dirty = index.dirty_rows(dir)?;
    Ok(dirty
        .into_iter()
        .filter(|(path, _)| !oversize.contains(path.as_str()))
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

/// The settings store. Settings are preferences, not a cache, so the file
/// lives in the config dir rather than beside the index.
fn settings(app: &tauri::AppHandle) -> Result<Arc<tauri_plugin_store::Store<tauri::Wry>>, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    app.store(dir.join("settings.json"))
        .map_err(|e| e.to_string())
}

/// Read and remove the plain file the last folder was kept in before the
/// settings store existed. `None` when there is none.
fn take_legacy_last_folder(file: &Path) -> Option<String> {
    let dir = std::fs::read_to_string(file).ok()?;
    if let Err(e) = std::fs::remove_file(file) {
        eprintln!("failed to remove {}: {e}", file.display());
    }
    Some(dir)
}

/// Load the settings at launch: move a legacy `last_folder` file into the
/// store once, then return the selected sidecar format. A store that cannot
/// be read is logged and falls back to the defaults.
pub fn load_settings(app: &tauri::AppHandle) -> SidecarFormat {
    let store = match settings(app) {
        Ok(store) => store,
        Err(e) => {
            eprintln!("failed to open the settings: {e}");
            return SidecarFormat::default();
        }
    };
    if !store.has("lastFolder") {
        let legacy = app
            .path()
            .app_config_dir()
            .ok()
            .and_then(|dir| take_legacy_last_folder(&dir.join("last_folder")));
        if let Some(dir) = legacy {
            store.set("lastFolder", dir);
            if let Err(e) = store.save() {
                eprintln!("failed to save the settings: {e}");
            }
        }
    }
    SidecarFormat::from_setting(store.get("sidecarFormat").as_ref().and_then(|v| v.as_str()))
}

/// Remember `dir` as the folder to reopen on the next launch. Failing to write
/// it only means starting with nothing open, so it is logged, not returned.
#[tauri::command]
pub fn remember_folder(app: tauri::AppHandle, dir: String) {
    let saved = settings(&app).and_then(|store| {
        store.set("lastFolder", dir);
        store.save().map_err(|e| e.to_string())
    });
    if let Err(e) = saved {
        eprintln!("failed to remember the folder: {e}");
    }
}

/// Open `dir` in the newest DxO PhotoLab installed under /Applications, the
/// hand-off from culling to developing. PhotoLab's bundle name carries its
/// major version (`DXOPhotoLab10.app`), so it is looked up rather than named.
#[tauri::command]
pub fn open_in_photolab(app: tauri::AppHandle, dir: String) -> Result<(), String> {
    let names = std::fs::read_dir("/Applications")
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok());
    let photolab = newest_photolab(names).ok_or("DxO PhotoLab is not installed")?;
    app.opener()
        .open_path(dir, Some(format!("/Applications/{photolab}")))
        .map_err(|e| e.to_string())
}

/// The `DXOPhotoLab<N>.app` name with the highest `N` among `names`.
fn newest_photolab(names: impl Iterator<Item = String>) -> Option<String> {
    names
        .filter_map(|name| {
            let version = name
                .strip_prefix("DXOPhotoLab")?
                .strip_suffix(".app")?
                .parse::<u32>()
                .ok()?;
            Some((version, name))
        })
        .max()
        .map(|(_, name)| name)
}

/// The folder remembered by `remember_folder`, or `None` if there is none or
/// it is no longer a directory.
#[tauri::command]
pub fn last_folder(app: tauri::AppHandle) -> Option<String> {
    let dir = settings(&app)
        .ok()?
        .get("lastFolder")?
        .as_str()?
        .to_string();
    Path::new(&dir).is_dir().then_some(dir)
}

#[tauri::command]
pub fn list_arw(dir: String) -> Result<Vec<String>, String> {
    list_arw_in(Path::new(&canonicalize(&dir)))
}

/// The folder a dropped path stands for: a directory is taken as it is, a
/// file is taken by its parent directory (dragging one ARW is the obvious
/// gesture; a file at a filesystem root resolves to that root, since
/// `Path::parent` only yields `None` for the root itself). Anything else — a
/// path that is neither a directory nor a regular file, such as one gone by
/// the time it lands, or a broken symlink or socket — is `None`.
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
pub async fn dropped_folder(path: String) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        dropped_dir(Path::new(&path)).map(|p| p.to_string_lossy().into_owned())
    })
    .await
    .map_err(|e| e.to_string())
}

/// The metadata shown in the right pane: the file name plus the shooting
/// settings, already formatted for display so the frontend only lays them out.
/// Fields the file does not carry are `None` and are left out of the pane.
#[derive(serde::Serialize)]
pub struct Metadata {
    name: String,
    camera: Option<String>,
    lens: Option<String>,
    aperture: Option<String>,
    shutter: Option<String>,
    iso: Option<String>,
    focal_length: Option<String>,
    exposure_bias: Option<String>,
    focus_distance: Option<String>,
    captured_at: Option<String>,
}

/// Format a rational as a decimal with at most `places` digits, with trailing
/// zeros dropped (2.80 -> "2.8", 50.0 -> "50").
fn decimal(r: riffle_core::arw::Rational, places: usize) -> Option<String> {
    let v = r.value()?;
    let text = format!("{v:.places$}");
    let text = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.')
    } else {
        text.as_str()
    };
    Some(text.to_string())
}

/// Shutter speed the way a camera shows it: `1/250` below a second, `1.3"`
/// at or above one.
fn shutter(r: riffle_core::arw::Rational) -> Option<String> {
    let v = r.value()?;
    if v <= 0.0 {
        return None;
    }
    if v >= 1.0 {
        return decimal(r, 1).map(|t| format!("{t}\""));
    }
    Some(format!("1/{}", (1.0 / v).round()))
}

fn read_metadata(path: &Path) -> Result<Metadata, String> {
    let name = path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let arw =
        riffle_core::reader::read_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let shot = arw.shot;
    // The model usually already starts with the make ("SONY" / "ILCE-7M5"),
    // so the two are joined rather than one being dropped.
    let camera = match (shot.make.as_deref(), shot.model.as_deref()) {
        (Some(make), Some(model)) => Some(format!("{make} {model}")),
        (make, model) => make.or(model).map(str::to_string),
    };
    Ok(Metadata {
        name,
        camera,
        lens: shot.lens_model,
        aperture: match (shot.f_number, shot.estimated_f_number) {
            (Some(r), _) => decimal(r, 1).map(|t| format!("f/{t}")),
            (None, Some(f)) => decimal(
                riffle_core::arw::Rational {
                    num: (f * 10.0).round() as i64,
                    den: 10,
                },
                1,
            )
            .map(|t| format!("f/{t} (est.)")),
            (None, None) => None,
        },
        shutter: shot.exposure_time.and_then(shutter),
        iso: shot.iso.map(|v| v.to_string()),
        focal_length: shot
            .focal_length
            .and_then(|r| decimal(r, 1))
            .map(|t| format!("{t} mm")),
        exposure_bias: shot.exposure_bias.and_then(|r| {
            let v = r.value()?;
            let text = decimal(r, 1)?;
            Some(if v > 0.0 {
                format!("+{text} EV")
            } else {
                format!("{text} EV")
            })
        }),
        focus_distance: shot
            .focus_distance_mm
            .map(|mm| format!("{:.2} m", f64::from(mm) / 1000.0)),
        captured_at: shot.capture_time,
    })
}

/// The metadata of one file, read from the same bounded prefix as `preview`.
#[tauri::command]
pub async fn metadata(path: String) -> Result<Metadata, String> {
    tauri::async_runtime::spawn_blocking(move || read_metadata(Path::new(&path)))
        .await
        .map_err(|e| e.to_string())?
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

/// The 1:1 crop of one file around its focus point, as raw RGBA. `width` and
/// `height` are the viewport in device pixels.
#[tauri::command]
pub async fn focus_crop(path: String, width: u32, height: u32) -> Result<Response, String> {
    let (orientation, crop, timing) = tauri::async_runtime::spawn_blocking(move || {
        read_focus_crop(Path::new(&path), width as usize, height as usize)
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(Response::new(crop_payload(orientation, &crop, timing)))
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
        let (dir, index, listed) = (dir.clone(), index.clone(), listed.clone());
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

    let format = *index::lock(&app.state::<AppSidecarFormat>().0);
    let dirty = {
        let (dir, index) = (dir.clone(), index);
        tauri::async_runtime::spawn_blocking(move || {
            reconcile_sidecars_of(&dir, &listed, &index, format)
        })
        .await
        .map_err(|e| e.to_string())?
    };
    match dirty {
        // A dirty row waited out its debounce in an earlier session already,
        // so it goes to the writer with none.
        Ok(dirty) => {
            if let Some(writer) = &app.state::<AppWriter>().0 {
                for (path, rating) in dirty {
                    if let Err(e) = writer.set_now(PathBuf::from(path), rating, format) {
                        log::error!("failed to queue a pending sidecar: {e}");
                    }
                }
            }
        }
        // The sidecars are a cache layer over the folder; failing to read them
        // must not stop the folder from opening.
        Err(e) => log::error!("failed to reconcile the sidecars of {dir}: {e}"),
    }

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

/// The sidecar writer, shared between `set_rating` and the quit-time drain.
/// `None` when there is no index to record judgements in, in which case
/// `set_rating` is an error rather than a silent no-op.
pub struct AppWriter(pub Option<Writer>);

/// The sidecar format selected in the settings, read once at launch.
pub struct AppSidecarFormat(pub Mutex<SidecarFormat>);

/// Record a judgement for one file: `-1` is a reject, `0` unrated and `1`-`5`
/// stars.
///
/// The `ratings` row is written before the writer is told, so a crash between
/// the two still leaves the row dirty and the sidecar is written on the next
/// open of that folder. The command is `async` because it touches SQLite; the
/// frontend redraws without awaiting it.
#[tauri::command]
pub async fn set_rating(app: tauri::AppHandle, path: String, rating: i8) -> Result<(), String> {
    if !(-1..=5).contains(&rating) {
        return Err(format!("rating {rating} is outside -1..=5"));
    }
    // 0 and "unrated" are the same state; the row keeps NULL and the sidecar
    // gets a `0` only when one already exists.
    let rating = Some(rating).filter(|r| *r != 0);
    let Some(index) = app.state::<AppIndex>().0.clone() else {
        return Err("no index cache available".to_string());
    };
    let dir = Path::new(&path)
        .parent()
        .map_or_else(String::new, |d| d.to_string_lossy().into_owned());
    {
        let path = path.clone();
        tauri::async_runtime::spawn_blocking(move || {
            index::lock(&index).set_rating(&dir, &path, rating)
        })
        .await
        .map_err(|e| e.to_string())??;
    }
    let format = *index::lock(&app.state::<AppSidecarFormat>().0);
    match &app.state::<AppWriter>().0 {
        Some(writer) => writer.set(PathBuf::from(path), rating, format),
        None => Err("the sidecar writer is not running".to_string()),
    }
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
    #[test]
    fn newest_photolab_picks_the_highest_major_version() {
        let names = [
            "DXOPhotoLab9.app",
            "Safari.app",
            "DXOPhotoLab10.app",
            "DXOPhotoLabX.app",
        ]
        .map(String::from);
        assert_eq!(
            super::newest_photolab(names.into_iter()).as_deref(),
            Some("DXOPhotoLab10.app")
        );
        assert_eq!(super::newest_photolab(std::iter::empty()), None);
    }

    use std::time::Duration;

    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("riffle-app-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    // Best effort, like the removal in temp_dir above: on Windows a directory
    // holding an open SQLite database cannot be removed, and several of these
    // tests still hold the Index when they finish. The next run's temp_dir
    // clears whatever is left over.
    fn remove_temp_dir(dir: &Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    /// A foreign sidecar, shaped like the one Bridge writes.
    fn sidecar(dir: &Path, name: &str, rating: i32) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(
            &path,
            format!(
                r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about="" xmlns:xmp="http://ns.adobe.com/xap/1.0/" xmp:Rating="{rating}"/>
 </rdf:RDF>
</x:xmpmeta>"#
            ),
        )
        .unwrap();
        path
    }

    fn sidecar_index(dir: &Path) -> Arc<Mutex<Index>> {
        Arc::new(Mutex::new(Index::open(&dir.join("index.sqlite")).unwrap()))
    }

    fn rating_of(index: &Arc<Mutex<Index>>, dir: &str, path: &str) -> Option<i8> {
        index::lock(index)
            .entries(dir)
            .unwrap()
            .into_iter()
            .find(|e| e.path == path)
            .unwrap()
            .rating
    }

    /// Give a file the `(size, mtime)` it had before it was rewritten, so a
    /// content change that the reconciliation is meant to miss really is
    /// invisible to it.
    fn restore_mtime(path: &Path, mtime_ns: i64) {
        let time = std::time::UNIX_EPOCH + Duration::from_nanos(mtime_ns as u64);
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(time)
            .unwrap();
    }

    #[test]
    fn a_foreign_sidecar_is_read_on_the_first_open_without_any_files_row() {
        let root = temp_dir("sidecar-first-open");
        let dir = root.to_string_lossy().into_owned();
        std::fs::write(root.join("a.ARW"), b"x").unwrap();
        sidecar(&root, "a.xmp", 4);
        let index = sidecar_index(&root);

        let listed = list_arw_in(&root).unwrap();
        let dirty = reconcile_sidecars_of(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();

        assert!(dirty.is_empty());
        // No `files` row exists yet, so `entries` cannot show it; the row is
        // there all the same, which is what the next scan will join against.
        assert!(index::lock(&index).entries(&dir).unwrap().is_empty());
        index::lock(&index)
            .write_batch(
                &dir,
                &[(index::stat(Path::new(&listed[0])).unwrap(), Err("x".into()))],
            )
            .unwrap();
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(4));

        remove_temp_dir(&root);
    }

    #[test]
    fn an_external_edit_wins_over_a_dirty_row_and_an_unchanged_stat_parses_nothing() {
        let root = temp_dir("sidecar-external");
        let dir = root.to_string_lossy().into_owned();
        std::fs::write(root.join("a.ARW"), b"x").unwrap();
        let file = sidecar(&root, "a.xmp", 2);
        let index = sidecar_index(&root);
        let listed = list_arw_in(&root).unwrap();

        // An app edit that never reached disk, then someone else's edit.
        index::lock(&index)
            .set_rating(&dir, &listed[0], Some(5))
            .unwrap();
        sidecar(&root, "a.xmp", 3);

        let dirty = reconcile_sidecars_of(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();
        assert!(dirty.is_empty(), "the sidecar won, so nothing is written");
        assert_eq!(
            index::lock(&index).dirty_rows(&dir).unwrap(),
            Vec::new(),
            "and the row is no longer dirty"
        );

        // The same stat parses nothing: the content is changed underneath
        // without touching size or mtime, and the stored rating does not move.
        let stat = index::stat(&file).unwrap();
        sidecar(&root, "a.xmp", 1);
        restore_mtime(&file, stat.mtime_ns);
        assert!(
            reconcile_sidecars_of(&dir, &listed, &index, SidecarFormat::Xmp)
                .unwrap()
                .is_empty()
        );

        index::lock(&index)
            .write_batch(
                &dir,
                &[(index::stat(Path::new(&listed[0])).unwrap(), Err("x".into()))],
            )
            .unwrap();
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(3));

        remove_temp_dir(&root);
    }

    #[test]
    fn a_deleted_sidecar_clears_the_rating_and_a_dirty_row_survives_it() {
        let root = temp_dir("sidecar-deleted");
        let dir = root.to_string_lossy().into_owned();
        for name in ["a.ARW", "b.ARW", "c.ARW"] {
            std::fs::write(root.join(name), b"x").unwrap();
        }
        let file = sidecar(&root, "a.xmp", 1);
        let index = sidecar_index(&root);
        let listed = list_arw_in(&root).unwrap();
        assert!(
            reconcile_sidecars_of(&dir, &listed, &index, SidecarFormat::Xmp)
                .unwrap()
                .is_empty()
        );

        // `a` had a sidecar and lost it: the truth is gone with it. `b` has a
        // judgement that never reached a sidecar, and must be written instead.
        // `c` has neither a sidecar nor a row, and is not a case at all.
        std::fs::remove_file(&file).unwrap();
        index::lock(&index)
            .set_rating(&dir, &listed[1], Some(-1))
            .unwrap();

        let dirty = reconcile_sidecars_of(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();
        assert_eq!(dirty, [(listed[1].clone(), Some(-1))]);

        let files: Vec<_> = listed
            .iter()
            .map(|p| (index::stat(Path::new(p)).unwrap(), Err("x".to_string())))
            .collect();
        index::lock(&index).write_batch(&dir, &files).unwrap();
        assert_eq!(rating_of(&index, &dir, &listed[0]), None);
        assert_eq!(rating_of(&index, &dir, &listed[2]), None);

        remove_temp_dir(&root);
    }

    #[test]
    fn a_sidecar_differing_only_in_case_is_the_one_that_is_read() {
        let root = temp_dir("sidecar-case");
        let dir = root.to_string_lossy().into_owned();
        std::fs::write(root.join("a.ARW"), b"x").unwrap();
        sidecar(&root, "A.XMP", 5);
        let index = sidecar_index(&root);
        let listed = list_arw_in(&root).unwrap();

        reconcile_sidecars_of(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();
        index::lock(&index)
            .write_batch(
                &dir,
                &[(index::stat(Path::new(&listed[0])).unwrap(), Err("x".into()))],
            )
            .unwrap();
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(5));

        remove_temp_dir(&root);
    }

    const PHOTOLAB_REJECT: &[u8] = include_bytes!("../../core/src/fixtures/dop/_DSC0002.ARW.dop");
    const PHOTOLAB_THREE: &[u8] = include_bytes!("../../core/src/fixtures/dop/_DSC0003.ARW.dop");

    /// Store the `files` rows of `listed` so `entries` can show their ratings.
    fn index_files(index: &Arc<Mutex<Index>>, dir: &str, listed: &[String]) {
        let files: Vec<_> = listed
            .iter()
            .map(|p| (index::stat(Path::new(p)).unwrap(), Err("x".to_string())))
            .collect();
        index::lock(index).write_batch(dir, &files).unwrap();
    }

    #[test]
    fn photolab_ratings_and_rejects_are_read_on_the_first_open_and_xmps_ignored() {
        let root = temp_dir("dop-first-open");
        let dir = root.to_string_lossy().into_owned();
        for name in ["a.ARW", "b.ARW", "c.ARW"] {
            std::fs::write(root.join(name), b"x").unwrap();
        }
        std::fs::write(root.join("a.ARW.dop"), PHOTOLAB_REJECT).unwrap();
        std::fs::write(root.join("b.ARW.dop"), PHOTOLAB_THREE).unwrap();
        sidecar(&root, "c.xmp", 4);
        let index = sidecar_index(&root);
        let listed = list_arw_in(&root).unwrap();

        let dirty = reconcile_sidecars_of(&dir, &listed, &index, SidecarFormat::Dop).unwrap();

        assert!(dirty.is_empty());
        index_files(&index, &dir, &listed);
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(-1));
        assert_eq!(rating_of(&index, &dir, &listed[1]), Some(3));
        assert_eq!(rating_of(&index, &dir, &listed[2]), None);

        remove_temp_dir(&root);
    }

    #[test]
    fn a_dop_differing_only_in_case_is_the_one_that_is_read() {
        let root = temp_dir("dop-case");
        let dir = root.to_string_lossy().into_owned();
        std::fs::write(root.join("a.ARW"), b"x").unwrap();
        std::fs::write(root.join("A.ARW.DOP"), PHOTOLAB_THREE).unwrap();
        let index = sidecar_index(&root);
        let listed = list_arw_in(&root).unwrap();

        reconcile_sidecars_of(&dir, &listed, &index, SidecarFormat::Dop).unwrap();
        index_files(&index, &dir, &listed);
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(3));

        remove_temp_dir(&root);
    }

    #[test]
    fn an_oversize_dop_is_neither_read_nor_handed_to_the_writer() {
        let root = temp_dir("dop-oversize");
        let dir = root.to_string_lossy().into_owned();
        std::fs::write(root.join("a.ARW"), b"x").unwrap();
        let mut big = PHOTOLAB_THREE.to_vec();
        big.resize(MAX_SIDECAR_BYTES as usize + 1, b'\n');
        std::fs::write(root.join("a.ARW.dop"), big).unwrap();
        let index = sidecar_index(&root);
        let listed = list_arw_in(&root).unwrap();
        index::lock(&index)
            .set_rating(&dir, &listed[0], Some(5))
            .unwrap();

        let dirty = reconcile_sidecars_of(&dir, &listed, &index, SidecarFormat::Dop).unwrap();

        assert!(dirty.is_empty(), "the writer must not patch it unread");
        index_files(&index, &dir, &listed);
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(5));

        remove_temp_dir(&root);
    }

    #[test]
    fn the_legacy_last_folder_file_is_taken_once() {
        let root = temp_dir("legacy-last-folder");
        let file = root.join("last_folder");
        assert_eq!(take_legacy_last_folder(&file), None);
        std::fs::write(&file, "/some/folder").unwrap();

        assert_eq!(
            take_legacy_last_folder(&file).as_deref(),
            Some("/some/folder")
        );
        assert!(!file.exists());
        assert_eq!(take_legacy_last_folder(&file), None);

        remove_temp_dir(&root);
    }

    #[test]
    fn lists_only_arw_files_sorted_by_name() {
        let dir = temp_dir("list");
        for name in [
            "b.ARW",
            "a.arw",
            "c.Arw",
            "d.jpg",
            "e.arw.txt",
            "f.DNG",
            "g.dng",
            "f.dop",
        ] {
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
        assert_eq!(names, ["a.arw", "b.ARW", "c.Arw", "f.DNG", "g.dng"]);
        assert!(files.iter().all(|p| Path::new(p).is_absolute()));

        remove_temp_dir(&dir);
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

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_files_parent_at_the_filesystem_root_is_the_root_not_none() {
        // The boundary the `dropped_dir` doc comment describes: `parent()`
        // only yields `None` for the root itself, so a file directly under
        // it resolves to the root. There is no writable file directly under
        // `/` to exercise `dropped_dir` itself against, so this pins the
        // `Path::parent` behaviour the function relies on instead.
        assert_eq!(Path::new("/a.arw").parent(), Some(Path::new("/")));
    }

    #[test]
    fn shutter_reads_as_a_fraction_below_a_second_and_seconds_above() {
        use riffle_core::arw::Rational;
        assert_eq!(
            shutter(Rational { num: 1, den: 250 }).as_deref(),
            Some("1/250")
        );
        assert_eq!(
            shutter(Rational { num: 13, den: 10 }).as_deref(),
            Some("1.3\"")
        );
        assert_eq!(shutter(Rational { num: 4, den: 1 }).as_deref(), Some("4\""));
        assert_eq!(shutter(Rational { num: 0, den: 1 }), None);
        assert_eq!(shutter(Rational { num: 1, den: 0 }), None);
    }

    #[test]
    fn decimals_drop_their_trailing_zeros() {
        use riffle_core::arw::Rational;
        assert_eq!(
            decimal(Rational { num: 28, den: 10 }, 1).as_deref(),
            Some("2.8")
        );
        assert_eq!(
            decimal(Rational { num: 500, den: 10 }, 1).as_deref(),
            Some("50")
        );
        assert_eq!(decimal(Rational { num: 1, den: 0 }, 1), None);
        assert_eq!(
            decimal(Rational { num: 100, den: 1 }, 0).as_deref(),
            Some("100")
        );
    }

    /// Encode a synthetic gradient so the test needs no image file.
    fn gradient_jpeg(w: usize, h: usize) -> Vec<u8> {
        let mut rgb = Vec::with_capacity(w * h * 3);
        for y in 0..h {
            for x in 0..w {
                rgb.extend_from_slice(&[(x * 255 / w) as u8, (y * 255 / h) as u8, 128]);
            }
        }
        let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        c.set_size(w, h);
        c.set_quality(90.0);
        let mut c = c.start_compress(Vec::new()).unwrap();
        c.write_scanlines(&rgb).unwrap();
        c.finish().unwrap()
    }

    /// A TIFF whose IFD0 holds a tiny preview and whose IFD1 points at the
    /// JpgFromRaw, mirroring the builder in `riffle_core::reader`'s tests.
    fn arw_with_full(jpeg: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"II\x2a\x00");
        buf.extend_from_slice(&8u32.to_le_bytes());
        let ifd1_at = 8 + 2 + 2 * 12 + 4;
        let at = 256;
        for (entries, next) in [
            ([(0x0201u16, 8u32), (0x0202, 1)], ifd1_at as u32),
            ([(0x0201, at as u32), (0x0202, jpeg.len() as u32)], 0),
        ] {
            buf.extend_from_slice(&2u16.to_le_bytes());
            for (tag, value) in entries {
                buf.extend_from_slice(&tag.to_le_bytes());
                buf.extend_from_slice(&4u16.to_le_bytes());
                buf.extend_from_slice(&1u32.to_le_bytes());
                buf.extend_from_slice(&value.to_le_bytes());
            }
            buf.extend_from_slice(&next.to_le_bytes());
        }
        buf.resize(at, 0);
        buf.extend_from_slice(jpeg);
        buf
    }

    #[test]
    fn crop_payload_header_encodes_every_field() {
        let crop = riffle_core::partial::FocusCrop {
            crop: riffle_core::partial::Crop {
                pixels: vec![1, 2, 3, 4, 5, 6, 7, 8],
                width: 2,
                height: 1,
                x: 16,
                y: 32,
            },
            point_x: 7,
            point_y: 9,
        };
        let out = crop_payload(
            8,
            &crop,
            CropTiming {
                read_us: 731,
                decode_us: 21_500,
            },
        );
        assert_eq!(out.len(), CROP_HEADER_LEN + crop.crop.pixels.len());
        let u16at = |at: usize| u16::from_le_bytes([out[at], out[at + 1]]);
        let u32at =
            |at: usize| u32::from_le_bytes([out[at], out[at + 1], out[at + 2], out[at + 3]]);
        assert_eq!(u16at(0), CROP_KIND_RGBA_V2);
        assert_eq!(u16at(2), 8);
        assert_eq!(u32at(4), 2);
        assert_eq!(u32at(8), 1);
        assert_eq!(u32at(12), 7);
        assert_eq!(u32at(16), 9);
        assert_eq!(u32at(20), 731);
        assert_eq!(u32at(24), 21_500);
        assert_eq!(u32at(28), 0);
        assert_eq!(&out[CROP_HEADER_LEN..], &crop.crop.pixels);
    }

    #[test]
    fn a_quarter_turn_swaps_the_viewport_and_the_cap_bounds_each_axis() {
        assert_eq!(crop_size(1, 300, 200), (300, 200));
        assert_eq!(crop_size(6, 300, 200), (200, 300));
        assert_eq!(crop_size(8, 300, 200), (200, 300));
        assert_eq!(crop_size(8, 4000, 2000), (CROP_MAX, CROP_MAX));
        assert_eq!(crop_size(1, 0, 0), (1, 1));
    }

    #[test]
    fn a_synthetic_arw_crops_around_the_centre_without_a_focus_point() {
        let dir = temp_dir("crop");
        let path = dir.join("a.arw");
        std::fs::write(&path, arw_with_full(&gradient_jpeg(400, 300))).unwrap();

        let (orientation, crop, _) = read_focus_crop(&path, 64, 48).unwrap();
        assert_eq!(orientation, 1);
        // The crop snaps to an MCU boundary horizontally, so it can come back
        // wider than asked for, never narrower.
        assert!((64..64 + 16).contains(&crop.crop.width));
        assert_eq!(crop.crop.height, 48);
        assert_eq!(crop.crop.pixels.len(), crop.crop.width * 48 * 4);
        // Centre of a 400x300 image, minus the crop's own snapped origin.
        assert_eq!(crop.point_x, 200 - crop.crop.x);
        assert_eq!(crop.crop.x % 16, 0);
        assert_eq!(crop.point_y, 24);

        remove_temp_dir(&dir);
    }

    #[test]
    fn missing_preview_is_an_error() {
        let dir = temp_dir("preview");
        let path = dir.join("broken.arw");
        std::fs::write(&path, b"II\x2a\x00\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00").unwrap();
        assert!(read_preview(&path).is_err());
        remove_temp_dir(&dir);
    }
}
