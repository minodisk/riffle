//! The single writer thread that turns judgements into sidecars, in the
//! format selected by the `sidecarFormat` setting (XMP or DxO PhotoLab `.dop`).
//!
//! A keypress must never wait on disk, so `set_rating` only writes the
//! `ratings` row and hands the judgement to this thread. Entries are
//! coalesced per path with a trailing debounce — mashing `1`, `2`, `3` on one
//! file writes the sidecar once, with `3` — and each write goes to a temp file
//! that is fsynced and renamed over the sidecar, so a crash mid-write leaves
//! the previous sidecar intact rather than a truncated one.
//!
//! Nothing here is async: the thread blocks on `recv_timeout` against the
//! earliest deadline, which makes the drain on quit a plain synchronous
//! handshake.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use riffle_core::{dop, xmp, Flag};

use crate::index::{lock, Index};

/// Which sidecar Riffle reads and writes. One format at a time: the other
/// one's files are neither read nor written.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidecarFormat {
    #[default]
    Xmp,
    Dop,
}

impl SidecarFormat {
    /// The format a stored `sidecarFormat` value names; anything but `"dop"`
    /// (missing, unknown) is `Xmp`.
    pub fn from_setting(value: Option<&str>) -> Self {
        match value {
            Some("dop") => Self::Dop,
            _ => Self::Xmp,
        }
    }

    /// The value stored under `sidecarFormat` for this format.
    pub fn setting(self) -> &'static str {
        match self {
            Self::Xmp => "xmp",
            Self::Dop => "dop",
        }
    }

    pub fn sidecar_path(self, arw: &Path) -> PathBuf {
        match self {
            Self::Xmp => xmp::sidecar_path(arw),
            Self::Dop => dop::sidecar_path(arw),
        }
    }

    // Temporary shim until Step 3 of the lightroom-xmp-flags-labels plan
    // carries `Flag` through the app: a reject is still reported as `-1`.
    pub fn read_rating(self, bytes: &[u8]) -> Result<Option<i8>, String> {
        let (rating, flag) = match self {
            Self::Xmp => (xmp::read_rating(bytes)?, xmp::read_flag(bytes)?),
            Self::Dop => (dop::read_rating(bytes)?, dop::read_flag(bytes)?),
        };
        Ok(if flag == Flag::Reject {
            Some(-1)
        } else {
            rating
        })
    }

    /// Whether the sidecar marks the file as picked. XMP picks are ignored
    /// until Step 3 of the lightroom-xmp-flags-labels plan.
    pub fn read_pick(self, bytes: &[u8]) -> Result<bool, String> {
        match self {
            Self::Xmp => Ok(false),
            Self::Dop => Ok(dop::read_flag(bytes)? == Flag::Pick),
        }
    }

    /// The raw colour label the sidecar holds, `None` when it has none.
    pub fn read_label(self, bytes: &[u8]) -> Result<Option<String>, String> {
        match self {
            Self::Xmp => xmp::read_label(bytes),
            Self::Dop => dop::read_label(bytes),
        }
    }

    /// `existing` (or a fresh sidecar of `arw`) with its colour label set to
    /// `label`, or removed when `None`.
    pub fn write_label(
        self,
        arw: &Path,
        existing: Option<&[u8]>,
        label: Option<&str>,
    ) -> Result<Vec<u8>, String> {
        match self {
            Self::Xmp => xmp::write_label(existing, label),
            Self::Dop => dop::write_label(
                existing,
                label,
                &raw_name(arw),
                &dop::timestamp(SystemTime::now()),
            ),
        }
    }

    /// The sidecar bytes of `arw` carrying `rating` and `pick`, patched from
    /// `existing` or freshly minted. XMP ignores `pick`.
    pub fn write_rating(
        self,
        arw: &Path,
        existing: Option<&[u8]>,
        rating: Option<i8>,
        pick: bool,
    ) -> Result<Vec<u8>, String> {
        // Temporary shim until Step 3 of the lightroom-xmp-flags-labels plan
        // replaces the app's `(rating, pick)` with `(stars, Flag)`.
        let flag = if rating == Some(-1) {
            Flag::Reject
        } else if pick && self == Self::Dop {
            Flag::Pick
        } else {
            Flag::None
        };
        let rating = rating.filter(|r| *r != -1);
        match self {
            Self::Xmp => xmp::write_rating(existing, rating, flag),
            Self::Dop => dop::write_rating(
                existing,
                rating,
                flag,
                &raw_name(arw),
                &dop::timestamp(SystemTime::now()),
            ),
        }
    }

    /// Whether a directory entry named `name` is a sidecar of this format:
    /// `*.xmp`, or a RAW file name plus `.dop` (`*.arw.dop`, `*.dng.dop`),
    /// all case-insensitive.
    pub fn matches(self, name: &str) -> bool {
        let name = name.to_ascii_lowercase();
        match self {
            Self::Xmp => Path::new(&name).extension().is_some_and(|x| x == "xmp"),
            Self::Dop => name
                .strip_suffix(".dop")
                .is_some_and(|raw| riffle_core::scan::is_raw_file(Path::new(raw))),
        }
    }
}

fn raw_name(arw: &Path) -> String {
    arw.file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

/// How long a path's last update is held before its sidecar is written.
pub const DEBOUNCE: Duration = Duration::from_millis(300);

/// Delay before the first retry of a failed write; each further retry doubles
/// it.
const RETRY_BASE: Duration = Duration::from_secs(1);

/// Retries of a failed write before it is given up on. Five retries (1 s, 2 s,
/// 4 s, 8 s, 16 s, ~31 s in all) ride out a card reconnect or a transient
/// lock, while a genuinely read-only folder stops reporting within a minute;
/// the dirty row is then written on the next folder open.
const RETRY_LIMIT: u32 = 5;

/// The delay before retry number `attempts` (1-based) of a failed write, or
/// `None` once `RETRY_LIMIT` retries have been made.
fn retry_delay(attempts: u32) -> Option<Duration> {
    (1..=RETRY_LIMIT)
        .contains(&attempts)
        .then(|| RETRY_BASE * 2u32.pow(attempts - 1))
}

/// Suffix of the temp file each write goes to before being renamed into place.
pub(crate) const TEMP_SUFFIX: &str = ".riffle-tmp";

/// Longest the quit path waits for the writer to drain.
pub const DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

/// One file's rating, pick and colour label, bundled so the writer's queue
/// and channel do not need one argument per field.
///
/// `label_known` is false when the caller has not learned this path's label
/// yet (e.g. a judgement made before the first `folder_entries` refresh, or
/// before a sidecar parse has landed); `label` is then ignored and the
/// sidecar's own current label, read from disk, is kept instead of being
/// cleared.
#[derive(Clone)]
struct Judgement {
    rating: Option<i8>,
    pick: bool,
    label: Option<String>,
    label_known: bool,
}

enum Message {
    /// Queue a judgement, to be written once `deadline` has passed.
    Set {
        path: PathBuf,
        judgement: Judgement,
        format: SidecarFormat,
        deadline: Instant,
    },
    /// Write everything pending now and answer on the channel.
    Flush(Sender<()>),
}

/// A handle on the writer thread. Dropping it ends the thread, but the quit
/// path does not rely on that: it flushes explicitly (see `flush`), because a
/// `Drop` during process teardown is not guaranteed to run.
pub struct Writer {
    tx: Mutex<Sender<Message>>,
}

impl Writer {
    /// Start the thread. `on_error` reports every failed attempt at writing a
    /// sidecar; the write is retried with a growing delay (see `retry_delay`),
    /// and once the retries run out the row stays dirty and is written on the
    /// next folder open.
    pub fn spawn<F>(index: Arc<Mutex<Index>>, on_error: F) -> Self
    where
        F: Fn(&Path, &str) + Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || run(rx, index, on_error));
        Self { tx: Mutex::new(tx) }
    }

    /// Queue one judgement, to be written `DEBOUNCE` after the last update of
    /// that path. Fails only once the thread is gone.
    pub fn set(
        &self,
        path: PathBuf,
        rating: Option<i8>,
        pick: bool,
        label: Option<String>,
        label_known: bool,
        format: SidecarFormat,
    ) -> Result<(), String> {
        self.send(
            path,
            Judgement {
                rating,
                pick,
                label,
                label_known,
            },
            format,
            Instant::now() + DEBOUNCE,
        )
    }

    /// Queue one judgement with no debounce, for the dirty rows a folder open
    /// finds: they were queued in an earlier session and have waited long
    /// enough already.
    pub fn set_now(
        &self,
        path: PathBuf,
        rating: Option<i8>,
        pick: bool,
        label: Option<String>,
        label_known: bool,
        format: SidecarFormat,
    ) -> Result<(), String> {
        self.send(
            path,
            Judgement {
                rating,
                pick,
                label,
                label_known,
            },
            format,
            Instant::now(),
        )
    }

    fn send(
        &self,
        path: PathBuf,
        judgement: Judgement,
        format: SidecarFormat,
        deadline: Instant,
    ) -> Result<(), String> {
        lock(&self.tx)
            .send(Message::Set {
                path,
                judgement,
                format,
                deadline,
            })
            .map_err(|_| "the sidecar writer is not running".to_string())
    }

    /// Write everything pending and wait for it, at most `timeout`. Used on
    /// quit so a normal Cmd+Q loses nothing.
    pub fn flush(&self, timeout: Duration) {
        let (tx, rx) = std::sync::mpsc::channel();
        if lock(&self.tx).send(Message::Flush(tx)).is_err() {
            return;
        }
        let _ = rx.recv_timeout(timeout);
    }
}

fn run<F>(rx: Receiver<Message>, index: Arc<Mutex<Index>>, on_error: F)
where
    F: Fn(&Path, &str),
{
    let mut pending: Pending = std::collections::HashMap::new();
    loop {
        let wait = pending
            .values()
            .map(|entry| entry.deadline.saturating_duration_since(Instant::now()))
            .min();
        let message = match wait {
            Some(wait) => rx.recv_timeout(wait),
            None => rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match message {
            Ok(Message::Set {
                path,
                judgement,
                format,
                deadline,
            }) => {
                pending.insert(
                    path,
                    Entry {
                        judgement,
                        format,
                        deadline,
                        attempts: 0,
                    },
                );
            }
            Ok(Message::Flush(reply)) => {
                flush(&mut pending, None, &index, &on_error);
                let _ = reply.send(());
            }
            Err(RecvTimeoutError::Timeout) => {
                flush(&mut pending, Some(Instant::now()), &index, &on_error);
            }
            Err(RecvTimeoutError::Disconnected) => {
                flush(&mut pending, None, &index, &on_error);
                return;
            }
        }
    }
}

/// A queued judgement, with the format selected when it was made, the instant
/// its sidecar is due and how many retries of it have already failed.
struct Entry {
    judgement: Judgement,
    format: SidecarFormat,
    deadline: Instant,
    attempts: u32,
}

/// The entry queued per path.
type Pending = std::collections::HashMap<PathBuf, Entry>;

/// Write the entries whose deadline has passed by `now`, or all of them when
/// `now` is `None`. A failed write is requeued for a retry, except in a drain
/// (`now` is `None`), which makes one attempt so quitting never waits out a
/// backoff.
fn flush<F>(pending: &mut Pending, now: Option<Instant>, index: &Arc<Mutex<Index>>, on_error: &F)
where
    F: Fn(&Path, &str),
{
    let due: Vec<PathBuf> = pending
        .iter()
        .filter(|(_, entry)| now.is_none_or(|now| now >= entry.deadline))
        .map(|(path, _)| path.clone())
        .collect();
    for path in due {
        let entry = pending.remove(&path).expect("just listed");
        let judgement = &entry.judgement;
        match write(&path, judgement, entry.format) {
            Ok((stat, resolved_label)) => {
                if let Err(e) = lock(index).mark_written(
                    &path.to_string_lossy(),
                    judgement.rating,
                    judgement.pick,
                    resolved_label.as_deref(),
                    judgement.label_known,
                    stat,
                ) {
                    on_error(&path, &e);
                }
            }
            Err(e) => {
                let attempts = entry.attempts + 1;
                match retry_delay(attempts).filter(|_| now.is_some()) {
                    Some(delay) => {
                        on_error(&path, &format!("{e} (retrying in {}s)", delay.as_secs()));
                        pending.insert(
                            path,
                            Entry {
                                deadline: Instant::now() + delay,
                                attempts,
                                ..entry
                            },
                        );
                    }
                    None => on_error(&path, &e),
                }
            }
        }
    }
}

/// The sidecar of `arw` as it exists on disk, preferring a name that differs
/// only in case (`FOO.XMP`) over the one `sidecar_path` would mint.
pub(crate) fn existing_sidecar(arw: &Path, format: SidecarFormat) -> Option<PathBuf> {
    let wanted = format.sidecar_path(arw);
    if wanted.exists() {
        return Some(wanted);
    }
    let name = wanted.file_name()?.to_string_lossy().into_owned();
    let dir = wanted.parent()?;
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(&name))
        })
}

/// What `write` produces: the sidecar's `(size, mtime_ns)` (or `None` when
/// there is nothing to write: clearing a rating on a file that has no
/// sidecar must not litter the folder with an empty one), alongside the
/// label actually written.
type WriteResult = Result<(Option<(i64, i64)>, Option<String>), String>;

/// Write one file's sidecar.
///
/// `judgement.label_known` is false when `judgement.label` was never learned
/// by the caller (see `Writer::set`); the label is then ignored and the
/// sidecar's own current label, read from disk, is kept and returned instead,
/// so an unknown label can never clear one PhotoLab or Lightroom already
/// wrote.
fn write(arw: &Path, judgement: &Judgement, format: SidecarFormat) -> WriteResult {
    let Judgement {
        rating,
        pick,
        label,
        label_known,
    } = judgement;
    let (rating, label_known) = (*rating, *label_known);
    let pick = *pick && format == SidecarFormat::Dop;
    let existing = existing_sidecar(arw, format);
    let current = existing
        .as_ref()
        .map(|target| std::fs::read(target).map_err(|e| format!("{}: {e}", target.display())))
        .transpose()?;
    let resolved_label = if label_known {
        label.clone()
    } else {
        current
            .as_deref()
            .map(|bytes| format.read_label(bytes))
            .transpose()?
            .flatten()
    };
    let label = resolved_label.as_deref();
    let (target, bytes) = match &existing {
        None => {
            if rating.is_none() && !pick && label.is_none() {
                return Ok((None, resolved_label));
            }
            let bytes = if rating.is_none() && !pick {
                format.write_label(arw, None, label)?
            } else {
                let rated = format.write_rating(arw, None, rating, pick)?;
                format.write_label(arw, Some(&rated), label)?
            };
            (format.sidecar_path(arw), bytes)
        }
        Some(target) => {
            let current = current.as_deref().expect("existing sidecar was read above");
            let rated = format.write_rating(arw, Some(current), rating, pick)?;
            // Clearing a label that is not there would still bump the `.dop`
            // timestamps, so it is skipped.
            let bytes = if label.is_none() && format.read_label(&rated)?.is_none() {
                rated
            } else {
                format.write_label(arw, Some(&rated), label)?
            };
            (target.clone(), bytes)
        }
    };

    let mut temp = target.clone().into_os_string();
    temp.push(TEMP_SUFFIX);
    let temp = PathBuf::from(temp);
    let write_temp = || -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(&bytes)?;
        file.sync_all()
    };
    if let Err(e) = write_temp() {
        let _ = std::fs::remove_file(&temp);
        return Err(format!("{}: {e}", temp.display()));
    }
    if let Err(e) = std::fs::rename(&temp, &target) {
        let _ = std::fs::remove_file(&temp);
        return Err(format!("{}: {e}", target.display()));
    }

    let meta = std::fs::metadata(&target).map_err(|e| format!("{}: {e}", target.display()))?;
    let mtime_ns = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos() as i64);
    Ok((Some((meta.len() as i64, mtime_ns)), resolved_label))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("riffle-sidecar-{name}-{}", std::process::id()));
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

    fn index(dir: &Path) -> Arc<Mutex<Index>> {
        Arc::new(Mutex::new(Index::open(&dir.join("index.sqlite")).unwrap()))
    }

    fn writer(index: Arc<Mutex<Index>>) -> Writer {
        Writer::spawn(index, |path, message| {
            eprintln!("{}: {message}", path.display())
        })
    }

    /// Wait for `check`, so the tests do not depend on how long the writer
    /// thread takes to be scheduled. The budget is far longer than the work
    /// needs because it only costs time when the assertion is failing anyway,
    /// and a loaded CI runner is much slower than a developer's machine.
    fn eventually(check: impl Fn() -> bool) -> bool {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if check() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        check()
    }

    fn arw(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, b"not really an ARW").unwrap();
        path
    }

    /// A Lightroom-shaped sidecar: the `Rating` as an element, plus a `crs:`
    /// block that must survive the patch byte for byte.
    fn lightroom_sidecar(rating: i32) -> String {
        format!(
            r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="Adobe XMP Core">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:xmp="http://ns.adobe.com/xap/1.0/"
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
   crs:Exposure2012="+0.35"
   crs:Contrast2012="+12">
   <xmp:Rating>{rating}</xmp:Rating>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#
        )
    }

    #[test]
    fn a_burst_within_the_debounce_window_writes_one_sidecar_with_the_last_value() {
        let dir = temp_dir("burst");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "a.ARW");

        for rating in [1i8, 2, 3] {
            lock(&index)
                .set_rating(
                    "d",
                    &path.to_string_lossy(),
                    Some(rating),
                    false,
                    None,
                    true,
                )
                .unwrap();
            writer
                .set(
                    path.clone(),
                    Some(rating),
                    false,
                    None,
                    true,
                    SidecarFormat::Xmp,
                )
                .unwrap();
        }

        let sidecar = xmp::sidecar_path(&path);
        assert!(eventually(|| sidecar.exists()));
        let bytes = std::fs::read(&sidecar).unwrap();
        assert_eq!(xmp::read_rating(&bytes).unwrap(), Some(3));
        assert!(
            eventually(|| lock(&index).dirty_rows("d").unwrap().is_empty()),
            "the write clears dirty"
        );
        // Nothing is left behind by a successful write.
        let names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            !names.iter().any(|n| n.ends_with(TEMP_SUFFIX)),
            "no temp file is left: {names:?}"
        );

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn an_existing_sidecar_is_patched_rather_than_replaced() {
        let dir = temp_dir("patch");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "b.ARW");
        let sidecar = xmp::sidecar_path(&path);
        std::fs::write(&sidecar, lightroom_sidecar(2)).unwrap();

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), Some(5), false, None, true)
            .unwrap();
        writer
            .set(path.clone(), Some(5), false, None, true, SidecarFormat::Xmp)
            .unwrap();

        assert!(eventually(
            || std::fs::read_to_string(&sidecar).unwrap() == lightroom_sidecar(5)
        ));

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn a_flush_writes_before_the_debounce_elapses() {
        let dir = temp_dir("flush");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "c.ARW");

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), Some(-1), false, None, true)
            .unwrap();
        // A deadline far enough out that a slow machine cannot blur the
        // difference between flushing now and waiting for it. Measuring
        // against DEBOUNCE itself made this fail on CI, where the write alone
        // can take longer than 300ms.
        let deadline = Instant::now() + Duration::from_secs(30);
        writer
            .send(
                path.clone(),
                Judgement {
                    rating: Some(-1),
                    pick: false,
                    label: None,
                    label_known: true,
                },
                SidecarFormat::Xmp,
                deadline,
            )
            .unwrap();
        let started = Instant::now();
        writer.flush(DRAIN_TIMEOUT);

        assert!(
            started.elapsed() < Duration::from_secs(15),
            "the flush did not wait it out"
        );
        let bytes = std::fs::read(xmp::sidecar_path(&path)).unwrap();
        assert_eq!(SidecarFormat::Xmp.read_rating(&bytes).unwrap(), Some(-1));

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn clearing_a_rating_writes_no_sidecar_when_there_is_none() {
        let dir = temp_dir("clear");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "d.ARW");

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), None, false, None, true)
            .unwrap();
        writer
            .set(path.clone(), None, false, None, true, SidecarFormat::Xmp)
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);

        assert!(!xmp::sidecar_path(&path).exists());
        assert!(lock(&index).dirty_rows("d").unwrap().is_empty());

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    #[cfg(unix)]
    fn a_failed_write_leaves_the_row_dirty() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_dir("readonly");
        let dir = root.join("locked");
        std::fs::create_dir_all(&dir).unwrap();
        let path = arw(&dir, "e.ARW");
        let index = index(&root);
        let writer = writer(index.clone());
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o500)).unwrap();

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), Some(4), false, None, true)
            .unwrap();
        writer
            .set(path.clone(), Some(4), false, None, true, SidecarFormat::Xmp)
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);

        let dirty = lock(&index).dirty_rows("d").unwrap();
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0].1, Some(4));

        drop(writer);
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        remove_temp_dir(&root);
    }

    #[test]
    fn the_retry_delay_grows_and_stops_at_the_limit() {
        let delays: Vec<_> = (1..=RETRY_LIMIT).map(|n| retry_delay(n).unwrap()).collect();
        assert_eq!(delays[0], RETRY_BASE);
        assert!(delays.windows(2).all(|w| w[1] > w[0]));
        assert_eq!(retry_delay(RETRY_LIMIT + 1), None);
    }

    #[cfg(unix)]
    fn counting_writer(index: Arc<Mutex<Index>>) -> (Writer, Arc<AtomicUsize>) {
        let errors = Arc::new(AtomicUsize::new(0));
        let count = errors.clone();
        let writer = Writer::spawn(index, move |path, message| {
            eprintln!("{}: {message}", path.display());
            count.fetch_add(1, Ordering::SeqCst);
        });
        (writer, errors)
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_write_is_retried_until_it_succeeds() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_dir("retry");
        let dir = root.join("locked");
        std::fs::create_dir_all(&dir).unwrap();
        let path = arw(&dir, "r.ARW");
        let index = index(&root);
        let (writer, errors) = counting_writer(index.clone());
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o500)).unwrap();

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), Some(3), false, None, true)
            .unwrap();
        writer
            .set(path.clone(), Some(3), false, None, true, SidecarFormat::Xmp)
            .unwrap();
        assert!(eventually(|| errors.load(Ordering::SeqCst) >= 1));
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();

        let sidecar = xmp::sidecar_path(&path);
        assert!(eventually(|| std::fs::read(&sidecar)
            .is_ok_and(|b| xmp::read_rating(&b) == Ok(Some(3)))
            && lock(&index).dirty_rows("d").unwrap().is_empty()));

        drop(writer);
        remove_temp_dir(&root);
    }

    #[cfg(unix)]
    #[test]
    fn a_newer_judgement_replaces_one_waiting_for_a_retry() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_dir("retry-latest");
        let dir = root.join("locked");
        std::fs::create_dir_all(&dir).unwrap();
        let path = arw(&dir, "l.ARW");
        let index = index(&root);
        let (writer, errors) = counting_writer(index.clone());
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o500)).unwrap();

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), Some(2), false, None, true)
            .unwrap();
        writer
            .set(path.clone(), Some(2), false, None, true, SidecarFormat::Xmp)
            .unwrap();
        assert!(eventually(|| errors.load(Ordering::SeqCst) >= 1));
        lock(&index)
            .set_rating("d", &path.to_string_lossy(), Some(5), false, None, true)
            .unwrap();
        writer
            .set(path.clone(), Some(5), false, None, true, SidecarFormat::Xmp)
            .unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();

        let sidecar = xmp::sidecar_path(&path);
        assert!(eventually(|| std::fs::read(&sidecar)
            .is_ok_and(|b| xmp::read_rating(&b) == Ok(Some(5)))
            && lock(&index).dirty_rows("d").unwrap().is_empty()));

        drop(writer);
        remove_temp_dir(&root);
    }

    const PHOTOLAB_0003: &[u8] = include_bytes!("../../core/src/fixtures/dop/_DSC0003.ARW.dop");

    #[test]
    fn the_format_setting_defaults_to_xmp_and_matches_names_case_insensitively() {
        assert_eq!(SidecarFormat::from_setting(None), SidecarFormat::Xmp);
        assert_eq!(
            SidecarFormat::from_setting(Some("tiff")),
            SidecarFormat::Xmp
        );
        assert_eq!(SidecarFormat::from_setting(Some("dop")), SidecarFormat::Dop);
        assert!(SidecarFormat::Xmp.matches("A.XMP"));
        assert!(!SidecarFormat::Xmp.matches("a.ARW.dop"));
        assert!(SidecarFormat::Dop.matches("A.ARW.DOP"));
        assert!(SidecarFormat::Dop.matches("L1000001.DNG.dop"));
        assert!(!SidecarFormat::Dop.matches("a.jpg.dop"));
        assert_eq!(
            SidecarFormat::from_setting(Some(SidecarFormat::Dop.setting())),
            SidecarFormat::Dop
        );
        assert_eq!(
            SidecarFormat::from_setting(Some(SidecarFormat::Xmp.setting())),
            SidecarFormat::Xmp
        );
        assert!(!SidecarFormat::Dop.matches("a.dop"));
        assert!(!SidecarFormat::Dop.matches("a.xmp"));
    }

    #[test]
    fn a_photolab_sidecar_is_patched_in_place() {
        let dir = temp_dir("dop-patch");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "_DSC0003.ARW");
        let sidecar = dop::sidecar_path(&path);
        std::fs::write(&sidecar, PHOTOLAB_0003).unwrap();

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), Some(-1), false, None, true)
            .unwrap();
        writer
            .set(
                path.clone(),
                Some(-1),
                false,
                None,
                true,
                SidecarFormat::Dop,
            )
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);

        let bytes = std::fs::read(&sidecar).unwrap();
        assert_eq!(SidecarFormat::Dop.read_rating(&bytes).unwrap(), Some(-1));
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.ends_with("}\n\r\n"), "the trailing CRLF survives");
        assert!(!xmp::sidecar_path(&path).exists(), "no XMP is written");

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn a_file_with_no_dop_gets_the_minimal_template() {
        let dir = temp_dir("dop-fresh");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "f.ARW");

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), Some(2), false, None, true)
            .unwrap();
        writer
            .set(path.clone(), Some(2), false, None, true, SidecarFormat::Dop)
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);

        let bytes = std::fs::read(dop::sidecar_path(&path)).unwrap();
        assert_eq!(dop::read_rating(&bytes).unwrap(), Some(2));
        assert!(String::from_utf8(bytes)
            .unwrap()
            .contains("Name = \"f.ARW\""));
        assert!(!xmp::sidecar_path(&path).exists());

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn clearing_a_rating_writes_no_dop_when_there_is_none() {
        let dir = temp_dir("dop-clear");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "g.ARW");

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), None, false, None, true)
            .unwrap();
        writer
            .set(path.clone(), None, false, None, true, SidecarFormat::Dop)
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);

        assert!(!dop::sidecar_path(&path).exists());
        assert!(lock(&index).dirty_rows("d").unwrap().is_empty());

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn a_pick_reaches_the_dop_and_survives_a_later_rating() {
        let dir = temp_dir("dop-pick");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "p.ARW");
        let key = path.to_string_lossy().into_owned();

        lock(&index)
            .set_rating("d", &key, None, true, None, true)
            .unwrap();
        writer
            .set(path.clone(), None, true, None, true, SidecarFormat::Dop)
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);
        let bytes = std::fs::read(dop::sidecar_path(&path)).unwrap();
        assert!(
            SidecarFormat::Dop.read_pick(&bytes).unwrap(),
            "a pick alone mints a .dop"
        );

        lock(&index)
            .set_rating("d", &key, Some(4), true, None, true)
            .unwrap();
        writer
            .set(path.clone(), Some(4), true, None, true, SidecarFormat::Dop)
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);
        let bytes = std::fs::read(dop::sidecar_path(&path)).unwrap();
        assert!(SidecarFormat::Dop.read_pick(&bytes).unwrap());
        assert_eq!(dop::read_rating(&bytes).unwrap(), Some(4));
        assert!(lock(&index).dirty_rows("d").unwrap().is_empty());

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn xmp_ignores_a_pick() {
        let dir = temp_dir("xmp-pick");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "q.ARW");

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), None, true, None, true)
            .unwrap();
        writer
            .set(path.clone(), None, true, None, true, SidecarFormat::Xmp)
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);

        assert!(
            !xmp::sidecar_path(&path).exists(),
            "a pick alone writes no XMP"
        );
        assert!(!dop::sidecar_path(&path).exists());
        assert!(!SidecarFormat::Xmp.read_pick(b"anything").unwrap());

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn a_dop_differing_only_in_case_is_the_one_patched() {
        let dir = temp_dir("dop-case");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "h.ARW");
        let sidecar = dir.join("H.ARW.DOP");
        std::fs::write(&sidecar, PHOTOLAB_0003).unwrap();

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), Some(5), false, None, true)
            .unwrap();
        writer
            .set(path.clone(), Some(5), false, None, true, SidecarFormat::Dop)
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);

        // On a case-insensitive file system the rename may take either
        // spelling; what matters is that there is still one sidecar.
        let dops: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| SidecarFormat::Dop.matches(&p.file_name().unwrap().to_string_lossy()))
            .collect();
        assert_eq!(dops.len(), 1, "no second sidecar is minted: {dops:?}");
        let bytes = std::fs::read(&dops[0]).unwrap();
        assert_eq!(dop::read_rating(&bytes).unwrap(), Some(5));

        drop(writer);
        remove_temp_dir(&dir);
    }
    /// Record and queue one full judgement, then wait for the write.
    fn judge(
        index: &Arc<Mutex<Index>>,
        writer: &Writer,
        path: &Path,
        rating: Option<i8>,
        pick: bool,
        label: Option<&str>,
        format: SidecarFormat,
    ) {
        lock(index)
            .set_rating("d", &path.to_string_lossy(), rating, pick, label, true)
            .unwrap();
        writer
            .set(
                path.to_path_buf(),
                rating,
                pick,
                label.map(str::to_string),
                true,
                format,
            )
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);
    }

    #[test]
    fn a_label_alone_mints_an_xmp_and_a_dop() {
        let dir = temp_dir("label-mint");
        let index = index(&dir);
        let writer = writer(index.clone());

        let a = arw(&dir, "a.ARW");
        judge(
            &index,
            &writer,
            &a,
            None,
            false,
            Some("Green"),
            SidecarFormat::Xmp,
        );
        let bytes = std::fs::read(xmp::sidecar_path(&a)).unwrap();
        assert_eq!(xmp::read_label(&bytes).unwrap().as_deref(), Some("Green"));

        let b = arw(&dir, "b.ARW");
        judge(
            &index,
            &writer,
            &b,
            None,
            false,
            Some("Orange"),
            SidecarFormat::Dop,
        );
        let bytes = std::fs::read(dop::sidecar_path(&b)).unwrap();
        assert_eq!(dop::read_label(&bytes).unwrap().as_deref(), Some("Orange"));
        assert!(lock(&index).dirty_rows("d").unwrap().is_empty());

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn a_label_patches_a_lightroom_sidecar_and_survives_a_later_rating() {
        let dir = temp_dir("label-lightroom");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "a.ARW");
        let sidecar = xmp::sidecar_path(&path);
        std::fs::write(&sidecar, lightroom_sidecar(2)).unwrap();

        judge(
            &index,
            &writer,
            &path,
            Some(2),
            false,
            Some("Red"),
            SidecarFormat::Xmp,
        );
        let text = std::fs::read_to_string(&sidecar).unwrap();
        assert_eq!(
            text.replace(r#" xmp:Label="Red""#, ""),
            lightroom_sidecar(2),
            "only the label is added"
        );

        judge(
            &index,
            &writer,
            &path,
            Some(4),
            false,
            Some("Red"),
            SidecarFormat::Xmp,
        );
        let bytes = std::fs::read(&sidecar).unwrap();
        assert_eq!(xmp::read_rating(&bytes).unwrap(), Some(4));
        assert_eq!(xmp::read_label(&bytes).unwrap().as_deref(), Some("Red"));

        judge(
            &index,
            &writer,
            &path,
            Some(4),
            false,
            None,
            SidecarFormat::Xmp,
        );
        let text = std::fs::read_to_string(&sidecar).unwrap();
        assert_eq!(text, lightroom_sidecar(4), "clearing removes the property");

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn clearing_a_label_on_a_dop_keeps_the_rating() {
        let dir = temp_dir("label-dop-clear");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "_DSC0003.ARW");
        let sidecar = dop::sidecar_path(&path);
        std::fs::write(&sidecar, PHOTOLAB_0003).unwrap();

        judge(
            &index,
            &writer,
            &path,
            Some(3),
            false,
            Some("Blue"),
            SidecarFormat::Dop,
        );
        let bytes = std::fs::read(&sidecar).unwrap();
        assert_eq!(dop::read_label(&bytes).unwrap().as_deref(), Some("Blue"));

        judge(
            &index,
            &writer,
            &path,
            Some(3),
            false,
            None,
            SidecarFormat::Dop,
        );
        let bytes = std::fs::read(&sidecar).unwrap();
        assert_eq!(dop::read_label(&bytes).unwrap(), None);
        assert_eq!(dop::read_rating(&bytes).unwrap(), Some(3));

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn a_judgement_before_the_label_is_known_keeps_the_sidecars_existing_label() {
        // Reproduces "judge before the row or a sidecar parse is known":
        // the writer must not treat an unknown label as "no label" and strip
        // the one PhotoLab already wrote.
        let dir = temp_dir("label-unknown-keeps");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "_DSC0003.ARW");
        let sidecar = dop::sidecar_path(&path);
        std::fs::write(&sidecar, PHOTOLAB_0003).unwrap();

        // PhotoLab already labelled the file, as if a prior session (or a
        // parse this session has not caught up with yet) put it there.
        judge(
            &index,
            &writer,
            &path,
            Some(3),
            false,
            Some("Red"),
            SidecarFormat::Dop,
        );
        let bytes = std::fs::read(&sidecar).unwrap();
        assert_eq!(dop::read_label(&bytes).unwrap().as_deref(), Some("Red"));

        // A judgement lands with the label unknown, mirroring `set_rating`
        // called before `folder_entries` (or a sidecar parse) has learned it.
        // There is no row for this path in a fresh index, but the guard must
        // hold even when one already exists.
        lock(&index)
            .set_rating("d", &path.to_string_lossy(), Some(4), false, None, false)
            .unwrap();
        writer
            .set(
                path.clone(),
                Some(4),
                false,
                None,
                false,
                SidecarFormat::Dop,
            )
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);

        let bytes = std::fs::read(&sidecar).unwrap();
        assert_eq!(
            dop::read_label(&bytes).unwrap().as_deref(),
            Some("Red"),
            "the existing label survives a judgement with an unknown label"
        );
        assert_eq!(dop::read_rating(&bytes).unwrap(), Some(4));
        assert!(lock(&index).dirty_rows("d").unwrap().is_empty());

        // The resolved label the write actually kept ("Red") is now stored
        // and marked known in `ratings`, not left `NULL`: a following-up
        // judgement that keeps the same label, this time with `label_known:
        // true`, must still keep it, and a folder-open replay of a dirty row
        // (see `dirty_rows`) must never again send "no label" for this path.
        let stored_label: Option<String> = lock(&index)
            .conn
            .query_row(
                "SELECT label FROM ratings WHERE path = ?1",
                [path.to_string_lossy()],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            stored_label.as_deref(),
            Some("Red"),
            "mark_written must persist the resolved label, not leave it NULL"
        );
        lock(&index)
            .set_rating(
                "d",
                &path.to_string_lossy(),
                Some(5),
                false,
                Some("Red"),
                true,
            )
            .unwrap();
        writer
            .set(
                path.clone(),
                Some(5),
                false,
                Some("Red".to_string()),
                true,
                SidecarFormat::Dop,
            )
            .unwrap();
        writer.flush(DRAIN_TIMEOUT);
        let bytes = std::fs::read(&sidecar).unwrap();
        assert_eq!(
            dop::read_label(&bytes).unwrap().as_deref(),
            Some("Red"),
            "a follow-up known judgement using the now-stored label keeps it"
        );
        assert!(lock(&index).dirty_rows("d").unwrap().is_empty());

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn clearing_a_label_writes_no_sidecar_when_there_is_none() {
        let dir = temp_dir("label-none");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "a.ARW");

        judge(
            &index,
            &writer,
            &path,
            None,
            false,
            None,
            SidecarFormat::Xmp,
        );
        judge(
            &index,
            &writer,
            &path,
            None,
            false,
            None,
            SidecarFormat::Dop,
        );

        assert!(!xmp::sidecar_path(&path).exists());
        assert!(!dop::sidecar_path(&path).exists());
        assert!(lock(&index).dirty_rows("d").unwrap().is_empty());

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn clearing_everything_strips_stars_and_label_from_an_xmp() {
        let dir = temp_dir("clear-all-xmp");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "a.ARW");
        let sidecar = xmp::sidecar_path(&path);
        std::fs::write(&sidecar, lightroom_sidecar(2)).unwrap();
        judge(
            &index,
            &writer,
            &path,
            Some(4),
            false,
            Some("Red"),
            SidecarFormat::Xmp,
        );

        judge(
            &index,
            &writer,
            &path,
            None,
            false,
            None,
            SidecarFormat::Xmp,
        );

        let bytes = std::fs::read(&sidecar).unwrap();
        assert_eq!(
            xmp::read_rating(&bytes).unwrap(),
            Some(0),
            "cleared stars are Rating 0"
        );
        assert!(!SidecarFormat::Xmp.read_pick(&bytes).unwrap());
        assert_eq!(xmp::read_label(&bytes).unwrap(), None);
        assert_eq!(
            std::fs::read_to_string(&sidecar).unwrap(),
            lightroom_sidecar(0)
        );
        assert!(lock(&index).dirty_rows("d").unwrap().is_empty());

        drop(writer);
        remove_temp_dir(&dir);
    }

    #[test]
    fn clearing_everything_strips_every_judgement_from_a_dop() {
        let dir = temp_dir("clear-all-dop");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "_DSC0003.ARW");
        let sidecar = dop::sidecar_path(&path);

        for (rating, pick, label) in [
            (Some(4), false, "Red"),
            (None, true, "Blue"),
            (Some(-1), false, "Green"),
        ] {
            std::fs::write(&sidecar, PHOTOLAB_0003).unwrap();
            judge(
                &index,
                &writer,
                &path,
                rating,
                pick,
                Some(label),
                SidecarFormat::Dop,
            );
            let bytes = std::fs::read(&sidecar).unwrap();
            assert_eq!(
                SidecarFormat::Dop.read_rating(&bytes).unwrap(),
                rating.or(Some(0))
            );
            assert_eq!(SidecarFormat::Dop.read_pick(&bytes).unwrap(), pick);
            assert_eq!(dop::read_label(&bytes).unwrap().as_deref(), Some(label));

            judge(
                &index,
                &writer,
                &path,
                None,
                false,
                None,
                SidecarFormat::Dop,
            );

            let bytes = std::fs::read(&sidecar).unwrap();
            assert_eq!(
                dop::read_rating(&bytes).unwrap(),
                Some(0),
                "{label}: unrated (Rating = 0) and not rejected"
            );
            assert!(
                !SidecarFormat::Dop.read_pick(&bytes).unwrap(),
                "{label}: not picked"
            );
            assert_eq!(dop::read_label(&bytes).unwrap(), None, "{label}: no label");
            assert!(lock(&index).dirty_rows("d").unwrap().is_empty());
        }

        drop(writer);
        remove_temp_dir(&dir);
    }
}
