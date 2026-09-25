//! The SQLite index of a folder's metadata and thumbnails, and the scan that
//! fills it.
//!
//! One database holds every folder ever opened; a row is valid for a file only
//! while the file's `size` and `mtime_ns` still match what was indexed and it
//! was written at the current `EXTRACTOR_VERSION`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant, UNIX_EPOCH};

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};

use riffle_core::arw::{Rational, Shot};
use riffle_core::faces::FaceCatch;
use riffle_core::scan::{extract_all, Entry};
use riffle_core::sharpness::manual_focus;
use riffle_core::Flag;

use crate::exif::{exif, Exif};

/// Schema version stored in `PRAGMA user_version`. Bump it when the layout
/// below changes; a version `prepare` cannot migrate is dropped and rebuilt.
///
/// From v2 on the database is no longer purely derivable: a `ratings` row with
/// `dirty = 1` is a judgment that has not reached its sidecar yet, so
/// discarding the database loses it. A future bump must migrate `ratings` or
/// flush every dirty row to its sidecar first. v3 added `ratings.pick`, and a
/// v2 database is migrated in place with an `ALTER TABLE`. v4 added the
/// shooting settings to `files`; a v2 or v3 database has its `files` table
/// dropped and recreated, so every folder is rescanned once, and keeps
/// `ratings`. v5 added `ratings.label`, the color label (`NULL` = none); a
/// v2, v3 or v4 database gains it in place with an `ALTER TABLE`. v6 added
/// `ratings.label_known`, persisting whether `label` reflects a judgment the
/// app actually asserted (see `set_rating`); older databases gain it in
/// place, defaulting to `1` since every label they hold was asserted. v7
/// added `files.sharpness`, the preview's focus-window score; a v2 to v6
/// database has its `files` table dropped and recreated, so every folder is
/// rescanned once, and keeps `ratings`. v8 added `folders`, when each folder
/// was last opened, for `evict`; a v7 database gains it in place, seeded with
/// every indexed folder opened "now", and keeps its `files` rows. v9 changed
/// how `files.sharpness` is scored (the sharpest tile when there is no
/// trustworthy AF point); a v2 to v8 database has its `files` table dropped
/// and recreated, so every folder is rescanned once, and keeps `ratings` and
/// `folders`. v10 changed it again (the eyes when a face is found); a v2 to v9
/// database likewise drops and recreates `files` and keeps `ratings` and
/// `folders`. v11 replaced `ratings.pick` with `ratings.flag` (`0` none, `1`
/// pick, `2` reject) and took the reject out of `rating`, which now only
/// holds `0`-`5`; older databases are migrated in place (a `-1` rating
/// becomes a reject with no stars, a pick a pick) and keep their `files`.
/// v12 added `files.extractor`, the `EXTRACTOR_VERSION` a row was written at;
/// a v10 or v11 database gains it in place with an `ALTER TABLE`, defaulting
/// to `0` so every existing row is re-extracted once, and keeps `files`,
/// `ratings` and `folders`. v13 added `files.frame_w` / `files.frame_h`, the
/// Sony AF frame size (`NULL` when not recorded), and `files.manual_focus`; a
/// v10 to v12 database gains them in place with an `ALTER TABLE` and keeps
/// `files`, `ratings` and `folders`. v14 added `files.face_catch`, the
/// face-catch state (`0` unknown, `1` caught, `2` missed); a v10 to v13
/// database gains it in place with an `ALTER TABLE` and keeps `files`,
/// `ratings` and `folders`.
const SCHEMA_VERSION: i64 = 14;

/// The version of what `riffle_core::scan::extract` produces, stored on every
/// `files` row. Bump it on any change to that output: ARW/DNG parsing or
/// embedded JPEG tier selection (`crates/core/src/arw.rs`), which preview
/// bytes are read (`crates/core/src/reader.rs`), thumbnail generation
/// (`crates/core/src/decode.rs`), face detection or the sharpness score
/// (`crates/core/src/scan.rs`, `sharpness.rs`, `faces.rs`). A bump re-extracts
/// every row, error rows included, on the next scan of each folder, and keeps
/// `ratings`. It starts at `1` so rows from before the column existed (`0`)
/// are stale; `2` fills in the AF frame size and the manual-focus flag; `3`
/// scores the AF window instead of a detected face's eyes when the AF point
/// lies outside the face; `4` fills in the face-catch state.
const EXTRACTOR_VERSION: i64 = 4;

/// Files per transaction while scanning. `thumbnail` / `folder_entries` read
/// through their own connection (`Index::open_reader`) and do not wait on
/// these transactions, so the size is set by what still shares the writer:
/// quitting mid-scan loses at most one small batch, and `set_rating` and the
/// sidecar writer thread, which take the writer mutex, wait behind at most
/// one. The per-commit cost is small: writing 5000 synthetic rows in batches of
/// 10 instead of 50 measured ~15 ms more in total (M3 Pro, release), about
/// 0.3% of a 5000-file first scan.
const BATCH: usize = 10;

/// A folder not opened for this long loses its rows at the next launch
/// (`evict`). Thumbnails are cheap to rebuild on the next open (a rescan) and a
/// month covers returning to a recent shoot, so it errs on keeping the cache
/// warm for what is actually in use.
pub const MAX_AGE_SECS: i64 = 30 * 24 * 60 * 60;

/// The size `evict` shrinks the database under, least-recently-opened folder
/// first, once the aged folders are gone. At the measured ~20.8 KB per row it
/// is ~50k files, a few seasons of shooting, while staying a small share of a
/// laptop's disk.
pub const MAX_BYTES: i64 = 1024 * 1024 * 1024;

/// What `evict` removes.
#[derive(Debug, Clone, Copy)]
pub struct EvictPolicy {
    pub max_age_secs: i64,
    pub max_bytes: i64,
}

pub const EVICT_POLICY: EvictPolicy = EvictPolicy {
    max_age_secs: MAX_AGE_SECS,
    max_bytes: MAX_BYTES,
};

/// How an `evict` ended.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EvictSummary {
    pub folders: usize,
    pub rows: usize,
    pub vacuumed: bool,
}

/// Seconds since the epoch, 0 for a clock set before it.
pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// Shortest interval between two progress notifications.
pub const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

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
    #[serde(serialize_with = "serialize_flag")]
    pub flag: Flag,
    pub label: Option<String>,
    pub has_sidecar: bool,
    pub sharpness: Option<f64>,
    /// `None` for a file whose extraction failed.
    pub exif: Option<Exif>,
}

/// The name the frontend uses for `flag`.
pub fn flag_name(flag: Flag) -> &'static str {
    match flag {
        Flag::None => "none",
        Flag::Pick => "pick",
        Flag::Reject => "reject",
    }
}

/// The flag `flag_name` names.
pub fn parse_flag(name: &str) -> Result<Flag, String> {
    match name {
        "none" => Ok(Flag::None),
        "pick" => Ok(Flag::Pick),
        "reject" => Ok(Flag::Reject),
        _ => Err(format!("unknown flag {name:?}")),
    }
}

fn serialize_flag<S: serde::Serializer>(flag: &Flag, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(flag_name(*flag))
}

/// The `ratings.flag` code of `flag`.
fn flag_code(flag: Flag) -> i64 {
    match flag {
        Flag::None => 0,
        Flag::Pick => 1,
        Flag::Reject => 2,
    }
}

fn flag_from_code(code: i64) -> Flag {
    match code {
        1 => Flag::Pick,
        2 => Flag::Reject,
        _ => Flag::None,
    }
}

/// The name the frontend uses for a face-catch state.
fn face_catch_name(state: FaceCatch) -> &'static str {
    match state {
        FaceCatch::Caught => "caught",
        FaceCatch::Missed => "missed",
        FaceCatch::Unknown => "unknown",
    }
}

fn serialize_face_catch<S: serde::Serializer>(state: &FaceCatch, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(face_catch_name(*state))
}

/// The `files.face_catch` code of `state`.
fn face_catch_code(state: FaceCatch) -> i64 {
    match state {
        FaceCatch::Unknown => 0,
        FaceCatch::Caught => 1,
        FaceCatch::Missed => 2,
    }
}

fn face_catch_from_code(code: i64) -> FaceCatch {
    match code {
        1 => FaceCatch::Caught,
        2 => FaceCatch::Missed,
        _ => FaceCatch::Unknown,
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Focus {
    pub sensor_w: u16,
    pub sensor_h: u16,
    pub x: u16,
    pub y: u16,
    /// The AF frame size, in the same sensor coordinates as `x` / `y`.
    pub frame: Option<FocusSize>,
    pub manual_focus: bool,
    #[serde(serialize_with = "serialize_face_catch")]
    pub face_catch: FaceCatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct FocusSize {
    pub width: u16,
    pub height: u16,
}

/// How a finished scan ended.
#[derive(Debug, Clone, Copy)]
pub struct ScanSummary {
    pub total: usize,
    pub errors: usize,
}

/// The columns `indexed_file` reads, before the `WHERE` clause.
const INDEXED_FILE: &str = "SELECT path, orientation, capture_time, subsec,
            focus_w, focus_h, focus_x, focus_y, thumb IS NOT NULL,
            ratings.rating, ratings.xmp_size IS NOT NULL,
            COALESCE(ratings.flag, 0), error IS NOT NULL,
            make, model, lens, f_num, f_den, f_estimated, exposure_num,
            exposure_den, iso, focal_num, focal_den, ratings.label,
            sharpness, frame_w, frame_h, manual_focus, face_catch
     FROM files LEFT JOIN ratings USING (path)";

fn indexed_file(r: &rusqlite::Row<'_>) -> rusqlite::Result<IndexedFile> {
    let focus = match (r.get::<_, Option<u16>>(4)?, r.get::<_, Option<u16>>(5)?) {
        (Some(sensor_w), Some(sensor_h)) => Some(Focus {
            sensor_w,
            sensor_h,
            x: r.get(6)?,
            y: r.get(7)?,
            frame: r
                .get::<_, Option<u16>>(26)?
                .zip(r.get::<_, Option<u16>>(27)?)
                .map(|(width, height)| FocusSize { width, height }),
            manual_focus: r.get(28)?,
            face_catch: face_catch_from_code(r.get(29)?),
        }),
        _ => None,
    };
    let rational = |num: usize| -> rusqlite::Result<Option<Rational>> {
        Ok(r.get::<_, Option<i64>>(num)?
            .zip(r.get::<_, Option<i64>>(num + 1)?)
            .map(|(num, den)| Rational { num, den }))
    };
    let exif = if r.get(12)? {
        None
    } else {
        Some(exif(&Shot {
            make: r.get(13)?,
            model: r.get(14)?,
            lens_model: r.get(15)?,
            f_number: rational(16)?,
            estimated_f_number: r.get(18)?,
            exposure_time: rational(19)?,
            iso: r.get(21)?,
            focal_length: rational(22)?,
            ..Shot::default()
        }))
    };
    Ok(IndexedFile {
        path: r.get(0)?,
        orientation: r.get::<_, Option<u16>>(1)?.unwrap_or(1),
        capture_time: r.get(2)?,
        subsec: r.get(3)?,
        focus,
        has_thumb: r.get(8)?,
        rating: r.get(9)?,
        flag: flag_from_code(r.get(11)?),
        label: r.get(24)?,
        has_sidecar: r.get(10)?,
        sharpness: r.get(25)?,
        exif,
    })
}

pub struct Index {
    pub(crate) conn: Connection,
}

/// Take a lock without caring whether a previous holder panicked. The scan
/// hands `on_item` to rayon workers, and a panic there would otherwise poison
/// the connection for the rest of the process.
pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Append a suffix to a path's file name, for the `-wal`/`-shm` sidecars SQLite
/// keeps next to the main database file.
pub(crate) fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
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
    /// Open, creating the file and the schema on first use. `v2` to `v4` are migrated
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

    /// Open a second, read-only connection on a database `open` has already
    /// returned `Ok` for, so the schema and the WAL files exist. Under WAL it
    /// reads a committed snapshot without waiting for the writer connection.
    pub fn open_reader(path: &Path) -> Result<Self, String> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| e.to_string())?;
        conn.busy_timeout(Duration::from_millis(300))
            .map_err(|e| e.to_string())?;
        Ok(Self { conn })
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
        if ![0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, SCHEMA_VERSION].contains(&version) {
            return Err(format!("unsupported index schema version {version}"));
        }
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        if version != 0 && version < 10 {
            tx.execute_batch("DROP TABLE IF EXISTS files;")
                .map_err(|e| e.to_string())?;
        }
        tx.execute_batch(
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
                     error TEXT,
                     make TEXT,
                     model TEXT,
                     lens TEXT,
                     f_num INTEGER,
                     f_den INTEGER,
                     f_estimated REAL,
                     exposure_num INTEGER,
                     exposure_den INTEGER,
                     iso INTEGER,
                     focal_num INTEGER,
                     focal_den INTEGER,
                     sharpness REAL,
                     extractor INTEGER NOT NULL DEFAULT 0,
                     frame_w INTEGER,
                     frame_h INTEGER,
                     manual_focus INTEGER NOT NULL DEFAULT 0,
                     face_catch INTEGER NOT NULL DEFAULT 0
                 );
                 CREATE INDEX IF NOT EXISTS files_dir ON files (dir);
                 CREATE INDEX IF NOT EXISTS files_capture ON files (capture_time, subsec);
                 -- `xmp_size` / `xmp_mtime_ns` are the stat of the effective
                 -- sidecar of the selected format (XMP or `.dop`; the newest
                 -- one under Both), whatever its name says.
                 -- `flag` is the pick / reject (`0` none, `1` pick, `2`
                 -- reject), kept apart from the `0`-`5` `rating` because the
                 -- two coexist in both an XMP and a `.dop`.
                 CREATE TABLE IF NOT EXISTS ratings (
                     path TEXT PRIMARY KEY,
                     dir TEXT NOT NULL,
                     rating INTEGER,
                     flag INTEGER NOT NULL DEFAULT 0,
                     label TEXT,
                     label_known INTEGER NOT NULL DEFAULT 1,
                     xmp_size INTEGER,
                     xmp_mtime_ns INTEGER,
                     dirty INTEGER NOT NULL DEFAULT 0
                 );
                 CREATE INDEX IF NOT EXISTS ratings_dir ON ratings (dir);
                 -- `opened_at` is seconds since the epoch, set by `reconcile`.
                 CREATE TABLE IF NOT EXISTS folders (
                     dir TEXT PRIMARY KEY,
                     opened_at INTEGER NOT NULL
                 );",
        )
        .map_err(|e| e.to_string())?;
        // `files` was just dropped, so this seeds nothing; kept so a v7
        // database still gains the `folders` table in the same way.
        if version == 7 {
            tx.execute(
                "INSERT OR IGNORE INTO folders (dir, opened_at)
                 SELECT DISTINCT dir, ?1 FROM files",
                params![now_secs()],
            )
            .map_err(|e| e.to_string())?;
        }
        if version == 2 {
            tx.execute_batch("ALTER TABLE ratings ADD COLUMN pick INTEGER NOT NULL DEFAULT 0;")
                .map_err(|e| e.to_string())?;
        }
        if version != 0 && version != SCHEMA_VERSION && version < 5 {
            tx.execute_batch("ALTER TABLE ratings ADD COLUMN label TEXT;")
                .map_err(|e| e.to_string())?;
        }
        if version != 0 && version != SCHEMA_VERSION && version < 6 {
            tx.execute_batch(
                "ALTER TABLE ratings ADD COLUMN label_known INTEGER NOT NULL DEFAULT 1;",
            )
            .map_err(|e| e.to_string())?;
        }
        if version != 0 && version < 11 {
            // Clean rows carry stats matching values written by the old,
            // lossy `read_pick` / `read_rating` shims (a Lightroom pick
            // stored as `pick = 0`, a starred reject stored with its stars
            // dropped). Invalidate their stat so `reconcile_sidecars`
            // re-parses the sidecar with the real readers instead of trusting
            // those stale values.
            tx.execute_batch(
                "ALTER TABLE ratings ADD COLUMN flag INTEGER NOT NULL DEFAULT 0;
                 UPDATE ratings SET flag = CASE WHEN rating = -1 THEN 2
                     WHEN pick = 1 THEN 1 ELSE 0 END;
                 UPDATE ratings SET rating = NULL WHERE rating = -1;
                 ALTER TABLE ratings DROP COLUMN pick;
                 UPDATE ratings SET xmp_size = NULL, xmp_mtime_ns = NULL WHERE dirty = 0;",
            )
            .map_err(|e| e.to_string())?;
        }
        // Below v10 `files` was just dropped and recreated with the column.
        if (10..12).contains(&version) {
            tx.execute_batch("ALTER TABLE files ADD COLUMN extractor INTEGER NOT NULL DEFAULT 0;")
                .map_err(|e| e.to_string())?;
        }
        if (10..13).contains(&version) {
            tx.execute_batch(
                "ALTER TABLE files ADD COLUMN frame_w INTEGER;
                 ALTER TABLE files ADD COLUMN frame_h INTEGER;
                 ALTER TABLE files ADD COLUMN manual_focus INTEGER NOT NULL DEFAULT 0;",
            )
            .map_err(|e| e.to_string())?;
        }
        if (10..14).contains(&version) {
            tx.execute_batch("ALTER TABLE files ADD COLUMN face_catch INTEGER NOT NULL DEFAULT 0;")
                .map_err(|e| e.to_string())?;
        }
        tx.pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }

    /// Drop the rows under `dir` whose file is gone, whose `size`/`mtime_ns`
    /// no longer match, or that were written at an older `EXTRACTOR_VERSION`,
    /// and return the files that have no valid row, in the
    /// order given. Paths are compared with the same lossy conversion
    /// `write_batch` uses to key rows, so a non-UTF-8 path matches the row it
    /// wrote instead of being rescanned on every open.
    pub fn reconcile(&mut self, dir: &str, files: &[FileStat]) -> Result<Vec<FileStat>, String> {
        let known: Vec<(String, i64, i64, i64)> = {
            let mut stmt = self
                .conn
                .prepare("SELECT path, size, mtime_ns, extractor FROM files WHERE dir = ?1")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![dir], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
                })
                .map_err(|e| e.to_string())?;
            rows.collect::<rusqlite::Result<_>>()
                .map_err(|e| e.to_string())?
        };
        let listed: std::collections::HashMap<String, (i64, i64, i64)> = files
            .iter()
            .map(|f| {
                (
                    f.path.to_string_lossy().into_owned(),
                    (f.size, f.mtime_ns, EXTRACTOR_VERSION),
                )
            })
            .collect();

        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO folders (dir, opened_at) VALUES (?1, ?2)
             ON CONFLICT (dir) DO UPDATE SET opened_at = excluded.opened_at",
            params![dir, now_secs()],
        )
        .map_err(|e| e.to_string())?;
        for (path, ..) in known
            .iter()
            .filter(|(p, s, m, v)| listed.get(p.as_str()) != Some(&(*s, *m, *v)))
        {
            tx.execute("DELETE FROM files WHERE path = ?1", params![path])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;

        let known: std::collections::HashMap<&str, (i64, i64, i64)> = known
            .iter()
            .map(|(p, s, m, v)| (p.as_str(), (*s, *m, *v)))
            .collect();
        Ok(files
            .iter()
            .filter(|f| {
                let path = f.path.to_string_lossy();
                known.get(path.as_ref()) != Some(&(f.size, f.mtime_ns, EXTRACTOR_VERSION))
            })
            .cloned()
            .collect())
    }

    /// Drop the `files` rows and clean `ratings` rows of folders last opened
    /// more than `policy.max_age_secs` before `now`, then of the
    /// least-recently-opened folders until the pages in use fit
    /// `policy.max_bytes`, and `VACUUM` if anything was deleted. A folder
    /// keeping a dirty rating keeps its `folders` row, so it is reconsidered
    /// next time. `VACUUM` rewrites the whole file, so call this with no scan
    /// running.
    pub fn evict(&mut self, now: i64, policy: EvictPolicy) -> Result<EvictSummary, String> {
        let folders: Vec<(String, i64)> = {
            let mut stmt = self
                .conn
                .prepare("SELECT dir, opened_at FROM folders ORDER BY opened_at, dir")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|e| e.to_string())?;
            rows.collect::<rusqlite::Result<_>>()
                .map_err(|e| e.to_string())?
        };
        let mut summary = EvictSummary::default();
        for (dir, opened_at) in &folders {
            if *opened_at >= now - policy.max_age_secs && self.used_bytes()? <= policy.max_bytes {
                break;
            }
            summary.rows += self.evict_folder(dir)?;
            summary.folders += 1;
        }
        self.vacuum_if_deleted(&mut summary)?;
        Ok(summary)
    }

    /// Drop the `files` rows and clean `ratings` rows of every folder, then
    /// `VACUUM` and truncate the WAL so the space is given back to the file
    /// system. A folder keeping a dirty rating keeps its `folders` row and
    /// that rating, as in `evict`. Like `evict`, call this with no scan
    /// running.
    pub fn clear(&mut self) -> Result<EvictSummary, String> {
        let folders: Vec<String> = {
            let mut stmt = self
                .conn
                .prepare("SELECT dir FROM folders ORDER BY dir")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| r.get(0))
                .map_err(|e| e.to_string())?;
            rows.collect::<rusqlite::Result<_>>()
                .map_err(|e| e.to_string())?
        };
        let mut summary = EvictSummary::default();
        for dir in &folders {
            summary.rows += self.evict_folder(dir)?;
            summary.folders += 1;
        }
        if self.vacuum_if_deleted(&mut summary)? {
            // `VACUUM` writes the rebuilt database through the WAL, which
            // keeps the old size on disk until a checkpoint truncates it. A
            // reader in a read transaction makes this fail, and the clear
            // itself still succeeded, so only log it.
            if let Err(e) = self
                .conn
                .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
            {
                log::warn!("could not truncate the index WAL: {e}");
            }
        }
        Ok(summary)
    }

    fn vacuum_if_deleted(&mut self, summary: &mut EvictSummary) -> Result<bool, String> {
        if summary.rows == 0 {
            return Ok(false);
        }
        self.conn
            .execute_batch("VACUUM")
            .map_err(|e| e.to_string())?;
        summary.vacuumed = true;
        Ok(true)
    }

    /// The bytes of the pages holding data, which a delete lowers at once
    /// while the file itself only shrinks at the `VACUUM`.
    fn used_bytes(&self) -> Result<i64, String> {
        let pragma = |name: &str| -> Result<i64, String> {
            self.conn
                .pragma_query_value(None, name, |r| r.get(0))
                .map_err(|e| e.to_string())
        };
        Ok((pragma("page_count")? - pragma("freelist_count")?) * pragma("page_size")?)
    }

    /// One folder's share of the deletion `evict` and `clear` do: the `files`
    /// rows, the clean `ratings` rows, and the `folders` row when no rating is
    /// left. Returns the rows deleted.
    fn evict_folder(&mut self, dir: &str) -> Result<usize, String> {
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        let mut rows = tx
            .execute("DELETE FROM files WHERE dir = ?1", params![dir])
            .map_err(|e| e.to_string())?;
        rows += tx
            .execute(
                "DELETE FROM ratings WHERE dir = ?1 AND dirty = 0",
                params![dir],
            )
            .map_err(|e| e.to_string())?;
        tx.execute(
            "DELETE FROM folders WHERE dir = ?1
                 AND NOT EXISTS (SELECT 1 FROM ratings WHERE dir = ?1)",
            params![dir],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(rows)
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
                    let shot = &entry.shot;
                    let focus = shot.focus;
                    tx.execute(
                        "INSERT OR REPLACE INTO files (path, dir, size, mtime_ns, orientation,
                             capture_time, subsec, focus_w, focus_h, focus_x, focus_y, thumb, error,
                             make, model, lens, f_num, f_den, f_estimated, exposure_num,
                             exposure_den, iso, focal_num, focal_den, sharpness, extractor,
                             frame_w, frame_h, manual_focus, face_catch)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, NULL,
                             ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25,
                             ?26, ?27, ?28, ?29)",
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
                            shot.make,
                            shot.model,
                            shot.lens_model,
                            shot.f_number.map(|r| r.num),
                            shot.f_number.map(|r| r.den),
                            shot.estimated_f_number,
                            shot.exposure_time.map(|r| r.num),
                            shot.exposure_time.map(|r| r.den),
                            shot.iso,
                            shot.focal_length.map(|r| r.num),
                            shot.focal_length.map(|r| r.den),
                            entry.sharpness,
                            EXTRACTOR_VERSION,
                            shot.focus_frame.map(|f| f.width),
                            shot.focus_frame.map(|f| f.height),
                            manual_focus(shot),
                            face_catch_code(entry.face_catch),
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                }
                Err(message) => {
                    tx.execute(
                        "INSERT OR REPLACE INTO files (path, dir, size, mtime_ns, orientation,
                             capture_time, subsec, focus_w, focus_h, focus_x, focus_y, thumb, error,
                             sharpness, extractor, frame_w, frame_h, manual_focus, face_catch)
                         VALUES (?1, ?2, ?3, ?4, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, ?5,
                             NULL, ?6, NULL, NULL, 0, 0)",
                        params![
                            path,
                            dir,
                            file.size,
                            file.mtime_ns,
                            message,
                            EXTRACTOR_VERSION
                        ],
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
            .prepare(&format!("{INDEXED_FILE} WHERE files.dir = ?1"))
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![dir], indexed_file)
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

    /// The indexed row of one file, `None` when the index has no row for it.
    pub fn entry(&self, path: &str) -> Result<Option<IndexedFile>, String> {
        self.conn
            .query_row(
                &format!("{INDEXED_FILE} WHERE files.path = ?1"),
                params![path],
                indexed_file,
            )
            .optional()
            .map_err(|e| e.to_string())
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

    /// Record a judgment for one file, pending a sidecar write.
    ///
    /// The row is independent of `files`, so a rescan or a failed extraction
    /// never drops a rating. `dirty = 1` until the writer has landed exactly
    /// this value in the sidecar.
    ///
    /// `label_known` is false when the caller has not yet learned this
    /// path's label (e.g. a judgment made before the first `folder_entries`
    /// refresh, or before the sidecar parse has stored it). The `label`
    /// column is then left untouched rather than being set to `label`
    /// (typically `None`), so a later sidecar parse or read is still free to
    /// fill it in and the app-side row never asserts a label it does not
    /// actually know. A fresh row instead records `label_known = 0`, so that
    /// state survives a crash before the writer drains it and a replayed
    /// `dirty` row (see `dirty_rows`) does not wrongly assert "no label" to
    /// the writer.
    pub fn set_rating(
        &mut self,
        dir: &str,
        path: &str,
        rating: Option<i8>,
        flag: Flag,
        label: Option<&str>,
        label_known: bool,
    ) -> Result<(), String> {
        let flag = flag_code(flag);
        let result = if label_known {
            self.conn.execute(
                "INSERT INTO ratings (path, dir, rating, flag, label, label_known, dirty)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, 1)
                 ON CONFLICT (path) DO UPDATE SET dir = ?2, rating = ?3, flag = ?4, label = ?5,
                     label_known = 1, dirty = 1",
                params![path, dir, rating, flag, label],
            )
        } else {
            self.conn.execute(
                "INSERT INTO ratings (path, dir, rating, flag, label_known, dirty)
                 VALUES (?1, ?2, ?3, ?4, 0, 1)
                 ON CONFLICT (path) DO UPDATE SET dir = ?2, rating = ?3, flag = ?4,
                     dirty = 1",
                params![path, dir, rating, flag],
            )
        };
        result.map(|_| ()).map_err(|e| format!("{path}: {e}"))
    }

    /// Clear `dirty` and store the sidecar's stat, but only while the row
    /// still holds the judgment that was written: a keypress during the write
    /// leaves the row dirty so the newer value is written in turn.
    ///
    /// `stat` is `None` when no sidecar exists (clearing a rating on a file
    /// that never had one writes nothing).
    ///
    /// `label_known` is false when the label `set_rating` was given for this
    /// judgment was not asserted by the app: the writer instead kept the
    /// sidecar's own current label, passed here as `label` (the label
    /// actually resolved and written, see `sidecar::write`). That resolved
    /// label is now known to match the sidecar, so it and `label_known = 1`
    /// are stored regardless, which is what lets a later folder open (or a
    /// crash before this write even lands) treat the label as known instead
    /// of replaying it as "no label" and stripping the sidecar's own label
    /// (see `dirty_rows` and its caller).
    pub fn mark_written(
        &mut self,
        path: &str,
        rating: Option<i8>,
        flag: Flag,
        label: Option<&str>,
        label_known: bool,
        stat: Option<(i64, i64)>,
    ) -> Result<bool, String> {
        let flag = flag_code(flag);
        let (size, mtime_ns) = (stat.map(|s| s.0), stat.map(|s| s.1));
        let n = if label_known {
            self.conn.execute(
                "UPDATE ratings SET dirty = 0, xmp_size = ?2, xmp_mtime_ns = ?3
                 WHERE path = ?1 AND rating IS ?4 AND flag = ?5 AND label IS ?6",
                params![path, size, mtime_ns, rating, flag, label],
            )
        } else {
            self.conn.execute(
                "UPDATE ratings SET dirty = 0, xmp_size = ?2, xmp_mtime_ns = ?3,
                     label = ?6, label_known = 1
                 WHERE path = ?1 AND rating IS ?4 AND flag = ?5
                     AND (label_known = 0 OR label IS ?6)",
                params![path, size, mtime_ns, rating, flag, label],
            )
        }
        .map_err(|e| format!("{path}: {e}"))?;
        Ok(n > 0)
    }

    /// Record the stat of the sidecar a `Both` write did manage to write
    /// before its other sidecar failed and the retries ran out, without
    /// clearing `dirty`: the row still holds a judgment unwritten to that
    /// other sidecar. Guarded on the row still holding the judgment that was
    /// written and still being dirty, the same way `mark_written` is, so a
    /// keypress during the write or a write that already got cleaned up does
    /// not have its stat overwritten. Without this, the next folder open
    /// would see the freshly written sidecar's changed stat and mistake it
    /// for an external edit (see `sidecar::write`), storing it and clearing
    /// `dirty` while the other sidecar stays stale for good.
    pub fn mark_partial_write(
        &mut self,
        path: &str,
        rating: Option<i8>,
        flag: Flag,
        stat: (i64, i64),
    ) -> Result<bool, String> {
        let flag = flag_code(flag);
        let n = self
            .conn
            .execute(
                "UPDATE ratings SET xmp_size = ?2, xmp_mtime_ns = ?3
                 WHERE path = ?1 AND rating IS ?4 AND flag = ?5 AND dirty = 1",
                params![path, stat.0, stat.1, rating, flag],
            )
            .map_err(|e| format!("{path}: {e}"))?;
        Ok(n > 0)
    }

    /// Forget every sidecar the index has seen, for a sidecar format switch:
    /// clean rows are dropped so the next open reads the newly selected
    /// format, and dirty rows keep their judgment but lose the old format's
    /// stat, so the next open writes them into the new one. Their whole
    /// judgment is kept, including the flag, since both formats hold it.
    pub fn reset_sidecars(&mut self) -> Result<(), String> {
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM ratings WHERE dirty = 0", [])
            .map_err(|e| e.to_string())?;
        tx.execute(
            "UPDATE ratings SET xmp_size = NULL, xmp_mtime_ns = NULL WHERE dirty = 1",
            [],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }

    /// The rows of `dir` whose judgment has not reached its sidecar yet,
    /// with each row's own `label_known` (see `set_rating`) rather than a
    /// blanket "known": a row created before the label was learned must
    /// still be replayed as unknown, or the writer would strip whatever
    /// label the sidecar already holds.
    pub fn dirty_rows(&self, dir: &str) -> Result<Vec<DirtyRow>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT path, rating, flag, label, label_known FROM ratings
                 WHERE dir = ?1 AND dirty = 1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![dir], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    flag_from_code(r.get(2)?),
                    r.get(3)?,
                    r.get(4)?,
                ))
            })
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
    /// clean clears the rating, flag and label, because the truth is gone with it; and a row
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
                            "UPDATE ratings SET rating = NULL, flag = 0, label = NULL,
                                 label_known = 1, xmp_size = NULL, xmp_mtime_ns = NULL
                                 WHERE path = ?1",
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

    /// Store what parsing the sidecars of `dir` found: the rating, flag and label
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
        for (path, rating, flag, label, size, mtime_ns, dirty) in rows {
            let flag = flag_code(*flag);
            tx.execute(
                "INSERT INTO ratings (path, dir, rating, flag, label, label_known, xmp_size, xmp_mtime_ns, dirty)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, 0)
                 ON CONFLICT (path) DO UPDATE SET dir = ?2, rating = ?3, flag = ?4, label = ?5,
                     label_known = 1, xmp_size = ?6, xmp_mtime_ns = ?7, dirty = 0
                 WHERE dirty = ?8",
                params![path, dir, rating, flag, label, size, mtime_ns, *dirty as i64],
            )
            .map_err(|e| format!("{path}: {e}"))?;
        }
        tx.commit().map_err(|e| e.to_string())
    }
}

/// The part of a `ratings` row the folder-open reconciliation looks at: the
/// stat of the sidecar as the app last saw it, and whether a judgment is
/// still waiting to be written.
struct RatingRow {
    stat: (Option<i64>, Option<i64>),
    dirty: bool,
}

/// What one file's sidecar looks like on disk, as seen by the single
/// directory listing a folder open does.
pub type SidecarStat = (PathBuf, i64, i64);

/// What parsing one file's sidecar found, for `store_sidecar_ratings`:
/// `(path, rating, flag, label, size, mtime_ns, dirty)`, the last being the
/// `dirty` flag `reconcile_sidecars` observed.
pub type ParsedSidecar = (String, Option<i8>, Flag, Option<String>, i64, i64, bool);

/// A row `dirty_rows` hands to the writer: `(path, rating, flag, label,
/// label_known)`.
pub type DirtyRow = (String, Option<i8>, Flag, Option<String>, bool);

/// Extract `files` and write them into `index` in batched transactions,
/// reporting `(done, total, ready)` through `progress` at most every
/// `progress_interval` (`PROGRESS_INTERVAL` in the app), always on the first
/// file and once more after the trailing flush. `ready` holds the paths whose
/// rows a batch committed since the previous notification, so a path is
/// reported only once the index can answer for it; a batch whose write failed
/// is not reported at all.
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
    progress_interval: Duration,
    progress: P,
) -> ScanSummary
where
    P: Fn(usize, usize, Vec<String>) + Send + Sync,
{
    let total = files.len();
    let paths: Vec<PathBuf> = files.iter().map(|f| f.path.clone()).collect();
    let pending: Mutex<Vec<(FileStat, Result<Entry, String>)>> = Mutex::new(Vec::new());
    let done = AtomicUsize::new(0);
    let errors = AtomicUsize::new(0);
    let last = Mutex::new(None::<Instant>);
    let ready: Mutex<Vec<String>> = Mutex::new(Vec::new());

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
            return;
        }
        lock(&ready).extend(
            batch
                .iter()
                .map(|(file, _)| file.path.to_string_lossy().into_owned()),
        );
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
            if last.is_none_or(|t| now.duration_since(t) >= progress_interval) {
                *last = Some(now);
                true
            } else {
                false
            }
        };
        if due {
            progress(done, total, std::mem::take(&mut *lock(&ready)));
        }
    };

    let started = Instant::now();
    if let Err(e) = extract_all(&paths, threads, on_item, cancel) {
        log::error!("scan of {dir} failed: {e}");
    }
    flush(std::mem::take(&mut *lock(&pending)));
    let done_count = done.load(Ordering::Relaxed);
    progress(done_count, total, std::mem::take(&mut *lock(&ready)));

    let summary = ScanSummary {
        total: done_count,
        errors: errors.load(Ordering::Relaxed),
    };
    log::info!(
        "scan extract: dir={dir} files={total} done={} errors={} threads={threads} canceled={} in {}ms",
        summary.total,
        summary.errors,
        cancel.load(Ordering::Relaxed),
        started.elapsed().as_millis()
    );
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use riffle_core::arw::{FocusFrame, FocusLocation, Shot};
    use std::collections::HashSet;

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
                make: Some("SONY".to_string()),
                model: Some("ILCE-7M5".to_string()),
                lens_model: Some("FE 50mm F1.4 GM".to_string()),
                exposure_time: Some(Rational { num: 10, den: 2500 }),
                f_number: Some(Rational { num: 14, den: 10 }),
                iso: Some(800),
                focal_length: Some(Rational { num: 500, den: 10 }),
                ..Shot::default()
            },
            thumbnail: vec![0xff, 0xd8, 0xff, 0xd9],
            sharpness: None,
            face_catch: FaceCatch::Unknown,
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
        assert_eq!(entries[0].exif, Some(exif(&entry().shot)));
        let e = entries[0].exif.as_ref().unwrap();
        assert_eq!(e.camera.as_deref(), Some("SONY ILCE-7M5"));
        assert_eq!(e.lens.as_deref(), Some("FE 50mm F1.4 GM"));
        assert_eq!(e.aperture.as_ref().unwrap().label, "f/1.4");
        assert_eq!(e.shutter.as_ref().unwrap().label, "1/250");
        assert_eq!(e.iso.as_ref().unwrap().label, "800");
        assert_eq!(e.focal_length.as_ref().unwrap().label, "50 mm");

        remove_temp_dir(&dir);
    }

    #[test]
    fn an_estimated_aperture_round_trips() {
        let dir = temp_dir("estimated");
        let a = file(&dir, "a.ARW", b"a");
        let mut index = open(&dir);
        let mut e = entry();
        e.shot = Shot {
            estimated_f_number: Some(2.0),
            ..Shot::default()
        };
        index.write_batch("d", &[(a, Ok(e))]).unwrap();
        let exif = index.entries("d").unwrap()[0].exif.clone().unwrap();
        assert_eq!(exif.aperture.unwrap().label, "f/2 (est.)");
        assert_eq!(exif.camera, None);

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v3_database_drops_its_files_rows_and_keeps_its_dirty_ratings() {
        let dir = temp_dir("migrate-v3");
        let db = dir.join("index.sqlite");
        {
            let conn = Connection::open(&db).unwrap();
            conn.execute_batch(
                "CREATE TABLE files (
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
                 INSERT INTO files (path, dir, size, mtime_ns, thumb)
                     VALUES ('/a.ARW', 'd', 1, 1, x'ffd8ffd9');
                 CREATE TABLE ratings (
                     path TEXT PRIMARY KEY,
                     dir TEXT NOT NULL,
                     rating INTEGER,
                     pick INTEGER NOT NULL DEFAULT 0,
                     xmp_size INTEGER,
                     xmp_mtime_ns INTEGER,
                     dirty INTEGER NOT NULL DEFAULT 0
                 );
                 INSERT INTO ratings (path, dir, rating, pick, dirty)
                     VALUES ('/a.ARW', 'd', 4, 1, 1);
                 PRAGMA user_version = 3;",
            )
            .unwrap();
        }

        let index = Index::open(&db).unwrap();
        assert!(index.entries("d").unwrap().is_empty());
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [("/a.ARW".to_string(), Some(4), Flag::Pick, None, true)]
        );
        drop(index);
        let index = Index::open(&db).unwrap();
        let version: i64 = index
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 14);
        assert_eq!(index.dirty_rows("d").unwrap().len(), 1);

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v6_database_drops_its_files_rows_and_keeps_its_ratings() {
        let dir = temp_dir("migrate-v6");
        let db = dir.join("index.sqlite");
        {
            let conn = Connection::open(&db).unwrap();
            conn.execute_batch(
                "CREATE TABLE files (
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
                     error TEXT,
                     make TEXT,
                     model TEXT,
                     lens TEXT,
                     f_num INTEGER,
                     f_den INTEGER,
                     f_estimated REAL,
                     exposure_num INTEGER,
                     exposure_den INTEGER,
                     iso INTEGER,
                     focal_num INTEGER,
                     focal_den INTEGER
                 );
                 INSERT INTO files (path, dir, size, mtime_ns, thumb)
                     VALUES ('/a.ARW', 'd', 1, 1, x'ffd8ffd9');
                 CREATE TABLE ratings (
                     path TEXT PRIMARY KEY,
                     dir TEXT NOT NULL,
                     rating INTEGER,
                     pick INTEGER NOT NULL DEFAULT 0,
                     label TEXT,
                     label_known INTEGER NOT NULL DEFAULT 1,
                     xmp_size INTEGER,
                     xmp_mtime_ns INTEGER,
                     dirty INTEGER NOT NULL DEFAULT 0
                 );
                 INSERT INTO ratings (path, dir, rating, pick, label, dirty)
                     VALUES ('/a.ARW', 'd', 4, 1, 'Red', 1);
                 PRAGMA user_version = 6;",
            )
            .unwrap();
        }

        let index = Index::open(&db).unwrap();
        assert!(index.entries("d").unwrap().is_empty());
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [(
                "/a.ARW".to_string(),
                Some(4),
                Flag::Pick,
                Some("Red".to_string()),
                true
            )]
        );
        let version: i64 = index
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 14);

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v10_database_turns_its_picks_and_rejects_into_flags_and_keeps_its_files() {
        let dir = temp_dir("migrate-v10");
        let db = dir.join("index.sqlite");
        let a = synthetic(&dir, 0);
        {
            let mut index = open(&dir);
            index.write_batch("d", &[(a.clone(), Ok(entry()))]).unwrap();
            index
                .conn
                .execute_batch(
                    "ALTER TABLE ratings RENAME COLUMN flag TO pick;
                     ALTER TABLE files DROP COLUMN extractor;
                     ALTER TABLE files DROP COLUMN frame_w;
                     ALTER TABLE files DROP COLUMN frame_h;
                     ALTER TABLE files DROP COLUMN manual_focus;
                     ALTER TABLE files DROP COLUMN face_catch;
                     INSERT INTO ratings (path, dir, rating, pick, dirty, xmp_size, xmp_mtime_ns) VALUES
                         ('/rejected.ARW', 'd', -1, 0, 1, 10, 20),
                         ('/picked.ARW', 'd', 3, 1, 0, 10, 20),
                         ('/plain.ARW', 'd', 2, 0, 0, 10, 20);
                     PRAGMA user_version = 10;",
                )
                .unwrap();
        }

        let index = Index::open(&db).unwrap();
        let version: i64 = index
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 14);
        assert_eq!(index.entries("d").unwrap().len(), 1, "files are kept");
        let rows: Vec<(String, Option<i8>, i64, i64)> = index
            .conn
            .prepare("SELECT path, rating, flag, dirty FROM ratings ORDER BY path")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(
            rows,
            [
                ("/picked.ARW".to_string(), Some(3), 1, 0),
                ("/plain.ARW".to_string(), Some(2), 0, 0),
                ("/rejected.ARW".to_string(), None, 2, 1),
            ]
        );
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [("/rejected.ARW".to_string(), None, Flag::Reject, None, true)]
        );
        let stats: Vec<(String, Option<i64>, Option<i64>)> = index
            .conn
            .prepare("SELECT path, xmp_size, xmp_mtime_ns FROM ratings ORDER BY path")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(
            stats,
            [
                ("/picked.ARW".to_string(), None, None),
                ("/plain.ARW".to_string(), None, None),
                ("/rejected.ARW".to_string(), Some(10), Some(20)),
            ],
            "the stat of clean rows is invalidated so reconcile_sidecars re-parses \
             them with the real readers, but a dirty row keeps its stat"
        );

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_sharpness_score_round_trips() {
        let dir = temp_dir("sharpness");
        let a = file(&dir, "a.ARW", b"a");
        let b = file(&dir, "b.ARW", b"b");
        let mut index = open(&dir);
        let mut scored = entry();
        scored.sharpness = Some(123.5);
        index
            .write_batch("d", &[(a, Ok(scored)), (b, Ok(entry()))])
            .unwrap();
        let entries = index.entries("d").unwrap();
        assert_eq!(entries[0].sharpness, Some(123.5));
        assert_eq!(entries[1].sharpness, None);

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
    fn a_row_at_an_older_extractor_version_is_re_extracted() {
        let dir = temp_dir("extractor");
        let a = file(&dir, "a.ARW", b"a");
        let b = file(&dir, "b.ARW", b"b");
        let mut index = open(&dir);
        index
            .write_batch(
                "d",
                &[
                    (a.clone(), Ok(entry())),
                    (b.clone(), Err("broken".to_string())),
                ],
            )
            .unwrap();
        index
            .conn
            .execute_batch("UPDATE files SET extractor = 0")
            .unwrap();

        let stale = index.reconcile("d", &[a.clone(), b.clone()]).unwrap();
        assert_eq!(
            stale.iter().map(|f| f.path.clone()).collect::<Vec<_>>(),
            [a.path.clone(), b.path.clone()],
            "both the success row and the error row are returned"
        );
        assert!(index.entries("d").unwrap().is_empty());

        index
            .write_batch(
                "d",
                &[
                    (a.clone(), Ok(entry())),
                    (b.clone(), Err("broken".to_string())),
                ],
            )
            .unwrap();
        assert_eq!(index.entries("d").unwrap().len(), 2);
        assert!(index.reconcile("d", &[a, b]).unwrap().is_empty());

        remove_temp_dir(&dir);
    }

    #[test]
    fn rows_at_the_current_extractor_version_are_kept() {
        let dir = temp_dir("extractor-current");
        let a = file(&dir, "a.ARW", b"a");
        let b = file(&dir, "b.ARW", b"b");
        let mut index = open(&dir);
        index
            .write_batch(
                "d",
                &[
                    (a.clone(), Ok(entry())),
                    (b.clone(), Err("broken".to_string())),
                ],
            )
            .unwrap();

        assert!(index.reconcile("d", &[a, b]).unwrap().is_empty());
        assert_eq!(index.entries("d").unwrap().len(), 2);

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v11_database_gains_the_extractor_column_and_keeps_its_files_and_ratings() {
        let dir = temp_dir("migrate-v11");
        let db = dir.join("index.sqlite");
        let a = file(&dir, "a.ARW", b"a");
        {
            let mut index = open(&dir);
            index.write_batch("d", &[(a.clone(), Ok(entry()))]).unwrap();
            index
                .conn
                .execute_batch(
                    "INSERT INTO ratings (path, dir, rating, flag, dirty) VALUES
                         ('/rated.ARW', 'd', 4, 1, 1);
                     ALTER TABLE files DROP COLUMN extractor;
                     ALTER TABLE files DROP COLUMN frame_w;
                     ALTER TABLE files DROP COLUMN frame_h;
                     ALTER TABLE files DROP COLUMN manual_focus;
                     ALTER TABLE files DROP COLUMN face_catch;
                     PRAGMA user_version = 11;",
                )
                .unwrap();
        }

        let mut index = Index::open(&db).unwrap();
        let version: i64 = index
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 14);
        assert_eq!(index.entries("d").unwrap().len(), 1, "files are kept");
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [("/rated.ARW".to_string(), Some(4), Flag::Pick, None, true)]
        );
        assert_eq!(index.reconcile("d", &[a]).unwrap().len(), 1);

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v12_database_gains_the_frame_and_manual_focus_columns_and_keeps_its_files_and_ratings() {
        let dir = temp_dir("migrate-v12");
        let db = dir.join("index.sqlite");
        let a = file(&dir, "a.ARW", b"a");
        {
            let mut index = open(&dir);
            index.write_batch("d", &[(a.clone(), Ok(entry()))]).unwrap();
            index
                .conn
                .execute_batch(
                    "INSERT INTO ratings (path, dir, rating, flag, dirty) VALUES
                         ('/rated.ARW', 'd', 4, 1, 1);
                     UPDATE files SET extractor = 1;
                     ALTER TABLE files DROP COLUMN frame_w;
                     ALTER TABLE files DROP COLUMN frame_h;
                     ALTER TABLE files DROP COLUMN manual_focus;
                     ALTER TABLE files DROP COLUMN face_catch;
                     PRAGMA user_version = 12;",
                )
                .unwrap();
        }

        let mut index = Index::open(&db).unwrap();
        let version: i64 = index
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 14);
        let entries = index.entries("d").unwrap();
        assert_eq!(entries.len(), 1, "files are kept");
        let focus = entries[0].focus.unwrap();
        assert!(focus.frame.is_none());
        assert!(!focus.manual_focus);
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [("/rated.ARW".to_string(), Some(4), Flag::Pick, None, true)]
        );
        assert_eq!(index.reconcile("d", &[a]).unwrap().len(), 1);

        remove_temp_dir(&dir);
    }

    #[test]
    fn the_af_frame_and_manual_focus_round_trip() {
        let dir = temp_dir("frame");
        let a = file(&dir, "a.ARW", b"a");
        let b = file(&dir, "b.ARW", b"b");
        let mut index = open(&dir);
        let mut framed = entry();
        framed.shot.focus_frame = Some(FocusFrame {
            width: 832,
            height: 624,
        });
        framed.shot.focus_mode = Some(0);
        index
            .write_batch("d", &[(a, Ok(framed)), (b, Ok(entry()))])
            .unwrap();

        let entries = index.entries("d").unwrap();
        let framed = entries[0].focus.unwrap();
        assert_eq!(
            framed.frame,
            Some(FocusSize {
                width: 832,
                height: 624
            })
        );
        assert!(framed.manual_focus);
        let plain = entries[1].focus.unwrap();
        assert!(plain.frame.is_none());
        assert!(!plain.manual_focus);

        remove_temp_dir(&dir);
    }

    #[test]
    fn the_face_catch_state_round_trips_as_a_string() {
        let dir = temp_dir("face-catch");
        let a = file(&dir, "a.ARW", b"a");
        let b = file(&dir, "b.ARW", b"b");
        let c = file(&dir, "c.ARW", b"c");
        let mut index = open(&dir);
        let mut caught = entry();
        caught.face_catch = FaceCatch::Caught;
        let mut missed = entry();
        missed.face_catch = FaceCatch::Missed;
        index
            .write_batch("d", &[(a, Ok(caught)), (b, Ok(missed)), (c, Ok(entry()))])
            .unwrap();

        let states: Vec<String> = index
            .entries("d")
            .unwrap()
            .iter()
            .map(|e| {
                let focus = serde_json::to_value(e.focus.unwrap()).unwrap();
                focus["face_catch"].as_str().unwrap().to_string()
            })
            .collect();
        assert_eq!(states, ["caught", "missed", "unknown"]);

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v13_database_gains_the_face_catch_column_and_keeps_its_files_and_ratings() {
        let dir = temp_dir("migrate-v13");
        let db = dir.join("index.sqlite");
        let a = file(&dir, "a.ARW", b"a");
        {
            let mut index = open(&dir);
            index.write_batch("d", &[(a.clone(), Ok(entry()))]).unwrap();
            index
                .conn
                .execute_batch(
                    "INSERT INTO ratings (path, dir, rating, flag, dirty) VALUES
                         ('/rated.ARW', 'd', 4, 1, 1);
                     UPDATE files SET extractor = 3;
                     ALTER TABLE files DROP COLUMN face_catch;
                     PRAGMA user_version = 13;",
                )
                .unwrap();
        }

        let mut index = Index::open(&db).unwrap();
        let version: i64 = index
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 14);
        let entries = index.entries("d").unwrap();
        assert_eq!(entries.len(), 1, "files are kept");
        assert_eq!(entries[0].focus.unwrap().face_catch, FaceCatch::Unknown);
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [("/rated.ARW".to_string(), Some(4), Flag::Pick, None, true)]
        );
        assert_eq!(index.reconcile("d", &[a]).unwrap().len(), 1);

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
        index
            .set_rating("d", &path, Some(2), Flag::Reject, None, true)
            .unwrap();
        let judged = &index.entries("d").unwrap()[0];
        assert_eq!((judged.rating, judged.flag), (Some(2), Flag::Reject));

        // A reconcile that drops the `files` row must leave `ratings` alone:
        // the sidecar, not the scan, is what a rating belongs to.
        assert!(index.reconcile("d", &[]).unwrap().is_empty());
        assert!(index.entries("d").unwrap().is_empty());
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [(path.clone(), Some(2), Flag::Reject, None, true)]
        );

        index.write_batch("d", &[(a, Ok(entry()))]).unwrap();
        let judged = &index.entries("d").unwrap()[0];
        assert_eq!((judged.rating, judged.flag), (Some(2), Flag::Reject));

        remove_temp_dir(&dir);
    }

    #[test]
    fn entry_reads_one_row_by_path() {
        let dir = temp_dir("entry");
        let a = file(&dir, "a.ARW", b"a");
        let b = file(&dir, "b.ARW", b"b");
        let path = b.path.to_string_lossy().into_owned();
        let mut index = open(&dir);
        index
            .write_batch("d", &[(a, Ok(entry())), (b, Ok(entry()))])
            .unwrap();
        index
            .set_rating("d", &path, Some(4), Flag::Pick, Some("Red"), true)
            .unwrap();

        let row = index.entry(&path).unwrap().unwrap();
        assert_eq!(row.path, path);
        assert_eq!(
            (row.rating, row.flag, row.label.as_deref()),
            (Some(4), Flag::Pick, Some("Red"))
        );
        assert!(index.entry("d/missing.ARW").unwrap().is_none());

        remove_temp_dir(&dir);
    }

    #[test]
    fn clearing_everything_nulls_the_rating_flag_and_label_of_a_row() {
        let dir = temp_dir("clear-all");
        let mut index = open(&dir);
        index
            .set_rating("d", "/a.ARW", Some(4), Flag::Pick, Some("Red"), true)
            .unwrap();
        index
            .mark_written("/a.ARW", Some(4), Flag::Pick, Some("Red"), true, None)
            .unwrap();

        index
            .set_rating("d", "/a.ARW", None, Flag::None, None, true)
            .unwrap();

        let row: (Option<i8>, i64, Option<String>, i64, i64) = index
            .conn
            .query_row(
                "SELECT rating, flag, label, label_known, dirty FROM ratings WHERE path = ?1",
                ["/a.ARW"],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();
        assert_eq!(row, (None, 0, None, 1, 1));

        remove_temp_dir(&dir);
    }

    #[test]
    fn mark_written_only_clears_a_row_that_still_holds_the_written_value() {
        let dir = temp_dir("written");
        let mut index = open(&dir);
        index
            .set_rating("d", "/a.ARW", Some(3), Flag::None, None, true)
            .unwrap();

        assert!(index
            .mark_written("/a.ARW", Some(3), Flag::None, None, true, Some((42, 7)))
            .unwrap());
        assert!(index.dirty_rows("d").unwrap().is_empty());

        index
            .set_rating("d", "/a.ARW", Some(5), Flag::None, None, true)
            .unwrap();
        assert!(
            !index
                .mark_written("/a.ARW", Some(3), Flag::None, None, true, Some((42, 7)))
                .unwrap(),
            "a keypress during the write keeps the row dirty"
        );
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [("/a.ARW".to_string(), Some(5), Flag::None, None, true)]
        );

        // An unrated row is matched by NULL, not skipped.
        index
            .set_rating("d", "/a.ARW", None, Flag::None, None, true)
            .unwrap();
        assert!(index
            .mark_written("/a.ARW", None, Flag::None, None, true, None)
            .unwrap());
        assert!(index.dirty_rows("d").unwrap().is_empty());

        remove_temp_dir(&dir);
    }

    #[test]
    fn mark_written_for_an_unknown_label_leaves_a_row_dirty_once_it_asserts_a_different_label() {
        let dir = temp_dir("written-unknown-label-superseded");
        let mut index = open(&dir);
        // An unknown-label write for "Red" is in flight...
        index
            .set_rating("d", "/a.ARW", Some(3), Flag::None, None, false)
            .unwrap();

        // ...but a newer judgment with the same rating/flag asserts "Blue"
        // before that write lands.
        index
            .set_rating("d", "/a.ARW", Some(3), Flag::None, Some("Blue"), true)
            .unwrap();

        // The stale unknown-label write must not overwrite the newer,
        // known label, and must leave the row dirty so it is retried.
        assert!(
            !index
                .mark_written(
                    "/a.ARW",
                    Some(3),
                    Flag::None,
                    Some("Red"),
                    false,
                    Some((42, 7))
                )
                .unwrap(),
            "a row that has since asserted a different label is left dirty"
        );
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [(
                "/a.ARW".to_string(),
                Some(3),
                Flag::None,
                Some("Blue".to_string()),
                true
            )]
        );

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_sidecar_reset_drops_clean_rows_and_clears_the_stat_of_dirty_ones() {
        let dir = temp_dir("reset-sidecars");
        let mut index = open(&dir);
        index
            .set_rating("d", "/a.ARW", Some(3), Flag::None, None, true)
            .unwrap();
        assert!(index
            .mark_written("/a.ARW", Some(3), Flag::None, None, true, Some((42, 7)))
            .unwrap());
        index
            .set_rating("d", "/b.ARW", Some(5), Flag::None, None, true)
            .unwrap();
        index
            .store_sidecar_ratings(
                "d",
                &[("/b.ARW".to_string(), Some(5), Flag::None, None, 9, 9, true)],
            )
            .unwrap();
        index
            .set_rating("d", "/b.ARW", None, Flag::Reject, None, true)
            .unwrap();

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
            [("/b.ARW".to_string(), None, Flag::Reject, None, true)]
        );

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_flag_is_stored_beside_the_rating_and_guards_mark_written() {
        let dir = temp_dir("flag");
        let a = file(&dir, "a.ARW", b"a");
        let path = a.path.to_string_lossy().into_owned();
        let mut index = open(&dir);
        index.write_batch("d", &[(a, Ok(entry()))]).unwrap();
        assert_eq!(
            index.entries("d").unwrap()[0].flag,
            Flag::None,
            "no ratings row"
        );

        index
            .set_rating("d", &path, Some(3), Flag::Pick, None, true)
            .unwrap();
        let entry = &index.entries("d").unwrap()[0];
        assert_eq!((entry.rating, entry.flag), (Some(3), Flag::Pick));

        assert!(
            !index
                .mark_written(&path, Some(3), Flag::None, None, true, None)
                .unwrap(),
            "an unpick during the write keeps the row dirty"
        );
        assert!(index
            .mark_written(&path, Some(3), Flag::Pick, None, true, None)
            .unwrap());

        index
            .store_sidecar_ratings(
                "d",
                &[(path.clone(), Some(0), Flag::None, None, 1, 2, false)],
            )
            .unwrap();
        assert_eq!(
            index.entries("d").unwrap()[0].flag,
            Flag::None,
            "the sidecar wins"
        );

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_label_is_stored_beside_the_rating_and_guards_mark_written() {
        let dir = temp_dir("label");
        let a = file(&dir, "a.ARW", b"a");
        let path = a.path.to_string_lossy().into_owned();
        let mut index = open(&dir);
        index.write_batch("d", &[(a, Ok(entry()))]).unwrap();
        assert_eq!(index.entries("d").unwrap()[0].label, None, "no ratings row");

        index
            .set_rating("d", &path, Some(3), Flag::None, Some("Red"), true)
            .unwrap();
        assert_eq!(index.entries("d").unwrap()[0].label.as_deref(), Some("Red"));

        assert!(
            !index
                .mark_written(&path, Some(3), Flag::None, Some("Blue"), true, None)
                .unwrap(),
            "a relabel during the write keeps the row dirty"
        );
        assert!(!index
            .mark_written(&path, Some(3), Flag::None, None, true, None)
            .unwrap());
        assert!(index
            .mark_written(&path, Some(3), Flag::None, Some("Red"), true, None)
            .unwrap());

        remove_temp_dir(&dir);
    }

    #[test]
    fn an_unknown_label_leaves_the_ratings_label_untouched() {
        let dir = temp_dir("label-unknown");
        let a = file(&dir, "a.ARW", b"a");
        let path = a.path.to_string_lossy().into_owned();
        let mut index = open(&dir);
        index.write_batch("d", &[(a, Ok(entry()))]).unwrap();

        // A judgment before the row's label is known must not stamp `label`
        // as `None`: a later sidecar parse still needs to be free to fill it
        // in, and mark_written must not be guarded on a value never asserted.
        index
            .set_rating("d", &path, Some(3), Flag::None, None, false)
            .unwrap();
        assert_eq!(
            index.entries("d").unwrap()[0].label,
            None,
            "no label was ever asserted, so the row stays as the sidecar parse left it"
        );

        // Simulate the sidecar parse landing afterwards: it is free to set
        // the label, since `set_rating` above never touched the column. The
        // row is still dirty from that judgment, so the snapshot passed
        // here must say so too.
        index
            .store_sidecar_ratings(
                "d",
                &[(
                    path.clone(),
                    Some(3),
                    Flag::None,
                    Some("Red".into()),
                    1,
                    2,
                    true,
                )],
            )
            .unwrap();
        assert_eq!(index.entries("d").unwrap()[0].label.as_deref(), Some("Red"));

        // `mark_written` for the same unknown-label judgment must not check
        // `label` at all, since `ratings.label` was never set to it, but the
        // resolved label the writer actually put in the sidecar ("Red", the
        // one already there) is stored and marked known regardless, so a
        // later replay of a still-dirty row (see `dirty_rows`) never sends
        // "no label" and strips it.
        assert!(index
            .mark_written(&path, Some(3), Flag::None, Some("Red"), false, None)
            .unwrap());
        assert_eq!(
            index.entries("d").unwrap()[0].label.as_deref(),
            Some("Red"),
            "the resolved label is now stored and marked known"
        );

        // A later judgment that again does not know the label yet leaves
        // that now-known label untouched, and a replay of the resulting
        // dirty row carries it as known.
        index
            .set_rating("d", &path, Some(4), Flag::None, None, false)
            .unwrap();
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [(
                path.clone(),
                Some(4),
                Flag::None,
                Some("Red".to_string()),
                true
            )],
            "the row survives a crash before the writer drains it: a replay must \
             not treat the label as unknown and strip it from the sidecar"
        );

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_sidecar_reset_keeps_the_judgment_of_a_dirty_row() {
        let dir = temp_dir("reset-label");
        let mut index = open(&dir);
        index
            .set_rating("d", "/a.ARW", Some(2), Flag::Pick, Some("Green"), true)
            .unwrap();

        index.reset_sidecars().unwrap();

        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [(
                "/a.ARW".to_string(),
                Some(2),
                Flag::Pick,
                Some("Green".to_string()),
                true
            )]
        );

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v4_database_gains_the_label_column_and_keeps_its_dirty_rows() {
        let dir = temp_dir("migrate-v4");
        let db = dir.join("index.sqlite");
        {
            let conn = Connection::open(&db).unwrap();
            conn.execute_batch(
                "CREATE TABLE ratings (
                     path TEXT PRIMARY KEY,
                     dir TEXT NOT NULL,
                     rating INTEGER,
                     pick INTEGER NOT NULL DEFAULT 0,
                     xmp_size INTEGER,
                     xmp_mtime_ns INTEGER,
                     dirty INTEGER NOT NULL DEFAULT 0
                 );
                 INSERT INTO ratings (path, dir, rating, pick, dirty)
                     VALUES ('/a.ARW', 'd', 4, 1, 1);
                 PRAGMA user_version = 4;",
            )
            .unwrap();
        }

        let index = Index::open(&db).unwrap();
        assert_eq!(
            index.dirty_rows("d").unwrap(),
            [("/a.ARW".to_string(), Some(4), Flag::Pick, None, true)]
        );
        drop(index);
        let index = Index::open(&db).unwrap();
        let version: i64 = index
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 14);
        assert_eq!(index.dirty_rows("d").unwrap().len(), 1);

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v2_database_gains_the_flag_column_and_keeps_its_dirty_rows() {
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
            [("/a.ARW".to_string(), Some(4), Flag::None, None, true)]
        );
        drop(index);
        let index = Index::open(&db).unwrap();
        assert_eq!(index.dirty_rows("d").unwrap().len(), 1, "a reopen is v11");

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

        index
            .set_rating("d", &path, Some(3), Flag::None, None, true)
            .unwrap();
        assert!(!has_sidecar(&index), "dirty row, nothing written yet");

        assert!(index
            .mark_written(&path, Some(3), Flag::None, None, true, Some((42, 7)))
            .unwrap());
        assert!(has_sidecar(&index));

        index
            .set_rating("d", &path, None, Flag::None, None, true)
            .unwrap();
        assert!(index
            .mark_written(&path, None, Flag::None, None, true, None)
            .unwrap());
        assert!(!has_sidecar(&index));

        index
            .store_sidecar_ratings(
                "d",
                &[(path.clone(), None, Flag::Reject, None, 10, 20, false)],
            )
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
        assert!(entries[0].exif.is_none());
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
            PROGRESS_INTERVAL,
            |done, total, ready| progress.lock().unwrap().push((done, total, ready)),
        );

        assert_eq!(summary.total, 8);
        assert_eq!(summary.errors, 0);
        let entries = lock(&index).entries("d").unwrap();
        assert_eq!(entries.len(), 8);
        assert!(entries.iter().all(|e| e.has_thumb && e.orientation == 6));
        let progress = progress.into_inner().unwrap();
        let (done, total, _) = progress.last().unwrap();
        assert_eq!((*done, *total), (8, 8), "the last file is reported");

        remove_temp_dir(&dir);
    }

    /// Every written path is reported exactly once, for a file count that is
    /// not a multiple of `BATCH`.
    #[test]
    fn a_scan_reports_every_written_path_exactly_once() {
        let dir = temp_dir("ready");
        let body = jpeg(64, 48);
        let files: Vec<FileStat> = (0..(BATCH * 2 + 3))
            .map(|i| file(&dir, &format!("{i:03}.ARW"), &fixture(6, &body)))
            .collect();

        let index = Mutex::new(open(&dir));
        let reported = Mutex::new(Vec::new());
        let summary = run_scan(
            &index,
            "d",
            &files,
            2,
            &AtomicBool::new(false),
            PROGRESS_INTERVAL,
            |_, _, ready| reported.lock().unwrap().extend(ready),
        );

        assert_eq!(summary.total, files.len());
        let mut reported = reported.into_inner().unwrap();
        let written: Vec<String> = {
            let mut paths: Vec<String> = lock(&index)
                .entries("d")
                .unwrap()
                .into_iter()
                .map(|e| e.path)
                .collect();
            paths.sort();
            paths
        };
        let unique: HashSet<&String> = reported.iter().collect();
        assert_eq!(unique.len(), reported.len(), "no path is reported twice");
        reported.sort();
        assert_eq!(reported, written, "every written path is reported");

        remove_temp_dir(&dir);
    }

    /// An extraction error is authoritative too: its row exists, so its path
    /// is reported like a successful one.
    #[test]
    fn a_scan_reports_the_paths_of_files_that_failed_extraction() {
        let dir = temp_dir("ready-err");
        let body = jpeg(64, 48);
        let mut files: Vec<FileStat> = (0..4)
            .map(|i| file(&dir, &format!("{i:03}.ARW"), &fixture(6, &body)))
            .collect();
        let bad = file(&dir, "bad.ARW", b"not a raw file at all");
        files.push(bad.clone());

        let index = Mutex::new(open(&dir));
        let reported = Mutex::new(Vec::new());
        let summary = run_scan(
            &index,
            "d",
            &files,
            2,
            &AtomicBool::new(false),
            PROGRESS_INTERVAL,
            |_, _, ready| reported.lock().unwrap().extend(ready),
        );

        assert_eq!(summary.errors, 1);
        let reported = reported.into_inner().unwrap();
        let bad_path = bad.path.to_string_lossy().into_owned();
        assert!(reported.contains(&bad_path), "the failed file is reported");
        let entries = lock(&index).entries("d").unwrap();
        assert!(
            entries
                .iter()
                .any(|e| e.path == bad_path && e.exif.is_none()),
            "the failed file has a row"
        );
        assert_eq!(reported.len(), entries.len());

        remove_temp_dir(&dir);
    }

    #[test]
    fn canceling_after_the_first_batch_keeps_what_was_written() {
        let dir = temp_dir("cancel");
        let body = jpeg(64, 48);
        // The cancel fires from `progress` once `done >= BATCH`, so the count
        // only has to leave room for the items already in flight on the
        // worker threads when it does; it no longer races the machine speed.
        let files: Vec<FileStat> = (0..(BATCH * 4))
            .map(|i| file(&dir, &format!("{i:03}.ARW"), &fixture(1, &body)))
            .collect();

        let index = Mutex::new(open(&dir));
        let cancel = AtomicBool::new(false);
        let reported = Mutex::new(Vec::new());
        // A full batch is flushed before `done` counts its last item, so
        // `done >= BATCH` means the first batch has been written.
        let summary = run_scan(
            &index,
            "d",
            &files,
            2,
            &cancel,
            Duration::ZERO,
            |done, _, ready| {
                reported.lock().unwrap().extend(ready);
                if done >= BATCH {
                    cancel.store(true, Ordering::Relaxed);
                }
            },
        );

        assert!(summary.total >= BATCH, "the first batch ran");
        assert!(summary.total < files.len(), "the rest did not");
        let written = lock(&index).entries("d").unwrap();
        assert_eq!(
            written.len(),
            summary.total,
            "everything scanned is persisted"
        );
        let mut reported = reported.into_inner().unwrap();
        reported.sort();
        let mut persisted: Vec<String> = written.into_iter().map(|e| e.path).collect();
        persisted.sort();
        assert_eq!(reported, persisted, "the reported paths are the rows kept");

        remove_temp_dir(&dir);
    }

    fn synthetic(dir: &Path, i: usize) -> FileStat {
        FileStat {
            path: dir.join(format!("{i:05}.ARW")),
            size: 1,
            mtime_ns: 1,
        }
    }

    #[test]
    fn the_reader_does_not_wait_on_an_open_write_transaction() {
        let dir = temp_dir("reader");
        let path = dir.join("index.sqlite");
        let mut writer = Index::open(&path).unwrap();
        let rows: Vec<_> = (0..3).map(|i| (synthetic(&dir, i), Ok(entry()))).collect();
        writer.write_batch("d", &rows).unwrap();
        let reader = Mutex::new(Index::open_reader(&path).unwrap());
        let first = rows[0].0.path.to_string_lossy().into_owned();

        let writer = Mutex::new(writer);
        // The reads are joined while the transaction is still open: a blocked
        // reader would hit the busy timeout `Index::open_reader` sets and fail
        // here, since the rollback below cannot run until the join returns.
        std::thread::scope(|scope| {
            let guard = lock(&writer);
            guard.conn.execute_batch("BEGIN IMMEDIATE").unwrap();
            let read = scope.spawn(|| {
                let reader = lock(&reader);
                let entries = reader.entries("d").unwrap();
                let thumb = reader.thumbnail(&first).unwrap();
                (entries.len(), thumb.0)
            });
            assert_eq!(read.join().unwrap(), (3, 6));
            guard.conn.execute_batch("ROLLBACK").unwrap();
        });

        lock(&writer)
            .write_batch("d", &[(synthetic(&dir, 3), Ok(entry()))])
            .unwrap();
        assert_eq!(lock(&reader).entries("d").unwrap().len(), 4);

        remove_temp_dir(&dir);
    }

    fn opened_at(index: &Index, dir: &str) -> Option<i64> {
        index
            .conn
            .query_row(
                "SELECT opened_at FROM folders WHERE dir = ?1",
                params![dir],
                |r| r.get(0),
            )
            .ok()
    }

    fn set_opened_at(index: &Index, dir: &str, at: i64) {
        index
            .conn
            .execute(
                "INSERT OR REPLACE INTO folders (dir, opened_at) VALUES (?1, ?2)",
                params![dir, at],
            )
            .unwrap();
    }

    fn big_entry() -> Entry {
        Entry {
            thumbnail: vec![0x55; 200_000],
            ..entry()
        }
    }

    const DAY: i64 = 24 * 60 * 60;
    const NOW: i64 = 100 * DAY;
    const NO_CAP: EvictPolicy = EvictPolicy {
        max_age_secs: MAX_AGE_SECS,
        max_bytes: i64::MAX,
    };

    #[test]
    fn reconcile_records_when_the_folder_was_opened() {
        let dir = temp_dir("opened-at");
        let mut index = open(&dir);
        set_opened_at(&index, "d", 0);
        let before = now_secs();
        index.reconcile("d", &[]).unwrap();
        assert!(opened_at(&index, "d").unwrap() >= before);

        remove_temp_dir(&dir);
    }

    #[test]
    fn an_old_folder_is_evicted_and_a_fresh_one_kept() {
        let dir = temp_dir("evict-age");
        let mut index = open(&dir);
        index
            .write_batch("old", &[(synthetic(&dir, 0), Ok(entry()))])
            .unwrap();
        index
            .write_batch("new", &[(synthetic(&dir, 1), Ok(entry()))])
            .unwrap();
        index
            .set_rating(
                "old",
                &synthetic(&dir, 0).path.to_string_lossy(),
                Some(3),
                Flag::None,
                None,
                true,
            )
            .unwrap();
        index
            .mark_written(
                &synthetic(&dir, 0).path.to_string_lossy(),
                Some(3),
                Flag::None,
                None,
                true,
                None,
            )
            .unwrap();
        set_opened_at(&index, "old", NOW - 31 * DAY);
        set_opened_at(&index, "new", NOW - 29 * DAY);

        let summary = index.evict(NOW, NO_CAP).unwrap();
        assert_eq!(summary.folders, 1);
        assert!(summary.vacuumed);
        assert!(index.entries("old").unwrap().is_empty());
        assert_eq!(opened_at(&index, "old"), None);
        assert_eq!(index.entries("new").unwrap().len(), 1);
        let ratings: i64 = index
            .conn
            .query_row("SELECT COUNT(*) FROM ratings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(ratings, 0);

        assert_eq!(index.evict(NOW, NO_CAP).unwrap(), EvictSummary::default());

        remove_temp_dir(&dir);
    }

    #[test]
    fn the_size_cap_evicts_the_least_recently_opened_folders_first() {
        let dir = temp_dir("evict-lru");
        let mut index = open(&dir);
        for (i, name) in ["a", "b", "c"].iter().enumerate() {
            index
                .write_batch(name, &[(synthetic(&dir, i), Ok(big_entry()))])
                .unwrap();
        }
        set_opened_at(&index, "b", NOW - 3 * DAY);
        set_opened_at(&index, "a", NOW - 2 * DAY);
        set_opened_at(&index, "c", NOW - DAY);
        let used = index.used_bytes().unwrap();
        let policy = EvictPolicy {
            max_age_secs: MAX_AGE_SECS,
            max_bytes: used - 250_000,
        };

        let summary = index.evict(NOW, policy).unwrap();
        assert_eq!((summary.folders, summary.rows), (2, 2));
        assert!(index.entries("b").unwrap().is_empty());
        assert!(index.entries("a").unwrap().is_empty());
        assert_eq!(index.entries("c").unwrap().len(), 1);

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_dirty_rating_survives_the_eviction_of_its_folder() {
        let dir = temp_dir("evict-dirty");
        let mut index = open(&dir);
        let path = synthetic(&dir, 0).path.to_string_lossy().into_owned();
        index
            .write_batch("d", &[(synthetic(&dir, 0), Ok(entry()))])
            .unwrap();
        index
            .set_rating("d", &path, Some(5), Flag::Pick, None, true)
            .unwrap();
        set_opened_at(&index, "d", 0);

        let summary = index.evict(NOW, NO_CAP).unwrap();
        assert_eq!(summary.rows, 1);
        assert!(index.entries("d").unwrap().is_empty());
        assert_eq!(index.dirty_rows("d").unwrap().len(), 1);
        assert_eq!(opened_at(&index, "d"), Some(0));

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_vacuum_after_eviction_shrinks_the_file() {
        let dir = temp_dir("evict-vacuum");
        let mut index = open(&dir);
        let rows: Vec<_> = (0..10)
            .map(|i| (synthetic(&dir, i), Ok(big_entry())))
            .collect();
        index.write_batch("d", &rows).unwrap();
        set_opened_at(&index, "d", 0);
        let pages = |index: &Index| -> i64 {
            index
                .conn
                .pragma_query_value(None, "page_count", |r| r.get(0))
                .unwrap()
        };
        let before = pages(&index);

        assert!(index.evict(NOW, NO_CAP).unwrap().vacuumed);
        assert!(
            pages(&index) * 10 < before,
            "{} -> {}",
            before,
            pages(&index)
        );

        remove_temp_dir(&dir);
    }

    #[test]
    fn clear_empties_the_index_and_gives_the_space_back() {
        let dir = temp_dir("clear-vacuum");
        let mut index = open(&dir);
        for name in ["a", "b"] {
            let rows: Vec<_> = (0..5)
                .map(|i| (synthetic(&dir, i), Ok(big_entry())))
                .collect();
            index.write_batch(name, &rows).unwrap();
            set_opened_at(&index, name, NOW);
        }
        let pages = |index: &Index| -> i64 {
            index
                .conn
                .pragma_query_value(None, "page_count", |r| r.get(0))
                .unwrap()
        };
        let on_disk = || -> u64 {
            let db = dir.join("index.sqlite");
            [db.clone(), with_suffix(&db, "-wal")]
                .iter()
                .map(|p| std::fs::metadata(p).map_or(0, |m| m.len()))
                .sum()
        };
        let before = pages(&index);
        let before_disk = on_disk();

        let summary = index.clear().unwrap();
        assert_eq!(summary.folders, 2);
        assert!(summary.vacuumed);
        assert!(index.entries("a").unwrap().is_empty());
        assert!(index.entries("b").unwrap().is_empty());
        assert_eq!(opened_at(&index, "a"), None);
        assert!(
            pages(&index) * 10 < before,
            "pages {} -> {}",
            before,
            pages(&index)
        );
        // Without the `wal_checkpoint(TRUNCATE)` the rebuilt database stays in
        // the WAL and the footprint on disk does not drop at all.
        assert!(
            on_disk() < before_disk / 10,
            "bytes {} -> {}",
            before_disk,
            on_disk()
        );

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_dirty_rating_survives_a_clear() {
        let dir = temp_dir("clear-dirty");
        let mut index = open(&dir);
        let path = synthetic(&dir, 0).path.to_string_lossy().into_owned();
        index
            .write_batch("d", &[(synthetic(&dir, 0), Ok(entry()))])
            .unwrap();
        index
            .set_rating("d", &path, Some(5), Flag::Pick, None, true)
            .unwrap();
        set_opened_at(&index, "d", NOW);

        let summary = index.clear().unwrap();
        assert_eq!(summary.rows, 1);
        assert!(index.entries("d").unwrap().is_empty());
        assert_eq!(index.dirty_rows("d").unwrap().len(), 1);
        assert_eq!(opened_at(&index, "d"), Some(NOW));

        remove_temp_dir(&dir);
    }

    #[test]
    fn clearing_an_empty_index_does_nothing() {
        let dir = temp_dir("clear-empty");
        let mut index = open(&dir);

        assert_eq!(index.clear().unwrap(), EvictSummary::default());

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v7_database_drops_its_files_rows_and_gains_folders() {
        let dir = temp_dir("migrate-v7");
        let db = dir.join("index.sqlite");
        let a = synthetic(&dir, 0);
        {
            let mut index = open(&dir);
            index.write_batch("d", &[(a.clone(), Ok(entry()))]).unwrap();
            index
                .conn
                .execute_batch(
                    "DROP TABLE folders;
                     ALTER TABLE ratings RENAME COLUMN flag TO pick;
                     PRAGMA user_version = 7;",
                )
                .unwrap();
        }

        let index = Index::open(&db).unwrap();
        let version: i64 = index
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 14);
        assert!(index.entries("d").unwrap().is_empty());
        assert_eq!(opened_at(&index, "d"), None);

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v8_database_drops_its_files_rows_and_keeps_ratings_and_folders() {
        let dir = temp_dir("migrate-v8");
        let db = dir.join("index.sqlite");
        let a = synthetic(&dir, 0);
        {
            let mut index = open(&dir);
            index.write_batch("d", &[(a.clone(), Ok(entry()))]).unwrap();
            set_opened_at(&index, "d", NOW);
            index
                .conn
                .execute_batch(
                    "INSERT INTO ratings (path, dir, rating) VALUES ('x', 'd', 3);
                     ALTER TABLE ratings RENAME COLUMN flag TO pick;
                     PRAGMA user_version = 8;",
                )
                .unwrap();
        }

        let index = Index::open(&db).unwrap();
        let version: i64 = index
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 14);
        assert!(index.entries("d").unwrap().is_empty());
        assert_eq!(opened_at(&index, "d"), Some(NOW));
        let rating: i64 = index
            .conn
            .query_row("SELECT rating FROM ratings WHERE path = 'x'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(rating, 3);

        remove_temp_dir(&dir);
    }

    #[test]
    fn a_v9_database_drops_its_files_rows_and_keeps_ratings_and_folders() {
        let dir = temp_dir("migrate-v9");
        let db = dir.join("index.sqlite");
        let a = synthetic(&dir, 0);
        {
            let mut index = open(&dir);
            index.write_batch("d", &[(a.clone(), Ok(entry()))]).unwrap();
            set_opened_at(&index, "d", NOW);
            index
                .conn
                .execute_batch(
                    "INSERT INTO ratings (path, dir, rating) VALUES ('x', 'd', 3);
                     ALTER TABLE ratings RENAME COLUMN flag TO pick;
                     PRAGMA user_version = 9;",
                )
                .unwrap();
        }

        let index = Index::open(&db).unwrap();
        let version: i64 = index
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 14);
        assert!(index.entries("d").unwrap().is_empty());
        assert_eq!(opened_at(&index, "d"), Some(NOW));
        let rating: i64 = index
            .conn
            .query_row("SELECT rating FROM ratings WHERE path = 'x'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(rating, 3);

        remove_temp_dir(&dir);
    }

    /// `VACUUM` time after evicting half of a ~100 MB index. Run with
    /// `cargo test -p riffle-app --release -- --ignored --nocapture vacuum_cost`.
    #[test]
    #[ignore]
    fn vacuum_cost_on_a_100_mb_index() {
        const N: usize = 5000;
        let dir = temp_dir("vacuum-cost");
        let mut index = open(&dir);
        let thumb = Entry {
            thumbnail: vec![0x55; 20_800],
            ..entry()
        };
        for (i, chunk) in (0..N).collect::<Vec<_>>().chunks(100).enumerate() {
            let name = format!("d{}", i % 2);
            let rows: Vec<_> = chunk
                .iter()
                .map(|&i| (synthetic(&dir, i), Ok(thumb.clone())))
                .collect();
            index.write_batch(&name, &rows).unwrap();
        }
        set_opened_at(&index, "d0", 0);
        set_opened_at(&index, "d1", NOW);
        let size = |index: &Index| index.used_bytes().unwrap() / 1_000_000;
        println!("before: {} MB in use", size(&index));
        let start = Instant::now();
        let summary = index.evict(NOW, NO_CAP).unwrap();
        println!(
            "evicted {} rows and vacuumed in {:.0} ms, {} MB left",
            summary.rows,
            start.elapsed().as_secs_f64() * 1000.0,
            size(&index)
        );
        remove_temp_dir(&dir);
    }

    /// Commit cost per chunk size. Run with
    /// `cargo test -p riffle-app --release -- --ignored --nocapture write_batch_cost`.
    #[test]
    #[ignore]
    fn write_batch_cost_per_chunk_size() {
        const N: usize = 5000;
        for chunk in [10, 50, 100] {
            let dir = temp_dir(&format!("cost-{chunk}"));
            let mut index = open(&dir);
            let rows: Vec<_> = (0..N).map(|i| (synthetic(&dir, i), Ok(entry()))).collect();
            let start = Instant::now();
            for batch in rows.chunks(chunk) {
                index.write_batch("d", batch).unwrap();
            }
            let total = start.elapsed();
            let per_tx = total.as_secs_f64() * 1000.0 / N.div_ceil(chunk) as f64;
            println!(
                "chunk {chunk}: {:.1} ms total, {per_tx:.3} ms per transaction",
                total.as_secs_f64() * 1000.0
            );
            drop(index);
            remove_temp_dir(&dir);
        }
    }
}
