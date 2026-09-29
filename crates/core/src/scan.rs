//! Extract one folder's worth of metadata and thumbnails in parallel.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use rayon::prelude::*;

use crate::arw::{FocusLocation, Shot};
use crate::candidate::{focus_cue_unless, Cue};
use crate::decode::{thumbnail_jpeg, thumbnail_jpeg_near};
use crate::faces::detect_around;
use crate::reader::read_preview;
use crate::sharpness::{eye_af_frame, score_preview, trusted_focus};

/// Quality of the cached thumbnails. 80 gives ~19KB for a 404x270 frame.
pub const THUMBNAIL_QUALITY: f32 = 80.0;

/// The long edge a JPEG file's thumbnail aims at: the ARW thumbnail's.
const JPEG_THUMBNAIL_LONG_EDGE: usize = 404;

/// Whether `path` has a RAW extension Riffle lists: `.ARW`, `.CR3`, `.DNG`
/// or `.NEF`, in any case.
pub fn is_raw_file(path: &Path) -> bool {
    path.extension().is_some_and(|e| {
        ["arw", "cr3", "dng", "nef"]
            .iter()
            .any(|raw| e.eq_ignore_ascii_case(raw))
    })
}

/// Whether `path` has a JPEG extension: `.jpg` or `.jpeg`, in any case.
pub fn is_jpeg_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
}

/// What one file contributes to the index.
#[derive(Debug, Clone)]
pub struct Entry {
    pub orientation: u16,
    pub shot: Shot,
    /// Unrotated thumbnail JPEG; the caller carries the Orientation.
    pub thumbnail: Vec<u8>,
    /// `sharpness::score_preview` of the preview; `None` when it could not be
    /// scored, which does not fail the file, and always for a JPEG file.
    pub sharpness: Option<f64>,
}

/// Read one file's metadata and thumbnail. Pure: no shared state, no IO beyond
/// `path`, and every failure comes back as `Err` rather than a panic. A JPEG
/// file gets no sharpness score and no face search.
pub fn extract(path: &Path) -> Result<Entry, String> {
    extract_unless(path, &AtomicBool::new(false)).expect("a never-set flag never abandons")
}

/// `extract`, abandoned (`None`) once `cancel` is set between its stages.
fn extract_unless(path: &Path, cancel: &AtomicBool) -> Option<Result<Entry, String>> {
    let (arw, preview) = match read_preview(path) {
        Ok(read) => read,
        Err(e) => return Some(Err(e.to_string())),
    };
    if canceled(cancel) {
        return None;
    }
    let jpeg = is_jpeg_file(path);
    // mozjpeg aborts through a panic, not an error return, when the bytes are
    // not a JPEG. A single corrupt file out of thousands must cost that file
    // and nothing else, so the decode runs inside `catch_unwind`. Nothing here
    // is shared or observable after a panic, hence `AssertUnwindSafe`.
    let thumbnail = match catch_unwind(AssertUnwindSafe(|| {
        if jpeg {
            thumbnail_jpeg_near(&preview, JPEG_THUMBNAIL_LONG_EDGE, THUMBNAIL_QUALITY)
        } else {
            thumbnail_jpeg(&preview, THUMBNAIL_QUALITY)
        }
    })) {
        Ok(Ok(thumbnail)) => thumbnail,
        Ok(Err(e)) => return Some(Err(e.to_string())),
        Err(_) => {
            return Some(Err(format!(
                "panic while encoding the thumbnail of {}",
                path.display()
            )))
        }
    };
    if canceled(cancel) {
        return None;
    }
    if jpeg {
        return Some(Ok(Entry {
            orientation: arw.orientation,
            shot: arw.shot,
            thumbnail,
            sharpness: None,
        }));
    }
    let eye_af = eye_af_frame(&arw.shot);
    let focus = trusted_focus(&arw.shot);
    // Faces steer the score only without an AF point, so they are searched for
    // on the whole image then and not at all otherwise. Any detection failure
    // is no face.
    let faces = match focus {
        Some(_) => Vec::new(),
        None => catch_unwind(AssertUnwindSafe(|| {
            detect_around(&preview, arw.orientation, None)
        }))
        .ok()
        .and_then(Result::ok)
        .map_or_else(Vec::new, |d| d.faces),
    };
    if canceled(cancel) {
        return None;
    }
    let sharpness = catch_unwind(AssertUnwindSafe(|| {
        score_preview(&preview, focus, eye_af.map(|(_, frame)| frame), &faces)
    }))
    .ok()
    .and_then(Result::ok);
    Some(Ok(Entry {
        orientation: arw.orientation,
        shot: arw.shot,
        thumbnail,
        sharpness,
    }))
}

/// Compute the focus candidate cue of one file. Only an unreadable file is
/// `Err`; a decode or detection failure, or a panic in either, is an
/// unknown cue.
pub fn extract_faces(path: &Path) -> Result<Cue, String> {
    extract_faces_unless(path, &AtomicBool::new(false)).expect("a never-set flag never abandons")
}

/// `extract_faces`, abandoned (`None`) once `cancel` is set between its
/// stages.
fn extract_faces_unless(path: &Path, cancel: &AtomicBool) -> Option<Result<Cue, String>> {
    let (arw, preview) = match read_preview(path) {
        Ok(read) => read,
        Err(e) => return Some(Err(e.to_string())),
    };
    if canceled(cancel) {
        return None;
    }
    cue(&preview, arw.orientation, trusted_focus(&arw.shot), cancel).map(Ok)
}

fn cue(
    preview: &[u8],
    orientation: u16,
    focus: Option<FocusLocation>,
    cancel: &AtomicBool,
) -> Option<Cue> {
    match catch_unwind(AssertUnwindSafe(|| {
        focus_cue_unless(preview, orientation, focus, cancel)
    })) {
        Ok(None) => None,
        Ok(Some(Ok(cue))) => Some(cue),
        Ok(Some(Err(_))) | Err(_) => Some(Cue::unknown()),
    }
}

fn canceled(cancel: &AtomicBool) -> bool {
    cancel.load(Ordering::Relaxed)
}

/// Run `extract` over `paths` on a pool of `threads` threads, handing each
/// result to `on_item` from the worker thread that produced it. Once `cancel`
/// is set no further file is started, and files already running stop at their
/// next stage and are not handed to `on_item` at all.
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
    for_each_path(paths, threads, extract_unless, on_item, cancel)
}

/// Run `extract_faces` over `paths` the way `extract_all` runs `extract`:
/// same pool, same `cancel`, same `threads == 0` error, and the same rule
/// that `on_item` must not panic.
pub fn extract_faces_all<F>(
    paths: &[PathBuf],
    threads: usize,
    on_item: F,
    cancel: &AtomicBool,
) -> Result<(), String>
where
    F: Fn(usize, Result<Cue, String>) + Send + Sync,
{
    for_each_path(paths, threads, extract_faces_unless, on_item, cancel)
}

fn for_each_path<T, E, F>(
    paths: &[PathBuf],
    threads: usize,
    per_file: E,
    on_item: F,
    cancel: &AtomicBool,
) -> Result<(), String>
where
    E: Fn(&Path, &AtomicBool) -> Option<Result<T, String>> + Send + Sync,
    F: Fn(usize, Result<T, String>) + Send + Sync,
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
            if canceled(cancel) {
                return;
            }
            if let Some(result) = per_file(path, cancel) {
                on_item(i, result);
            }
        })
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::FocusCandidate;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

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
    fn canceling_stops_the_scan_early() {
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
    fn the_faces_pass_delivers_every_index_once_and_stops_on_cancel() {
        let dir = dir("faces-all");
        let jpeg = jpeg(64, 48);
        let paths: Vec<PathBuf> = (0..200)
            .map(|i| write(&dir, &format!("{i:03}.ARW"), &fixture(1, &jpeg)))
            .collect();

        let seen = Mutex::new(Vec::new());
        extract_faces_all(
            &paths,
            4,
            |i, r| seen.lock().unwrap().push((i, r.unwrap().state)),
            &AtomicBool::new(false),
        )
        .unwrap();
        let mut seen = seen.into_inner().unwrap();
        seen.sort_by_key(|&(i, _)| i);
        assert_eq!(
            seen,
            (0..200)
                .map(|i| (i, FocusCandidate::Unknown))
                .collect::<Vec<_>>(),
            "each index exactly once"
        );

        let cancel = AtomicBool::new(false);
        let done = Mutex::new(0usize);
        extract_faces_all(
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

        assert!(extract_faces_all(&paths, 0, |_, _| {}, &AtomicBool::new(false)).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_file_canceled_mid_pipeline_is_not_delivered() {
        let dir = dir("cancel-mid");
        let path = write(&dir, "a.ARW", &fixture(1, &jpeg(64, 48)));
        let set = AtomicBool::new(true);
        assert!(extract_unless(&path, &set).is_none());
        assert!(extract_faces_unless(&path, &set).is_none());
        assert!(
            extract_unless(&dir.join("missing.ARW"), &set).is_some_and(|r| r.is_err()),
            "an unreadable file is still an error"
        );
        let focus = Some(FocusLocation {
            sensor_w: 6000,
            sensor_h: 4000,
            x: 3000,
            y: 2000,
        });
        assert_eq!(cue(&jpeg(64, 48), 1, focus, &set), None);

        // The flag is set while each file is in flight, after the pre-dispatch
        // check has let it through.
        let paths: Vec<PathBuf> = (0..8)
            .map(|i| write(&dir, &format!("{i}.ARW"), &fixture(1, &jpeg(64, 48))))
            .collect();
        for faces in [false, true] {
            let cancel = AtomicBool::new(false);
            let delivered = Mutex::new(0usize);
            for_each_path(
                &paths,
                2,
                |path, cancel: &AtomicBool| {
                    let started = Instant::now();
                    cancel.store(true, Ordering::Relaxed);
                    let result = if faces {
                        extract_faces_unless(path, cancel).map(|r| r.map(|_| ()))
                    } else {
                        extract_unless(path, cancel).map(|r| r.map(|_| ()))
                    };
                    assert!(started.elapsed() < Duration::from_secs(5));
                    result
                },
                |_, _| *delivered.lock().unwrap() += 1,
                &cancel,
            )
            .unwrap();
            assert_eq!(delivered.into_inner().unwrap(), 0, "faces pass: {faces}");
        }
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
    fn a_jpeg_file_yields_its_exif_and_a_thumbnail_but_no_score() {
        use crate::jpeg::tests::{plain_jpeg, with_exif, W};
        let dir = dir("jpeg");
        let w = W(false);
        let tiff = w.tiff(
            &[w.short(0x0112, 6)],
            &[w.ascii(0x9003, "2026:09:27 12:34:56")],
        );
        let e = extract(&write(
            &dir,
            "a.jpg",
            &with_exif(&plain_jpeg(64, 48), &tiff),
        ))
        .unwrap();
        assert_eq!(e.orientation, 6);
        assert_eq!(e.shot.capture_time.as_deref(), Some("2026:09:27 12:34:56"));
        assert_eq!(e.sharpness, None);
        assert_eq!(&e.thumbnail[..2], &[0xFF, 0xD8]);
        let bare = extract(&write(&dir, "b.JPEG", &plain_jpeg(64, 48))).unwrap();
        assert_eq!((bare.orientation, bare.sharpness), (1, None));
        assert!(!bare.thumbnail.is_empty());
        assert!(extract(&write(&dir, "c.jpg", b"not a jpeg")).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_mixed_list_of_raws_and_jpegs_delivers_every_index() {
        use crate::jpeg::tests::plain_jpeg;
        let dir = dir("mixed");
        let paths: Vec<PathBuf> = (0..8)
            .map(|i| {
                if i % 2 == 0 {
                    write(&dir, &format!("{i}.ARW"), &fixture(3, &jpeg(64, 48)))
                } else {
                    write(&dir, &format!("{i}.jpg"), &plain_jpeg(64, 48))
                }
            })
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
            (0..8)
                .map(|i| (i, if i % 2 == 0 { 3u16 } else { 1 }))
                .collect::<Vec<_>>()
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn jpeg_and_raw_extensions_are_told_apart_in_any_case() {
        for name in ["a.jpg", "a.JPG", "a.jpeg", "a.JpEg"] {
            assert!(is_jpeg_file(Path::new(name)) && !is_raw_file(Path::new(name)));
        }
        for name in ["a.ARW", "a.png", "a.jpg.xmp", "jpg"] {
            assert!(!is_jpeg_file(Path::new(name)), "{name}");
        }
        for name in ["a.ARW", "a.dng", "a.NEF", "a.nef", "a.CR3", "a.cr3"] {
            assert!(is_raw_file(Path::new(name)), "{name}");
        }
        for name in ["a.nef.xmp", "nef", "a.cr3.dop", "cr3"] {
            assert!(!is_raw_file(Path::new(name)), "{name}");
        }
    }

    #[test]
    fn the_cue_is_unknown_without_an_af_point_or_a_decodable_preview() {
        let dir = dir("faces");
        let no_af = extract_faces(&write(&dir, "noaf.ARW", &fixture(1, &jpeg(64, 48)))).unwrap();
        assert_eq!(no_af.state, FocusCandidate::Unknown);
        assert!(no_af.detection.is_none());
        let bad = extract_faces(&write(&dir, "bad.ARW", &fixture(1, &[0u8; 512]))).unwrap();
        assert_eq!(bad.state, FocusCandidate::Unknown);
        let focus = Some(FocusLocation {
            sensor_w: 6000,
            sensor_h: 4000,
            x: 3000,
            y: 2000,
        });
        let never = AtomicBool::new(false);
        assert_eq!(cue(&[0u8; 512], 1, focus, &never), Some(Cue::unknown()));
        let flat = cue(&jpeg(64, 48), 1, focus, &never).unwrap();
        assert_eq!(flat.state, FocusCandidate::Unknown);
        assert!(flat.detection.is_some_and(|d| d.point.is_some()));
        assert!(extract_faces(&dir.join("missing.ARW")).is_err());
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
