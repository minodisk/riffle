//! The SQLite index of a folder's metadata and thumbnails, and the scan that
//! fills it.
//!
//! One database holds every folder ever opened; a row is valid for a file only
//! while the file's `size` and `mtime_ns` still match what was indexed.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant, UNIX_EPOCH};

use rusqlite::{params, Connection};

use riffle_core::scan::{extract_all, Entry};

/// Schema version stored in `PRAGMA user_version`. Bump it when the layout
/// below changes; the old database is then dropped and rebuilt.
const SCHEMA_VERSION: i64 = 1;

/// Files per transaction while scanning. Small enough that quitting mid-scan
/// loses at most a second of work, large enough that the per-transaction fsync
/// is not paid per file.
const BATCH: usize = 50;

/// Shortest interval between two progress notifications.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

/// What `stat` says about one listed file.
#[derive(Debug, Clone)]
pub struct FileStat {
    pub path: PathBuf,
    pub size: i64,
    pub mtime_ns: i64,
}

/// One indexed file, as the frontend sees it.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexedFile {
    pub path: String,
    pub orientation: u16,
    pub capture_time: Option<String>,
    pub subsec: Option<String>,
    pub focus: Option<Focus>,
    pub has_thumb: bool,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Focus {
    pub sensor_w: u16,
    pub sensor_h: u16,
    pub x: u16,
    pub y: u16,
}

/// How a finished scan ended.
#[derive(Debug, Clone, Copy)]
pub struct ScanSummary {
    pub total: usize,
    pub errors: usize,
}

pub struct Index {
    conn: Connection,
}

/// Take a lock without caring whether a previous holder panicked. The scan
/// hands `on_item` to rayon workers, and a panic there would otherwise poison
/// the connection for the rest of the process.
pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// `stat` a file for validity checking. Times before the epoch, which no real
/// photo has, collapse to 0 rather than failing the scan.
pub fn stat(path: &Path) -> Result<FileStat, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mtime_ns = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos() as i64);
    Ok(FileStat {
        path: path.to_path_buf(),
        size: meta.len() as i64,
        mtime_ns,
    })
}

impl Index {
    /// Open, creating the file and the schema on first use. A database written
    /// by a different schema version is discarded: it is a cache.
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let mut index = Self { conn };
        if index.prepare().is_err() {
            let _ = std::fs::remove_file(path);
            let conn = Connection::open(path).map_err(|e| e.to_string())?;
            index = Self { conn };
            index.prepare()?;
        }
        Ok(index)
    }

    fn prepare(&mut self) -> Result<(), String> {
        self.conn
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| e.to_string())?;
        let version: i64 = self
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if version != 0 && version != SCHEMA_VERSION {
            return Err(format!("unsupported index schema version {version}"));
        }
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS files (
                     path TEXT PRIMARY KEY,
                     dir TEXT NOT NULL,
                     size INTEGER NOT NULL,
                     mtime_ns INTEGER NOT NULL,
                     orientation INTEGER,
                     capture_time TEXT,
                     subsec TEXT,
                     focus_w INTEGER,
                     focus_h INTEGER,
                     focus_x INTEGER,
                     focus_y INTEGER,
                     thumb BLOB,
                     error TEXT
                 );
                 CREATE INDEX IF NOT EXISTS files_dir ON files (dir);
                 CREATE INDEX IF NOT EXISTS files_capture ON files (capture_time, subsec);",
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Drop the rows under `dir` whose file is gone and return the files that
    /// have no valid row, in the order given.
    pub fn reconcile(&mut self, dir: &str, files: &[FileStat]) -> Result<Vec<FileStat>, String> {
        let known: Vec<(String, i64, i64)> = {
            let mut stmt = self
                .conn
                .prepare("SELECT path, size, mtime_ns FROM files WHERE dir = ?1")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![dir], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map_err(|e| e.to_string())?;
            rows.collect::<rusqlite::Result<_>>()
                .map_err(|e| e.to_string())?
        };
        let listed: std::collections::HashSet<&str> = files
            .iter()
            .filter_map(|f| f.path.to_str())
            .collect::<std::collections::HashSet<_>>();

        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        for (path, _, _) in known
            .iter()
            .filter(|(p, _, _)| !listed.contains(p.as_str()))
        {
            tx.execute("DELETE FROM files WHERE path = ?1", params![path])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;

        let valid: std::collections::HashMap<&str, (i64, i64)> = known
            .iter()
            .map(|(p, s, m)| (p.as_str(), (*s, *m)))
            .collect();
        Ok(files
            .iter()
            .filter(|f| {
                f.path
                    .to_str()
                    .is_none_or(|p| valid.get(p) != Some(&(f.size, f.mtime_ns)))
            })
            .cloned()
            .collect())
    }

    /// Write one batch of results in a single transaction.
    pub fn write_batch(
        &mut self,
        dir: &str,
        rows: &[(FileStat, Result<Entry, String>)],
    ) -> Result<(), String> {
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        for (file, result) in rows {
            let path = file.path.to_string_lossy();
            match result {
                Ok(entry) => {
                    let focus = entry.focus;
                    tx.execute(
                        "INSERT OR REPLACE INTO files (path, dir, size, mtime_ns, orientation,
                             capture_time, subsec, focus_w, focus_h, focus_x, focus_y, thumb, error)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, NULL)",
                        params![
                            path,
                            dir,
                            file.size,
                            file.mtime_ns,
                            entry.orientation,
                            entry.capture_time,
                            entry.subsec,
                            focus.map(|f| f.sensor_w),
                            focus.map(|f| f.sensor_h),
                            focus.map(|f| f.x),
                            focus.map(|f| f.y),
                            entry.thumbnail,
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                }
                Err(message) => {
                    tx.execute(
                        "INSERT OR REPLACE INTO files (path, dir, size, mtime_ns, orientation,
                             capture_time, subsec, focus_w, focus_h, focus_x, focus_y, thumb, error)
                         VALUES (?1, ?2, ?3, ?4, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, ?5)",
                        params![path, dir, file.size, file.mtime_ns, message],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }
        tx.commit().map_err(|e| e.to_string())
    }

    /// The indexed files of `dir`, in file-name order like `list_arw`.
    pub fn entries(&self, dir: &str) -> Result<Vec<IndexedFile>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT path, orientation, capture_time, subsec,
                        focus_w, focus_h, focus_x, focus_y, thumb IS NOT NULL
                 FROM files WHERE dir = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![dir], |r| {
                let focus = match (r.get::<_, Option<u16>>(4)?, r.get::<_, Option<u16>>(5)?) {
                    (Some(sensor_w), Some(sensor_h)) => Some(Focus {
                        sensor_w,
                        sensor_h,
                        x: r.get(6)?,
                        y: r.get(7)?,
                    }),
                    _ => None,
                };
                Ok(IndexedFile {
                    path: r.get(0)?,
                    orientation: r.get::<_, Option<u16>>(1)?.unwrap_or(1),
                    capture_time: r.get(2)?,
                    subsec: r.get(3)?,
                    focus,
                    has_thumb: r.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut entries: Vec<IndexedFile> = rows
            .collect::<rusqlite::Result<_>>()
            .map_err(|e| e.to_string())?;
        entries.sort_by(|a, b| {
            Path::new(&a.path)
                .file_name()
                .cmp(&Path::new(&b.path).file_name())
        });
        Ok(entries)
    }

    /// The cached thumbnail of one file with its Orientation.
    pub fn thumbnail(&self, path: &str) -> Result<(u16, Vec<u8>), String> {
        self.conn
            .query_row(
                "SELECT orientation, thumb FROM files WHERE path = ?1",
                params![path],
                |r| Ok((r.get::<_, Option<u16>>(0)?, r.get::<_, Option<Vec<u8>>>(1)?)),
            )
            .map_err(|e| format!("{path}: {e}"))
            .and_then(|(orientation, thumb)| match thumb {
                Some(thumb) => Ok((orientation.unwrap_or(1), thumb)),
                None => Err(format!("{path}: no cached thumbnail")),
            })
    }
}

/// Extract `files` and write them into `index` in batched transactions,
/// reporting `(done, total)` through `progress` at most every 100ms and always
/// on the last file.
///
/// `on_item` runs on rayon worker threads and a panic there would abort the
/// whole scan, so nothing inside it unwraps: locks are taken with `lock` (which
/// ignores poisoning) and a failed write is counted like a failed file.
pub fn run_scan<P>(
    index: &Mutex<Index>,
    dir: &str,
    files: &[FileStat],
    threads: usize,
    cancel: &AtomicBool,
    progress: P,
) -> ScanSummary
where
    P: Fn(usize, usize) + Send + Sync,
{
    let total = files.len();
    let paths: Vec<PathBuf> = files.iter().map(|f| f.path.clone()).collect();
    let pending: Mutex<Vec<(FileStat, Result<Entry, String>)>> = Mutex::new(Vec::new());
    let done = AtomicUsize::new(0);
    let errors = AtomicUsize::new(0);
    let last = Mutex::new(None::<Instant>);

    let flush = |batch: Vec<(FileStat, Result<Entry, String>)>| {
        if batch.is_empty() {
            return;
        }
        if lock(index).write_batch(dir, &batch).is_err() {
            errors.fetch_add(batch.len(), Ordering::Relaxed);
        }
    };

    let on_item = |i: usize, result: Result<Entry, String>| {
        if result.is_err() {
            errors.fetch_add(1, Ordering::Relaxed);
        }
        let batch = {
            let mut pending = lock(&pending);
            pending.push((files[i].clone(), result));
            if pending.len() >= BATCH {
                std::mem::take(&mut *pending)
            } else {
                Vec::new()
            }
        };
        flush(batch);

        let done = done.fetch_add(1, Ordering::Relaxed) + 1;
        let now = Instant::now();
        let due = {
            let mut last = lock(&last);
            if done == total || last.is_none_or(|t| now.duration_since(t) >= PROGRESS_INTERVAL) {
                *last = Some(now);
                true
            } else {
                false
            }
        };
        if due {
            progress(done, total);
        }
    };

    if let Err(e) = extract_all(&paths, threads, on_item, cancel) {
        eprintln!("scan of {dir} failed: {e}");
    }
    flush(std::mem::take(&mut *lock(&pending)));

    ScanSummary {
        total: done.load(Ordering::Relaxed),
        errors: errors.load(Ordering::Relaxed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riffle_core::arw::FocusLocation;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("riffle-index-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn open(dir: &Path) -> Index {
        Index::open(&dir.join("index.sqlite")).unwrap()
    }

    fn file(dir: &Path, name: &str, bytes: &[u8]) -> FileStat {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        stat(&path).unwrap()
    }

    fn entry() -> Entry {
        Entry {
            orientation: 6,
            capture_time: Some("2026:09:13 09:23:33".to_string()),
            subsec: Some("122".to_string()),
            focus: Some(FocusLocation {
                sensor_w: 7008,
                sensor_h: 4672,
                x: 3613,
                y: 1732,
            }),
            thumbnail: vec![0xff, 0xd8, 0xff, 0xd9],
        }
    }

    #[test]
    fn a_written_row_survives_re_opening_the_database() {
        let dir = temp_dir("reopen");
        let a = file(&dir, "a.ARW", b"a");
        let mut index = open(&dir);
        index.write_batch("d", &[(a.clone(), Ok(entry()))]).unwrap();
        drop(index);

        let mut index = open(&dir);
        assert!(index
            .reconcile("d", std::slice::from_ref(&a))
            .unwrap()
            .is_empty());
        let entries = index.entries("d").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].orientation, 6);
        assert_eq!(entries[0].subsec.as_deref(), Some("122"));
        assert_eq!(entries[0].focus.unwrap().x, 3613);
        assert!(entries[0].has_thumb);
        assert_eq!(index.thumbnail(&entries[0].path).unwrap().0, 6);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_changed_mtime_invalidates_the_row() {
        let dir = temp_dir("mtime");
        let a = file(&dir, "a.ARW", b"a");
        let mut index = open(&dir);
        index.write_batch("d", &[(a.clone(), Ok(entry()))]).unwrap();

        let changed = FileStat {
            mtime_ns: a.mtime_ns + 1,
            ..a.clone()
        };
        assert_eq!(index.reconcile("d", &[changed]).unwrap().len(), 1);

        let bigger = FileStat { size: 99, ..a };
        assert_eq!(index.reconcile("d", &[bigger]).unwrap().len(), 1);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_deleted_file_loses_its_row_on_the_next_reconciliation() {
        let dir = temp_dir("deleted");
        let a = file(&dir, "a.ARW", b"a");
        let b = file(&dir, "b.ARW", b"b");
        let mut index = open(&dir);
        index
            .write_batch("d", &[(a.clone(), Ok(entry())), (b, Ok(entry()))])
            .unwrap();
        assert_eq!(index.entries("d").unwrap().len(), 2);

        assert!(index
            .reconcile("d", std::slice::from_ref(&a))
            .unwrap()
            .is_empty());
        let entries = index.entries("d").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, a.path.to_string_lossy());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_failed_file_is_remembered_as_an_error_row_without_a_thumbnail() {
        let dir = temp_dir("error");
        let a = file(&dir, "a.ARW", b"a");
        let mut index = open(&dir);
        index
            .write_batch("d", &[(a.clone(), Err("broken".to_string()))])
            .unwrap();

        let entries = index.entries("d").unwrap();
        assert!(!entries[0].has_thumb);
        assert!(index.thumbnail(&entries[0].path).is_err());
        assert!(index.reconcile("d", &[a]).unwrap().is_empty());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A TIFF shell whose IFD0 carries an Orientation and a preview of `body`,
    /// the same synthetic fixture the core's scan tests use.
    fn fixture(orientation: u16, body: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"II\x2a\x00");
        buf.extend_from_slice(&8u32.to_le_bytes());
        buf.extend_from_slice(&3u16.to_le_bytes());
        let at = 8 + 2 + 3 * 12 + 4;
        for (tag, ty, value) in [
            (0x0112u16, 3u16, orientation as u32),
            (0x0201, 4, at as u32),
            (0x0202, 4, body.len() as u32),
        ] {
            buf.extend_from_slice(&tag.to_le_bytes());
            buf.extend_from_slice(&ty.to_le_bytes());
            buf.extend_from_slice(&1u32.to_le_bytes());
            buf.extend_from_slice(&value.to_le_bytes());
        }
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(body);
        buf
    }

    fn jpeg(w: usize, h: usize) -> Vec<u8> {
        let rgb = vec![128u8; w * h * 3];
        let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        c.set_size(w, h);
        c.set_quality(80.0);
        let mut c = c.start_compress(Vec::new()).unwrap();
        c.write_scanlines(&rgb).unwrap();
        c.finish().unwrap()
    }

    #[test]
    fn a_scan_writes_every_file_and_reports_progress() {
        let dir = temp_dir("scan");
        let body = jpeg(64, 48);
        let files: Vec<FileStat> = (0..8)
            .map(|i| file(&dir, &format!("{i:02}.ARW"), &fixture(6, &body)))
            .collect();

        let index = Mutex::new(open(&dir));
        let progress = Mutex::new(Vec::new());
        let summary = run_scan(
            &index,
            "d",
            &files,
            2,
            &AtomicBool::new(false),
            |done, total| progress.lock().unwrap().push((done, total)),
        );

        assert_eq!(summary.total, 8);
        assert_eq!(summary.errors, 0);
        let entries = lock(&index).entries("d").unwrap();
        assert_eq!(entries.len(), 8);
        assert!(entries.iter().all(|e| e.has_thumb && e.orientation == 6));
        let progress = progress.into_inner().unwrap();
        assert_eq!(progress.last(), Some(&(8, 8)), "the last file is reported");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn cancelling_after_the_first_batch_keeps_what_was_written() {
        let dir = temp_dir("cancel");
        let body = jpeg(64, 48);
        let files: Vec<FileStat> = (0..(BATCH * 4))
            .map(|i| file(&dir, &format!("{i:03}.ARW"), &fixture(1, &body)))
            .collect();

        let index = Mutex::new(open(&dir));
        let cancel = AtomicBool::new(false);
        let finished = AtomicBool::new(false);
        // Progress is throttled to ~10/s, so the cancel cannot be driven from
        // it; a watcher polls the rows instead and fires once a batch landed.
        let summary = std::thread::scope(|scope| {
            scope.spawn(|| {
                while !finished.load(Ordering::Relaxed) {
                    if lock(&index).entries("d").map_or(0, |e| e.len()) >= BATCH {
                        cancel.store(true, Ordering::Relaxed);
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            });
            let summary = run_scan(&index, "d", &files, 2, &cancel, |_, _| {});
            finished.store(true, Ordering::Relaxed);
            summary
        });

        assert!(summary.total >= BATCH, "the first batch ran");
        assert!(summary.total < files.len(), "the rest did not");
        let written = lock(&index).entries("d").unwrap().len();
        assert_eq!(written, summary.total, "everything scanned is persisted");

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
