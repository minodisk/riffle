//! The folder tree's listing commands: the top-level roots, and one folder's
//! subfolders with its own RAW count.

use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, serde::Serialize)]
pub struct FolderNode {
    name: String,
    path: String,
}

#[derive(Debug, serde::Serialize)]
pub struct Folder {
    raw_count: usize,
    children: Vec<FolderNode>,
}

#[tauri::command]
pub async fn folder_roots(app: AppHandle) -> Result<Vec<FolderNode>, String> {
    let home = app.path().home_dir().ok();
    tauri::async_runtime::spawn_blocking(move || roots(home))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_subfolders(dir: String) -> Result<Folder, String> {
    tauri::async_runtime::spawn_blocking(move || list(Path::new(&dir)))
        .await
        .map_err(|e| e.to_string())?
}

/// The home directory, then the mounted volumes. A volume that resolves to
/// the home directory (a plain duplicate), or on macOS to one of its
/// ancestors (`/Volumes/Macintosh HD` is a link to `/`, an ancestor of
/// home), is dropped, since home already stands for it. On other platforms
/// only the duplicate is dropped: Windows' `C:\` is a real, independently
/// browsable root even when home lives under it, unlike macOS's volume
/// alias.
fn roots(home: Option<PathBuf>) -> Vec<FolderNode> {
    let home = home.and_then(|h| std::fs::canonicalize(&h).ok().map(|c| (h, c)));
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut nodes = Vec::new();
    if let Some((path, canonical)) = &home {
        seen.push(canonical.clone());
        nodes.push(node(path));
    }
    for path in volumes() {
        let Ok(canonical) = std::fs::canonicalize(&path) else {
            continue;
        };
        if seen.contains(&canonical) || is_ancestor_alias(&home, &canonical) {
            continue;
        }
        seen.push(canonical);
        nodes.push(node(&path));
    }
    nodes
}

#[cfg(target_os = "macos")]
fn is_ancestor_alias(home: &Option<(PathBuf, PathBuf)>, canonical: &Path) -> bool {
    home.as_ref().is_some_and(|(_, h)| h.starts_with(canonical))
}

#[cfg(not(target_os = "macos"))]
fn is_ancestor_alias(_home: &Option<(PathBuf, PathBuf)>, _canonical: &Path) -> bool {
    false
}

#[cfg(target_os = "macos")]
fn volumes() -> Vec<PathBuf> {
    child_dirs(Path::new("/Volumes"))
}

#[cfg(target_os = "windows")]
fn volumes() -> Vec<PathBuf> {
    (b'A'..=b'Z')
        .map(|letter| PathBuf::from(format!("{}:\\", letter as char)))
        .filter(|p| p.exists())
        .collect()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn volumes() -> Vec<PathBuf> {
    let mut volumes = child_dirs(Path::new("/mnt"));
    for parent in ["/media", "/run/media"] {
        for dir in child_dirs(Path::new(parent)) {
            volumes.extend(child_dirs(&dir));
        }
    }
    volumes
}

/// The directories directly under `dir`, sorted by name; none when it cannot
/// be read.
#[cfg(not(target_os = "windows"))]
fn child_dirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// One `read_dir` of `dir`: how many RAW files it holds, and its visible
/// subfolders sorted case-insensitively. Children are not probed for RAW
/// files, which would cost one more listing each (slow on a network share).
fn list(dir: &Path) -> Result<Folder, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut raw_count = 0;
    let mut children = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some((is_dir, is_file)) = kind(&entry) else {
            continue;
        };
        if is_dir {
            if !entry.file_name().to_string_lossy().starts_with('.') && !is_hidden(&entry) {
                children.push(node(&path));
            }
        } else if is_file && riffle_core::scan::is_raw_file(&path) {
            raw_count += 1;
        }
    }
    children.sort_by_cached_key(|c| c.name.to_lowercase());
    Ok(Folder {
        raw_count,
        children,
    })
}

/// Whether `entry` is a directory and whether it is a file, following a
/// symlink; `None` if that cannot be told (a broken link, say).
/// `file_type()` does not follow symlinks, so a symlinked folder or RAW file
/// is reported as neither a dir nor a file. Fall back to one `metadata()`
/// call (which does follow the link) only in that rare case, keeping the
/// common path free of a per-entry stat: `file_type()` comes with the
/// directory listing on Windows.
fn kind(entry: &DirEntry) -> Option<(bool, bool)> {
    let file_type = entry.file_type().ok()?;
    if file_type.is_symlink() {
        let m = std::fs::metadata(entry.path()).ok()?;
        Some((m.is_dir(), m.is_file()))
    } else {
        Some((file_type.is_dir(), file_type.is_file()))
    }
}

/// Whether `entry` is a file, following a symlink, as `kind` tells it.
pub(crate) fn is_file(entry: &DirEntry) -> bool {
    kind(entry).is_some_and(|(_, is_file)| is_file)
}

#[cfg(target_os = "windows")]
fn is_hidden(entry: &DirEntry) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    entry
        .metadata()
        .is_ok_and(|m| m.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0)
}

#[cfg(not(target_os = "windows"))]
fn is_hidden(_: &DirEntry) -> bool {
    false
}

/// A drive root such as `C:\` has no file name, so it is named by its path.
fn node(path: &Path) -> FolderNode {
    let path_string = path.to_string_lossy().into_owned();
    let name = path
        .file_name()
        .map_or_else(|| path_string.clone(), |n| n.to_string_lossy().into_owned());
    FolderNode {
        name,
        path: path_string,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("riffle-folders-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn children_are_sorted_case_insensitively_without_dot_dirs() {
        let dir = temp_dir("children");
        for name in ["beta", "Alpha", ".hidden", "gamma"] {
            std::fs::create_dir(dir.join(name)).unwrap();
        }
        std::fs::write(dir.join("Delta"), b"").unwrap();
        let folder = list(&dir).unwrap();
        let names: Vec<&str> = folder.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["Alpha", "beta", "gamma"]);
        assert_eq!(folder.children[0].path, dir.join("Alpha").to_string_lossy());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn raw_count_counts_only_arw_and_dng_files() {
        let dir = temp_dir("raw-count");
        for name in ["a.ARW", "b.arw", "c.DNG", "d.jpg", "a.xmp", "e.dop"] {
            std::fs::write(dir.join(name), b"").unwrap();
        }
        std::fs::create_dir(dir.join("f.ARW")).unwrap();
        let folder = list(&dir).unwrap();
        assert_eq!(folder.raw_count, 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_dir_is_an_error() {
        let dir = temp_dir("missing").join("nope");
        let err = list(&dir).unwrap_err();
        assert!(err.starts_with(&dir.display().to_string()), "{err}");
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_dir_and_raw_file_are_counted() {
        let dir = temp_dir("symlinks");
        let target_dir = temp_dir("symlinks-target-dir");
        std::os::unix::fs::symlink(&target_dir, dir.join("linked-dir")).unwrap();
        let target_file = temp_dir("symlinks-target-file").join("photo.ARW");
        std::fs::write(&target_file, b"").unwrap();
        std::os::unix::fs::symlink(&target_file, dir.join("linked.ARW")).unwrap();

        let folder = list(&dir).unwrap();
        let names: Vec<&str> = folder.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["linked-dir"]);
        assert_eq!(folder.raw_count, 1);

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&target_dir);
        let _ = std::fs::remove_dir_all(target_file.parent().unwrap());
    }

    #[test]
    fn roots_start_with_home_and_all_exist() {
        let home = temp_dir("home");
        let roots = roots(Some(home.clone()));
        assert_eq!(roots[0].path, home.to_string_lossy());
        assert!(roots.iter().all(|r| Path::new(&r.path).exists()));
        let _ = std::fs::remove_dir_all(&home);
    }
}
