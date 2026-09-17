//! The single writer thread that turns judgements into XMP sidecars.
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
use std::time::{Duration, Instant, UNIX_EPOCH};

use riffle_core::xmp;

use crate::index::{lock, Index};

/// How long a path's last update is held before its sidecar is written.
pub const DEBOUNCE: Duration = Duration::from_millis(300);

/// Suffix of the temp file each write goes to before being renamed into place.
const TEMP_SUFFIX: &str = ".riffle-tmp";

/// Longest the quit path waits for the writer to drain.
pub const DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

enum Message {
    /// Queue a judgement, to be written `DEBOUNCE` after the last update of
    /// that path.
    Set { path: PathBuf, rating: Option<i8> },
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
    /// Start the thread. `on_error` reports a sidecar that could not be
    /// written; the row stays dirty and is retried on the next folder open.
    pub fn spawn<F>(index: Arc<Mutex<Index>>, on_error: F) -> Self
    where
        F: Fn(&Path, &str) + Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || run(rx, index, on_error));
        Self { tx: Mutex::new(tx) }
    }

    /// Queue one judgement. Fails only once the thread is gone.
    pub fn set(&self, path: PathBuf, rating: Option<i8>) -> Result<(), String> {
        lock(&self.tx)
            .send(Message::Set { path, rating })
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
    let mut pending: std::collections::HashMap<PathBuf, (Option<i8>, Instant)> =
        std::collections::HashMap::new();
    loop {
        let wait = pending
            .values()
            .map(|(_, at)| (*at + DEBOUNCE).saturating_duration_since(Instant::now()))
            .min();
        let message = match wait {
            Some(wait) => rx.recv_timeout(wait),
            None => rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match message {
            Ok(Message::Set { path, rating }) => {
                pending.insert(path, (rating, Instant::now()));
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

/// Write the entries whose debounce has elapsed by `now`, or all of them when
/// `now` is `None`.
fn flush<F>(
    pending: &mut std::collections::HashMap<PathBuf, (Option<i8>, Instant)>,
    now: Option<Instant>,
    index: &Arc<Mutex<Index>>,
    on_error: &F,
) where
    F: Fn(&Path, &str),
{
    let due: Vec<PathBuf> = pending
        .iter()
        .filter(|(_, (_, at))| now.is_none_or(|now| now.duration_since(*at) >= DEBOUNCE))
        .map(|(path, _)| path.clone())
        .collect();
    for path in due {
        let (rating, _) = pending.remove(&path).expect("just listed");
        match write(&path, rating) {
            Ok(stat) => {
                if let Err(e) = lock(index).mark_written(&path.to_string_lossy(), rating, stat) {
                    on_error(&path, &e);
                }
            }
            Err(e) => on_error(&path, &e),
        }
    }
}

/// The sidecar of `arw` as it exists on disk, preferring a name that differs
/// only in case (`FOO.XMP`) over the one `sidecar_path` would mint.
fn existing_sidecar(arw: &Path) -> Option<PathBuf> {
    let wanted = xmp::sidecar_path(arw);
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

/// Write one file's sidecar, returning its `(size, mtime_ns)`, or `None` when
/// there is nothing to write: clearing a rating on a file that has no sidecar
/// must not litter the folder with an empty one.
fn write(arw: &Path, rating: Option<i8>) -> Result<Option<(i64, i64)>, String> {
    let existing = existing_sidecar(arw);
    let (target, bytes) = match existing {
        None => {
            if rating.is_none() {
                return Ok(None);
            }
            (xmp::sidecar_path(arw), xmp::write_rating(None, rating)?)
        }
        Some(target) => {
            let current =
                std::fs::read(&target).map_err(|e| format!("{}: {e}", target.display()))?;
            let bytes = xmp::write_rating(Some(&current), rating)?;
            (target, bytes)
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
    Ok(Some((meta.len() as i64, mtime_ns)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("riffle-sidecar-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn index(dir: &Path) -> Arc<Mutex<Index>> {
        Arc::new(Mutex::new(Index::open(&dir.join("index.sqlite")).unwrap()))
    }

    fn writer(index: Arc<Mutex<Index>>) -> Writer {
        Writer::spawn(index, |path, message| {
            eprintln!("{}: {message}", path.display())
        })
    }

    /// Wait for `check` for up to a second, so the tests do not depend on how
    /// long the writer thread takes to be scheduled.
    fn eventually(check: impl Fn() -> bool) -> bool {
        let deadline = Instant::now() + Duration::from_secs(1);
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
                .set_rating("d", &path.to_string_lossy(), Some(rating))
                .unwrap();
            writer.set(path.clone(), Some(rating)).unwrap();
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
        std::fs::remove_dir_all(&dir).unwrap();
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
            .set_rating("d", &path.to_string_lossy(), Some(5))
            .unwrap();
        writer.set(path.clone(), Some(5)).unwrap();

        assert!(eventually(
            || std::fs::read_to_string(&sidecar).unwrap() == lightroom_sidecar(5)
        ));

        drop(writer);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_flush_writes_before_the_debounce_elapses() {
        let dir = temp_dir("flush");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "c.ARW");

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), Some(-1))
            .unwrap();
        writer.set(path.clone(), Some(-1)).unwrap();
        let started = Instant::now();
        writer.flush(DRAIN_TIMEOUT);

        assert!(
            started.elapsed() < DEBOUNCE,
            "the flush did not wait it out"
        );
        let bytes = std::fs::read(xmp::sidecar_path(&path)).unwrap();
        assert_eq!(xmp::read_rating(&bytes).unwrap(), Some(-1));

        drop(writer);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn clearing_a_rating_writes_no_sidecar_when_there_is_none() {
        let dir = temp_dir("clear");
        let index = index(&dir);
        let writer = writer(index.clone());
        let path = arw(&dir, "d.ARW");

        lock(&index)
            .set_rating("d", &path.to_string_lossy(), None)
            .unwrap();
        writer.set(path.clone(), None).unwrap();
        writer.flush(DRAIN_TIMEOUT);

        assert!(!xmp::sidecar_path(&path).exists());
        assert!(lock(&index).dirty_rows("d").unwrap().is_empty());

        drop(writer);
        std::fs::remove_dir_all(&dir).unwrap();
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
            .set_rating("d", &path.to_string_lossy(), Some(4))
            .unwrap();
        writer.set(path.clone(), Some(4)).unwrap();
        writer.flush(DRAIN_TIMEOUT);

        let dirty = lock(&index).dirty_rows("d").unwrap();
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0].1, Some(4));

        drop(writer);
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::remove_dir_all(&root).unwrap();
    }
}
