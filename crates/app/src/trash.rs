//! Moving the rejected files of folders to the OS trash, RAW first and its
//! sidecars after. The collection of the rejects and the per-file loop are
//! pure so they can be tested without touching the real Trash; only
//! `commands::trash_rejected_preview` and `commands::trash_rejected_run`
//! supply the index rows and the mover that calls into the `trash` crate.
//! Each run that moved something is recorded in `Runs`, and `restore` puts
//! a recorded run back for `commands::trash_rejected_undo`, again through
//! closures so the tests never touch the real Trash. What an undo brought
//! back is kept in `Runs` too, and `redo` moves exactly those files to the
//! Trash again for `commands::trash_rejected_redo`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use riffle_core::Flag;
use serde::Serialize;

use crate::commands::{read_listing, stat_sidecars, MAX_SIDECAR_BYTES};
use crate::index::{RowFlag, SidecarStat};
use crate::sidecar::SidecarFormat;

/// One RAW and the sidecars that exist on disk for it, in the order they are
/// moved.
#[derive(Debug, PartialEq, Eq)]
pub struct Group {
    pub raw: PathBuf,
    pub sidecars: Vec<PathBuf>,
}

/// The rejects of one folder.
#[derive(Debug)]
pub struct FolderPlan {
    pub dir: String,
    pub groups: Vec<Group>,
}

/// What `collect` finds: every folder it read, in visiting order, and the
/// files and folders it could not tell the flag of.
#[derive(Debug, Default)]
pub struct Collection {
    pub folders: Vec<FolderPlan>,
    pub failed: Vec<Failure>,
}

impl Collection {
    /// How many RAWs are rejected over all the folders.
    pub fn count(&self) -> usize {
        self.folders.iter().map(|f| f.groups.len()).sum()
    }
}

/// One file or folder that could not be collected or moved.
#[derive(Debug, Serialize)]
pub struct Failure {
    pub path: String,
    pub message: String,
}

/// What `run` produces: the RAWs that went to the trash, the ones a move
/// attempt failed on, and the files or folders `collect` could not even read
/// (never attempted, so they must not count as a failed move).
#[derive(Debug, Default, Serialize)]
pub struct Summary {
    pub moved: Vec<String>,
    pub failed: Vec<Failure>,
    pub unread: Vec<Failure>,
    /// The recorded run `trash_rejected_undo` takes back, `None` when nothing
    /// moved.
    pub run_id: Option<u64>,
}

/// One file a run moved to the Trash: where it came from and, when the
/// platform tells, where it went (`None` on Windows / Linux, where the
/// Trash is listed at undo time instead).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Trashed {
    pub path: PathBuf,
    pub trashed_at: Option<PathBuf>,
    /// `(dev, ino)` of the file at `trashed_at`, taken right after the move
    /// (macOS only; `None` elsewhere or when the file could not be stat'd).
    /// The Trash can reuse a name once it is free, so `restore_recorded`
    /// checks this against the file it is about to rename back, not just
    /// that a file exists at `trashed_at`.
    pub trashed_id: Option<(u64, u64)>,
    /// Where an undo puts the file back when a folder above it was renamed
    /// since the run (`Runs::rename_dir`); `None` puts it back at `path`,
    /// which stays the key the Trash is looked up by.
    pub restore_to: Option<PathBuf>,
}

impl Trashed {
    /// Where an undo puts the file back.
    pub fn destination(&self) -> &Path {
        self.restore_to.as_deref().unwrap_or(&self.path)
    }
}

/// One recorded run: every file it moved, each RAW followed by its sidecars
/// in the order they were moved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrashRun {
    pub id: u64,
    pub moved: Vec<Trashed>,
}

/// How many runs `Runs` keeps, the frontend's undo history limit.
pub const MAX_RUNS: usize = 100;

/// The error `trash_rejected_undo` returns for a run `Runs` no longer holds.
pub const RUN_GONE: &str = "this run can no longer be undone";
/// The error `trash_rejected_redo` returns for an undone run `Runs` no longer
/// holds.
pub const UNDONE_GONE: &str = "this run can no longer be redone";
/// The failure of a file redo finds gone from where the undo put it back.
pub const GONE: &str = "no longer at its location";

/// The recorded runs, the newest `MAX_RUNS`, managed as app state, and as
/// many undone runs, each holding only the files its undo brought back. The
/// ids come from a counter, so a forgotten run's id is never handed out
/// again.
#[derive(Default)]
pub struct Runs(Mutex<RunsState>);

#[derive(Default)]
struct RunsState {
    next_id: u64,
    runs: Vec<TrashRun>,
    undone: Vec<(u64, Vec<PathBuf>)>,
}

impl Runs {
    /// Record a run that moved `moved`, dropping the oldest past `MAX_RUNS`.
    /// Nothing moved records nothing.
    pub fn record(&self, moved: Vec<Trashed>) -> Option<u64> {
        if moved.is_empty() {
            return None;
        }
        let mut state = crate::index::lock(&self.0);
        state.next_id += 1;
        let id = state.next_id;
        state.runs.push(TrashRun { id, moved });
        if state.runs.len() > MAX_RUNS {
            state.runs.remove(0);
        }
        Some(id)
    }

    /// Remove and return the run `id`: an undone run is forgotten, whatever
    /// the outcome of its restore.
    pub fn take(&self, id: u64) -> Option<TrashRun> {
        let mut state = crate::index::lock(&self.0);
        let at = state.runs.iter().position(|run| run.id == id)?;
        Some(state.runs.remove(at))
    }

    /// Keep the files the undo of run `id` brought back, in the recorded
    /// order, for its redo, dropping the oldest past `MAX_RUNS`. Nothing
    /// back keeps nothing.
    pub fn undone(&self, id: u64, back: Vec<PathBuf>) {
        if back.is_empty() {
            return;
        }
        let mut state = crate::index::lock(&self.0);
        state.undone.push((id, back));
        if state.undone.len() > MAX_RUNS {
            state.undone.remove(0);
        }
    }

    /// Remove and return the files the undo of run `id` brought back: a
    /// redone run is forgotten, whatever the outcome of its move.
    pub fn take_undone(&self, id: u64) -> Option<Vec<PathBuf>> {
        let mut state = crate::index::lock(&self.0);
        let at = state.undone.iter().position(|(undone, _)| *undone == id)?;
        Some(state.undone.remove(at).1)
    }

    /// Follow the rename of the folder `old` to `new` (both canonical, as
    /// `Index::rename_dir` takes them): a recorded file under `old` is put
    /// back under `new` by its undo, still looked up in the Trash by its
    /// original path, and an undone run's files, which sit on disk under
    /// the renamed folder now, are redone from there.
    pub fn rename_dir(&self, old: &str, new: &str) {
        if old == new {
            return;
        }
        let old_prefix = format!("{old}{}", std::path::MAIN_SEPARATOR);
        let new_prefix = format!("{new}{}", std::path::MAIN_SEPARATOR);
        let rebase = |path: &Path| {
            path.to_string_lossy()
                .strip_prefix(&old_prefix)
                .map(|rest| PathBuf::from(format!("{new_prefix}{rest}")))
        };
        let mut state = crate::index::lock(&self.0);
        let state = &mut *state;
        for trashed in state.runs.iter_mut().flat_map(|run| run.moved.iter_mut()) {
            if let Some(to) = rebase(trashed.destination()) {
                trashed.restore_to = (to != trashed.path).then_some(to);
            }
        }
        for path in state
            .undone
            .iter_mut()
            .flat_map(|(_, back)| back.iter_mut())
        {
            if let Some(to) = rebase(path) {
                *path = to;
            }
        }
    }
}

/// What `trash_rejected_undo` returns: the RAWs that came back and every
/// file, RAW or sidecar, that did not, with the reason.
#[derive(Debug, Default, Serialize)]
pub struct Restored {
    pub restored: Vec<String>,
    pub failed: Vec<Failure>,
    /// Every file, RAW or sidecar, that came back, in the recorded order:
    /// what a redo moves to the Trash again.
    #[serde(skip)]
    pub back: Vec<PathBuf>,
}

impl Restored {
    /// Every file of `run` failed with `message`, for a Trash that cannot be
    /// read or restored from at all.
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    pub fn none(run: &TrashRun, message: &str) -> Self {
        Restored {
            restored: Vec::new(),
            failed: run
                .moved
                .iter()
                .map(|trashed| Failure {
                    path: trashed.destination().to_string_lossy().into_owned(),
                    message: message.to_string(),
                })
                .collect(),
            back: Vec::new(),
        }
    }
}

/// One folder's row in the confirmation dialog: how many RAWs are rejected
/// there and the bytes they and their sidecars take.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct FolderCount {
    pub dir: String,
    pub count: usize,
    pub bytes: u64,
}

/// What `trash_rejected_preview` returns: every folder read, in visiting
/// order (those without rejects included, the dialog folds them into one
/// line), the files and folders that could not be read, and the totals, the
/// size already formatted for display.
#[derive(Debug, Serialize)]
pub struct Preview {
    pub folders: Vec<FolderCount>,
    pub failed: Vec<Failure>,
    pub total_files: usize,
    pub total_bytes: u64,
    pub size_text: String,
}

/// The preview of `collection`, `size_text` formatting the total bytes.
pub fn preview(collection: Collection, size_text: impl Fn(u64) -> String) -> Preview {
    let folders: Vec<FolderCount> = collection
        .folders
        .iter()
        .map(|folder| FolderCount {
            dir: folder.dir.clone(),
            count: folder.groups.len(),
            bytes: bytes(&folder.groups),
        })
        .collect();
    let total_files = folders.iter().map(|f| f.count).sum();
    let total_bytes = folders.iter().map(|f| f.bytes).sum();
    Preview {
        folders,
        failed: collection.failed,
        total_files,
        total_bytes,
        size_text: size_text(total_bytes),
    }
}

/// The bytes the RAWs of `groups` and their sidecars take on disk. A file
/// gone since the collection counts as zero.
fn bytes(groups: &[Group]) -> u64 {
    groups
        .iter()
        .flat_map(|group| std::iter::once(&group.raw).chain(&group.sidecars))
        .map(|path| std::fs::metadata(path).map_or(0, |m| m.len()))
        .sum()
}

/// The rejects of `dirs`, each followed by its subfolders depth-first by
/// name when `recursive`; a folder reached twice is read once. `rows` gives
/// the index rows of one folder, keyed by the same canonical spelling as
/// `dirs`. `format` is the configured sidecar format: the flag is decided
/// from its sidecars only, the same rule the folder open applies, so a
/// leftover sidecar of the other format cannot make the collector disagree
/// with what the strip shows. A folder that cannot be read is a failure of
/// that folder and the others still go on.
pub fn collect(
    dirs: &[String],
    recursive: bool,
    format: SidecarFormat,
    rows: impl Fn(&str) -> Result<HashMap<String, RowFlag>, String>,
) -> Collection {
    let mut collection = Collection::default();
    let mut seen = HashSet::new();
    for dir in dirs {
        visit(
            Path::new(dir),
            recursive,
            format,
            &rows,
            &mut seen,
            &mut collection,
        );
    }
    collection
}

fn visit(
    dir: &Path,
    recursive: bool,
    format: SidecarFormat,
    rows: &impl Fn(&str) -> Result<HashMap<String, RowFlag>, String>,
    seen: &mut HashSet<String>,
    collection: &mut Collection,
) {
    let key = dir.to_string_lossy().into_owned();
    if !seen.insert(key.clone()) {
        return;
    }
    match collect_folder(dir, &key, format, rows) {
        Ok((groups, failed)) => {
            collection.failed.extend(failed);
            collection.folders.push(FolderPlan { dir: key, groups });
        }
        Err(message) => {
            collection.failed.push(Failure { path: key, message });
            return;
        }
    }
    if recursive {
        for child in subfolders(dir) {
            visit(&child, true, format, rows, seen, collection);
        }
    }
}

/// The rejects of the RAWs listed directly in `dir`, grouped with every
/// sidecar of either format that exists for them (a format switch can leave
/// both on disk), and the files whose sidecar could not be read. The flag
/// itself is decided from `format`'s sidecars only, keeping in step with the
/// folder open; `Both` is used only to gather what gets moved with a reject.
/// A JPEG-only folder yields nothing.
fn collect_folder(
    dir: &Path,
    key: &str,
    format: SidecarFormat,
    rows: &impl Fn(&str) -> Result<HashMap<String, RowFlag>, String>,
) -> Result<(Vec<Group>, Vec<Failure>), String> {
    let listing = read_listing(dir, Some(SidecarFormat::Both))?;
    let raws: Vec<&String> = listing
        .files
        .iter()
        .filter(|path| riffle_core::scan::is_raw_file(Path::new(path)))
        .collect();
    let mut groups = Vec::new();
    let mut failed = Vec::new();
    if raws.is_empty() {
        return Ok((groups, failed));
    }
    let sidecars = stat_sidecars(&listing.sidecars);
    let rows = rows(key)?;
    for path in raws {
        let found = sidecars_of(SidecarFormat::Both, path, &sidecars);
        let effective = sidecars_of(format, path, &sidecars);
        let newest = crate::sidecar::newest(effective.iter().map(|stat| (*stat, stat.2)));
        match flag_of(rows.get(path), newest) {
            Ok(Flag::Reject) => groups.push(Group {
                raw: PathBuf::from(path),
                sidecars: found.iter().map(|stat| stat.0.clone()).collect(),
            }),
            Ok(_) => {}
            Err(message) => failed.push(Failure {
                path: path.clone(),
                message,
            }),
        }
    }
    Ok((groups, failed))
}

/// The sidecars of `format`'s kinds that exist for `path`, from the stats
/// keyed by lowercase file name.
fn sidecars_of<'a>(
    format: SidecarFormat,
    path: &str,
    sidecars: &'a HashMap<String, SidecarStat>,
) -> Vec<&'a SidecarStat> {
    format
        .kinds()
        .iter()
        .filter_map(|kind| {
            let name = kind
                .sidecar_path(Path::new(path))
                .file_name()?
                .to_string_lossy()
                .to_lowercase();
            sidecars.get(&name)
        })
        .collect()
}

/// The flag of one RAW, by the rule the folder open applies in
/// `reconcile_sidecars_of`: a dirty row (a judgment the writer has not
/// flushed) wins, a clean row is trusted while its stored stat is the newest
/// sidecar's, and otherwise the sidecar is read. A clean row whose sidecar is
/// gone has lost its judgment with it.
fn flag_of(row: Option<&RowFlag>, sidecar: Option<&SidecarStat>) -> Result<Flag, String> {
    match (row, sidecar) {
        (Some(row), _) if row.dirty => Ok(row.flag),
        (Some(row), Some((_, size, mtime_ns))) if row.stat == (Some(*size), Some(*mtime_ns)) => {
            Ok(row.flag)
        }
        (_, Some(sidecar)) => read_flag(sidecar),
        (Some(row), None) if row.stat == (None, None) => Ok(row.flag),
        _ => Ok(Flag::None),
    }
}

fn read_flag((sidecar, size, _): &SidecarStat) -> Result<Flag, String> {
    let name = sidecar
        .file_name()
        .map_or_else(|| sidecar.to_string_lossy(), |n| n.to_string_lossy());
    if *size > MAX_SIDECAR_BYTES {
        return Err(format!(
            "{name}: larger than {} MiB, not read",
            MAX_SIDECAR_BYTES / 1024 / 1024
        ));
    }
    let kind = if SidecarFormat::Dop.matches(&name) {
        SidecarFormat::Dop
    } else {
        SidecarFormat::Xmp
    };
    std::fs::read(sidecar)
        .map_err(|e| e.to_string())
        .and_then(|bytes| kind.read_flag(&bytes))
        .map_err(|e| format!("{name}: {e}"))
}

/// The visible subfolders of `dir` by the folder tree's rules (no dot
/// folder, no hidden one on Windows), sorted case-insensitively. A symlinked
/// folder is not followed, so a link back up the tree cannot loop. None when
/// `dir` cannot be read: its listing already reported that.
fn subfolders(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| {
            entry.file_type().is_ok_and(|t| t.is_dir())
                && !entry.file_name().to_string_lossy().starts_with('.')
                && !crate::folders::is_hidden(entry)
        })
        .map(|entry| entry.path())
        .collect();
    dirs.sort_by_cached_key(|p| {
        p.file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    });
    dirs
}

/// The error `trash_rejected_preview` returns when `dirs` hold no reject at all.
pub fn nothing_to_trash(dirs: &[String], recursive: bool) -> String {
    let [dir] = dirs else {
        return format!("No rejected files in {} folders", dirs.len());
    };
    let name = Path::new(dir)
        .file_name()
        .map_or_else(|| dir.clone(), |n| n.to_string_lossy().into_owned());
    if recursive {
        format!("No rejected files in {name} or its subfolders")
    } else {
        format!("No rejected files in {name}")
    }
}

/// Move every group with `mover`, never stopping at a failure. The RAW goes
/// first: when it fails its sidecars are left alone, so the judgment stays
/// with the file. `mover` returns where the file went when the platform
/// tells; every file moved is listed, in order, next to the summary.
pub fn run(
    groups: Vec<Group>,
    mut mover: impl FnMut(&Path) -> Result<Option<PathBuf>, String>,
) -> (Summary, Vec<Trashed>) {
    let mut summary = Summary::default();
    let mut moved = Vec::new();
    for group in groups {
        let raw = group.raw.to_string_lossy().into_owned();
        match mover(&group.raw) {
            Ok(trashed_at) => {
                let trashed_id = trashed_at.as_deref().and_then(file_id);
                moved.push(Trashed {
                    path: group.raw,
                    trashed_at,
                    trashed_id,
                    restore_to: None,
                });
            }
            Err(message) => {
                summary.failed.push(Failure { path: raw, message });
                continue;
            }
        }
        summary.moved.push(raw);
        for sidecar in group.sidecars {
            match mover(&sidecar) {
                Ok(trashed_at) => {
                    let trashed_id = trashed_at.as_deref().and_then(file_id);
                    moved.push(Trashed {
                        path: sidecar,
                        trashed_at,
                        trashed_id,
                        restore_to: None,
                    });
                }
                Err(message) => summary.failed.push(Failure {
                    path: sidecar.to_string_lossy().into_owned(),
                    message,
                }),
            }
        }
    }
    (summary, moved)
}

/// `(dev, ino)` of `path`, `None` when it cannot be stat'd or the platform is
/// not Unix. A rename within the same volume keeps this identity, but the
/// Trash reusing a freed name after emptying does not.
#[cfg(unix)]
fn file_id(path: &Path) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(path).ok()?;
    Some((metadata.dev(), metadata.ino()))
}

#[cfg(not(unix))]
fn file_id(_path: &Path) -> Option<(u64, u64)> {
    None
}

/// The failure of a file whose original location holds a file again.
pub const ALREADY_EXISTS: &str = "already exists at the original location";
/// The failure of a file the Trash no longer holds.
pub const NOT_IN_TRASH: &str = "not in the Trash (emptied or restored by hand)";
/// The failure of a sidecar left in the Trash because its RAW did not come
/// back.
pub const RAW_NOT_RESTORED: &str = "left in the Trash: its RAW could not be restored";

/// Put the files of `run` back in the recorded order, never stopping at a
/// failure and never overwriting: a file whose destination `exists` fails,
/// as does one `in_trash` finds no item for; otherwise `mover` restores the
/// item. The RAW goes first, and when it does not come back its sidecars stay
/// in the Trash (each reported), the mirror of `run`'s rule: a reject sidecar
/// restored next to a different file that took the RAW's name would pin the
/// judgment on that file.
pub fn restore<I>(
    run: &TrashRun,
    mut in_trash: impl FnMut(&Trashed) -> Option<I>,
    exists: impl Fn(&Path) -> bool,
    mut mover: impl FnMut(&Trashed, I) -> Result<(), String>,
) -> Restored {
    let mut restored = Restored::default();
    let mut raw_back = true;
    for trashed in &run.moved {
        let is_raw = riffle_core::scan::is_raw_file(&trashed.path);
        let destination = trashed.destination();
        let path = destination.to_string_lossy().into_owned();
        let result = if !is_raw && !raw_back {
            Err(RAW_NOT_RESTORED.to_string())
        } else if exists(destination) {
            Err(ALREADY_EXISTS.to_string())
        } else {
            match in_trash(trashed) {
                Some(item) => mover(trashed, item),
                None => Err(NOT_IN_TRASH.to_string()),
            }
        };
        if is_raw {
            raw_back = result.is_ok();
        }
        match result {
            Ok(()) => {
                restored.back.push(destination.to_path_buf());
                if is_raw {
                    restored.restored.push(path);
                }
            }
            Err(message) => restored.failed.push(Failure { path, message }),
        }
    }
    restored
}

/// Move `paths`, what an undo brought back (each RAW followed by its
/// sidecars), to the Trash again with `run`, so its rule holds: a sidecar
/// moves only after its RAW. No reject is collected again; a file that no
/// longer `exists` fails without reaching `mover`.
pub fn redo(
    paths: Vec<PathBuf>,
    exists: impl Fn(&Path) -> bool,
    mut mover: impl FnMut(&Path) -> Result<Option<PathBuf>, String>,
) -> (Summary, Vec<Trashed>) {
    let mut groups: Vec<Group> = Vec::new();
    for path in paths {
        match groups.last_mut() {
            Some(group) if !riffle_core::scan::is_raw_file(&path) => group.sidecars.push(path),
            _ => groups.push(Group {
                raw: path,
                sidecars: Vec::new(),
            }),
        }
    }
    run(groups, |path| {
        if exists(path) {
            mover(path)
        } else {
            Err(GONE.to_string())
        }
    })
}

/// Put `run` back from where each file went, as recorded at trash time
/// (macOS): a file whose recorded location is gone (the Trash was emptied or
/// the file put back by hand) is "not in the Trash", else it is renamed back,
/// within the volume its Trash lives on. `std::fs::rename` replaces an
/// existing file on Unix, so `restore`'s `exists` check is the only guard
/// against overwriting. A file existing at the recorded location is not
/// enough: the Trash can hand the same name to a different file once the
/// original is gone (the Trash emptied, then something else trashed under
/// the same name), so this also requires the file's `(dev, ino)` to still
/// match the one recorded right after the move.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn restore_recorded(run: &TrashRun) -> Restored {
    restore(
        run,
        |trashed| {
            let at = trashed.trashed_at.clone()?;
            if file_id(&at) != trashed.trashed_id {
                return None;
            }
            Some(at)
        },
        Path::exists,
        |trashed, at| std::fs::rename(at, trashed.destination()).map_err(|e| e.to_string()),
    )
}

/// Move a file the Trash put back at its original path `from` to `to`, where
/// a folder rename since the run moved its folder (Windows / Linux, whose
/// Trash only restores to the original path), then remove the folders the
/// Trash recreated on the way to `from`, each only while it is empty. A
/// failed move leaves the file at `from`.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub fn relocate(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::rename(from, to).map_err(|e| {
        format!(
            "put back at {} but not moved into the renamed folder: {e}",
            from.display()
        )
    })?;
    let mut dir = from.parent();
    while let Some(parent) = dir {
        if std::fs::remove_dir(parent).is_err() {
            break;
        }
        dir = parent.parent();
    }
    Ok(())
}

/// Move `path` to the Trash through `NSFileManager`, as the `trash` crate's
/// `DeleteMethod::NsFileManager` does (no Automation permission needed),
/// but keep the URL the file got inside the Trash, which the crate discards.
#[cfg(target_os = "macos")]
pub fn trash_file(path: &Path) -> Result<PathBuf, String> {
    use objc2_foundation::{NSFileManager, NSString, NSURL};

    let bytes = path.as_os_str().as_encoded_bytes();
    let path = match std::str::from_utf8(bytes) {
        Ok(utf8) => NSString::from_str(utf8),
        Err(_) => NSString::from_str(&percent_encode(bytes)),
    };
    let url = NSURL::fileURLWithPath(&path);
    let mut resulting = None;
    NSFileManager::defaultManager()
        .trashItemAtURL_resultingItemURL_error(&url, Some(&mut resulting))
        .map_err(|e| format!("`trashItemAtURL` failed: {e}"))?;
    resulting
        .and_then(|url| url.path())
        .map(|path| PathBuf::from(path.to_string()))
        .ok_or_else(|| "the Trash did not tell where the file went".to_string())
}

/// `input` with every byte that is not part of valid UTF-8 percent-encoded,
/// the `trash` crate's spelling of a non-UTF-8 path.
#[cfg(target_os = "macos")]
fn percent_encode(input: &[u8]) -> String {
    let mut encoded = String::with_capacity(input.len());
    for chunk in input.utf8_chunks() {
        encoded.push_str(chunk.valid());
        for byte in chunk.invalid() {
            encoded.push_str(percent_encoding::percent_encode_byte(*byte));
        }
    }
    encoded
}

/// `path` spelled the way the Trash's listing is matched against: without
/// the Windows verbatim prefix the app's canonical paths carry (the listing
/// has none) and, on Windows, lowercase, since its file names are
/// case-insensitive.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub fn trash_key(path: &Path) -> String {
    let path = path.to_string_lossy();
    let path = if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else {
        path.strip_prefix(r"\\?\").unwrap_or(&path).to_string()
    };
    if cfg!(windows) {
        path.to_lowercase()
    } else {
        path
    }
}

/// The items of the Trash keyed by `trash_key` of their original path, the
/// newest `time_deleted` winning when the same path was trashed more than
/// once.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub fn newest_by_path(items: Vec<::trash::TrashItem>) -> HashMap<String, ::trash::TrashItem> {
    let mut newest: HashMap<String, ::trash::TrashItem> = HashMap::new();
    for item in items {
        let key = trash_key(&item.original_path());
        if newest
            .get(&key)
            .is_none_or(|kept| kept.time_deleted < item.time_deleted)
        {
            newest.insert(key, item);
        }
    }
    newest
}

#[cfg(test)]
mod tests {
    use super::{
        bytes, collect, newest_by_path, nothing_to_trash, preview, redo, relocate, restore,
        restore_recorded, run, trash_key, Collection, FolderCount, Group, Restored, Runs, TrashRun,
        Trashed, ALREADY_EXISTS, GONE, MAX_RUNS, NOT_IN_TRASH, RAW_NOT_RESTORED,
    };
    use crate::index::RowFlag;
    use crate::sidecar::SidecarFormat;
    use riffle_core::Flag;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("riffle-trash-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path) {
        std::fs::write(path, b"x").unwrap();
    }

    fn key(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    /// A sidecar of `format` holding `flag`, next to `raw`, named as the app
    /// mints it.
    fn sidecar(raw: &Path, format: SidecarFormat, flag: Flag) -> PathBuf {
        let path = format.sidecar_path(raw);
        let bytes = format
            .write_rating(raw, None, None, flag, None, None)
            .unwrap();
        std::fs::write(&path, bytes).unwrap();
        path
    }

    fn stat(path: &Path) -> (Option<i64>, Option<i64>) {
        let stat = crate::index::stat(path).unwrap();
        (Some(stat.size), Some(stat.mtime_ns))
    }

    fn set_mtime(path: &Path, secs: u64) {
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs))
            .unwrap();
    }

    fn no_rows(_: &str) -> Result<HashMap<String, RowFlag>, String> {
        Ok(HashMap::new())
    }

    /// The rejected RAW file names of each folder, in collection order.
    fn names(collection: &Collection) -> Vec<(String, Vec<String>)> {
        collection
            .folders
            .iter()
            .map(|folder| {
                let name = Path::new(&folder.dir)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                let raws = folder
                    .groups
                    .iter()
                    .map(|g| g.raw.file_name().unwrap().to_string_lossy().into_owned())
                    .collect();
                (name, raws)
            })
            .collect()
    }

    /// A mover that renames into `trash` instead of the real Trash, and fails
    /// for any path whose file name contains `fail`.
    fn mover(trash: PathBuf) -> impl FnMut(&Path) -> Result<Option<PathBuf>, String> {
        move |path: &Path| {
            let name = path.file_name().unwrap();
            if name.to_string_lossy().contains("fail") {
                return Err("refused".to_string());
            }
            let to = trash.join(name);
            std::fs::rename(path, &to).map_err(|e| e.to_string())?;
            Ok(Some(to))
        }
    }

    fn trashed(path: &str) -> Trashed {
        Trashed {
            path: PathBuf::from(path),
            trashed_at: None,
            trashed_id: None,
            restore_to: None,
        }
    }

    fn trash_run(paths: &[&str]) -> TrashRun {
        TrashRun {
            id: 1,
            moved: paths.iter().map(|p| trashed(p)).collect(),
        }
    }

    /// Restore `run` against a Trash holding `in_trash` and a disk holding
    /// `on_disk`, recording the paths the mover was asked to restore.
    fn restore_with(
        run: &TrashRun,
        in_trash: &[&str],
        on_disk: &[&str],
    ) -> (Restored, Vec<String>) {
        let mut asked = Vec::new();
        let restored = restore(
            run,
            |t| {
                let path = t.path.to_string_lossy().into_owned();
                in_trash.contains(&path.as_str()).then_some(path)
            },
            |p| on_disk.contains(&p.to_string_lossy().as_ref()),
            |_, item| {
                asked.push(item);
                Ok(())
            },
        );
        (restored, asked)
    }

    fn failures(restored: &Restored) -> Vec<(&str, &str)> {
        restored
            .failed
            .iter()
            .map(|f| (f.path.as_str(), f.message.as_str()))
            .collect()
    }

    #[test]
    fn a_full_run_is_restored_raw_first() {
        let run = trash_run(&["a.ARW", "a.xmp", "a.ARW.dop", "b.DNG"]);
        let (restored, asked) = restore_with(&run, &["a.ARW", "a.xmp", "a.ARW.dop", "b.DNG"], &[]);
        assert_eq!(restored.restored, ["a.ARW", "b.DNG"]);
        assert!(restored.failed.is_empty());
        assert_eq!(asked, ["a.ARW", "a.xmp", "a.ARW.dop", "b.DNG"]);
        assert_eq!(
            restored.back,
            ["a.ARW", "a.xmp", "a.ARW.dop", "b.DNG"].map(PathBuf::from)
        );
    }

    #[test]
    fn only_what_came_back_is_kept_for_a_redo() {
        let run = trash_run(&["a.ARW", "a.xmp", "b.ARW", "b.xmp", "c.ARW", "c.xmp"]);
        let (restored, _) = restore_with(&run, &["a.ARW", "b.ARW", "b.xmp", "c.ARW"], &["a.ARW"]);
        assert_eq!(
            restored.back,
            ["b.ARW", "b.xmp", "c.ARW"].map(PathBuf::from)
        );
    }

    #[test]
    fn a_raw_back_already_keeps_its_sidecars_in_the_trash() {
        let run = trash_run(&["a.ARW", "a.xmp", "b.ARW", "b.xmp"]);
        let (restored, asked) =
            restore_with(&run, &["a.ARW", "a.xmp", "b.ARW", "b.xmp"], &["a.ARW"]);
        assert_eq!(restored.restored, ["b.ARW"]);
        assert_eq!(
            failures(&restored),
            [("a.ARW", ALREADY_EXISTS), ("a.xmp", RAW_NOT_RESTORED)]
        );
        assert_eq!(asked, ["b.ARW", "b.xmp"]);
    }

    #[test]
    fn a_raw_whose_restore_fails_keeps_its_sidecars_in_the_trash() {
        let run = trash_run(&["a.ARW", "a.xmp", "b.ARW"]);
        let mut asked = Vec::new();
        let restored = restore(
            &run,
            |t| Some(t.path.clone()),
            |_| false,
            |t, _| {
                asked.push(t.path.clone());
                if t.path == Path::new("a.ARW") {
                    Err("refused".to_string())
                } else {
                    Ok(())
                }
            },
        );
        assert_eq!(restored.restored, ["b.ARW"]);
        assert_eq!(
            failures(&restored),
            [("a.ARW", "refused"), ("a.xmp", RAW_NOT_RESTORED)]
        );
        assert_eq!(asked, [PathBuf::from("a.ARW"), PathBuf::from("b.ARW")]);
    }

    #[test]
    fn a_sidecar_missing_from_the_trash_is_reported_and_its_raw_comes_back() {
        let run = trash_run(&["a.ARW", "a.xmp", "a.ARW.dop"]);
        let (restored, asked) = restore_with(&run, &["a.ARW", "a.ARW.dop"], &[]);
        assert_eq!(restored.restored, ["a.ARW"]);
        assert_eq!(failures(&restored), [("a.xmp", NOT_IN_TRASH)]);
        assert_eq!(asked, ["a.ARW", "a.ARW.dop"]);
    }

    #[test]
    fn an_emptied_trash_reports_every_file() {
        let run = trash_run(&["a.ARW", "a.xmp", "b.ARW"]);
        let (restored, asked) = restore_with(&run, &[], &[]);
        assert!(restored.restored.is_empty());
        assert_eq!(
            failures(&restored),
            [
                ("a.ARW", NOT_IN_TRASH),
                ("a.xmp", RAW_NOT_RESTORED),
                ("b.ARW", NOT_IN_TRASH)
            ]
        );
        assert!(asked.is_empty());
    }

    #[test]
    fn an_unreadable_trash_fails_every_file() {
        let run = trash_run(&["a.ARW", "a.xmp"]);
        assert_eq!(
            failures(&Restored::none(&run, "no Trash")),
            [("a.ARW", "no Trash"), ("a.xmp", "no Trash")]
        );
    }

    #[test]
    fn a_recorded_run_is_renamed_back_never_over_a_file() {
        let dir = temp_dir("recorded");
        let trash = dir.join("trash");
        std::fs::create_dir(&trash).unwrap();
        let recorded = |name: &str| {
            let at = trash.join(name);
            write(&at);
            Trashed {
                path: dir.join(name),
                trashed_at: Some(at.clone()),
                trashed_id: super::file_id(&at),
                restore_to: None,
            }
        };
        let back = recorded("a.ARW");
        let back_sidecar = recorded("a.xmp");
        let taken = recorded("b.ARW");
        write(&taken.path);
        let emptied = Trashed {
            // Nothing is ever written at this path, so this covers the
            // Trash-emptied case: no file at `trashed_at` at all.
            path: dir.join("c.ARW"),
            trashed_at: Some(trash.join("c.ARW")),
            trashed_id: Some((0, 0)),
            restore_to: None,
        };
        let reused = Trashed {
            // A different file now occupies the recorded name (macOS Trash
            // name reuse), so this covers the id-mismatch case: something
            // exists at `trashed_at`, but its id does not match what was
            // recorded at trash time.
            path: dir.join("d.ARW"),
            trashed_at: Some(trash.join("d.ARW")),
            trashed_id: Some((0, 0)),
            restore_to: None,
        };
        write(&reused.trashed_at.clone().unwrap());
        let run = TrashRun {
            id: 1,
            moved: vec![
                back.clone(),
                back_sidecar.clone(),
                taken.clone(),
                emptied,
                reused.clone(),
            ],
        };
        let restored = restore_recorded(&run);
        assert_eq!(
            restored.restored,
            [back.path.to_string_lossy().into_owned()]
        );
        let failed: Vec<(String, &str)> = restored
            .failed
            .iter()
            .map(|f| (f.path.clone(), f.message.as_str()))
            .collect();
        assert_eq!(
            failed,
            [
                (taken.path.to_string_lossy().into_owned(), ALREADY_EXISTS),
                (
                    dir.join("c.ARW").to_string_lossy().into_owned(),
                    NOT_IN_TRASH
                ),
                (
                    dir.join("d.ARW").to_string_lossy().into_owned(),
                    NOT_IN_TRASH
                )
            ]
        );
        assert!(back.path.exists() && back_sidecar.path.exists());
        assert!(!back.trashed_at.unwrap().exists());
        assert!(taken.trashed_at.unwrap().exists());
        assert!(reused.trashed_at.unwrap().exists());
        assert!(!reused.path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn ns_file_manager_tells_where_the_file_went() {
        let dir = temp_dir("nsfilemanager");
        let file = dir.join("a.ARW");
        write(&file);
        let at = super::trash_file(&file).unwrap();
        assert!(!file.exists());
        assert!(at.exists());
        std::fs::rename(&at, &file).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn item(parent: &str, name: &str, time_deleted: i64) -> ::trash::TrashItem {
        ::trash::TrashItem {
            id: format!("{parent}/{name}@{time_deleted}").into(),
            name: name.into(),
            original_parent: PathBuf::from(parent),
            time_deleted,
        }
    }

    #[test]
    fn the_newest_item_of_a_path_wins() {
        let items = newest_by_path(vec![
            item("/photos", "a.ARW", 10),
            item("/photos", "a.ARW", 30),
            item("/photos", "a.ARW", 20),
            item("/photos", "b.ARW", 5),
        ]);
        assert_eq!(items.len(), 2);
        let a = &items[&trash_key(&Path::new("/photos").join("a.ARW"))];
        assert_eq!(a.time_deleted, 30);
    }

    #[cfg(not(windows))]
    #[test]
    fn a_plain_path_keys_as_it_is_spelled_off_windows() {
        assert_eq!(trash_key(Path::new("/photos/A.ARW")), "/photos/A.ARW");
    }

    #[cfg(windows)]
    #[test]
    fn a_verbatim_path_matches_the_listing_whatever_the_case() {
        assert_eq!(
            trash_key(Path::new(r"\\?\C:\Photos\A.ARW")),
            trash_key(Path::new(r"C:\photos\a.arw"))
        );
        assert_eq!(
            trash_key(Path::new(r"\\?\UNC\nas\share\A.ARW")),
            trash_key(Path::new(r"\\NAS\share\a.ARW"))
        );
        assert_eq!(trash_key(Path::new(r"\\?\C:\A.ARW")), r"c:\a.arw");
        let items = newest_by_path(vec![item(r"C:\Photos", "A.ARW", 1)]);
        assert!(items.contains_key(&trash_key(Path::new(r"\\?\C:\photos\a.ARW"))));
    }

    #[test]
    fn runs_keep_the_newest_and_forget_an_undone_one() {
        let runs = Runs::default();
        assert_eq!(runs.record(Vec::new()), None);
        let ids: Vec<u64> = (0..=MAX_RUNS)
            .map(|_| runs.record(vec![trashed("a.ARW")]).unwrap())
            .collect();
        assert_eq!(runs.take(ids[0]), None);
        let last = *ids.last().unwrap();
        assert_eq!(runs.take(ids[1]).unwrap().id, ids[1]);
        assert_eq!(runs.take(last).unwrap().moved, [trashed("a.ARW")]);
        assert_eq!(runs.take(last), None);
        assert!(runs.record(vec![trashed("b.ARW")]).unwrap() > last);
    }

    #[test]
    fn undone_runs_keep_the_newest_and_forget_a_redone_one() {
        let runs = Runs::default();
        runs.undone(1, Vec::new());
        assert_eq!(runs.take_undone(1), None);
        for id in 0..=MAX_RUNS as u64 {
            runs.undone(id, vec![PathBuf::from("a.ARW")]);
        }
        assert_eq!(runs.take_undone(0), None);
        assert_eq!(runs.take_undone(1), Some(vec![PathBuf::from("a.ARW")]));
        assert_eq!(runs.take_undone(1), None);
    }

    fn moved_to(path: &Path, to: Option<&Path>) -> Trashed {
        Trashed {
            restore_to: to.map(Path::to_path_buf),
            ..trashed(&key(path))
        }
    }

    #[test]
    fn a_folder_rename_moves_where_a_run_is_restored_to_but_not_its_trash_key() {
        let photos = Path::new("photos");
        let shoot = photos.join("shoot");
        let day1 = photos.join("day1");
        let day2 = photos.join("day2");
        let a = shoot.join("a.ARW");
        let b = shoot.join("sub").join("b.ARW");
        let sibling = photos.join("shoot2").join("c.ARW");
        let elsewhere = Path::new("other").join("d.ARW");
        let runs = Runs::default();
        let record = || {
            runs.record(
                [&a, &b, &sibling, &elsewhere]
                    .map(|path| trashed(&key(path)))
                    .to_vec(),
            )
            .unwrap()
        };
        let id = record();
        runs.rename_dir(&key(&shoot), &key(&shoot));
        runs.rename_dir(&key(&shoot), &key(&day1));
        assert_eq!(
            runs.take(id).unwrap().moved,
            [
                moved_to(&a, Some(&day1.join("a.ARW"))),
                moved_to(&b, Some(&day1.join("sub").join("b.ARW"))),
                moved_to(&sibling, None),
                moved_to(&elsewhere, None),
            ]
        );
        let id = record();
        runs.rename_dir(&key(&shoot), &key(&day1));
        runs.rename_dir(&key(&day1), &key(&day2));
        assert_eq!(
            runs.take(id).unwrap().moved[0],
            moved_to(&a, Some(&day2.join("a.ARW")))
        );
        let id = record();
        runs.rename_dir(&key(&shoot), &key(&day1));
        runs.rename_dir(&key(&day1), &key(&shoot));
        assert_eq!(runs.take(id).unwrap().moved[0], moved_to(&a, None));
    }

    #[test]
    fn a_folder_rename_moves_the_files_an_undo_brought_back() {
        let photos = Path::new("photos");
        let shoot = photos.join("shoot");
        let day1 = photos.join("day1");
        let runs = Runs::default();
        let back = vec![
            shoot.join("a.ARW"),
            shoot.join("a.xmp"),
            photos.join("shoot2").join("c.ARW"),
        ];
        runs.undone(1, back.clone());
        runs.undone(2, back.clone());
        runs.rename_dir(&key(&shoot), &key(&shoot));
        assert_eq!(runs.take_undone(2), Some(back.clone()));
        runs.rename_dir(&key(&shoot), &key(&day1));
        assert_eq!(
            runs.take_undone(1),
            Some(vec![
                day1.join("a.ARW"),
                day1.join("a.xmp"),
                photos.join("shoot2").join("c.ARW"),
            ])
        );
    }

    #[test]
    fn a_renamed_run_is_restored_to_its_destination() {
        let run = TrashRun {
            id: 1,
            moved: vec![
                moved_to(Path::new("a.ARW"), Some(Path::new("new/a.ARW"))),
                moved_to(Path::new("a.xmp"), Some(Path::new("new/a.xmp"))),
                moved_to(Path::new("b.ARW"), Some(Path::new("new/b.ARW"))),
            ],
        };
        let mut asked = Vec::new();
        let restored = restore(
            &run,
            |t| Some(key(&t.path)),
            |p| ["a.ARW", "new/b.ARW"].contains(&key(p).as_str()),
            |t, item| {
                asked.push((item, key(t.destination())));
                Ok(())
            },
        );
        assert_eq!(restored.restored, ["new/a.ARW"]);
        assert_eq!(failures(&restored), [("new/b.ARW", ALREADY_EXISTS)]);
        assert_eq!(restored.back, ["new/a.ARW", "new/a.xmp"].map(PathBuf::from));
        assert_eq!(
            asked,
            [
                ("a.ARW".to_string(), "new/a.ARW".to_string()),
                ("a.xmp".to_string(), "new/a.xmp".to_string())
            ]
        );
    }

    #[test]
    fn a_file_restored_under_the_old_name_is_moved_into_the_renamed_folder() {
        let dir = temp_dir("relocate");
        let from = dir.join("shoot").join("sub").join("a.ARW");
        let to = dir.join("day1").join("sub").join("a.ARW");
        std::fs::create_dir_all(from.parent().unwrap()).unwrap();
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        write(&from);
        relocate(&from, &to).unwrap();
        assert!(to.exists());
        assert!(!dir.join("shoot").exists());
        assert!(dir.exists());

        let from = dir.join("shoot").join("b.ARW");
        std::fs::create_dir_all(from.parent().unwrap()).unwrap();
        write(&from);
        assert!(relocate(&from, &dir.join("gone").join("b.ARW")).is_err());
        assert!(from.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn index_rows_alone_give_the_rejects() {
        let dir = temp_dir("rows");
        for name in ["a.ARW", "b.ARW", "c.ARW"] {
            write(&dir.join(name));
        }
        let rows = HashMap::from([
            (
                key(&dir.join("a.ARW")),
                RowFlag {
                    flag: Flag::Reject,
                    stat: (None, None),
                    dirty: false,
                },
            ),
            (
                key(&dir.join("b.ARW")),
                RowFlag {
                    flag: Flag::Pick,
                    stat: (None, None),
                    dirty: false,
                },
            ),
        ]);
        let wanted = key(&dir);
        let collection = collect(&[key(&dir)], false, SidecarFormat::Both, |d| {
            assert_eq!(d, wanted);
            Ok(rows.clone())
        });
        assert_eq!(
            names(&collection),
            [(
                dir.file_name().unwrap().to_string_lossy().into_owned(),
                vec!["a.ARW".to_string()]
            )]
        );
        assert!(collection.failed.is_empty());
    }

    #[test]
    fn sidecars_alone_give_the_rejects_of_both_formats() {
        let dir = temp_dir("sidecars");
        for name in ["a.ARW", "b.ARW", "c.ARW", "d.ARW"] {
            write(&dir.join(name));
        }
        sidecar(&dir.join("a.ARW"), SidecarFormat::Xmp, Flag::Reject);
        sidecar(&dir.join("b.ARW"), SidecarFormat::Dop, Flag::Reject);
        sidecar(&dir.join("c.ARW"), SidecarFormat::Xmp, Flag::Pick);
        let collection = collect(&[key(&dir)], false, SidecarFormat::Both, no_rows);
        assert_eq!(
            names(&collection)[0].1,
            ["a.ARW".to_string(), "b.ARW".to_string()]
        );
        let groups = &collection.folders[0].groups;
        assert_eq!(groups[0].sidecars, [dir.join("a.xmp")]);
        assert_eq!(groups[1].sidecars, [dir.join("b.ARW.dop")]);
    }

    #[test]
    fn a_reject_is_grouped_with_the_sidecars_of_both_formats() {
        let dir = temp_dir("group");
        write(&dir.join("a.ARW"));
        sidecar(&dir.join("a.ARW"), SidecarFormat::Xmp, Flag::None);
        std::fs::write(dir.join("A.ARW.DOP"), b"not read").unwrap();
        let rows = HashMap::from([(
            key(&dir.join("a.ARW")),
            RowFlag {
                flag: Flag::Reject,
                stat: (None, None),
                dirty: true,
            },
        )]);
        let collection = collect(&[key(&dir)], false, SidecarFormat::Both, |_| {
            Ok(rows.clone())
        });
        let [Group { raw, sidecars }] = &collection.folders[0].groups[..] else {
            panic!("expected one group, got {collection:?}");
        };
        assert_eq!(raw, &dir.join("a.ARW"));
        let mut names: Vec<_> = sidecars
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, ["A.ARW.DOP", "a.xmp"]);
    }

    #[test]
    fn a_sidecar_changed_since_a_clean_row_overrides_it() {
        let dir = temp_dir("stale");
        write(&dir.join("a.ARW"));
        write(&dir.join("b.ARW"));
        sidecar(&dir.join("a.ARW"), SidecarFormat::Xmp, Flag::Pick);
        sidecar(&dir.join("b.ARW"), SidecarFormat::Xmp, Flag::Reject);
        let rows = HashMap::from([
            (
                key(&dir.join("a.ARW")),
                RowFlag {
                    flag: Flag::Reject,
                    stat: (Some(1), Some(1)),
                    dirty: false,
                },
            ),
            (
                key(&dir.join("b.ARW")),
                RowFlag {
                    flag: Flag::None,
                    stat: (Some(1), Some(1)),
                    dirty: false,
                },
            ),
        ]);
        let collection = collect(&[key(&dir)], false, SidecarFormat::Both, |_| {
            Ok(rows.clone())
        });
        assert_eq!(names(&collection)[0].1, ["b.ARW".to_string()]);
    }

    #[test]
    fn a_stray_dop_newer_than_a_matching_xmp_row_is_ignored_under_the_xmp_setting() {
        let dir = temp_dir("stray-dop");
        write(&dir.join("a.ARW"));
        let xmp = sidecar(&dir.join("a.ARW"), SidecarFormat::Xmp, Flag::None);
        set_mtime(&xmp, 1_000);
        // A leftover reject `.dop`, newer than the clean XMP row: PhotoLab (or
        // an earlier `Both` setting) could have left this behind. Under the
        // XMP setting the folder open never looks at it, so the collector
        // must not either.
        let dop = sidecar(&dir.join("a.ARW"), SidecarFormat::Dop, Flag::Reject);
        set_mtime(&dop, 1_010);
        let rows = HashMap::from([(
            key(&dir.join("a.ARW")),
            RowFlag {
                flag: Flag::None,
                stat: stat(&xmp),
                dirty: false,
            },
        )]);
        let collection = collect(
            &[key(&dir)],
            false,
            SidecarFormat::Xmp,
            |_| Ok(rows.clone()),
        );
        assert!(collection.folders[0].groups.is_empty());

        // The same setup under `Both` is collected: the `.dop` is the newer
        // sidecar of the pair, so the format argument is what decides.
        let rows_both = HashMap::from([(
            key(&dir.join("a.ARW")),
            RowFlag {
                flag: Flag::None,
                stat: stat(&xmp),
                dirty: false,
            },
        )]);
        let collection_both = collect(&[key(&dir)], false, SidecarFormat::Both, |_| {
            Ok(rows_both.clone())
        });
        assert_eq!(collection_both.folders[0].groups.len(), 1);
    }

    #[test]
    fn a_clean_row_matching_its_sidecar_is_trusted_and_a_dirty_row_wins() {
        let dir = temp_dir("dirty");
        write(&dir.join("a.ARW"));
        write(&dir.join("b.ARW"));
        let a = sidecar(&dir.join("a.ARW"), SidecarFormat::Xmp, Flag::None);
        sidecar(&dir.join("b.ARW"), SidecarFormat::Xmp, Flag::None);
        let rows = HashMap::from([
            (
                key(&dir.join("a.ARW")),
                RowFlag {
                    flag: Flag::Reject,
                    stat: stat(&a),
                    dirty: false,
                },
            ),
            (
                key(&dir.join("b.ARW")),
                RowFlag {
                    flag: Flag::Reject,
                    stat: (Some(1), Some(1)),
                    dirty: true,
                },
            ),
        ]);
        let collection = collect(&[key(&dir)], false, SidecarFormat::Both, |_| {
            Ok(rows.clone())
        });
        assert_eq!(
            names(&collection)[0].1,
            ["a.ARW".to_string(), "b.ARW".to_string()]
        );
    }

    #[test]
    fn a_clean_row_whose_sidecar_is_gone_is_not_a_reject() {
        let dir = temp_dir("gone");
        write(&dir.join("a.ARW"));
        let rows = HashMap::from([(
            key(&dir.join("a.ARW")),
            RowFlag {
                flag: Flag::Reject,
                stat: (Some(1), Some(1)),
                dirty: false,
            },
        )]);
        let collection = collect(&[key(&dir)], false, SidecarFormat::Both, |_| {
            Ok(rows.clone())
        });
        assert!(collection.folders[0].groups.is_empty());
    }

    #[test]
    fn an_unparsable_sidecar_is_a_failure_not_a_reject() {
        let dir = temp_dir("unparsable");
        write(&dir.join("a.ARW"));
        std::fs::write(dir.join("a.ARW.dop"), b"{ not a dop").unwrap();
        let collection = collect(&[key(&dir)], false, SidecarFormat::Both, no_rows);
        assert!(collection.folders[0].groups.is_empty());
        assert_eq!(collection.failed.len(), 1);
        assert_eq!(collection.failed[0].path, key(&dir.join("a.ARW")));
    }

    #[test]
    fn a_jpeg_only_folder_yields_nothing() {
        let dir = temp_dir("jpeg");
        write(&dir.join("a.jpg"));
        let rows = HashMap::from([(
            key(&dir.join("a.jpg")),
            RowFlag {
                flag: Flag::Reject,
                stat: (None, None),
                dirty: true,
            },
        )]);
        let collection = collect(&[key(&dir)], false, SidecarFormat::Both, |_| {
            Ok(rows.clone())
        });
        assert_eq!(collection.count(), 0);
        assert!(collection.failed.is_empty());
    }

    #[test]
    fn recursion_finds_a_nested_reject_and_skips_a_dot_folder() {
        let root = temp_dir("recursive");
        let nested = root.join("B").join("inner");
        let dot = root.join(".cache");
        let other = root.join("a");
        for dir in [&nested, &dot, &other] {
            std::fs::create_dir_all(dir).unwrap();
        }
        for dir in [&nested, &dot] {
            write(&dir.join("x.ARW"));
            sidecar(&dir.join("x.ARW"), SidecarFormat::Xmp, Flag::Reject);
        }
        let collection = collect(&[key(&root)], true, SidecarFormat::Both, no_rows);
        let root_name = root.file_name().unwrap().to_string_lossy().into_owned();
        assert_eq!(
            names(&collection),
            [
                (root_name, vec![]),
                ("a".to_string(), vec![]),
                ("B".to_string(), vec![]),
                ("inner".to_string(), vec!["x.ARW".to_string()]),
            ]
        );
        let flat = collect(&[key(&root)], false, SidecarFormat::Both, no_rows);
        assert_eq!(flat.folders.len(), 1);
        assert_eq!(flat.count(), 0);
    }

    #[test]
    fn an_unreadable_folder_is_reported_and_the_rest_go_on() {
        let dir = temp_dir("unreadable");
        write(&dir.join("a.ARW"));
        sidecar(&dir.join("a.ARW"), SidecarFormat::Xmp, Flag::Reject);
        let missing = dir.join("missing");
        let collection = collect(
            &[key(&missing), key(&dir)],
            false,
            SidecarFormat::Both,
            no_rows,
        );
        assert_eq!(collection.failed.len(), 1);
        assert_eq!(collection.failed[0].path, key(&missing));
        assert_eq!(collection.folders.len(), 1);
        assert_eq!(collection.count(), 1);
    }

    #[test]
    fn an_index_failure_is_a_failure_of_that_folder() {
        let dir = temp_dir("index-failure");
        write(&dir.join("a.ARW"));
        let collection = collect(&[key(&dir)], false, SidecarFormat::Both, |_| {
            Err("locked".to_string())
        });
        assert!(collection.folders.is_empty());
        assert_eq!(collection.failed[0].path, key(&dir));
        assert_eq!(collection.failed[0].message, "locked");
    }

    #[test]
    fn the_byte_sum_counts_the_raw_and_both_sidecars_and_skips_a_missing_file() {
        let dir = temp_dir("bytes");
        std::fs::write(dir.join("a.ARW"), [0; 100]).unwrap();
        std::fs::write(dir.join("a.xmp"), [0; 20]).unwrap();
        std::fs::write(dir.join("a.ARW.dop"), [0; 3]).unwrap();
        let groups = [
            Group {
                raw: dir.join("a.ARW"),
                sidecars: vec![dir.join("a.xmp"), dir.join("a.ARW.dop")],
            },
            Group {
                raw: dir.join("gone.ARW"),
                sidecars: vec![dir.join("gone.xmp")],
            },
        ];
        assert_eq!(bytes(&groups), 123);
    }

    #[test]
    fn the_preview_counts_each_folder_as_the_collection_does() {
        let root = temp_dir("preview");
        let nested = root.join("b");
        let empty = root.join("a");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::create_dir_all(&empty).unwrap();
        let len = |path: &Path| std::fs::metadata(path).unwrap().len();
        let mut nested_bytes = 0;
        for name in ["x.ARW", "y.ARW"] {
            std::fs::write(nested.join(name), [0; 10]).unwrap();
            let xmp = sidecar(&nested.join(name), SidecarFormat::Xmp, Flag::Reject);
            nested_bytes += 10 + len(&xmp);
        }
        std::fs::write(root.join("z.ARW"), [0; 10]).unwrap();
        let xmp = sidecar(&root.join("z.ARW"), SidecarFormat::Xmp, Flag::Reject);
        let root_bytes = 10 + len(&xmp);
        let collection = collect(&[key(&root)], true, SidecarFormat::Both, no_rows);
        let counts: Vec<usize> = collection
            .folders
            .iter()
            .map(|folder| folder.groups.len())
            .collect();
        let preview = preview(collection, |b| format!("{b} bytes"));
        assert_eq!(
            preview.folders,
            [
                FolderCount {
                    dir: key(&root),
                    count: 1,
                    bytes: root_bytes,
                },
                FolderCount {
                    dir: key(&empty),
                    count: 0,
                    bytes: 0,
                },
                FolderCount {
                    dir: key(&nested),
                    count: 2,
                    bytes: nested_bytes,
                },
            ]
        );
        assert_eq!(
            preview.folders.iter().map(|f| f.count).collect::<Vec<_>>(),
            counts
        );
        assert_eq!(preview.total_files, 3);
        assert_eq!(preview.total_bytes, root_bytes + nested_bytes);
        assert_eq!(
            preview.size_text,
            format!("{} bytes", root_bytes + nested_bytes)
        );
        assert!(preview.failed.is_empty());
    }

    #[test]
    fn nothing_to_trash_names_the_folder_or_counts_them() {
        let one = ["/photos/2026".to_string()];
        assert_eq!(nothing_to_trash(&one, false), "No rejected files in 2026");
        assert_eq!(
            nothing_to_trash(&one, true),
            "No rejected files in 2026 or its subfolders"
        );
        let two = ["/a".to_string(), "/b".to_string()];
        assert_eq!(
            nothing_to_trash(&two, false),
            "No rejected files in 2 folders"
        );
    }

    #[test]
    fn a_failing_raw_keeps_its_sidecars_and_is_reported() {
        let dir = temp_dir("raw-failure");
        let trash = dir.join("trash");
        std::fs::create_dir(&trash).unwrap();
        for name in ["ok.ARW", "ok.xmp", "fail.ARW", "fail.xmp"] {
            write(&dir.join(name));
        }
        let groups = ["fail", "ok"]
            .into_iter()
            .map(|name| Group {
                raw: dir.join(format!("{name}.ARW")),
                sidecars: vec![dir.join(format!("{name}.xmp"))],
            })
            .collect();
        let (summary, moved) = run(groups, mover(trash.clone()));
        assert_eq!(
            moved,
            [
                Trashed {
                    path: dir.join("ok.ARW"),
                    trashed_at: Some(trash.join("ok.ARW")),
                    trashed_id: super::file_id(&trash.join("ok.ARW")),
                    restore_to: None,
                },
                Trashed {
                    path: dir.join("ok.xmp"),
                    trashed_at: Some(trash.join("ok.xmp")),
                    trashed_id: super::file_id(&trash.join("ok.xmp")),
                    restore_to: None,
                },
            ]
        );
        assert_eq!(summary.moved, [key(&dir.join("ok.ARW"))]);
        assert_eq!(summary.failed.len(), 1);
        assert_eq!(summary.failed[0].path, key(&dir.join("fail.ARW")));
        assert!(dir.join("fail.ARW").exists());
        assert!(dir.join("fail.xmp").exists());
        assert!(trash.join("ok.ARW").exists());
        assert!(trash.join("ok.xmp").exists());
    }

    #[test]
    fn a_failing_sidecar_is_reported_after_its_raw_went() {
        let dir = temp_dir("sidecar-failure");
        let trash = dir.join("trash");
        std::fs::create_dir(&trash).unwrap();
        write(&dir.join("d.ARW"));
        write(&dir.join("d.ARW.dop"));
        let groups = vec![Group {
            raw: dir.join("d.ARW"),
            sidecars: vec![dir.join("fail.xmp"), dir.join("d.ARW.dop")],
        }];
        let (summary, moved) = run(groups, mover(trash.clone()));
        assert_eq!(
            moved.iter().map(|t| t.path.clone()).collect::<Vec<_>>(),
            [dir.join("d.ARW"), dir.join("d.ARW.dop")]
        );
        assert_eq!(summary.moved, [key(&dir.join("d.ARW"))]);
        assert_eq!(summary.failed.len(), 1);
        assert_eq!(summary.failed[0].path, key(&dir.join("fail.xmp")));
        assert!(trash.join("d.ARW").exists());
        assert!(trash.join("d.ARW.dop").exists());
    }

    #[test]
    fn a_redo_reports_a_file_gone_meanwhile_and_moves_the_rest() {
        let dir = temp_dir("redo");
        let trash = dir.join("trash");
        std::fs::create_dir(&trash).unwrap();
        for name in ["a.ARW", "a.xmp", "c.ARW", "c.ARW.dop"] {
            write(&dir.join(name));
        }
        let paths = ["a.ARW", "a.xmp", "b.ARW", "b.xmp", "c.ARW", "c.ARW.dop"]
            .map(|name| dir.join(name))
            .to_vec();
        let (summary, moved) = redo(paths, Path::exists, mover(trash.clone()));
        assert_eq!(
            summary.moved,
            [key(&dir.join("a.ARW")), key(&dir.join("c.ARW"))]
        );
        assert_eq!(summary.failed.len(), 1);
        assert_eq!(summary.failed[0].path, key(&dir.join("b.ARW")));
        assert_eq!(summary.failed[0].message, GONE);
        assert_eq!(
            moved.iter().map(|t| t.path.clone()).collect::<Vec<_>>(),
            ["a.ARW", "a.xmp", "c.ARW", "c.ARW.dop"].map(|name| dir.join(name))
        );
        for name in ["a.ARW", "a.xmp", "c.ARW", "c.ARW.dop"] {
            assert!(trash.join(name).exists());
            assert!(!dir.join(name).exists());
        }
    }
}
