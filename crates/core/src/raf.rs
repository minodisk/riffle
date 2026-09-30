//! Read a Fujifilm RAF into the `Arw` shape the scan, the index and the meta
//! pane already consume. A RAF opens with a fixed big-endian header of offsets;
//! the one embedded JPEG it points at is a plain Exif JPEG, and its Exif is the
//! file's: orientation, capture time and the shooting settings are all read
//! from it through the path `jpeg` uses. That JPEG is both the preview and the
//! full-size JPEG.
//!
//! The AF point is the Fujifilm MakerNote's `FocusPixel`, in the embedded
//! JPEG's unrotated frame.

use anyhow::{anyhow, ensure, Result};

use crate::arw::{Arw, Codec, Embedded, FocusLocation, Shot};
use crate::exif::{integer, TAG_EXIF_IFD, TYPE_SHORT};
use crate::jpeg;
use crate::sequence::{find_exif_tiff, Tiff};

const MAGIC: &[u8] = b"FUJIFILMCCD-RAW ";
/// The header words holding the embedded JPEG's offset and length.
const JPEG_OFFSET_AT: usize = 0x54;
const JPEG_LENGTH_AT: usize = 0x58;
const TAG_MAKER_NOTE: u16 = 0x927c;
const TAG_PIXEL_X_DIMENSION: u16 = 0xa002;
const TAG_PIXEL_Y_DIMENSION: u16 = 0xa003;
/// A Fujifilm MakerNote is `FUJIFILM`, then the little-endian offset of its
/// IFD; every offset in it is relative to the note start.
const FUJIFILM_HEADER: &[u8] = b"FUJIFILM";
const TAG_FOCUS_MODE: u16 = 0x1021;
const TAG_FOCUS_PIXEL: u16 = 0x1023;
/// `FocusMode` for manual focus.
const FOCUS_MODE_MANUAL: u32 = 1;

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
    let (orientation, mut shot) = match jpeg::read_exif(jpeg) {
        Ok(exif) => exif,
        Err(_) if complete => (1, Shot::default()),
        Err(e) => return Err(e.context("RAF JPEG Exif out of range")),
    };
    shot.focus = af_point(jpeg);
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

/// `FocusPixel` in the frame of the JPEG's `PixelXDimension` x
/// `PixelYDimension`, which is the JPEG's own unrotated size. `None` on
/// manual focus (`FocusMode` 1), which still writes a `FocusPixel`, and when
/// any piece is missing. Only called once `read_exif` found the whole Exif
/// segment in `jpeg`, so everything read here lies in the buffer.
fn af_point(jpeg: &[u8]) -> Option<FocusLocation> {
    let (base, end) = find_exif_tiff(jpeg).ok()?;
    let tiff = Tiff::new(jpeg, base, end).ok()?;
    let ifd0 = tiff.ifd_entries(tiff.u32(4).ok()? as usize).ok()?;
    let exif_ifd = ifd0.iter().find(|e| e.tag == TAG_EXIF_IFD)?;
    let exif = tiff
        .ifd_entries(tiff.u32(exif_ifd.value_field).ok()? as usize)
        .ok()?;
    let dimension = |tag| {
        let e = exif.iter().find(|e| e.tag == tag)?;
        u16::try_from(integer(&tiff, e)?).ok().filter(|&v| v > 0)
    };
    let (sensor_w, sensor_h) = (
        dimension(TAG_PIXEL_X_DIMENSION)?,
        dimension(TAG_PIXEL_Y_DIMENSION)?,
    );
    let e = exif.iter().find(|e| e.tag == TAG_MAKER_NOTE)?;
    let at = base.checked_add(tiff.u32(e.value_field).ok()? as usize)?;
    let note = Tiff::with_order(jpeg, at, end, true).ok()?;
    if note.bytes(0, FUJIFILM_HEADER.len()).ok()? != FUJIFILM_HEADER {
        return None;
    }
    let entries = note
        .ifd_entries(note.u32(FUJIFILM_HEADER.len()).ok()? as usize)
        .ok()?;
    if let Some(e) = entries.iter().find(|e| e.tag == TAG_FOCUS_MODE) {
        if integer(&note, e) == Some(FOCUS_MODE_MANUAL) {
            return None;
        }
    }
    let e = entries.iter().find(|e| e.tag == TAG_FOCUS_PIXEL)?;
    if e.typ != TYPE_SHORT || e.count != 2 {
        return None;
    }
    Some(FocusLocation {
        sensor_w,
        sensor_h,
        x: note.u16(e.value_field).ok()?,
        y: note.u16(e.value_field + 2).ok()?,
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

    /// A Fujifilm MakerNote holding `FocusMode` (when given) and `FocusPixel`,
    /// always little-endian with note-relative offsets.
    fn fuji_note(focus_mode: Option<u16>, pixel: [u16; 2]) -> Vec<u8> {
        let mut entries: Vec<(u16, u16, u32, [u8; 4])> = Vec::new();
        if let Some(m) = focus_mode {
            let v = m.to_le_bytes();
            entries.push((TAG_FOCUS_MODE, TYPE_SHORT, 1, [v[0], v[1], 0, 0]));
        }
        let (x, y) = (pixel[0].to_le_bytes(), pixel[1].to_le_bytes());
        entries.push((TAG_FOCUS_PIXEL, TYPE_SHORT, 2, [x[0], x[1], y[0], y[1]]));
        let mut out = FUJIFILM_HEADER.to_vec();
        out.extend_from_slice(&12u32.to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        for (tag, typ, count, value) in entries {
            out.extend_from_slice(&tag.to_le_bytes());
            out.extend_from_slice(&typ.to_le_bytes());
            out.extend_from_slice(&count.to_le_bytes());
            out.extend_from_slice(&value);
        }
        out.extend_from_slice(&0u32.to_le_bytes());
        out
    }

    /// An Exif JPEG whose Exif IFD carries `PixelXDimension` x
    /// `PixelYDimension` (when given) and `note` as its MakerNote.
    fn af_jpeg(w: &W, dimensions: Option<(u32, u32)>, note: &[u8]) -> Vec<u8> {
        let mut exif = vec![(TAG_MAKER_NOTE, 7, note.len() as u32, note.to_vec())];
        if let Some((pw, ph)) = dimensions {
            exif.push(w.long(TAG_PIXEL_X_DIMENSION, pw));
            exif.push(w.short(TAG_PIXEL_Y_DIMENSION, ph as u16));
        }
        let tiff = w.tiff(&[w.ascii(TAG_MAKE, "FUJIFILM")], &exif);
        with_exif(&plain_jpeg(16, 16), &tiff)
    }

    #[test]
    fn reads_focus_pixel_in_the_jpeg_frame_in_both_byte_orders() {
        for le in [true, false] {
            let w = W(le);
            for mode in [None, Some(0)] {
                let jpeg = af_jpeg(&w, Some((4416, 2944)), &fuji_note(mode, [1592, 1472]));
                let a = parse(&raf(JPEG_AT, &jpeg)).unwrap();
                assert_eq!(
                    a.shot.focus,
                    Some(FocusLocation {
                        sensor_w: 4416,
                        sensor_h: 2944,
                        x: 1592,
                        y: 1472,
                    }),
                    "le {le}, mode {mode:?}"
                );
                assert!(a.shot.focus_mode.is_none() && a.shot.focus_frame.is_none());
            }
        }
    }

    #[test]
    fn manual_focus_has_no_af_point() {
        let jpeg = af_jpeg(
            &W(true),
            Some((4416, 2944)),
            &fuji_note(Some(1), [2517, 1472]),
        );
        assert!(parse(&raf(JPEG_AT, &jpeg)).unwrap().shot.focus.is_none());
    }

    #[test]
    fn a_missing_or_malformed_piece_has_no_af_point() {
        let w = W(false);
        let note = fuji_note(Some(0), [100, 200]);
        let mut not_fuji = note.clone();
        not_fuji[..8].copy_from_slice(b"FUJIFILX");
        let mut bad_ifd = note.clone();
        bad_ifd[8..12].copy_from_slice(&0xFFFFu32.to_le_bytes());
        let mut one_value = note.clone();
        // The FocusPixel entry's count.
        let count_at = 12 + 2 + 12 + 4;
        one_value[count_at..count_at + 4].copy_from_slice(&1u32.to_le_bytes());
        for (dimensions, note) in [
            (None, &note),
            (Some((0, 2944)), &note),
            (Some((4416, 2944)), &not_fuji),
            (Some((4416, 2944)), &bad_ifd),
            (Some((4416, 2944)), &one_value),
            (Some((4416, 2944)), &note[..6].to_vec()),
        ] {
            let a = parse(&raf(JPEG_AT, &af_jpeg(&w, dimensions, note))).unwrap();
            assert!(a.shot.focus.is_none(), "{dimensions:?} {note:?}");
            assert_eq!(a.shot.make.as_deref(), Some("FUJIFILM"));
        }
    }

    #[test]
    fn a_prefix_cut_inside_the_maker_note_is_an_error() {
        let note = fuji_note(Some(0), [1592, 1472]);
        let buf = raf(JPEG_AT, &af_jpeg(&W(true), Some((4416, 2944)), &note));
        let at = buf.windows(12).position(|b| b == &note[..12]).unwrap();
        for cut in [at + 4, at + 12, at + note.len() - 1] {
            assert!(parse(&buf[..cut]).is_err(), "cut at {cut}");
        }
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
