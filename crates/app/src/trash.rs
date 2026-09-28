//! Moving the rejected files of folders to the OS trash, RAW first and its
//! sidecars after. The collection of the rejects and the per-file loop are
//! pure so they can be tested without touching the real Trash; only
//! `commands::trash_rejected` supplies the index rows and the mover that
//! calls into the `trash` crate.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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

/// The error `trash_rejected` returns when `dirs` hold no reject at all.
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
/// with the file.
pub fn run(groups: Vec<Group>, mut mover: impl FnMut(&Path) -> Result<(), String>) -> Summary {
    let mut summary = Summary::default();
    for group in groups {
        let raw = group.raw.to_string_lossy().into_owned();
        if let Err(message) = mover(&group.raw) {
            summary.failed.push(Failure { path: raw, message });
            continue;
        }
        summary.moved.push(raw);
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
    use super::{collect, nothing_to_trash, run, Collection, Group};
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
        // A leftover reject `.dop`, newer than the clean XMP row: PhotoLab (or
        // an earlier `Both` setting) could have left this behind. Under the
        // XMP setting the folder open never looks at it, so the collector
        // must not either.
        sidecar(&dir.join("a.ARW"), SidecarFormat::Dop, Flag::Reject);
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
        let summary = run(groups, mover(trash.clone()));
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
        let summary = run(groups, mover(trash.clone()));
        assert_eq!(summary.moved, [key(&dir.join("d.ARW"))]);
        assert_eq!(summary.failed.len(), 1);
        assert_eq!(summary.failed[0].path, key(&dir.join("fail.xmp")));
        assert!(trash.join("d.ARW").exists());
        assert!(trash.join("d.ARW.dop").exists());
    }
}
