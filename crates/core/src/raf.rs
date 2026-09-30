//! Read a Fujifilm RAF into the `Arw` shape the scan, the index and the meta
//! pane already consume. A RAF opens with a fixed big-endian header of offsets;
//! the one embedded JPEG it points at is a plain Exif JPEG, and its Exif is the
//! file's: orientation, capture time and the shooting settings are all read
//! from it through the path `jpeg` uses. That JPEG is both the preview and the
//! full-size JPEG.

use anyhow::{anyhow, ensure, Result};

use crate::arw::{Arw, Codec, Embedded, Shot};
use crate::jpeg;

const MAGIC: &[u8] = b"FUJIFILMCCD-RAW ";
/// The header words holding the embedded JPEG's offset and length.
const JPEG_OFFSET_AT: usize = 0x54;
const JPEG_LENGTH_AT: usize = 0x58;

/// Parse a RAF (or a prefix of one). A header or a JPEG start past `buf` is an
/// error, and so is a JPEG cut off by the end of `buf` before its Exif segment
/// ends, so a prefix too short to hold them is never taken for a file without
/// Exif. A JPEG that lies wholly in `buf` without an Exif segment is
/// orientation 1 and a default `Shot`, as in `jpeg::parse`.
pub fn parse(buf: &[u8]) -> Result<Arw> {
    ensure!(
        buf.starts_with(MAGIC),
        "not a RAF file (missing FUJIFILMCCD-RAW magic)"
    );
    let word = |at: usize| {
        buf.get(at..at + 4)
            .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
            .ok_or_else(|| anyhow!("RAF header out of range"))
    };
    let (offset, length) = (word(JPEG_OFFSET_AT)?, word(JPEG_LENGTH_AT)?);
    ensure!(
        offset > 0 && length > 0 && offset & 0x8000 == 0,
        "RAF without an embedded JPEG"
    );
    let end = offset
        .checked_add(length)
        .ok_or_else(|| anyhow!("RAF JPEG out of range"))?;
    ensure!(offset < buf.len(), "RAF JPEG out of range");
    let complete = end <= buf.len();
    let jpeg = &buf[offset..end.min(buf.len())];
    ensure!(
        jpeg.starts_with(&[0xFF, 0xD8]),
        "RAF JPEG is not a JPEG (missing SOI marker)"
    );
    let (orientation, shot) = match jpeg::read_exif(jpeg) {
        Ok(exif) => exif,
        Err(_) if complete => (1, Shot::default()),
        Err(e) => return Err(e.context("RAF JPEG Exif out of range")),
    };
    let embedded = Some(Embedded {
        offset,
        length,
        codec: Codec::Jpeg,
    });
    Ok(Arw {
        preview: embedded,
        full: embedded,
        orientation,
        shot,
        hevc: false,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::exif::*;
    use crate::jpeg::tests::{plain_jpeg, with_exif, W};

    /// Where the camera writes the JPEG.
    pub(crate) const JPEG_AT: usize = 0x94;

    /// A RAF header pointing at `jpeg` placed at `at`, then `jpeg`.
    pub(crate) fn raf(at: usize, jpeg: &[u8]) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(b"0201");
        out.resize(0x1c, 0);
        out.extend_from_slice(b"X-T5");
        out.resize(JPEG_OFFSET_AT, 0);
        out.extend_from_slice(&(at as u32).to_be_bytes());
        out.extend_from_slice(&(jpeg.len() as u32).to_be_bytes());
        out.resize(at, 0);
        out.extend_from_slice(jpeg);
        out
    }

    pub(crate) fn exif_jpeg(w: &W, orientation: u16) -> Vec<u8> {
        let tiff = w.tiff(
            &[
                w.ascii(TAG_MAKE, "FUJIFILM"),
                w.ascii(TAG_MODEL, "X-T5"),
                w.short(TAG_ORIENTATION, orientation),
            ],
            &[
                w.ascii(TAG_DATE_TIME_ORIGINAL, "2026:09:30 10:11:12"),
                w.ascii(TAG_SUB_SEC_TIME_ORIGINAL, "45"),
                w.short(TAG_ISO, 400),
            ],
        );
        with_exif(&plain_jpeg(16, 16), &tiff)
    }

    #[test]
    fn reads_the_jpeg_and_its_exif_in_both_byte_orders() {
        for le in [true, false] {
            let jpeg = exif_jpeg(&W(le), 6);
            let a = parse(&raf(JPEG_AT, &jpeg)).unwrap();
            let (p, f) = (a.preview.unwrap(), a.full.unwrap());
            assert_eq!((p.offset, p.length), (JPEG_AT, jpeg.len()), "le {le}");
            assert_eq!((f.offset, f.length), (JPEG_AT, jpeg.len()));
            assert_eq!(a.orientation, 6);
            assert_eq!(a.shot.make.as_deref(), Some("FUJIFILM"));
            assert_eq!(a.shot.model.as_deref(), Some("X-T5"));
            assert_eq!(a.shot.capture_time.as_deref(), Some("2026:09:30 10:11:12"));
            assert_eq!(a.shot.subsec.as_deref(), Some("45"));
            assert_eq!(a.shot.iso, Some(400));
            assert!(a.shot.focus.is_none() && a.shot.focus_mode.is_none());
        }
    }

    #[test]
    fn a_prefix_holding_the_exif_but_not_the_whole_jpeg_parses() {
        let mut jpeg = exif_jpeg(&W(true), 8);
        jpeg.resize(jpeg.len() + 4096, 0);
        let buf = raf(JPEG_AT, &jpeg);
        let a = parse(&buf[..buf.len() - 4000]).unwrap();
        assert_eq!(a.orientation, 8);
        assert_eq!(a.preview.unwrap().length, jpeg.len());
    }

    #[test]
    fn a_prefix_cut_inside_the_exif_segment_is_an_error() {
        let buf = raf(JPEG_AT, &exif_jpeg(&W(false), 6));
        // SOI, then the APP1 marker and its length, which counts itself.
        let len = u16::from_be_bytes([buf[JPEG_AT + 4], buf[JPEG_AT + 5]]) as usize;
        let app1_end = JPEG_AT + 4 + len;
        for cut in [JPEG_AT + 1, JPEG_AT + 3, JPEG_AT + 20, app1_end - 1] {
            assert!(parse(&buf[..cut]).is_err(), "cut at {cut}");
        }
        assert_eq!(parse(&buf[..app1_end]).unwrap().orientation, 6);
    }

    #[test]
    fn a_whole_jpeg_without_exif_is_the_defaults() {
        let a = parse(&raf(JPEG_AT, &plain_jpeg(16, 16))).unwrap();
        assert_eq!(a.orientation, 1);
        assert!(a.shot.capture_time.is_none() && a.shot.make.is_none());
        assert!(a.preview.is_some());
    }

    #[test]
    fn a_bad_magic_or_header_is_an_error() {
        let buf = raf(JPEG_AT, &exif_jpeg(&W(true), 1));
        let mut bad = buf.clone();
        bad[0] = b'X';
        assert!(parse(&bad).is_err());
        assert!(parse(&buf[..JPEG_LENGTH_AT + 2]).is_err());
        assert!(parse(&[]).is_err());

        let mut not_jpeg = buf.clone();
        not_jpeg[JPEG_AT] = 0;
        assert!(parse(&not_jpeg).is_err());
    }

    #[test]
    fn a_zero_or_flagged_jpeg_offset_or_length_is_an_error() {
        let buf = raf(JPEG_AT, &exif_jpeg(&W(true), 1));
        for (at, value) in [
            (JPEG_OFFSET_AT, 0u32),
            (JPEG_LENGTH_AT, 0),
            (JPEG_OFFSET_AT, 0x8000 | JPEG_AT as u32),
        ] {
            let mut bad = buf.clone();
            bad[at..at + 4].copy_from_slice(&value.to_be_bytes());
            assert!(parse(&bad).is_err(), "{at:#x} = {value:#x}");
        }
    }

    #[test]
    fn a_jpeg_past_the_buffer_is_an_error() {
        let buf = raf(JPEG_AT, &exif_jpeg(&W(true), 1));
        for offset in [buf.len() as u32, 0x7FFF_0000] {
            let mut bad = buf.clone();
            bad[JPEG_OFFSET_AT..JPEG_OFFSET_AT + 4].copy_from_slice(&offset.to_be_bytes());
            assert!(parse(&bad).is_err(), "{offset:#x}");
        }
        let mut bad = buf.clone();
        bad[JPEG_LENGTH_AT..JPEG_LENGTH_AT + 4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(
            parse(&bad).is_ok(),
            "the JPEG's end is the reader's to check"
        );
    }
}
