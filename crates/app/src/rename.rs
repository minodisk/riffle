//! Renaming a folder from the folder tree. The name check and the target
//! planning are pure so they can be tested on temp dirs; `rename_folder`
//! releases the folder watcher, renames on disk and carries the index rows
//! along, so nothing is re-extracted and no judgment is lost.

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::Manager;

use crate::commands::{AppIndex, AppWriter, Scans, SCAN_RUNNING};
use crate::index;

/// What a rename produced: the new path, spelled the caller's way, and a
/// warning when the disk rename succeeded but something after it (carrying
/// the index rows along) did not.
#[derive(Debug, Serialize)]
pub struct Renamed {
    pub path: String,
    pub warning: Option<String>,
}

/// Refuse a name that is not a single valid file name component. The
/// characters Windows forbids are refused on every platform, so a name typed
/// on macOS still copies to an NTFS card.
pub fn check_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("the name is empty".to_string());
    }
    if name == "." || name == ".." {
        return Err(format!("{name}: not a valid name"));
    }
    if let Some(c) = name.chars().find(|c| {
        matches!(c, '/' | '\\' | '<' | '>' | ':' | '"' | '|' | '?' | '*') || c.is_control()
    }) {
        return Err(format!("{name}: a name cannot contain {c:?}"));
    }
    if name.ends_with('.') || name.ends_with(' ') {
        return Err(format!("{name}: a name cannot end with a dot or a space"));
    }
    Ok(())
}

/// The path `dir` is renamed to: `name` in `dir`'s parent. Refuses a `dir`
/// with no parent (a drive root or `/`), an invalid name, and a target that
/// exists as another entry. A target that canonicalizes to `dir` itself is
/// `dir` spelled in another case on a case-insensitive file system, which is
/// a case-only rename, not a collision.
pub fn folder_target(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let parent = dir
        .parent()
        .ok_or_else(|| format!("{}: a root cannot be renamed", dir.display()))?;
    check_name(name)?;
    let target = parent.join(name);
    if target.symlink_metadata().is_ok() {
        match (std::fs::canonicalize(&target), std::fs::canonicalize(dir)) {
            (Ok(a), Ok(b)) if a == b => {}
            _ => return Err(format!("{name} already exists")),
        }
    }
    Ok(target)
}

/// Rename the folder `dir` to `name`, refusing while a scan runs. The sidecar
/// writer is drained first and the `Scans` lock held across the rename and the
/// index write, as in `trash_rejected`. The watcher is released before the
/// rename (Windows refuses to rename a watched folder or its parent); the
/// frontend's reopen under the new path sets it again. A failed rename
/// changes nothing, and puts the released watch back since there is no reopen
/// to set it again; a failed index write after a successful rename still
/// returns `Ok` (the disk is the source of truth, so the caller must rebase
/// and reopen under the new path) with a warning, and only costs a
/// re-extraction on the next open.
#[tauri::command]
pub async fn rename_folder(
    app: tauri::AppHandle,
    dir: String,
    name: String,
) -> Result<Renamed, String> {
    if index::lock(&app.state::<Scans>().0).scanning() {
        return Err(SCAN_RUNNING.to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let target = folder_target(Path::new(&dir), &name)?;
        if let Some(writer) = &app.state::<AppWriter>().0 {
            writer.flush(crate::sidecar::DRAIN_TIMEOUT);
        }
        let scans = app.state::<Scans>();
        let state = index::lock(&scans.0);
        if state.scanning() {
            return Err(SCAN_RUNNING.to_string());
        }
        let old = std::fs::canonicalize(&dir)
            .map_err(|e| format!("{dir}: {e}"))?
            .to_string_lossy()
            .into_owned();
        let released = crate::watch::release_under(&app, &old);
        if let Err(e) = std::fs::rename(&dir, &target) {
            if let Some((dir, owner)) = released {
                crate::watch::set(&app, &dir, &owner);
            }
            return Err(format!("{name}: {e}"));
        }
        let new = std::fs::canonicalize(&target).map_or_else(
            |_| target.to_string_lossy().into_owned(),
            |p| p.to_string_lossy().into_owned(),
        );
        let mut warning = None;
        if let Some(index) = app.state::<AppIndex>().0.clone() {
            match index::lock(&index).rename_dir(&old, &new) {
                Ok(moved) => log::info!("renamed folder: {old} -> {new} files={moved}"),
                Err(e) => {
                    log::warn!("renamed folder {old} -> {new} but not its index rows: {e}");
                    warning = Some(format!(
                        "the folder was renamed but its cache was not ({e}); \
                         it is rebuilt on the next open"
                    ));
                }
            }
        }
        drop(state);
        Ok(Renamed {
            path: target.to_string_lossy().into_owned(),
            warning,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::{check_name, folder_target};
    use std::path::{Path, PathBuf};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("riffle-rename-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn check_name_accepts_an_ordinary_name() {
        assert!(check_name("2026-09-28 Tokyo").is_ok());
        assert!(check_name("a.b").is_ok());
        assert!(check_name(".hidden").is_ok());
    }

    #[test]
    fn check_name_refuses_what_a_file_system_would() {
        for name in [
            "", ".", "..", "a/b", "a\\b", "a<b", "a>b", "a:b", "a\"b", "a|b", "a?b", "a*b", "a\tb",
            "a\u{0}b", "a.", "a ",
        ] {
            assert!(check_name(name).is_err(), "{name:?} was accepted");
        }
    }

    #[test]
    fn folder_target_joins_the_name_to_the_parent() {
        let root = temp_dir("target");
        let dir = root.join("photos");
        std::fs::create_dir(&dir).unwrap();
        assert_eq!(folder_target(&dir, "shoot").unwrap(), root.join("shoot"));
        assert!(folder_target(&dir, "a/b").is_err());
    }

    #[test]
    fn folder_target_refuses_an_existing_entry() {
        let root = temp_dir("collision");
        let dir = root.join("photos");
        std::fs::create_dir(&dir).unwrap();
        std::fs::create_dir(root.join("taken")).unwrap();
        std::fs::write(root.join("file"), b"x").unwrap();
        assert!(folder_target(&dir, "taken").is_err());
        assert!(folder_target(&dir, "file").is_err());
    }

    #[test]
    fn folder_target_allows_a_case_only_rename() {
        let root = temp_dir("case");
        let dir = root.join("photos");
        std::fs::create_dir(&dir).unwrap();
        let target = folder_target(&dir, "Photos").unwrap();
        std::fs::rename(&dir, &target).unwrap();
        // On a case-insensitive file system the result may keep either
        // spelling as far as `exists()` is concerned; compare canonically.
        assert_eq!(
            std::fs::canonicalize(&target).unwrap(),
            std::fs::canonicalize(root.join("Photos")).unwrap()
        );
    }

    #[test]
    fn folder_target_refuses_a_root() {
        let root = if cfg!(windows) {
            Path::new("C:\\")
        } else {
            Path::new("/")
        };
        assert!(folder_target(root, "x").is_err());
    }
}
