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
/// below changes; a version `prepare` cannot migrate is dropped and rebuilt.
///
/// From v2 on the database is no longer purely derivable: a `ratings` row with
/// `dirty = 1` is a judgement that has not reached its sidecar yet, so
/// discarding the database loses it. A future bump must migrate `ratings` or
/// flush every dirty row to its sidecar first. v3 added `ratings.pick`, and a
/// v2 database is migrated in place with an `ALTER TABLE`.
const SCHEMA_VERSION: i64 = 3;

/// Files per transaction while scanning. Small enough that quitting mid-scan
/// loses at most a fifth of a second of work and that the single index mutex,
/// which every `thumbnail` / `folder_entries` / `set_rating` also takes, is not
/// held for a long transaction, large enough that the commit (a WAL append
/// under `synchronous = NORMAL`, not an fsync) and the mutex re-acquisition are
/// not paid per file. Dropping this from 50 to 10 is a 5x increase in
/// transactions; that throughput cost has not been measured against the
/// `scan` benchmark in `crates/cli`, and was judged acceptable because it is
/// still one transaction per UI progress tick, not per file.
const BATCH: usize = 10;

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
///
/// `has_sidecar` is whether a sidecar was on disk the last time the app read
/// or wrote one (`ratings.xmp_size` is set), not a live stat.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexedFile {
    pub path: String,
    pub orientation: u16,
    pub capture_time: Option<String>,
    pub subsec: Option<String>,
    pub focus: Option<Focus>,
    pub has_thumb: bool,
    pub rating: Option<i8>,
    pub pick: bool,
    pub has_sidecar: bool,
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

/// Append a suffix to a path's file name, for the `-wal`/`-shm` sidecars SQLite
/// keeps next to the main database file.
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
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
    /// Open, creating the file and the schema on first use. `v2` is migrated
    /// to the current schema in place; a database written by any other
    /// unrecognized schema version is discarded, since it is a cache.
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let mut index = Self { conn };
        if let Err(e) = index.prepare() {
            log::warn!("discarding the index cache at {}: {e}", path.display());
            drop(index);
            let _ = std::fs::remove_file(path);
            let _ = std::fs::remove_file(with_suffix(path, "-wal"));
            let _ = std::fs::remove_file(with_suffix(path, "-shm"));
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
        // A commit is then a WAL append with no fsync, which is what keeps the
        // `set_rating` write off the critical path of a keypress.
        self.conn
            .pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| e.to_string())?;
        let version: i64 = self
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if version != 0 && version != 2 && version != SCHEMA_VERSION {
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
                 CREATE INDEX IF NOT EXISTS files_capture ON files (capture_time, subsec);
                 -- `xmp_size` / `xmp_mtime_ns` are the stat of the sidecar of
                 -- the selected format (XMP or `.dop`), whatever its name says.
                 -- `pick` is PhotoLab's pick flag, kept apart from `rating`
                 -- because the two coexist in a `.dop`.
                 CREATE TABLE IF NOT EXISTS ratings (
                     path TEXT PRIMARY KEY,
                     dir TEXT NOT NULL,
                     rating INTEGER,
                     pick INTEGER NOT NULL DEFAULT 0,
                     xmp_size INTEGER,
                     xmp_mtime_ns INTEGER,
                     dirty INTEGER NOT NULL DEFAULT 0
                 );
                 CREATE INDEX IF NOT EXISTS ratings_dir ON ratings (dir);",
            )
            .map_err(|e| e.to_string())?;
        if version == 2 {
            let tx = self.conn.transaction().map_err(|e| e.to_string())?;
            tx.execute_batch("ALTER TABLE ratings ADD COLUMN pick INTEGER NOT NULL DEFAULT 0;")
                .map_err(|e| e.to_string())?;
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)
                .map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;
        } else {
            self.conn
                .pragma_update(None, "user_version", SCHEMA_VERSION)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Drop the rows under `dir` whose file is gone or whose `size`/`mtime_ns`
    /// no longer match, and return the files that have no valid row, in the
    /// order given. Paths are compared with the same lossy conversion
    /// `write_batch` uses to key rows, so a non-UTF-8 path matches the row it
    /// wrote instead of being rescanned on every open.
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
        let listed: std::collections::HashMap<String, (i64, i64)> = files
            .iter()
            .map(|f| (f.path.to_string_lossy().into_owned(), (f.size, f.mtime_ns)))
            .collect();

        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        for (path, ..) in known
            .iter()
            .filter(|(p, s, m)| listed.get(p.as_str()) != Some(&(*s, *m)))
        {
            tx.execute("DELETE FROM files WHERE path = ?1", params![path])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;

        let known: std::collections::HashMap<&str, (i64, i64)> = known
            .iter()
            .map(|(p, s, m)| (p.as_str(), (*s, *m)))
            .collect();
        Ok(files
            .iter()
            .filter(|f| {
                let path = f.path.to_string_lossy();
                known.get(path.as_ref()) != Some(&(f.size, f.mtime_ns))
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
                    let focus = entry.shot.focus;
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
                            entry.shot.capture_time,
                            entry.shot.subsec,
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
                        focus_w, focus_h, focus_x, focus_y, thumb IS NOT NULL,
                        ratings.rating, ratings.xmp_size IS NOT NULL,
                        COALESCE(ratings.pick, 0)
                 FROM files LEFT JOIN ratings USING (path) WHERE files.dir = ?1",
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
                    rating: r.get(9)?,
                    pick: r.get(11)?,
                    has_sidecar: r.get(10)?,
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

    /// Record a judgement for one file, pending a sidecar write.
    ///
    /// The row is independent of `files`, so a rescan or a failed extraction
    /// never drops a rating. `dirty = 1` until the writer has landed exactly
    /// this value in the sidecar.
    pub fn set_rating(
        &mut self,
        dir: &str,
        path: &str,
        rating: Option<i8>,
        pick: bool,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO ratings (path, dir, rating, pick, dirty) VALUES (?1, ?2, ?3, ?4, 1)
                 ON CONFLICT (path) DO UPDATE SET dir = ?2, rating = ?3, pick = ?4, dirty = 1",
                params![path, dir, rating, pick],
            )
            .map(|_| ())
            .map_err(|e| format!("{path}: {e}"))
    }

    /// Clear `dirty` and store the sidecar's stat, but only while the row
    /// still holds the judgement that was written: a keypress during the write
    /// leaves the row dirty so the newer value is written in turn.
    ///
    /// `stat` is `None` when no sidecar exists (clearing a rating on a file
    /// that never had one writes nothing).
    pub fn mark_written(
        &mut self,
        path: &str,
        rating: Option<i8>,
        pick: bool,
        stat: Option<(i64, i64)>,
    ) -> Result<bool, String> {
        let (size, mtime_ns) = (stat.map(|s| s.0), stat.map(|s| s.1));
        self.conn
            .execute(
                "UPDATE ratings SET dirty = 0, xmp_size = ?2, xmp_mtime_ns = ?3
                 WHERE path = ?1 AND rating IS ?4 AND pick = ?5",
                params![path, size, mtime_ns, rating, pick],
            )
            .map(|n| n > 0)
            .map_err(|e| format!("{path}: {e}"))
    }

    /// Forget every sidecar the index has seen, for a sidecar format switch:
    /// clean rows are dropped so the next open reads the newly selected
    /// format, and dirty rows keep their judgement but lose the old format's
    /// stat, so the next open writes them into the new one. Their pick is
    /// dropped: it can only have come from `.dop`, and a switch away from it
    /// lands in XMP, which has none, while a switch to it starts from none.
    pub fn reset_sidecars(&mut self) -> Result<(), String> {
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM ratings WHERE dirty = 0", [])
            .map_err(|e| e.to_string())?;
        tx.execute(
            "UPDATE ratings SET xmp_size = NULL, xmp_mtime_ns = NULL, pick = 0 WHERE dirty = 1",
            [],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }

    /// The rows of `dir` whose judgement has not reached its sidecar yet.
    pub fn dirty_rows(&self, dir: &str) -> Result<Vec<(String, Option<i8>, bool)>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, rating, pick FROM ratings WHERE dir = ?1 AND dirty = 1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![dir], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<rusqlite::Result<_>>()
            .map_err(|e| e.to_string())
    }

    /// Apply the reconciliation rules for the sidecars of `dir` on a folder
    /// open. `sidecars` pairs each listed file with its sidecar's stat, or
    /// `None` when the folder listing found none.
    ///
    /// The sidecar is the source of truth: a sidecar that changed under us is
    /// read back, never silently overwritten, even when the row is dirty. The
    /// rules are, per file: a sidecar whose stat differs from the stored one
    /// is parsed (which is also the first open of a folder Lightroom wrote);
    /// an unchanged stat costs nothing, which is what keeps the second open of
    /// a 5000-file folder cheap; a sidecar that is gone while the row is
    /// clean clears the rating, because the truth is gone with it; and a row
    /// that is still dirty is left dirty, for `dirty_rows` to hand to the
    /// writer once the parsed sidecars have been stored.
    ///
    /// Returns the sidecars to read and parse outside the lock, each paired
    /// with the `dirty` flag observed here; their result goes back through
    /// `store_sidecar_ratings`, which must only apply the "sidecar wins over
    /// a dirty row" rule to the dirtiness this snapshot actually saw, not to
    /// a `set_rating` that lands while the lock is released for the parse.
    pub fn reconcile_sidecars(
        &mut self,
        dir: &str,
        sidecars: &[(String, Option<SidecarStat>)],
    ) -> Result<Vec<(String, SidecarStat, bool)>, String> {
        let known: std::collections::HashMap<String, RatingRow> = {
            let mut stmt = self
                .conn
                .prepare("SELECT path, xmp_size, xmp_mtime_ns, dirty FROM ratings WHERE dir = ?1")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![dir], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        RatingRow {
                            stat: (r.get(1)?, r.get(2)?),
                            dirty: r.get::<_, i64>(3)? != 0,
                        },
                    ))
                })
                .map_err(|e| e.to_string())?;
            rows.collect::<rusqlite::Result<_>>()
                .map_err(|e| e.to_string())?
        };

        let mut to_parse = Vec::new();
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        for (path, stat) in sidecars {
            let row = known.get(path);
            let dirty = row.is_some_and(|r| r.dirty);
            match stat {
                Some((sidecar, size, mtime_ns)) => {
                    if row.map(|r| r.stat) != Some((Some(*size), Some(*mtime_ns))) {
                        to_parse.push((path.clone(), (sidecar.clone(), *size, *mtime_ns), dirty));
                    }
                }
                None => {
                    if !dirty && row.is_some_and(|r| r.stat.0.is_some() || r.stat.1.is_some()) {
                        tx.execute(
                            "UPDATE ratings SET rating = NULL, pick = 0, xmp_size = NULL,
                                 xmp_mtime_ns = NULL WHERE path = ?1",
                            params![path],
                        )
                        .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(to_parse)
    }

    /// Store what parsing the sidecars of `dir` found: the rating and the pick
    /// become the sidecar's and `dirty` is cleared, since the sidecar wins over an app
    /// edit that never reached disk.
    ///
    /// `rows` carries the `dirty` flag `reconcile_sidecars` observed for each
    /// path before the lock was released for the parse. The update is
    /// conditioned on the row's `dirty` still matching that snapshot: a
    /// `set_rating` landing in that window makes the row dirty when the
    /// snapshot was not, and such a row is left untouched here rather than
    /// having its fresh rating and `dirty` flag overwritten by a sidecar read
    /// taken before the edit.
    pub fn store_sidecar_ratings(
        &mut self,
        dir: &str,
        rows: &[ParsedSidecar],
    ) -> Result<(), String> {
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        for (path, rating, pick, size, mtime_ns, dirty) in rows {
            tx.execute(
                "INSERT INTO ratings (path, dir, rating, pick, xmp_size, xmp_mtime_ns, dirty)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)
                 ON CONFLICT (path) DO UPDATE SET dir = ?2, rating = ?3, pick = ?4,
                     xmp_size = ?5, xmp_mtime_ns = ?6, dirty = 0
                 WHERE dirty = ?7",
                params![path, dir, rating, pick, size, mtime_ns, *dirty as i64],
            )
            .map_err(|e| format!("{path}: {e}"))?;
        }
        tx.commit().map_err(|e| e.to_string())
    }
}

/// The part of a `ratings` row the folder-open reconciliation looks at: the
/// stat of the sidecar as the app last saw it, and whether a judgement is
/// still waiting to be written.
struct RatingRow {
    stat: (Option<i64>, Option<i64>),
    dirty: bool,
}

/// What one file's sidecar looks like on disk, as seen by the single
/// directory listing a folder open does.
pub type SidecarStat = (PathBuf, i64, i64);

/// What parsing one file's sidecar found, for `store_sidecar_ratings`:
/// `(path, rating, pick, size, mtime_ns, dirty)`, the last being the `dirty`
/// flag `reconcile_sidecars` observed.
pub type ParsedSidecar = (String, Option<i8>, bool, i64, i64, bool);

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
        if let Err(e) = lock(index).write_batch(dir, &batch) {
            log::error!("failed to write a batch for {dir}: {e}");
            // Files that failed extraction are already counted in `on_item`;
            // only the ones that would otherwise have landed as a success are
            // newly lost here.
            let newly_failed = batch.iter().filter(|(_, r)| r.is_ok()).count();
            errors.fetch_add(newly_failed, Ordering::Relaxed);
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
        log::error!("scan of {dir} failed: {e}");
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
    use riffle_core::arw::{FocusLocation, Shot};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("riffle-index-{name}-{}", std::process::id()));
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
            shot: Shot {
                capture_time: Some("2026:09:13 09:23:33".to_string()),
                subsec: Some("122".to_string()),
                focus: Some(FocusLocation {
                    sensor_w: 7008,
                    sensor_h: 4672,
                    x: 3613,
                    y: 1732,
                }),
                ..Shot::default()
            },
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

        remove_temp_dir(&dir);
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
        // The stale row must be gone too, not just reported as "to scan":
        // otherwise `entries` would keep serving it until the rescan finishes.
        assert!(index.entries("d").unwrap().is_empty());

        index.write_batch("d", &[(a.clone(), Ok(entry()))]).unwrap();
        let bigger = FileStat { size: 99, ..a };
        assert_eq!(index.reconcile("d", &[bigger]).unwrap().len(), 1);
        assert!(index.entries("d").unwrap().is_empty());

        remove_temp_dir(&dir);
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

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_rating_survives_a_rescan_and_shows_up_on_the_entry() {
        let dir = temp_dir("rating");
        let a = file(&dir, "a.ARW", b"a");
        let path = a.path.to_string_lossy().into_owned();
        let mut index = open(&dir);
        index.write_batch("d", &[(a.clone(), Ok(entry()))]).unwrap();
        index.set_rating("d", &path, Some(-1), false).unwrap();
        assert_eq!(index.entries("d").unwrap()[0].rating, Some(-1));

        // A reconcile that drops the `files` row must leave `ratings` alone:
        // the sidecar, not the scan, is what a rating belongs to.
        assert!(index.reconcile("d", &[]).unwrap().is_empty());
        assert!(index.entries("d").unwrap().is_empty());
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [(path.clone(), Some(-1), false)]
        );

        index.write_batch("d", &[(a, Ok(entry()))]).unwrap();
        assert_eq!(index.entries("d").unwrap()[0].rating, Some(-1));

        remove_temp_dir(&dir);
    }

    #[test]
    fn mark_written_only_clears_a_row_that_still_holds_the_written_value() {
        let dir = temp_dir("written");
        let mut index = open(&dir);
        index.set_rating("d", "/a.ARW", Some(3), false).unwrap();

        assert!(index
            .mark_written("/a.ARW", Some(3), false, Some((42, 7)))
            .unwrap());
        assert!(index.dirty_rows("d").unwrap().is_empty());

        index.set_rating("d", "/a.ARW", Some(5), false).unwrap();
        assert!(
            !index
                .mark_written("/a.ARW", Some(3), false, Some((42, 7)))
                .unwrap(),
            "a keypress during the write keeps the row dirty"
        );
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [("/a.ARW".to_string(), Some(5), false)]
        );

        // An unrated row is matched by NULL, not skipped.
        index.set_rating("d", "/a.ARW", None, false).unwrap();
        assert!(index.mark_written("/a.ARW", None, false, None).unwrap());
        assert!(index.dirty_rows("d").unwrap().is_empty());

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_sidecar_reset_drops_clean_rows_and_clears_the_stat_of_dirty_ones() {
        let dir = temp_dir("reset-sidecars");
        let mut index = open(&dir);
        index.set_rating("d", "/a.ARW", Some(3), false).unwrap();
        assert!(index
            .mark_written("/a.ARW", Some(3), false, Some((42, 7)))
            .unwrap());
        index.set_rating("d", "/b.ARW", Some(5), false).unwrap();
        index
            .store_sidecar_ratings("d", &[("/b.ARW".to_string(), Some(5), false, 9, 9, true)])
            .unwrap();
        index.set_rating("d", "/b.ARW", Some(-1), false).unwrap();

        index.reset_sidecars().unwrap();

        let rows: Vec<(String, bool)> = index
            .conn
            .prepare("SELECT path, xmp_size IS NULL AND xmp_mtime_ns IS NULL FROM ratings")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(rows, [("/b.ARW".to_string(), true)]);
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [("/b.ARW".to_string(), Some(-1), false)]
        );

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_pick_is_stored_beside_the_rating_and_guards_mark_written() {
        let dir = temp_dir("pick");
        let a = file(&dir, "a.ARW", b"a");
        let path = a.path.to_string_lossy().into_owned();
        let mut index = open(&dir);
        index.write_batch("d", &[(a, Ok(entry()))]).unwrap();
        assert!(!index.entries("d").unwrap()[0].pick, "no ratings row");

        index.set_rating("d", &path, Some(3), true).unwrap();
        let entry = &index.entries("d").unwrap()[0];
        assert_eq!((entry.rating, entry.pick), (Some(3), true));

        assert!(
            !index.mark_written(&path, Some(3), false, None).unwrap(),
            "an unpick during the write keeps the row dirty"
        );
        assert!(index.mark_written(&path, Some(3), true, None).unwrap());

        index
            .store_sidecar_ratings("d", &[(path.clone(), Some(0), false, 1, 2, false)])
            .unwrap();
        assert!(!index.entries("d").unwrap()[0].pick, "the sidecar wins");

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v2_database_gains_the_pick_column_and_keeps_its_dirty_rows() {
        let dir = temp_dir("migrate-v2");
        let db = dir.join("index.sqlite");
        {
            let conn = Connection::open(&db).unwrap();
            conn.execute_batch(
                "CREATE TABLE ratings (
                     path TEXT PRIMARY KEY,
                     dir TEXT NOT NULL,
                     rating INTEGER,
                     xmp_size INTEGER,
                     xmp_mtime_ns INTEGER,
                     dirty INTEGER NOT NULL DEFAULT 0
                 );
                 INSERT INTO ratings (path, dir, rating, dirty) VALUES ('/a.ARW', 'd', 4, 1);
                 PRAGMA user_version = 2;",
            )
            .unwrap();
        }

        let index = Index::open(&db).unwrap();
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [("/a.ARW".to_string(), Some(4), false)]
        );
        drop(index);
        let index = Index::open(&db).unwrap();
        assert_eq!(index.dirty_rows("d").unwrap().len(), 1, "a reopen is v3");

        remove_temp_dir(&dir);
    }

    #[test]
    fn has_sidecar_follows_the_stored_sidecar_stat() {
        let dir = temp_dir("has-sidecar");
        let a = file(&dir, "a.ARW", b"a");
        let path = a.path.to_string_lossy().into_owned();
        let mut index = open(&dir);
        index.write_batch("d", &[(a, Ok(entry()))]).unwrap();
        let has_sidecar = |index: &Index| index.entries("d").unwrap()[0].has_sidecar;

        assert!(!has_sidecar(&index), "no ratings row");

        index.set_rating("d", &path, Some(3), false).unwrap();
        assert!(!has_sidecar(&index), "dirty row, nothing written yet");

        assert!(index
            .mark_written(&path, Some(3), false, Some((42, 7)))
            .unwrap());
        assert!(has_sidecar(&index));

        index.set_rating("d", &path, None, false).unwrap();
        assert!(index.mark_written(&path, None, false, None).unwrap());
        assert!(!has_sidecar(&index));

        index
            .store_sidecar_ratings("d", &[(path.clone(), Some(-1), false, 10, 20, false)])
            .unwrap();
        assert!(has_sidecar(&index));

        remove_temp_dir(&dir);
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

        remove_temp_dir(&dir);
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

        remove_temp_dir(&dir);
    }

    #[test]
    fn cancelling_after_the_first_batch_keeps_what_was_written() {
        let dir = temp_dir("cancel");
        let body = jpeg(64, 48);
        // Enough files that, however fast the machine, the run cannot finish
        // before the watcher below has a chance to see the first batch land
        // and fire the cancel; too small a margin here made this test flaky
        // on quieter/faster CI runners.
        let files: Vec<FileStat> = (0..(BATCH * 40))
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

        remove_temp_dir(&dir);
    }
}
