//! Moving the rejected files of a folder to the OS trash, RAW first and its
//! sidecars after. The planning and the per-file loop are pure so they can be
//! tested without touching the real Trash; only `commands::trash_rejected`
//! supplies the mover that calls into the `trash` crate.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::sidecar::{existing_sidecar, SidecarFormat};

/// One RAW and the sidecars that exist on disk for it, in the order they are
/// moved.
#[derive(Debug, PartialEq, Eq)]
pub struct Group {
    pub raw: PathBuf,
    pub sidecars: Vec<PathBuf>,
}

/// One file that could not be moved.
#[derive(Debug, Serialize)]
pub struct Failure {
    pub path: String,
    pub message: String,
}

/// What `run` produces: how many RAWs went to the trash and everything that
/// did not.
#[derive(Debug, Default, Serialize)]
pub struct Summary {
    pub trashed: usize,
    pub failed: Vec<Failure>,
}

/// Group the RAWs of `dir` with their sidecars, refusing anything that is not
/// a RAW directly inside `dir`. Both sidecar formats are collected whatever
/// the current setting: a format switch can leave both on disk, and a sidecar
/// whose RAW is gone is garbage.
pub fn plan(dir: &Path, paths: &[String]) -> Result<Vec<Group>, String> {
    let dir = dir.to_string_lossy();
    paths
        .iter()
        .map(|path| {
            let raw = PathBuf::from(path);
            if !riffle_core::scan::is_raw_file(&raw) {
                return Err(format!("not a RAW file: {path}"));
            }
            if raw.parent().map(|p| p.to_string_lossy()) != Some(dir.clone()) {
                return Err(format!("not in the open folder: {path}"));
            }
            let sidecars = [SidecarFormat::Xmp, SidecarFormat::Dop]
                .into_iter()
                .filter_map(|format| existing_sidecar(&raw, format))
                .collect();
            Ok(Group { raw, sidecars })
        })
        .collect()
}

/// Move every group with `mover`, never stopping at a failure. The RAW goes
/// first: when it fails its sidecars are left alone, so the judgement stays
/// with the file.
pub fn run(groups: Vec<Group>, mut mover: impl FnMut(&Path) -> Result<(), String>) -> Summary {
    let mut summary = Summary::default();
    for group in groups {
        if let Err(message) = mover(&group.raw) {
            summary.failed.push(Failure {
                path: group.raw.to_string_lossy().into_owned(),
                message,
            });
            continue;
        }
        summary.trashed += 1;
        for sidecar in &group.sidecars {
            if let Err(message) = mover(sidecar) {
                summary.failed.push(Failure {
                    path: sidecar.to_string_lossy().into_owned(),
                    message,
                });
            }
        }
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::{plan, run, Group};
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

    /// A mover that renames into `trash` instead of the real Trash, and fails
    /// for any path whose file name contains `fail`.
    fn mover(trash: PathBuf) -> impl FnMut(&Path) -> Result<(), String> {
        move |path: &Path| {
            let name = path.file_name().unwrap();
            if name.to_string_lossy().contains("fail") {
                return Err("refused".to_string());
            }
            std::fs::rename(path, trash.join(name)).map_err(|e| e.to_string())
        }
    }

    #[test]
    fn a_raw_is_grouped_with_the_sidecars_of_both_formats() {
        let dir = temp_dir("group");
        write(&dir.join("a.ARW"));
        write(&dir.join("a.xmp"));
        write(&dir.join("A.ARW.DOP"));
        let groups = plan(&dir, &[dir.join("a.ARW").to_string_lossy().into_owned()]).unwrap();
        let [Group { raw, sidecars }] = &groups[..] else {
            panic!("expected one group, got {groups:?}");
        };
        assert_eq!(raw, &dir.join("a.ARW"));
        let mut names: Vec<_> = sidecars
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        // On a case-insensitive file system `exists()` already matches the
        // `A.ARW.DOP` spelling through the minted `a.ARW.dop`, so only the
        // name's case may differ here.
        assert_eq!(names.len(), 2);
        assert!(names[0].eq_ignore_ascii_case("a.ARW.dop"));
        assert_eq!(names[1], "a.xmp");
    }

    #[test]
    fn a_path_outside_the_folder_or_not_a_raw_is_refused() {
        let dir = temp_dir("refuse");
        let outside = dir.parent().unwrap().join("b.ARW");
        write(&outside);
        write(&dir.join("c.jpg"));
        assert!(plan(&dir, &[outside.to_string_lossy().into_owned()]).is_err());
        assert!(plan(&dir, &[dir.join("c.jpg").to_string_lossy().into_owned()]).is_err());
    }

    #[test]
    fn a_failing_raw_keeps_its_sidecars_and_is_reported() {
        let dir = temp_dir("raw-failure");
        let trash = dir.join("trash");
        std::fs::create_dir(&trash).unwrap();
        for name in ["ok.ARW", "ok.xmp", "fail.ARW", "fail.xmp"] {
            write(&dir.join(name));
        }
        let paths = ["fail.ARW", "ok.ARW"]
            .into_iter()
            .map(|name| dir.join(name).to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let summary = run(plan(&dir, &paths).unwrap(), mover(trash.clone()));
        assert_eq!(summary.trashed, 1);
        assert_eq!(summary.failed.len(), 1);
        assert_eq!(
            summary.failed[0].path,
            dir.join("fail.ARW").to_string_lossy()
        );
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
        let summary = run(groups, mover(trash.clone()));
        assert_eq!(summary.trashed, 1);
        assert_eq!(summary.failed.len(), 1);
        assert_eq!(
            summary.failed[0].path,
            dir.join("fail.xmp").to_string_lossy()
        );
        assert!(trash.join("d.ARW").exists());
        assert!(trash.join("d.ARW.dop").exists());
    }
}
