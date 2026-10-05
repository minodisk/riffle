//! The folder tree's `Rewrite Sidecars from Index…`: the commands that count
//! and then rewrite the sidecars of one folder from its `ratings` rows.
//!
//! Every write goes through the sidecar writer, which patches only the
//! judgment fields of an existing sidecar (so Lightroom's develop settings
//! and PhotoLab's corrections survive) and mints one only for a row holding
//! a judgment. Only the current `SidecarFormat`'s kinds are written.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use riffle_core::xmp::LabelNames;
use riffle_core::Flag;
use serde::Serialize;
use tauri::Manager;

use crate::commands::{
    canonicalize, read_listing, AppIndex, AppLabelNames, AppSidecarFormat, AppWriter, Scans,
    SCAN_RUNNING,
};
use crate::index::{self, DirtyRow, Index};
use crate::sidecar::{SidecarFormat, Writer, DRAIN_TIMEOUT};

/// Longest a rewrite waits for the writer to drain. `DRAIN_TIMEOUT` is for
/// quit; a folder of thousands of files on Windows takes minutes (each write
/// is a read, a temp write, `sync_all` and a rename with retries).
const REWRITE_DRAIN: Duration = Duration::from_secs(10 * 60);

/// What `rewrite_sidecars_preview` counts for the confirmation dialog: the
/// rows with a judgment, the rows without one (whose existing sidecar is
/// patched to "no judgment"), the listed RAWs with no row, which are
/// skipped, and the format written (`SidecarFormat::setting`).
#[derive(Debug, PartialEq, Serialize)]
pub struct Preview {
    judged: usize,
    unjudged: usize,
    skipped: usize,
    format: &'static str,
}

/// What `rewrite_sidecars_run` reports: the files handed to the writer whose
/// rows came back clean, those still dirty after the drain, and whether the
/// drain finished within `REWRITE_DRAIN`. A row still dirty is retried on the
/// next open of the folder; one a judgment re-dirtied during the drain is
/// counted as failed too.
#[derive(Debug, PartialEq, Serialize)]
pub struct Summary {
    written: usize,
    failed: usize,
    flushed: bool,
}

fn folder_name(dir: &str) -> String {
    Path::new(dir)
        .file_name()
        .map_or_else(|| dir.to_string(), |n| n.to_string_lossy().into_owned())
}

fn judged((_, rating, flag, label, _): &DirtyRow) -> bool {
    rating.is_some_and(|r| r != 0) || *flag != Flag::None || label.is_some()
}

/// The rows of `dir` to rewrite, those of the RAWs the folder lists, and the
/// number of listed RAWs without a row. A row whose RAW is no longer listed
/// is left out, so no sidecar is minted next to a file that is gone. A folder
/// listing no RAW (a JPEG folder is view-only) and one with no row for any
/// listed RAW are refused.
fn targets(dir: &str, index: &Arc<Mutex<Index>>) -> Result<(Vec<DirtyRow>, usize), String> {
    let name = folder_name(dir);
    let listing = read_listing(Path::new(dir), None)?;
    let raws: HashSet<&str> = listing
        .files
        .iter()
        .filter(|path| riffle_core::scan::is_raw_file(Path::new(path)))
        .map(String::as_str)
        .collect();
    if raws.is_empty() {
        return Err(format!(
            "{name}: rewriting sidecars applies to RAW files only"
        ));
    }
    let rows: Vec<DirtyRow> = index::lock(index)
        .rows_of(dir)?
        .into_iter()
        .filter(|(path, _, _, _, _)| raws.contains(path.as_str()))
        .collect();
    if rows.is_empty() {
        return Err(format!(
            "No judgments indexed for {name}; open the folder first"
        ));
    }
    let skipped = raws.len() - rows.len();
    Ok((rows, skipped))
}

/// The counts of `targets` for `dir`, which is canonical.
fn preview(dir: &str, index: &Arc<Mutex<Index>>, format: SidecarFormat) -> Result<Preview, String> {
    let (rows, skipped) = targets(dir, index)?;
    let judged = rows.iter().filter(|row| judged(row)).count();
    Ok(Preview {
        judged,
        unjudged: rows.len() - judged,
        skipped,
        format: format.setting(),
    })
}

/// Mark the rows of `targets` dirty, hand each to `writer` with no debounce
/// and wait for it to drain, at most `bound`. `dir` is canonical.
fn rewrite(
    dir: &str,
    index: &Arc<Mutex<Index>>,
    writer: &Writer,
    format: SidecarFormat,
    names: &LabelNames,
    bound: Duration,
) -> Result<Summary, String> {
    let (rows, _) = targets(dir, index)?;
    let paths: Vec<String> = rows.into_iter().map(|(path, ..)| path).collect();
    let dirty = index::lock(index).mark_dirty(dir, &paths)?;
    let total = dirty.len();
    for (path, rating, flag, label, label_known) in dirty {
        writer.set_now(
            PathBuf::from(path),
            rating,
            flag,
            label,
            label_known,
            format,
            names.clone(),
        )?;
    }
    let flushed = writer.flush(bound);
    let wanted: HashSet<&str> = paths.iter().map(String::as_str).collect();
    let failed = index::lock(index)
        .dirty_rows(dir)?
        .iter()
        .filter(|(path, _, _, _, _)| wanted.contains(path.as_str()))
        .count();
    Ok(Summary {
        written: total.saturating_sub(failed),
        failed,
        flushed,
    })
}

/// Count what `rewrite_sidecars_run` would write in `dir`, for the
/// confirmation dialog. A running scan is refused and the sidecar writer is
/// drained first, so the rows read are the flushed ones. The `Scans` lock is
/// not held: the dialog stays open for as long as the user wants.
#[tauri::command]
pub async fn rewrite_sidecars_preview(
    app: tauri::AppHandle,
    dir: String,
) -> Result<Preview, String> {
    {
        let scans = app.state::<Scans>();
        let state = index::lock(&scans.0);
        if state.scanning() {
            return Err(SCAN_RUNNING.to_string());
        }
    }
    tauri::async_runtime::spawn_blocking(move || -> Result<Preview, String> {
        if let Some(writer) = &app.state::<AppWriter>().0 {
            writer.flush(DRAIN_TIMEOUT);
        }
        let Some(index) = app.state::<AppIndex>().0.clone() else {
            return Err("no index cache available".to_string());
        };
        let format = *index::lock(&app.state::<AppSidecarFormat>().0);
        preview(&canonicalize(&dir), &index, format)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Rewrite the sidecars of `dir` from its `ratings` rows once the dialog
/// `rewrite_sidecars_preview` fed was confirmed. The writer is drained first,
/// as in `trash_rejected_run`, and a running scan is refused under the
/// `Scans` lock, which is held across the index write and the drain.
#[tauri::command]
pub async fn rewrite_sidecars_run(app: tauri::AppHandle, dir: String) -> Result<Summary, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<Summary, String> {
        let Some(writer) = &app.state::<AppWriter>().0 else {
            return Err("the sidecar writer is not running".to_string());
        };
        writer.flush(DRAIN_TIMEOUT);
        let Some(index) = app.state::<AppIndex>().0.clone() else {
            return Err("no index cache available".to_string());
        };
        let scans = app.state::<Scans>();
        let state = index::lock(&scans.0);
        if state.scanning() {
            return Err(SCAN_RUNNING.to_string());
        }
        let format = *index::lock(&app.state::<AppSidecarFormat>().0);
        let names = index::lock(&app.state::<AppLabelNames>().0).clone();
        let dir = canonicalize(&dir);
        let summary = rewrite(&dir, &index, writer, format, &names, REWRITE_DRAIN);
        log::info!("rewrote the sidecars of {dir}: {summary:?}");
        drop(state);
        summary
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use riffle_core::{dop, xmp};

    const PHOTOLAB: &[u8] = include_bytes!("../../core/src/fixtures/dop/_DSC0003.ARW.dop");

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "riffle-foldersidecars-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    struct Fixture {
        dir: PathBuf,
        key: String,
        index: Arc<Mutex<Index>>,
        writer: Writer,
        errors: Arc<Mutex<Vec<String>>>,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let dir = temp_dir(name);
            let index = Arc::new(Mutex::new(Index::open(&dir.join("index.sqlite")).unwrap()));
            let errors = Arc::new(Mutex::new(Vec::new()));
            let sink = errors.clone();
            let writer = Writer::spawn(index.clone(), move |path, message| {
                index::lock(&sink).push(format!("{}: {message}", path.display()));
            });
            Self {
                key: dir.to_string_lossy().into_owned(),
                dir,
                index,
                writer,
                errors,
            }
        }

        fn raw(&self, name: &str) -> PathBuf {
            let path = self.dir.join(name);
            std::fs::write(&path, b"not really an ARW").unwrap();
            path
        }

        /// A clean row, as a folder open stores what it parsed.
        fn row(&self, path: &Path, rating: Option<i8>, flag: Flag, label: Option<&str>) {
            let path = path.to_string_lossy();
            let mut index = index::lock(&self.index);
            index
                .set_rating(&self.key, &path, rating, flag, label, true)
                .unwrap();
            index
                .mark_written(&path, rating, flag, label, true, None)
                .unwrap();
        }

        fn rewrite(&self, format: SidecarFormat) -> Summary {
            let summary = rewrite(
                &self.key,
                &self.index,
                &self.writer,
                format,
                &LabelNames::default(),
                Duration::from_secs(30),
            )
            .unwrap();
            let errors = index::lock(&self.errors);
            assert!(errors.is_empty(), "writer errors: {errors:?}");
            summary
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn lightroom_xmp() -> String {
        r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="Adobe XMP Core">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:xmp="http://ns.adobe.com/xap/1.0/"
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
   crs:Exposure2012="+0.35"
   crs:Contrast2012="+12">
   <xmp:Rating>1</xmp:Rating>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#
            .to_string()
    }

    #[test]
    fn a_judged_row_with_no_sidecar_mints_one() {
        let f = Fixture::new("mint");
        let a = f.raw("a.ARW");
        f.row(&a, Some(4), Flag::None, None);

        let summary = f.rewrite(SidecarFormat::Xmp);

        assert_eq!(
            summary,
            Summary {
                written: 1,
                failed: 0,
                flushed: true
            }
        );
        let bytes = std::fs::read(xmp::sidecar_path(&a)).unwrap();
        assert_eq!(xmp::read_rating(&bytes).unwrap(), Some(4));
        assert!(!dop::sidecar_path(&a).exists(), "only the current format");
    }

    #[test]
    fn existing_sidecars_are_patched_with_their_foreign_content_intact() {
        let f = Fixture::new("patch");
        let a = f.raw("_DSC0003.ARW");
        std::fs::write(xmp::sidecar_path(&a), lightroom_xmp()).unwrap();
        std::fs::write(dop::sidecar_path(&a), PHOTOLAB).unwrap();
        f.row(&a, Some(5), Flag::Reject, None);

        f.rewrite(SidecarFormat::Both);

        let text = std::fs::read_to_string(xmp::sidecar_path(&a)).unwrap();
        assert_eq!(xmp::read_rating(text.as_bytes()).unwrap(), Some(5));
        assert_eq!(xmp::read_flag(text.as_bytes()).unwrap(), Flag::Reject);
        assert!(text.contains(r#"crs:Exposure2012="+0.35""#), "{text}");
        assert!(text.contains(r#"crs:Contrast2012="+12""#), "{text}");

        let original = String::from_utf8(PHOTOLAB.to_vec()).unwrap();
        let start = original.find("Settings = {").unwrap();
        let end = original.find("ShotDate").unwrap();
        let text = std::fs::read_to_string(dop::sidecar_path(&a)).unwrap();
        assert_eq!(dop::read_rating(text.as_bytes()).unwrap(), Some(5));
        assert_eq!(dop::read_flag(text.as_bytes()).unwrap(), Flag::Reject);
        assert!(text.contains(&original[start..end]), "{text}");
    }

    #[test]
    fn an_unjudged_row_clears_an_existing_sidecar_and_mints_nothing_without_one() {
        let f = Fixture::new("clear");
        let a = f.raw("a.ARW");
        let b = f.raw("b.ARW");
        std::fs::write(xmp::sidecar_path(&a), lightroom_xmp()).unwrap();
        f.row(&a, None, Flag::None, None);
        f.row(&b, None, Flag::None, None);

        let summary = f.rewrite(SidecarFormat::Xmp);

        assert_eq!(summary.written, 2);
        let bytes = std::fs::read(xmp::sidecar_path(&a)).unwrap();
        assert_eq!(xmp::read_rating(&bytes).unwrap(), Some(0));
        assert_eq!(xmp::read_flag(&bytes).unwrap(), Flag::None);
        assert!(!xmp::sidecar_path(&b).exists());
    }

    #[test]
    fn a_listed_raw_without_a_row_is_skipped_and_an_unlisted_row_is_not_written() {
        let f = Fixture::new("skip");
        let a = f.raw("a.ARW");
        let b = f.raw("b.ARW");
        let gone = f.dir.join("gone.ARW");
        f.row(&a, Some(2), Flag::None, None);
        f.row(&gone, Some(3), Flag::Pick, None);

        assert_eq!(
            preview(&f.key, &f.index, SidecarFormat::Xmp).unwrap(),
            Preview {
                judged: 1,
                unjudged: 0,
                skipped: 1,
                format: "xmp"
            }
        );
        let summary = f.rewrite(SidecarFormat::Xmp);

        assert_eq!(summary.written, 1);
        assert!(xmp::sidecar_path(&a).exists());
        assert!(!xmp::sidecar_path(&b).exists());
        assert!(!xmp::sidecar_path(&gone).exists());
    }

    #[test]
    fn under_both_both_kinds_are_written() {
        let f = Fixture::new("both");
        let a = f.raw("a.ARW");
        f.row(&a, None, Flag::None, Some("Red"));

        assert_eq!(
            preview(&f.key, &f.index, SidecarFormat::Both)
                .unwrap()
                .format,
            "both"
        );
        f.rewrite(SidecarFormat::Both);

        let names = LabelNames::default();
        let bytes = std::fs::read(xmp::sidecar_path(&a)).unwrap();
        assert_eq!(
            xmp::read_label(&bytes, &names).unwrap().as_deref(),
            Some("Red")
        );
        let bytes = std::fs::read(dop::sidecar_path(&a)).unwrap();
        assert_eq!(dop::read_label(&bytes).unwrap().as_deref(), Some("Red"));
    }

    #[test]
    fn a_folder_with_no_row_or_no_raw_is_refused() {
        let f = Fixture::new("refuse");
        f.raw("a.ARW");
        let err = preview(&f.key, &f.index, SidecarFormat::Xmp).unwrap_err();
        assert!(err.starts_with("No judgments indexed for "), "{err}");

        let jpegs = Fixture::new("refuse-jpeg");
        std::fs::write(jpegs.dir.join("a.jpg"), b"not really a JPEG").unwrap();
        let err = preview(&jpegs.key, &jpegs.index, SidecarFormat::Xmp).unwrap_err();
        assert!(err.ends_with("applies to RAW files only"), "{err}");
    }
}
