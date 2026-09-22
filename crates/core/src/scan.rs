//! Extract one folder's worth of metadata and thumbnails in parallel.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use rayon::prelude::*;

use crate::arw::Shot;
use crate::decode::{apply_orientation, decode_rgb, thumbnail_jpeg};
use crate::faces::{self, Face};
use crate::reader::read_preview;
use crate::sharpness::{eye_af_frame, score_preview, trusted_focus};

/// Quality of the cached thumbnails. 80 gives ~19KB for a 404x270 frame.
pub const THUMBNAIL_QUALITY: f32 = 80.0;

/// Whether `path` has a RAW extension Riffle lists: `.ARW` or `.DNG`, in any
/// case.
pub fn is_raw_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("arw") || e.eq_ignore_ascii_case("dng"))
}

/// What one file contributes to the index.
#[derive(Debug, Clone)]
pub struct Entry {
    pub orientation: u16,
    pub shot: Shot,
    /// Unrotated thumbnail JPEG; the caller carries the Orientation.
    pub thumbnail: Vec<u8>,
    /// `sharpness::score_preview` of the preview; `None` when it could not be
    /// scored, which does not fail the file.
    pub sharpness: Option<f64>,
}

/// Read one file's metadata and thumbnail. Pure: no shared state, no IO beyond
/// `path`, and every failure comes back as `Err` rather than a panic.
pub fn extract(path: &Path) -> Result<Entry, String> {
    let (arw, preview) = read_preview(path).map_err(|e| e.to_string())?;
    // mozjpeg aborts through a panic, not an error return, when the bytes are
    // not a JPEG. A single corrupt file out of thousands must cost that file
    // and nothing else, so the decode runs inside `catch_unwind`. Nothing here
    // is shared or observable after a panic, hence `AssertUnwindSafe`.
    let thumbnail = catch_unwind(AssertUnwindSafe(|| {
        thumbnail_jpeg(&preview, THUMBNAIL_QUALITY)
    }))
    .map_err(|_| format!("panic while encoding the thumbnail of {}", path.display()))?
    .map_err(|e| e.to_string())?;
    let eye_af = eye_af_frame(&arw.shot);
    let faces = if eye_af.is_some() {
        Vec::new()
    } else {
        catch_unwind(AssertUnwindSafe(|| detect_faces(&preview, arw.orientation)))
            .unwrap_or_default()
    };
    let sharpness = catch_unwind(AssertUnwindSafe(|| {
        score_preview(
            &preview,
            trusted_focus(&arw.shot),
            eye_af.map(|(_, frame)| frame),
            &faces,
        )
    }))
    .ok()
    .and_then(Result::ok);
    Ok(Entry {
        orientation: arw.orientation,
        shot: arw.shot,
        thumbnail,
        sharpness,
    })
}

/// Faces in `preview`, in its stored coordinates. The detector runs on the
/// upright image; any failure is no face.
fn detect_faces(preview: &[u8], orientation: u16) -> Vec<Face> {
    let Ok((rgb, w, h)) = decode_rgb(preview) else {
        return Vec::new();
    };
    let (mut upright, uw, uh) = apply_orientation(&rgb, w, h, orientation);
    if orientation == 3 {
        upright = upright.chunks_exact(3).rev().flatten().copied().collect();
    }
    faces::detect(&upright, uw, uh)
        .unwrap_or_default()
        .into_iter()
        .map(|f| faces::to_stored(f, orientation, w, h))
        .collect()
}

/// Run `extract` over `paths` on a pool of `threads` threads, handing each
/// result to `on_item` from the worker thread that produced it. Once `cancel`
/// is set no further file is started; files already running finish.
/// `threads` must be at least 1; `0` is rejected rather than silently falling
/// back to rayon's default pool size.
///
/// The pool is built here rather than taken from rayon's global one, so the
/// caller sizes it and nothing else in the process shares it.
///
/// `on_item` must not panic: a panic inside it unwinds out of the worker's
/// `for_each`, aborts the scan, and propagates out of `extract_all` as a
/// panic rather than an `Err`, leaving an arbitrary subset of indices
/// delivered. Callers that share state behind a mutex (as Step 4's Tauri
/// command does) must keep `on_item` panic-free or wrap it themselves.
pub fn extract_all<F>(
    paths: &[PathBuf],
    threads: usize,
    on_item: F,
    cancel: &AtomicBool,
) -> Result<(), String>
where
    F: Fn(usize, Result<Entry, String>) + Send + Sync,
{
    if threads == 0 {
        return Err("threads must be at least 1".to_string());
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|e| e.to_string())?;
    pool.install(|| {
        paths.par_iter().enumerate().for_each(|(i, path)| {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            on_item(i, extract(path));
        })
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn jpeg(w: usize, h: usize) -> Vec<u8> {
        let rgb = vec![128u8; w * h * 3];
        let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        c.set_size(w, h);
        c.set_quality(80.0);
        let mut c = c.start_compress(Vec::new()).unwrap();
        c.write_scanlines(&rgb).unwrap();
        c.finish().unwrap()
    }

    /// A TIFF shell whose IFD0 carries an Orientation and a preview of `body`.
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
        assert_eq!(buf.len(), at);
        buf.extend_from_slice(body);
        buf
    }

    fn dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("riffle-scan-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn every_index_is_delivered_once() {
        let dir = dir("all");
        let jpeg = jpeg(64, 48);
        let paths: Vec<PathBuf> = (0..16)
            .map(|i| write(&dir, &format!("{i:02}.ARW"), &fixture(6, &jpeg)))
            .collect();

        let seen = Mutex::new(Vec::new());
        extract_all(
            &paths,
            4,
            |i, r| seen.lock().unwrap().push((i, r.unwrap().orientation)),
            &AtomicBool::new(false),
        )
        .unwrap();

        let mut seen = seen.into_inner().unwrap();
        seen.sort();
        assert_eq!(
            seen,
            (0..16).map(|i| (i, 6u16)).collect::<Vec<_>>(),
            "each index exactly once"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn cancelling_stops_the_scan_early() {
        let dir = dir("cancel");
        let jpeg = jpeg(64, 48);
        let paths: Vec<PathBuf> = (0..200)
            .map(|i| write(&dir, &format!("{i:03}.ARW"), &fixture(1, &jpeg)))
            .collect();

        let cancel = AtomicBool::new(false);
        let done = Mutex::new(0usize);
        extract_all(
            &paths,
            2,
            |_, _| {
                let mut done = done.lock().unwrap();
                *done += 1;
                if *done >= 4 {
                    cancel.store(true, Ordering::Relaxed);
                }
            },
            &cancel,
        )
        .unwrap();

        let done = done.into_inner().unwrap();
        assert!(done >= 4, "the files before the cancel are delivered");
        assert!(done < paths.len(), "the rest are not, got {done}");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_preview_that_cannot_be_scored_still_yields_a_thumbnail() {
        let dir = dir("unscored");
        let ok = extract(&write(&dir, "ok.ARW", &fixture(1, &jpeg(64, 48)))).unwrap();
        assert!(ok.sharpness.is_some());
        let tiny = extract(&write(&dir, "tiny.ARW", &fixture(1, &jpeg(2, 2)))).unwrap();
        assert_eq!(tiny.sharpness, None);
        assert!(!tiny.thumbnail.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_file_whose_preview_is_not_a_jpeg_is_one_error_not_a_crash() {
        let dir = dir("corrupt");
        let jpeg = jpeg(64, 48);
        let mut paths: Vec<PathBuf> = (0..4)
            .map(|i| write(&dir, &format!("{i}.ARW"), &fixture(1, &jpeg)))
            .collect();
        paths.push(write(&dir, "bad.ARW", &fixture(1, &[0u8; 512])));

        let errors = Mutex::new(Vec::new());
        extract_all(
            &paths,
            4,
            |i, r| {
                if r.is_err() {
                    errors.lock().unwrap().push(i)
                }
            },
            &AtomicBool::new(false),
        )
        .unwrap();

        assert_eq!(errors.into_inner().unwrap(), vec![4]);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
