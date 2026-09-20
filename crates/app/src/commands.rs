//! The commands the frontend invokes: folder picking, ARW enumeration and
//! preview extraction.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tauri::ipc::Response;
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;

use crate::index::{self, FileStat, Index, IndexedFile, SidecarStat};
use crate::shortcuts::{Binding, Keymap};
use crate::sidecar::{SidecarFormat, Writer};
use crate::trash;

/// Size of the header that precedes the JPEG bytes in a `preview` payload.
pub const PREVIEW_HEADER_LEN: usize = 8;

/// Tag identifying the payload kind and version: an IFD0 preview JPEG, v1.
pub const PREVIEW_KIND_JPEG_V1: u16 = 1;

/// Tag for the other payload kind: a cached thumbnail JPEG, v1.
pub const THUMBNAIL_KIND_JPEG_V1: u16 = 2;

/// Size of the header that precedes the pixels in a `focus_crop` payload.
pub const CROP_HEADER_LEN: usize = 32;

/// Tag for the third payload kind: a raw RGBA focus crop, v3. v1 was the same
/// pixels behind a 24-byte header without the timings; v2 left the last word
/// reserved instead of carrying the full JPEG size.
pub const CROP_KIND_RGBA_V3: u16 = 5;

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
/// | 0      | 2    | kind/version tag (`CROP_KIND_RGBA_V3`)    |
/// | 2      | 2    | EXIF Orientation (1..8)                   |
/// | 4      | 4    | crop width in pixels                      |
/// | 8      | 4    | crop height in pixels                     |
/// | 12     | 4    | point of interest x, in crop pixels       |
/// | 16     | 4    | point of interest y, in crop pixels       |
/// | 20     | 4    | ranged read of the JpgFromRaw, in us      |
/// | 24     | 4    | partial decode, in us                     |
/// | 28     | 2    | full JPEG width in pixels                 |
/// | 30     | 2    | full JPEG height in pixels                |
///
/// The crop is cut in unrotated JPEG coordinates; the Orientation is the one
/// the frontend already applies to the preview.
fn crop_payload(
    orientation: u16,
    crop: &riffle_core::partial::FocusCrop,
    timing: CropTiming,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(CROP_HEADER_LEN + crop.crop.pixels.len());
    out.extend_from_slice(&CROP_KIND_RGBA_V3.to_le_bytes());
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
    out.extend_from_slice(&(crop.crop.image_width as u16).to_le_bytes());
    out.extend_from_slice(&(crop.crop.image_height as u16).to_le_bytes());
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
    list_dir(dir, None).map(|(files, _)| files)
}

/// The RAW files of `dir` as `list_arw_in` lists them, plus the sidecars of
/// `format` keyed by lower-cased file name so that a `FOO.XMP` written by
/// another tool is found for `FOO.ARW`.
///
/// One listing of the directory for both, rather than a second `read_dir` or
/// a `stat` of 5000 guessed names: the second open of a folder is meant to
/// cost no more than the listing the RAWs already pay for.
fn list_folder_in(
    dir: &Path,
    format: SidecarFormat,
) -> Result<(Vec<String>, HashMap<String, SidecarStat>), String> {
    list_dir(dir, Some(format))
}

fn list_dir(
    dir: &Path,
    format: Option<SidecarFormat>,
) -> Result<(Vec<String>, HashMap<String, SidecarStat>), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut files: Vec<PathBuf> = Vec::new();
    let mut sidecars = HashMap::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && riffle_core::scan::is_raw_file(&path) {
            files.push(path);
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !format.is_some_and(|f| f.matches(&name)) {
            continue;
        }
        if let Ok(stat) = index::stat(&path) {
            sidecars.insert(name.to_lowercase(), (stat.path, stat.size, stat.mtime_ns));
        }
    }
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    let files = files
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    Ok((files, sidecars))
}

/// Largest sidecar the folder-open pass reads. A Lightroom sidecar is tens of
/// KB; anything past this is not a sidecar this app should be parsing, and is
/// left alone rather than failing the open.
const MAX_SIDECAR_BYTES: i64 = 4 * 1024 * 1024;

/// A sidecar the folder-open pass could not use, reported to the frontend.
#[derive(Debug, serde::Serialize)]
pub struct SidecarError {
    /// The RAW file's path, not the sidecar's: it has to match the key the
    /// `sidecar-error` event uses for a write failure so the frontend's
    /// `ErrorList` replaces one with the other instead of showing both. The
    /// sidecar's file name goes in `message`.
    path: String,
    message: String,
}

/// Bring the `ratings` rows of `dir` in line with the sidecars on disk and
/// return the judgements that still have to be written, along with the
/// sidecars that could not be used.
///
/// The sidecar is the source of truth, so anything whose stat changed since
/// the app last saw it is read back here; a sidecar that cannot be read or
/// parsed leaves its row as it was rather than erroring the open.
fn reconcile_sidecars_of(
    dir: &str,
    listed: &[String],
    sidecars: &HashMap<String, SidecarStat>,
    index: &Arc<Mutex<Index>>,
    format: SidecarFormat,
) -> Result<(Vec<index::DirtyRow>, Vec<SidecarError>), String> {
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
    let mut parsed: Vec<index::ParsedSidecar> = Vec::new();
    let mut problems: Vec<SidecarError> = Vec::new();
    for (path, (sidecar, size, mtime_ns), dirty) in &to_parse {
        let problem = |message: String| SidecarError {
            path: path.clone(),
            message: format!(
                "{}: {message}",
                sidecar
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_else(|| sidecar.to_string_lossy())
            ),
        };
        if oversize.contains(path.as_str()) {
            problems.push(problem(format!(
                "larger than {} MiB, not read",
                MAX_SIDECAR_BYTES / 1024 / 1024
            )));
            continue;
        }
        let read = std::fs::read(sidecar)
            .map_err(|e| e.to_string())
            .and_then(|bytes| {
                Ok((
                    format.read_rating(&bytes)?,
                    format.read_pick(&bytes)?,
                    format.read_label(&bytes)?,
                ))
            });
        match read {
            Ok((rating, pick, label)) => {
                parsed.push((path.clone(), rating, pick, label, *size, *mtime_ns, *dirty))
            }
            Err(e) => problems.push(problem(e)),
        }
    }
    let mut index = index::lock(index);
    index.store_sidecar_ratings(dir, &parsed)?;
    // After storing, so a row the sidecar just won stays out of this: only
    // what still has nowhere to be read back from is written out. An
    // oversize sidecar's row is excluded too, even if still dirty, since the
    // writer must not patch a sidecar whose contents were never read.
    let dirty = index.dirty_rows(dir)?;
    let dirty = dirty
        .into_iter()
        .filter(|(path, _, _, _, _)| !oversize.contains(path.as_str()))
        .collect();
    Ok((dirty, problems))
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
/// be read is logged and falls back to the defaults, followed by the keymap
/// and the `autoAdvance` setting.
pub fn load_settings(app: &tauri::AppHandle) -> (SidecarFormat, Keymap, bool) {
    let store = match settings(app) {
        Ok(store) => store,
        Err(e) => {
            eprintln!("failed to open the settings: {e}");
            let format = SidecarFormat::default();
            return (format, Keymap::defaults(), auto_advance_setting(None));
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
    let format =
        SidecarFormat::from_setting(store.get("sidecarFormat").as_ref().and_then(|v| v.as_str()));
    let keymap = Keymap::from_overrides(store.get("shortcuts").as_ref());
    let auto_advance = auto_advance_setting(store.get("autoAdvance").as_ref());
    (format, keymap, auto_advance)
}

/// The stored `autoAdvance` value; missing or non-boolean means off.
fn auto_advance_setting(value: Option<&Value>) -> bool {
    value.and_then(Value::as_bool).unwrap_or(false)
}

/// Switch the sidecar format to `format`: apply the new format first so any
/// rating set while this runs is queued in it, drain what the writer still
/// holds queued in the old format, persist the new format, and reset the
/// index so the next folder open reads the new format's sidecars. Blocks on
/// the drain and SQLite, so it must not run on the main thread.
///
/// Holds `AppSwitchLock` for its whole body and re-checks the current format
/// once inside it, so two switches started back to back cannot interleave
/// and the second one is a no-op when it already matches.
pub fn switch_sidecar_format(app: &tauri::AppHandle, format: SidecarFormat) -> Result<(), String> {
    let switch_lock = app.state::<AppSwitchLock>();
    let _guard = index::lock(&switch_lock.0);
    let current = app.state::<AppSidecarFormat>();
    let writer = app.state::<AppWriter>();
    let index = app.state::<AppIndex>();
    switch_format(
        &current.0,
        writer.0.as_ref(),
        index.0.as_ref(),
        format,
        |format| {
            settings(app).and_then(|store| {
                store.set("sidecarFormat", format.setting());
                store.save().map_err(|e| e.to_string())
            })
        },
    )
}

/// The body of `switch_sidecar_format`, without the `AppHandle`: the state
/// write, the drain, `persist` (a save failure is logged, not returned) and
/// the index reset. The caller serialises switches (the command does it with
/// `AppSwitchLock`); this function does no locking of its own beyond the
/// state and index mutexes.
fn switch_format(
    current: &Mutex<SidecarFormat>,
    writer: Option<&Writer>,
    index: Option<&Arc<Mutex<Index>>>,
    format: SidecarFormat,
    persist: impl FnOnce(SidecarFormat) -> Result<(), String>,
) -> Result<(), String> {
    if *index::lock(current) == format {
        return Ok(());
    }
    *index::lock(current) = format;
    if let Some(writer) = writer {
        writer.flush(crate::sidecar::DRAIN_TIMEOUT);
    }
    if let Err(e) = persist(format) {
        eprintln!("failed to save the sidecar format: {e}");
    }
    match index {
        Some(index) => index::lock(index).reset_sidecars(),
        None => Ok(()),
    }
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

/// The strip sort order a stored `sortOrder` value names; anything unknown or
/// missing is the default, file name order.
fn parse_sort_order(value: Option<&str>) -> &'static str {
    match value {
        Some("capture") => "capture",
        Some("rating") => "rating",
        _ => "name",
    }
}

/// The strip sort order remembered by `set_sort_order`.
#[tauri::command]
pub fn sort_order(app: tauri::AppHandle) -> &'static str {
    let store = settings(&app).ok();
    parse_sort_order(
        store
            .as_ref()
            .and_then(|s| s.get("sortOrder"))
            .as_ref()
            .and_then(|v| v.as_str()),
    )
}

/// Remember the strip sort order. Failing to write it only means starting in
/// file name order, so it is logged, not returned.
#[tauri::command]
pub fn set_sort_order(app: tauri::AppHandle, order: String) {
    let saved = settings(&app).and_then(|store| {
        store.set("sortOrder", parse_sort_order(Some(&order)));
        store.save().map_err(|e| e.to_string())
    });
    if let Err(e) = saved {
        eprintln!("failed to remember the sort order: {e}");
    }
}

#[tauri::command]
pub fn list_arw(dir: String) -> Result<Vec<String>, String> {
    let dir = canonicalize(&dir);
    let started = std::time::Instant::now();
    let files = list_arw_in(Path::new(&dir))?;
    log::info!(
        "open list: dir={dir} raws={} in {}ms",
        files.len(),
        started.elapsed().as_millis()
    );
    Ok(files)
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

fn read_metadata(path: &Path) -> Result<Metadata, String> {
    let name = path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let arw =
        riffle_core::reader::read_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let shot = arw.shot;
    let exif = crate::exif::exif(&shot);
    let label = |l: Option<crate::exif::Labelled>| l.map(|l| l.label);
    Ok(Metadata {
        name,
        camera: exif.camera,
        lens: exif.lens,
        aperture: label(exif.aperture),
        shutter: label(exif.shutter),
        iso: label(exif.iso),
        focal_length: label(exif.focal_length),
        exposure_bias: shot.exposure_bias.and_then(|r| {
            let v = r.value()?;
            let text = crate::exif::decimal(r, 1)?;
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
    /// The scan `start_scan` most recently spawned, with the id it was
    /// spawned under so the task's own clean-up can tell whether the entry
    /// is still its own. `Some` does not mean a scan is running; see
    /// `scanning`.
    running: Option<(u64, Arc<AtomicBool>, tauri::async_runtime::JoinHandle<()>)>,
    pending: HashMap<u64, PendingScan>,
    /// The number of `scan_folder` calls currently past minting their id but
    /// not yet done (listing, `reconcile`, `reconcile_sidecars_of`), i.e.
    /// past the point where `running` is invisible for them. `clear_index`
    /// refuses while this is nonzero, or it could run against a folder that
    /// is mid-open with no scan indicated by `running`. See `Preparing`.
    preparing: u32,
}

impl ScansState {
    fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    /// Whether a scan is genuinely in progress: a `scan_folder` past minting
    /// its id and not yet returned, a prepared scan waiting for `start_scan`,
    /// or a spawned scan task that has not yet run to completion. The single
    /// definition every guard uses.
    fn scanning(&self) -> bool {
        self.preparing > 0 || self.running.is_some() || !self.pending.is_empty()
    }

    /// Drop the entry of the scan spawned under `scan_id`, if it is still the
    /// one stored. A later `scan_folder` may have taken it and a later
    /// `start_scan` stored a newer one, which this must not clear.
    fn finish(&mut self, scan_id: u64) {
        if self
            .running
            .as_ref()
            .is_some_and(|(id, _, _)| *id == scan_id)
        {
            self.running = None;
        }
    }
}

struct PendingScan {
    dir: String,
    todo: Vec<FileStat>,
    cancel: Arc<AtomicBool>,
}

/// Marks one `scan_folder` call as being in its prepare phase (listing,
/// `reconcile`, `reconcile_sidecars_of`) by holding `ScansState::preparing`
/// above zero for as long as it is alive. `scan_folder` mints its id, takes
/// `running` and spends this whole phase with no lock held and `running ==
/// None`, so without this, a `clear_index` issued while a folder is opening
/// would pass both of its checks and run against the folder's
/// soon-to-be-reconciled rows. `start` increments under the very guard that
/// mints the id and takes `running`, so `preparing` is never zero while that
/// guard is released and the previous scan's task is still alive; the
/// decrement on drop covers every early return of `scan_folder` as well as
/// its normal one.
struct Preparing {
    app: tauri::AppHandle,
}

impl Preparing {
    /// Increments `preparing` under `state`, the same guard the caller used
    /// to mint the scan id and take `running`.
    fn start(app: tauri::AppHandle, state: &mut ScansState) -> Self {
        state.preparing += 1;
        Self { app }
    }
}

impl Drop for Preparing {
    fn drop(&mut self) {
        let scans = self.app.state::<Scans>();
        let mut state = index::lock(&scans.0);
        state.preparing -= 1;
        let scanning = state.scanning();
        // Every `scan-state` emit in this file happens under the `Scans`
        // lock, at the moment the state changes, so the mutex serialises
        // them in the order the state actually changed; releasing the lock
        // first (or emitting a value read earlier) would let two emits from
        // different threads interleave with no ordering guarantee. Holding
        // the lock across `emit` is safe here: no Rust-side `listen` handler in this
        // codebase takes the `Scans` lock, and `emit` only queues the event
        // to the webviews rather than running their listeners synchronously.
        let _ = self.app.emit("scan-state", scanning);
    }
}

/// The index, shared between the commands and the scan thread. It is a cache:
/// `None` when the cache directory could not be opened (see `main.rs`), in
/// which case the commands below degrade to "no thumbnails" instead of the
/// app failing to launch.
pub struct AppIndex(pub Option<Arc<Mutex<Index>>>);

/// A read-only connection on the same index for `folder_entries` and
/// `thumbnail`, so they do not wait behind the scan's write transactions. It
/// is the writer's `Arc` when the reader could not be opened.
pub struct AppIndexReader(pub Option<Arc<Mutex<Index>>>);

/// Evict stale folders from the index once, on a thread of its own. It holds
/// the `Scans` lock throughout and skips if a scan has already been started,
/// so no scan runs during the `VACUUM`: a `scan_folder`/`start_scan` issued
/// meanwhile waits for it (~0.2 s on a ~100 MB index, M3 Pro).
pub fn spawn_eviction(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let Some(index) = app.state::<AppIndex>().0.clone() else {
            return;
        };
        let scans = app.state::<Scans>();
        let state = index::lock(&scans.0);
        if state.running.is_some() {
            return;
        }
        match index::lock(&index).evict(index::now_secs(), index::EVICT_POLICY) {
            Ok(summary) => log::info!("index eviction: {summary:?}"),
            Err(e) => log::error!("failed to evict stale folders from the index: {e}"),
        }
        drop(state);
    });
}

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
/// folder. `sidecar_errors` lists the sidecars the open could not read.
#[derive(serde::Serialize)]
pub struct ScanStarted {
    total: usize,
    scan_id: u64,
    sidecar_errors: Vec<SidecarError>,
}

/// Bring the index of `dir` up to date: wait for a previous scan's last write
/// to land, then drop the rows of files that are gone or changed and report
/// how many have no valid row. The actual scan does not start until the
/// frontend calls `start_scan` with the returned `scan_id`, so this only
/// prepares the work; call `start_scan` right after storing the id. A no-op
/// (nothing to scan) when the index cache is unavailable; `start_scan` still
/// emits `scan-done` for this `scan_id` in that case.
#[tauri::command]
pub async fn scan_folder(app: tauri::AppHandle, dir: String) -> Result<ScanStarted, String> {
    let scans = app.state::<Scans>();
    let (scan_id, previous, _preparing) = {
        let mut state = index::lock(&scans.0);
        let scan_id = state.next_id();
        state.latest_id = scan_id;
        let previous = state.running.take();
        let preparing = Preparing::start(app.clone(), &mut state);
        let scanning = state.scanning();
        // Emit while still holding the lock, for the same reason as
        // `Preparing::drop` and `start_scan` (see the comment in
        // `Preparing::drop`): whichever of this and a concurrent
        // `Preparing::drop` runs second is guaranteed to emit after the
        // other's `app.emit` call has returned.
        let _ = app.emit("scan-state", scanning);
        (scan_id, previous, preparing)
    };
    let canonical = canonicalize(&dir);
    crate::watch::set(&app, &canonical, &dir);
    let dir = canonical;
    let cancel = Arc::new(AtomicBool::new(false));

    if let Some((_, previous_cancel, previous_handle)) = previous {
        previous_cancel.store(true, Ordering::Relaxed);
        let _ = previous_handle.await;
    }

    let Some(index) = app.state::<AppIndex>().0.clone() else {
        return Ok(ScanStarted {
            total: 0,
            scan_id,
            sidecar_errors: Vec::new(),
        });
    };

    let format = *index::lock(&app.state::<AppSidecarFormat>().0);
    let scan_started = std::time::Instant::now();
    let (listed, sidecars) = {
        let dir = dir.clone();
        tauri::async_runtime::spawn_blocking(move || list_folder_in(Path::new(&dir), format))
            .await
            .map_err(|e| e.to_string())??
    };
    log::info!(
        "scan list: dir={dir} scan_id={scan_id} raws={} sidecars={} in {}ms",
        listed.len(),
        sidecars.len(),
        scan_started.elapsed().as_millis()
    );

    let reconcile_started = std::time::Instant::now();
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

    log::info!(
        "scan reconcile: dir={dir} scan_id={scan_id} todo={} in {}ms",
        todo.len(),
        reconcile_started.elapsed().as_millis()
    );

    let sidecars_started = std::time::Instant::now();
    let dirty = {
        let (dir, index) = (dir.clone(), index);
        tauri::async_runtime::spawn_blocking(move || {
            reconcile_sidecars_of(&dir, &listed, &sidecars, &index, format)
        })
        .await
        .map_err(|e| e.to_string())?
    };
    let sidecars_ms = sidecars_started.elapsed().as_millis();
    let mut sidecar_errors = Vec::new();
    let mut dirty_count = 0;
    match dirty {
        // A dirty row waited out its debounce in an earlier session already,
        // so it goes to the writer with none. Its own `label_known` (see
        // `Index::set_rating`/`mark_written`) travels with it: a row created
        // before the app learned the label, and never written before this
        // open (e.g. a crash), must still reach the writer as unknown, or an
        // existing sidecar label would be stripped.
        Ok((dirty, problems)) => {
            sidecar_errors = problems;
            dirty_count = dirty.len();
            if let Some(writer) = &app.state::<AppWriter>().0 {
                for (path, rating, pick, label, label_known) in dirty {
                    if let Err(e) = writer.set_now(
                        PathBuf::from(path),
                        rating,
                        pick,
                        label,
                        label_known,
                        format,
                    ) {
                        log::error!("failed to queue a pending sidecar: {e}");
                    }
                }
            }
        }
        // The sidecars are a cache layer over the folder; failing to read them
        // must not stop the folder from opening.
        Err(e) => log::error!("failed to reconcile the sidecars of {dir}: {e}"),
    }
    log::info!("scan sidecars: dir={dir} scan_id={scan_id} dirty={dirty_count} in {sidecars_ms}ms");
    log::info!(
        "scan prepare: dir={dir} scan_id={scan_id} todo={} in {}ms",
        todo.len(),
        scan_started.elapsed().as_millis()
    );

    let total = todo.len();
    let mut state = index::lock(&scans.0);
    if state.latest_id != scan_id {
        // A later `scan_folder` has superseded this one while it was
        // listing/reconciling; its own `start_scan` (or none at all) owns
        // `pending`/`running` now, so queuing this work would either be
        // overwritten or, worse, race the id check below in `start_scan`.
        return Ok(ScanStarted {
            total,
            scan_id,
            sidecar_errors,
        });
    }
    // Any entry still here belongs to a scan this one has already superseded
    // (only one `scan_folder`/`start_scan` pair is ever live at a time); it
    // never started, so there is nothing to cancel or join, just drop it.
    state.pending.clear();
    state
        .pending
        .insert(scan_id, PendingScan { dir, todo, cancel });
    drop(state);
    Ok(ScanStarted {
        total,
        scan_id,
        sidecar_errors,
    })
}

/// Emit a `scan-done` with no work done, for a `scan_id` `start_scan` is not
/// going to spawn a real scan for. The frontend keys its `scanRunning` flag
/// off `scan-done` alone, so every `scan_id` it hands a scan for must
/// eventually get one, even the ids that turn out to be no-ops here.
fn emit_empty_scan_done(app: &tauri::AppHandle, dir: &str, scan_id: u64) {
    let _ = app.emit(
        "scan-done",
        Done {
            dir,
            scan_id,
            total: 0,
            errors: 0,
        },
    );
}

/// Start the scan `scan_folder` prepared for `scan_id`, in the background.
/// Still emits `scan-done` (with no work done) when there is no pending work
/// under that id (the index cache was unavailable, or this scan has since
/// been superseded), so the frontend's `scanRunning` flag always clears.
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
        let dir = state
            .pending
            .remove(&scan_id)
            .map(|pending| pending.dir)
            .unwrap_or_default();
        drop(state);
        emit_empty_scan_done(&app, &dir, scan_id);
        return Ok(());
    }
    let Some(pending) = state.pending.remove(&scan_id) else {
        drop(state);
        emit_empty_scan_done(&app, "", scan_id);
        return Ok(());
    };
    let Some(index) = app.state::<AppIndex>().0.clone() else {
        drop(state);
        emit_empty_scan_done(&app, &pending.dir, scan_id);
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
                index::PROGRESS_INTERVAL,
                |done, total, ready| {
                    let _ = app.emit(
                        "scan-progress",
                        Progress {
                            dir: &dir,
                            scan_id,
                            done,
                            total,
                            ready,
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
            // Emit while still holding the lock: `start_scan` below emits its
            // own `true` under the same lock, before ever releasing it, so
            // whichever of the two critical sections runs second (this one,
            // if the scan finishes fast enough to race the store below) is
            // guaranteed to emit after the other's `app.emit` call has
            // returned. Emitting with the lock dropped would let these two
            // `app.emit` calls interleave freely on separate threads with no
            // ordering guarantee, which is exactly the race that used to let
            // a stale `true` land after this correct `false` and leave the
            // settings window stuck showing "scanning" forever.
            let scans = app.state::<Scans>();
            let mut state = index::lock(&scans.0);
            state.finish(scan_id);
            let scanning = state.scanning();
            let _ = app.emit("scan-state", scanning);
        }
    });
    state.running = Some((scan_id, cancel, handle));
    // See the comment above the task's own emit: kept under the same lock
    // for the same reason. `state` has been held continuously since the top
    // of this function, so the task cannot have taken the lock (and hence
    // cannot have emitted) before this call.
    let _ = app.emit("scan-state", true);
    Ok(())
}

#[derive(serde::Serialize, Clone)]
struct Progress<'a> {
    dir: &'a str,
    scan_id: u64,
    done: usize,
    total: usize,
    /// The paths the index committed since the previous event, so the
    /// frontend can request exactly those thumbnails instead of every cell
    /// it has nothing for yet.
    ready: Vec<String>,
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
    let Some(index) = app.state::<AppIndexReader>().0.clone() else {
        return Ok(Vec::new());
    };
    let started = std::time::Instant::now();
    let rows = {
        let dir = dir.clone();
        tauri::async_runtime::spawn_blocking(move || index::lock(&index).entries(&dir))
            .await
            .map_err(|e| e.to_string())??
    };
    log::info!(
        "open entries: dir={dir} rows={} in {}ms",
        rows.len(),
        started.elapsed().as_millis()
    );
    Ok(rows)
}

/// The cached thumbnail of one file, in the same envelope as `preview`.
#[tauri::command]
pub async fn thumbnail(app: tauri::AppHandle, path: String) -> Result<Response, String> {
    let Some(index) = app.state::<AppIndexReader>().0.clone() else {
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

/// The culling keymap resolved from the defaults and the `shortcuts` setting.
pub struct AppKeymap(pub Mutex<Keymap>);

/// Whether the selection moves to the next file after a judgement.
pub struct AppAutoAdvance(pub AtomicBool);

/// Serializes `switch_sidecar_format` calls, so two quick clicks cannot run
/// concurrent switches whose drain, save, state write and reset would
/// otherwise interleave.
pub struct AppSwitchLock(pub Mutex<()>);

/// The sidecar format currently selected, as stored under `sidecarFormat`
/// (`"xmp"` or `"dop"`), so the frontend knows whether a pick can be kept.
#[tauri::command]
pub fn sidecar_format(app: tauri::AppHandle) -> &'static str {
    index::lock(&app.state::<AppSidecarFormat>().0).setting()
}

/// Whether auto-advance is on.
#[tauri::command]
pub fn auto_advance(app: tauri::AppHandle) -> bool {
    app.state::<AppAutoAdvance>().0.load(Ordering::Relaxed)
}

/// Turn auto-advance on or off, persist it under `autoAdvance` and tell both
/// windows. A save failure is logged and the in-memory change stands.
#[tauri::command]
pub fn set_auto_advance(app: tauri::AppHandle, enabled: bool) {
    app.state::<AppAutoAdvance>()
        .0
        .store(enabled, Ordering::Relaxed);
    let saved = settings(&app).and_then(|store| {
        store.set("autoAdvance", enabled);
        store.save().map_err(|e| e.to_string())
    });
    if let Err(e) = saved {
        log::warn!("failed to save the auto-advance setting: {e}");
    }
    let _ = app.emit("auto-advance", enabled);
}

/// The resolved keymap, one binding per action in the order the shortcuts
/// panel shows them.
#[tauri::command]
pub fn shortcuts(app: tauri::AppHandle) -> Vec<Binding> {
    index::lock(&app.state::<AppKeymap>().0).bindings()
}

/// Add `key` to `action`'s keys and persist the overrides.
#[tauri::command]
pub fn add_shortcut_key(
    app: tauri::AppHandle,
    action: String,
    key: String,
) -> Result<Vec<Binding>, String> {
    update_keymap(&app, |keymap| keymap.add(&action, &key))
}

/// Remove `key` from `action`'s keys and persist the overrides.
#[tauri::command]
pub fn remove_shortcut_key(
    app: tauri::AppHandle,
    action: String,
    key: String,
) -> Result<Vec<Binding>, String> {
    update_keymap(&app, |keymap| keymap.remove(&action, &key))
}

/// Restore `action`'s default keys and persist the overrides.
#[tauri::command]
pub fn reset_shortcut(app: tauri::AppHandle, action: String) -> Result<Vec<Binding>, String> {
    update_keymap(&app, |keymap| keymap.reset(&action))
}

/// Restore every default key and drop the `shortcuts` setting.
#[tauri::command]
pub fn reset_shortcuts(app: tauri::AppHandle) -> Vec<Binding> {
    update_keymap(&app, |keymap| {
        keymap.reset_all();
        Ok(())
    })
    .unwrap_or_default()
}

/// The menu accelerators the keymap gives the two menu-backed actions.
fn accelerators(keymap: &Keymap) -> (Option<String>, Option<String>) {
    (
        keymap.accelerator_for("open"),
        keymap.accelerator_for("photolab"),
    )
}

/// Apply `change` to the keymap and save its overrides under `shortcuts`,
/// removing the key when there are none. A save failure is logged and the
/// in-memory change stands, as in `remember_folder`.
fn update_keymap(
    app: &tauri::AppHandle,
    change: impl FnOnce(&mut Keymap) -> Result<(), String>,
) -> Result<Vec<Binding>, String> {
    let state = app.state::<AppKeymap>();
    let mut keymap = index::lock(&state.0);
    let before = accelerators(&keymap);
    change(&mut keymap)?;
    if accelerators(&keymap) != before {
        if let Err(e) = crate::app_menu::refresh(app, &keymap) {
            log::warn!("failed to refresh the app menu: {e}");
        }
    }
    let overrides = keymap.overrides();
    let saved = settings(app).and_then(|store| {
        if overrides.as_object().is_some_and(|o| o.is_empty()) {
            store.delete("shortcuts");
        } else {
            store.set("shortcuts", overrides);
        }
        store.save().map_err(|e| e.to_string())
    });
    if let Err(e) = saved {
        log::warn!("failed to save the shortcuts: {e}");
    }
    let bindings = keymap.bindings();
    // The settings window edits the keymap; the main window culls with it.
    let _ = app.emit("shortcuts-changed", &bindings);
    Ok(bindings)
}

/// Record a judgement for one file: `-1` is a reject, `0` unrated and `1`-`5`
/// stars, plus PhotoLab's pick flag and the colour label beside them. A pick
/// is only kept while `.dop` is selected; with XMP it is dropped, as XMP has
/// nowhere to put it. The label is any raw name, kept under both formats; an
/// empty one is no label.
///
/// `label_known` is false when the frontend has not yet learned this path's
/// label from `folder_entries` (e.g. a judgement made before the first
/// refresh lands, or before the sidecar has even been parsed): `label` is
/// then ignored, the `ratings.label` column is left untouched rather than
/// being cleared, and the writer keeps whatever label the sidecar itself
/// currently holds instead of stripping it (see `Index::set_rating` and
/// `sidecar::write`).
///
/// The `ratings` row is written before the writer is told, so a crash between
/// the two still leaves the row dirty and the sidecar is written on the next
/// open of that folder. The command is `async` because it touches SQLite; the
/// frontend redraws without awaiting it.
#[tauri::command]
pub async fn set_rating(
    app: tauri::AppHandle,
    path: String,
    rating: i8,
    pick: bool,
    label: Option<String>,
    label_known: bool,
) -> Result<(), String> {
    if !(-1..=5).contains(&rating) {
        return Err(format!("rating {rating} is outside -1..=5"));
    }
    // 0 and "unrated" are the same state; the row keeps NULL and the sidecar
    // gets a `0` only when one already exists.
    let rating = Some(rating).filter(|r| *r != 0);
    let format = *index::lock(&app.state::<AppSidecarFormat>().0);
    let pick = pick && rating != Some(-1) && format == SidecarFormat::Dop;
    let Some(index) = app.state::<AppIndex>().0.clone() else {
        return Err("no index cache available".to_string());
    };
    let label = label.filter(|l| !l.is_empty());
    let dir = Path::new(&path)
        .parent()
        .map_or_else(String::new, |d| d.to_string_lossy().into_owned());
    {
        let (path, label) = (path.clone(), label.clone());
        tauri::async_runtime::spawn_blocking(move || {
            index::lock(&index).set_rating(&dir, &path, rating, pick, label.as_deref(), label_known)
        })
        .await
        .map_err(|e| e.to_string())??;
    }
    match &app.state::<AppWriter>().0 {
        Some(writer) => writer.set(
            PathBuf::from(path),
            rating,
            pick,
            label,
            label_known,
            format,
        ),
        None => Err("the sidecar writer is not running".to_string()),
    }
}

/// The base the size figure is divided by. The label mirrors what the user's
/// file manager shows next to the same file: Finder counts 1 GB as 1000^3,
/// Explorer (and the Linux file managers) as 1024^3, so the base follows the
/// platform rather than the labels changing to `GiB`.
const SIZE_BASE: u64 = if cfg!(target_os = "macos") {
    1000
} else {
    1024
};

/// The message shown when the button is pressed while a scan is running. The
/// clear is refused rather than cancelling the scan.
const SCAN_RUNNING: &str = "a scan is running; wait for it to finish";

/// Format `bytes` for display with plain `B`/`KB`/`MB`/`GB` labels: whole
/// bytes below `base`, one decimal above it. Steps up a unit once the
/// *rounded* value would reach `base` (e.g. 999_999_999 rounds to 1000.0 MB
/// at base 1000, so it must show as 1.0 GB, not 1000.0 MB), not just the raw
/// one, or a value one rounding step below a power of the base prints a
/// figure equal to the base.
fn format_bytes(bytes: u64, base: u64) -> String {
    if bytes < base {
        return format!("{bytes} B");
    }
    let base = base as f64;
    let round1 = |v: f64| (v * 10.0).round() / 10.0;
    let kb = bytes as f64 / base;
    if round1(kb) < base {
        return format!("{:.1} KB", kb);
    }
    let mb = kb / base;
    if round1(mb) < base {
        return format!("{:.1} MB", mb);
    }
    format!("{:.1} GB", mb / base)
}

/// The index's footprint on disk: the database file plus the `-wal` and
/// `-shm` files SQLite keeps beside it. A missing file counts as zero.
fn on_disk_bytes(path: &Path) -> u64 {
    [
        path.to_path_buf(),
        index::with_suffix(path, "-wal"),
        index::with_suffix(path, "-shm"),
    ]
    .iter()
    .map(|p| std::fs::metadata(p).map_or(0, |m| m.len()))
    .sum()
}

/// Whether a scan is in progress right now, for a settings window that opened
/// mid-scan; `scan-state` carries every change from then on.
#[tauri::command]
pub async fn scan_running(app: tauri::AppHandle) -> bool {
    let scans = app.state::<Scans>();
    let scanning = index::lock(&scans.0).scanning();
    scanning
}

/// The path the index is opened from, recomputed as `main.rs` does; the path
/// itself is not kept in any managed state.
fn index_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path()
        .app_cache_dir()
        .ok()
        .map(|dir| dir.join("index.sqlite"))
}

/// The index cache's size on disk, already formatted for display. No index
/// cache (or no file yet) is `"0 B"`, not an error: there is nothing to
/// report and nothing to clear.
#[tauri::command]
pub async fn index_size(app: tauri::AppHandle) -> Result<String, String> {
    let path = index_path(&app).filter(|_| app.state::<AppIndex>().0.is_some());
    let Some(path) = path else {
        return Ok(format_bytes(0, SIZE_BASE));
    };
    let bytes = tauri::async_runtime::spawn_blocking(move || on_disk_bytes(&path))
        .await
        .map_err(|e| e.to_string())?;
    Ok(format_bytes(bytes, SIZE_BASE))
}

/// Empty the index cache after a native confirmation; `false` when the user
/// cancelled. A running scan is refused before the dialog is shown, and again
/// under the `Scans` lock in case one started while the dialog was up.
///
/// Lock order is `Scans` then the writer, as in `spawn_eviction`, and the
/// clear runs in `spawn_blocking`: the drain, the `VACUUM` and the WAL
/// checkpoint all block.
#[tauri::command]
pub async fn clear_index(app: tauri::AppHandle) -> Result<bool, String> {
    {
        let scans = app.state::<Scans>();
        let state = index::lock(&scans.0);
        if state.scanning() {
            return Err(SCAN_RUNNING.to_string());
        }
    }
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    let mut dialog = app
        .dialog()
        .message(
            "Every cached thumbnail and the cached metadata of every folder are removed. \
             Judgements are kept. The next open of a folder scans it again.",
        )
        .title("Clear the index cache?")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Clear".to_string(),
            "Cancel".to_string(),
        ));
    if let Some(window) = app.get_webview_window("settings") {
        dialog = dialog.parent(&window);
    }
    dialog.show(move |confirmed| {
        let _ = tx.try_send(confirmed);
    });
    if !rx.recv().await.unwrap_or(false) {
        return Ok(false);
    }
    let _ = app.emit("index-clearing", ());
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        if let Some(writer) = &handle.state::<AppWriter>().0 {
            writer.flush(crate::sidecar::DRAIN_TIMEOUT);
        }
        let Some(index) = handle.state::<AppIndex>().0.clone() else {
            return Ok(());
        };
        let scans = handle.state::<Scans>();
        let state = index::lock(&scans.0);
        if state.scanning() {
            return Err(SCAN_RUNNING.to_string());
        }
        let summary = index::lock(&index).clear()?;
        log::info!("index cleared: {summary:?}");
        drop(state);
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;
    let _ = app.emit("index-cleared", ());
    Ok(true)
}

/// Move the rejected files of `dir` to the OS trash after a native
/// confirmation, together with the `.xmp` and `.dop` sidecars that exist on
/// disk for them; `None` when the user cancelled. `paths` is the frontend's
/// list of rejects and every path is validated against `dir` before anything
/// is moved.
///
/// A running scan is refused before the dialog and again under the `Scans`
/// lock, and the sidecar writer is drained first, as in `clear_index`: a
/// judgement still inside its debounce window would otherwise mint a sidecar
/// for a file that is already gone.
#[tauri::command]
pub async fn trash_rejected(
    app: tauri::AppHandle,
    dir: String,
    paths: Vec<String>,
) -> Result<Option<trash::Summary>, String> {
    {
        let scans = app.state::<Scans>();
        let state = index::lock(&scans.0);
        if state.scanning() {
            return Err(SCAN_RUNNING.to_string());
        }
    }
    if paths.is_empty() {
        return Err("no rejected files to move".to_string());
    }
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    let message = if paths.len() == 1 {
        "Move 1 rejected file to the Trash?".to_string()
    } else {
        format!("Move {} rejected files to the Trash?", paths.len())
    };
    let mut dialog = app
        .dialog()
        .message(message)
        .title("Move Rejected to Trash")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Move to Trash".to_string(),
            "Cancel".to_string(),
        ));
    if let Some(window) = app.get_webview_window("main") {
        dialog = dialog.parent(&window);
    }
    dialog.show(move |confirmed| {
        let _ = tx.try_send(confirmed);
    });
    if !rx.recv().await.unwrap_or(false) {
        return Ok(None);
    }
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<Option<trash::Summary>, String> {
        if let Some(writer) = &handle.state::<AppWriter>().0 {
            writer.flush(crate::sidecar::DRAIN_TIMEOUT);
        }
        let scans = handle.state::<Scans>();
        let state = index::lock(&scans.0);
        if state.scanning() {
            return Err(SCAN_RUNNING.to_string());
        }
        let dir = canonicalize(&dir);
        let groups = trash::plan(Path::new(&dir), &paths)?;
        let context = trash_context();
        let summary = trash::run(groups, |path| {
            context.delete(path).map_err(|e| e.to_string())
        });
        log::info!("moved rejected files to the trash: {summary:?}");
        drop(state);
        Ok(Some(summary))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The trash context the mover uses. On macOS the crate defaults to driving
/// the Finder through AppleScript, which needs Automation permission; the
/// `NSFileManager` route needs none.
fn trash_context() -> ::trash::TrashContext {
    #[allow(unused_mut)]
    let mut context = ::trash::TrashContext::default();
    #[cfg(target_os = "macos")]
    {
        use ::trash::macos::{DeleteMethod, TrashContextExtMacos};
        context.set_delete_method(DeleteMethod::NsFileManager);
    }
    context
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
    use super::{format_bytes, Scans, ScansState, SIZE_BASE};
    use crate::index;
    use std::sync::{atomic::AtomicBool, mpsc, Arc, Mutex};

    /// Spawn a task that clears its own entry once `go` fires, the way the
    /// real scan task does at the end of `start_scan`'s closure.
    fn spawn_scan(
        scans: &Arc<Scans>,
        scan_id: u64,
    ) -> (mpsc::Sender<()>, tauri::async_runtime::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let scans = scans.clone();
        let handle = tauri::async_runtime::spawn_blocking(move || {
            let _ = rx.recv();
            index::lock(&scans.0).finish(scan_id);
        });
        (tx, handle)
    }

    #[test]
    fn a_finished_scan_leaves_no_scan_in_progress() {
        let scans = Arc::new(Scans(Mutex::new(ScansState::default())));
        let (go, handle) = spawn_scan(&scans, 1);
        index::lock(&scans.0).running = Some((1, Arc::new(AtomicBool::new(false)), handle));
        let _ = go.send(());
        // Wait for the task the way `scan_folder` does, without taking the
        // entry: what is asserted is that the task cleared it itself.
        let waited = tauri::async_runtime::spawn_blocking({
            let scans = scans.clone();
            move || {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
                while index::lock(&scans.0).running.is_some()
                    && std::time::Instant::now() < deadline
                {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        });
        tauri::async_runtime::block_on(waited).unwrap();
        assert!(!index::lock(&scans.0).scanning());
    }

    #[test]
    fn a_superseded_scan_does_not_clear_the_newer_scan() {
        let scans = Arc::new(Scans(Mutex::new(ScansState::default())));
        let (old_go, old_handle) = spawn_scan(&scans, 1);
        let old_cancel = Arc::new(AtomicBool::new(false));
        index::lock(&scans.0).running = Some((1, old_cancel, old_handle));

        // `scan_folder` supersedes it: take the entry, cancel, join.
        let (_, old_cancel, old_handle) = index::lock(&scans.0).running.take().unwrap();
        old_cancel.store(true, std::sync::atomic::Ordering::Relaxed);

        // The newer scan is already stored when the old task gets to run.
        let (new_go, new_handle) = spawn_scan(&scans, 2);
        index::lock(&scans.0).running = Some((2, Arc::new(AtomicBool::new(false)), new_handle));

        let _ = old_go.send(());
        tauri::async_runtime::block_on(old_handle).unwrap();

        let state = index::lock(&scans.0);
        assert!(state.scanning());
        assert_eq!(state.running.as_ref().unwrap().0, 2);
        drop(state);
        let _ = new_go.send(());
    }

    #[test]
    fn a_folder_being_prepared_counts_as_a_scan_in_progress() {
        let mut state = ScansState::default();
        assert!(!state.scanning());
        state.preparing = 1;
        assert!(state.scanning());
    }

    #[test]
    fn format_bytes_uses_whole_bytes_below_the_base() {
        assert_eq!(format_bytes(0, 1000), "0 B");
        assert_eq!(format_bytes(0, 1024), "0 B");
        assert_eq!(format_bytes(999, 1000), "999 B");
        assert_eq!(format_bytes(1023, 1024), "1023 B");
    }

    #[test]
    fn format_bytes_steps_up_a_unit_at_each_power_of_the_base() {
        assert_eq!(format_bytes(1000, 1000), "1.0 KB");
        assert_eq!(format_bytes(1024, 1024), "1.0 KB");
        assert_eq!(format_bytes(1000 * 1000, 1000), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024, 1024), "1.0 MB");
        assert_eq!(format_bytes(1000 * 1000 * 1000, 1000), "1.0 GB");
        assert_eq!(format_bytes(1024 * 1024 * 1024, 1024), "1.0 GB");
    }

    #[test]
    fn format_bytes_shows_one_decimal_from_kb_up() {
        assert_eq!(format_bytes(1_200_000_000, 1000), "1.2 GB");
        assert_eq!(format_bytes(1288490189, 1024), "1.2 GB");
        assert_eq!(format_bytes(312_000, 1000), "312.0 KB");
    }

    #[test]
    fn format_bytes_steps_up_when_rounding_would_reach_the_base() {
        // Each of these divides to a rounded X.0 at the current unit, so it
        // must already step up rather than print e.g. "1000.0 MB".
        assert_eq!(format_bytes(999_999_999, 1000), "1.0 GB");
        assert_eq!(format_bytes(1_073_741_823, 1024), "1.0 GB");
        assert_eq!(format_bytes(999_999, 1000), "1.0 MB");
    }

    #[test]
    fn the_size_base_follows_the_platforms_file_manager() {
        // macOS Finder reports sizes in decimal units (1 GB = 1000^3 bytes);
        // Windows Explorer and most Linux file managers report binary units
        // (1 GB = 1024^3 bytes).
        let expected = if cfg!(target_os = "macos") {
            "1.1 GB"
        } else {
            "1.0 GB"
        };
        assert_eq!(format_bytes(1_073_741_824, SIZE_BASE), expected);
    }

    #[test]
    fn auto_advance_setting_reads_a_boolean_or_defaults_to_off() {
        use serde_json::json;
        assert!(!super::auto_advance_setting(None));
        assert!(super::auto_advance_setting(Some(&json!(true))));
        assert!(!super::auto_advance_setting(Some(&json!(false))));
        assert!(!super::auto_advance_setting(Some(&json!("true"))));
    }

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

    /// `reconcile_sidecars_of` over a fresh listing of `dir`, as `scan_folder`
    /// hands it the sidecars of its own single listing.
    fn reconcile_listed(
        dir: &str,
        listed: &[String],
        index: &Arc<Mutex<Index>>,
        format: SidecarFormat,
    ) -> Result<Vec<index::DirtyRow>, String> {
        reconcile_listed_with_errors(dir, listed, index, format).map(|(dirty, _)| dirty)
    }

    fn reconcile_listed_with_errors(
        dir: &str,
        listed: &[String],
        index: &Arc<Mutex<Index>>,
        format: SidecarFormat,
    ) -> Result<(Vec<index::DirtyRow>, Vec<SidecarError>), String> {
        let (_, sidecars) = list_folder_in(Path::new(dir), format)?;
        reconcile_sidecars_of(dir, listed, &sidecars, index, format)
    }

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
        let dirty = reconcile_listed(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();

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
            .set_rating(&dir, &listed[0], Some(5), false, None, true)
            .unwrap();
        sidecar(&root, "a.xmp", 3);

        let dirty = reconcile_listed(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();
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
        assert!(reconcile_listed(&dir, &listed, &index, SidecarFormat::Xmp)
            .unwrap()
            .is_empty());

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
        assert!(reconcile_listed(&dir, &listed, &index, SidecarFormat::Xmp)
            .unwrap()
            .is_empty());

        // `a` had a sidecar and lost it: the truth is gone with it. `b` has a
        // judgement that never reached a sidecar, and must be written instead.
        // `c` has neither a sidecar nor a row, and is not a case at all.
        std::fs::remove_file(&file).unwrap();
        index::lock(&index)
            .set_rating(&dir, &listed[1], Some(-1), false, None, true)
            .unwrap();

        let dirty = reconcile_listed(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();
        assert_eq!(dirty, [(listed[1].clone(), Some(-1), false, None, true)]);

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

        reconcile_listed(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();
        index::lock(&index)
            .write_batch(
                &dir,
                &[(index::stat(Path::new(&listed[0])).unwrap(), Err("x".into()))],
            )
            .unwrap();
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(5));

        remove_temp_dir(&root);
    }

    const PHOTOLAB_PICK: &[u8] = include_bytes!("../../core/src/fixtures/dop/_DSC0001.ARW.dop");
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

        let dirty = reconcile_listed(&dir, &listed, &index, SidecarFormat::Dop).unwrap();

        assert!(dirty.is_empty());
        index_files(&index, &dir, &listed);
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(-1));
        assert_eq!(rating_of(&index, &dir, &listed[1]), Some(3));
        assert_eq!(rating_of(&index, &dir, &listed[2]), None);

        remove_temp_dir(&root);
    }

    fn label_of(index: &Arc<Mutex<Index>>, dir: &str, path: &str) -> Option<String> {
        index::lock(index)
            .entries(dir)
            .unwrap()
            .into_iter()
            .find(|e| e.path == path)
            .unwrap()
            .label
    }

    #[test]
    fn photolab_labels_are_read_on_the_first_open() {
        let root = temp_dir("dop-labels");
        let dir = root.to_string_lossy().into_owned();
        let fixtures: [(&[u8], &str); 7] = [
            (
                include_bytes!("../../core/src/fixtures/dop/_DSC0009.ARW.dop"),
                "Red",
            ),
            (
                include_bytes!("../../core/src/fixtures/dop/_DSC0010.ARW.dop"),
                "Orange",
            ),
            (
                include_bytes!("../../core/src/fixtures/dop/_DSC0011.ARW.dop"),
                "Yellow",
            ),
            (
                include_bytes!("../../core/src/fixtures/dop/_DSC0012.ARW.dop"),
                "Green",
            ),
            (
                include_bytes!("../../core/src/fixtures/dop/_DSC0013.ARW.dop"),
                "Blue",
            ),
            (
                include_bytes!("../../core/src/fixtures/dop/_DSC0014.ARW.dop"),
                "Pink",
            ),
            (
                include_bytes!("../../core/src/fixtures/dop/_DSC0015.ARW.dop"),
                "Purple",
            ),
        ];
        for (i, (bytes, _)) in fixtures.iter().enumerate() {
            std::fs::write(root.join(format!("_DSC00{}.ARW", 9 + i)), b"x").unwrap();
            std::fs::write(root.join(format!("_DSC00{}.ARW.dop", 9 + i)), bytes).unwrap();
        }
        let index = sidecar_index(&root);
        let listed = list_arw_in(&root).unwrap();

        reconcile_listed(&dir, &listed, &index, SidecarFormat::Dop).unwrap();
        index_files(&index, &dir, &listed);

        for (i, (_, label)) in fixtures.iter().enumerate() {
            let path = root.join(format!("_DSC00{}.ARW", 9 + i));
            let path = listed
                .iter()
                .find(|p| Path::new(p).file_name() == path.file_name())
                .unwrap();
            assert_eq!(label_of(&index, &dir, path).as_deref(), Some(*label));
        }

        remove_temp_dir(&root);
    }

    #[test]
    fn an_xmp_label_is_read_and_a_deleted_sidecar_clears_it() {
        let root = temp_dir("xmp-label");
        let dir = root.to_string_lossy().into_owned();
        std::fs::write(root.join("a.ARW"), b"x").unwrap();
        let file = root.join("a.xmp");
        std::fs::write(
            &file,
            r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about="" xmlns:xmp="http://ns.adobe.com/xap/1.0/"
   xmp:Rating="2" xmp:Label="Blue"/>
 </rdf:RDF>
</x:xmpmeta>"#,
        )
        .unwrap();
        let index = sidecar_index(&root);
        let listed = list_arw_in(&root).unwrap();

        reconcile_listed(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();
        index_files(&index, &dir, &listed);
        assert_eq!(label_of(&index, &dir, &listed[0]).as_deref(), Some("Blue"));

        std::fs::remove_file(&file).unwrap();
        reconcile_listed(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();
        assert_eq!(label_of(&index, &dir, &listed[0]), None);

        remove_temp_dir(&root);
    }

    #[test]
    fn a_photolab_pick_is_read_with_dop_and_ignored_with_xmp() {
        let root = temp_dir("dop-pick");
        let dir = root.to_string_lossy().into_owned();
        for name in ["a.ARW", "b.ARW"] {
            std::fs::write(root.join(name), b"x").unwrap();
        }
        std::fs::write(root.join("a.ARW.dop"), PHOTOLAB_PICK).unwrap();
        std::fs::write(root.join("b.ARW.dop"), PHOTOLAB_THREE).unwrap();
        let index = sidecar_index(&root);
        let listed = list_arw_in(&root).unwrap();
        let picks = |index: &Arc<Mutex<Index>>| -> Vec<bool> {
            index::lock(index)
                .entries(&dir)
                .unwrap()
                .into_iter()
                .map(|e| e.pick)
                .collect()
        };

        reconcile_listed(&dir, &listed, &index, SidecarFormat::Dop).unwrap();
        index_files(&index, &dir, &listed);
        assert_eq!(picks(&index), [true, false]);
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(0));

        index::lock(&index).reset_sidecars().unwrap();
        reconcile_listed(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();
        assert_eq!(picks(&index), [false, false]);

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

        reconcile_listed(&dir, &listed, &index, SidecarFormat::Dop).unwrap();
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
            .set_rating(&dir, &listed[0], Some(5), false, None, true)
            .unwrap();

        let (dirty, problems) =
            reconcile_listed_with_errors(&dir, &listed, &index, SidecarFormat::Dop).unwrap();

        assert!(dirty.is_empty(), "the writer must not patch it unread");
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].path, listed[0]);
        assert!(problems[0].message.starts_with("a.ARW.dop: "));
        index_files(&index, &dir, &listed);
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(5));

        remove_temp_dir(&root);
    }

    #[test]
    fn an_unparseable_sidecar_is_reported_on_every_open_and_leaves_the_rating() {
        let root = temp_dir("sidecar-unparseable");
        let dir = root.to_string_lossy().into_owned();
        std::fs::write(root.join("a.ARW"), b"x").unwrap();
        let index = sidecar_index(&root);
        let listed = list_arw_in(&root).unwrap();
        index::lock(&index)
            .set_rating(&dir, &listed[0], Some(2), false, None, true)
            .unwrap();
        let xmp = root.join("a.xmp");
        std::fs::write(
            &xmp,
            r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about="" xmlns:xmp="http://ns.adobe.com/xap/1.0/" xmp:Rating="4"#,
        )
        .unwrap();
        for _ in 0..2 {
            let (_, problems) =
                reconcile_listed_with_errors(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();
            assert_eq!(problems.len(), 1);
            assert_eq!(problems[0].path, listed[0]);
            assert!(problems[0].message.starts_with("a.xmp: "));
        }
        index_files(&index, &dir, &listed);
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(2));

        remove_temp_dir(&root);
    }

    #[test]
    fn scan_started_serialises_the_fields_the_frontend_reads() {
        let started = ScanStarted {
            total: 3,
            scan_id: 7,
            sidecar_errors: vec![SidecarError {
                path: "/d/a.xmp".into(),
                message: "bad".into(),
            }],
        };
        assert_eq!(
            serde_json::to_value(&started).unwrap(),
            serde_json::json!({
                "total": 3,
                "scan_id": 7,
                "sidecar_errors": [{ "path": "/d/a.xmp", "message": "bad" }],
            })
        );
    }

    #[test]
    fn after_a_format_switch_only_the_selected_formats_ratings_are_read() {
        let root = temp_dir("format-switch");
        let dir = root.to_string_lossy().into_owned();
        std::fs::write(root.join("a.ARW"), b"x").unwrap();
        std::fs::write(root.join("b.ARW"), b"x").unwrap();
        sidecar(&root, "a.xmp", 4);
        std::fs::write(root.join("b.ARW.dop"), PHOTOLAB_THREE).unwrap();
        let index = sidecar_index(&root);
        let listed = list_arw_in(&root).unwrap();
        reconcile_listed(&dir, &listed, &index, SidecarFormat::Xmp).unwrap();
        index_files(&index, &dir, &listed);
        assert_eq!(rating_of(&index, &dir, &listed[0]), Some(4));
        assert_eq!(rating_of(&index, &dir, &listed[1]), None);

        index::lock(&index).reset_sidecars().unwrap();
        let dirty = reconcile_listed(&dir, &listed, &index, SidecarFormat::Dop).unwrap();

        assert!(dirty.is_empty());
        assert_eq!(rating_of(&index, &dir, &listed[0]), None);
        assert_eq!(rating_of(&index, &dir, &listed[1]), Some(3));

        remove_temp_dir(&root);
    }

    #[test]
    fn an_unknown_sort_order_falls_back_to_name() {
        assert_eq!(parse_sort_order(Some("capture")), "capture");
        assert_eq!(parse_sort_order(Some("rating")), "rating");
        assert_eq!(parse_sort_order(Some("size")), "name");
        assert_eq!(parse_sort_order(None), "name");
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
    fn one_listing_finds_the_raw_files_and_the_sidecars_of_each_format() {
        let dir = temp_dir("list-folder");
        for name in [
            "b.ARW",
            "a.arw",
            "c.DNG",
            "a.xmp",
            "B.XMP",
            "a.arw.dop",
            "C.DNG.DOP",
            "d.jpg",
        ] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        std::fs::create_dir(dir.join("sub.arw")).unwrap();

        for format in [SidecarFormat::Xmp, SidecarFormat::Dop] {
            let (files, sidecars) = list_folder_in(&dir, format).unwrap();
            assert_eq!(files, list_arw_in(&dir).unwrap());

            let expected: HashMap<String, SidecarStat> = std::fs::read_dir(&dir)
                .unwrap()
                .flatten()
                .filter(|e| format.matches(&e.file_name().to_string_lossy()))
                .map(|e| {
                    let stat = index::stat(&e.path()).unwrap();
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    (name, (stat.path, stat.size, stat.mtime_ns))
                })
                .collect();
            let mut names: Vec<&String> = sidecars.keys().collect();
            names.sort();
            match format {
                SidecarFormat::Xmp => assert_eq!(names, ["a.xmp", "b.xmp"]),
                SidecarFormat::Dop => assert_eq!(names, ["a.arw.dop", "c.dng.dop"]),
            }
            assert_eq!(sidecars, expected);
        }

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
                image_width: 6000,
                image_height: 4000,
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
        assert_eq!(u16at(0), CROP_KIND_RGBA_V3);
        assert_eq!(u16at(2), 8);
        assert_eq!(u32at(4), 2);
        assert_eq!(u32at(8), 1);
        assert_eq!(u32at(12), 7);
        assert_eq!(u32at(16), 9);
        assert_eq!(u32at(20), 731);
        assert_eq!(u32at(24), 21_500);
        assert_eq!(u16at(28), 6000);
        assert_eq!(u16at(30), 4000);
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
        assert_eq!((crop.crop.image_width, crop.crop.image_height), (400, 300));

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

    fn switch_writer(index: Arc<Mutex<Index>>) -> Writer {
        Writer::spawn(index, |path, message| {
            eprintln!("{}: {message}", path.display())
        })
    }

    /// A judgement made in the window between the format swap and
    /// `reset_sidecars` must land in the newly selected format and leave the
    /// row clean, not be replayed on the next open.
    #[test]
    fn a_rating_set_during_a_switch_lands_in_the_new_format_and_leaves_no_dirty_row() {
        let root = temp_dir("switch-race-rating");
        let dir = root.to_string_lossy().into_owned();
        std::fs::write(root.join("a.ARW"), b"x").unwrap();
        let index = sidecar_index(&root);
        let writer = switch_writer(index.clone());
        let listed = list_arw_in(&root).unwrap();
        let path = listed[0].clone();
        let current = Mutex::new(SidecarFormat::Xmp);
        let observed = Mutex::new(None);

        switch_format(
            &current,
            Some(&writer),
            Some(&index),
            SidecarFormat::Dop,
            |_| {
                let format = *index::lock(&current);
                *index::lock(&observed) = Some(format);
                index::lock(&index)
                    .set_rating(&dir, &path, Some(4), false, None, true)
                    .unwrap();
                writer
                    .set(PathBuf::from(&path), Some(4), false, None, true, format)
                    .unwrap();
                Ok(())
            },
        )
        .unwrap();
        writer.flush(crate::sidecar::DRAIN_TIMEOUT);

        assert_eq!(*index::lock(&observed), Some(SidecarFormat::Dop));
        let dop = SidecarFormat::Dop.sidecar_path(Path::new(&path));
        let bytes = std::fs::read(&dop).unwrap();
        assert_eq!(SidecarFormat::Dop.read_rating(&bytes).unwrap(), Some(4));
        assert!(!SidecarFormat::Xmp.sidecar_path(Path::new(&path)).exists());
        // `entries` joins `files`, which only a scan fills in.
        index::lock(&index)
            .write_batch(
                &dir,
                &[(index::stat(Path::new(&path)).unwrap(), Err("x".into()))],
            )
            .unwrap();
        assert_eq!(rating_of(&index, &dir, &path), Some(4));
        assert!(index::lock(&index).dirty_rows(&dir).unwrap().is_empty());

        remove_temp_dir(&root);
    }

    /// The same race with a pick: `reset_sidecars` must not zero the pick of
    /// the row the writer is about to confirm, or `mark_written`'s `pick`
    /// guard misses it and the next open replays it over the `.dop`.
    #[test]
    fn a_pick_set_during_a_switch_lands_in_the_dop_and_leaves_no_dirty_row() {
        let root = temp_dir("switch-race-pick");
        let dir = root.to_string_lossy().into_owned();
        std::fs::write(root.join("a.ARW"), b"x").unwrap();
        let index = sidecar_index(&root);
        let writer = switch_writer(index.clone());
        let listed = list_arw_in(&root).unwrap();
        let path = listed[0].clone();
        let current = Mutex::new(SidecarFormat::Xmp);

        switch_format(
            &current,
            Some(&writer),
            Some(&index),
            SidecarFormat::Dop,
            |_| {
                let format = *index::lock(&current);
                index::lock(&index)
                    .set_rating(&dir, &path, Some(4), true, None, true)
                    .unwrap();
                writer
                    .set(PathBuf::from(&path), Some(4), true, None, true, format)
                    .unwrap();
                Ok(())
            },
        )
        .unwrap();
        writer.flush(crate::sidecar::DRAIN_TIMEOUT);

        let dop = SidecarFormat::Dop.sidecar_path(Path::new(&path));
        let bytes = std::fs::read(&dop).unwrap();
        assert_eq!(SidecarFormat::Dop.read_rating(&bytes).unwrap(), Some(4));
        assert!(SidecarFormat::Dop.read_pick(&bytes).unwrap());
        assert!(!SidecarFormat::Xmp.sidecar_path(Path::new(&path)).exists());
        // `entries` joins `files`, which only a scan fills in.
        index::lock(&index)
            .write_batch(
                &dir,
                &[(index::stat(Path::new(&path)).unwrap(), Err("x".into()))],
            )
            .unwrap();
        let entry = index::lock(&index)
            .entries(&dir)
            .unwrap()
            .into_iter()
            .find(|e| e.path == path)
            .unwrap();
        assert_eq!(entry.rating, Some(4));
        assert!(entry.pick);
        assert!(index::lock(&index).dirty_rows(&dir).unwrap().is_empty());

        remove_temp_dir(&root);
    }
}
