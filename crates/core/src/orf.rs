//! Read an OM System / Olympus ORF into the `Arw` shape the scan, the index
//! and the meta pane already consume. An ORF is a TIFF whose header carries
//! the magic `RO` (or `RS` on old bodies) instead of 42; IFD0 and the Exif IFD
//! are read as in any TIFF.
//!
//! The only embedded JPEG is the 3200x2400 preview inside the Olympus
//! MakerNote, located by its CameraSettings sub-IFD. IFD0 has no JPEG of its
//! own and its strips are the raw data, so the preview is also `full`: the 1:1
//! view is limited to it.
//!
//! The MakerNote opens with `OLYMPUS\0` or `OM SYSTEM\0\0\0`, then its own
//! byte order and a version, and no TIFF header; every offset in it is
//! relative to the note start. The note holds the preview, so it runs past the
//! bounded prefix `reader` reads; only its front (the IFDs) must be in `buf`.

use anyhow::{bail, Result};

use crate::arw::{Arw, Embedded, Shot};
use crate::exif::{self, integer, TYPE_LONG};
use crate::sequence::Tiff;

/// The TIFF magic of an ORF: `RO`, and `RS` on some old bodies.
const MAGIC: &[u16] = &[0x4f52, 0x5352];
const TAG_MAKER_NOTE: u16 = 0x927c;
const OLYMPUS_HEADER: &[u8] = b"OLYMPUS\0";
const OM_SYSTEM_HEADER: &[u8] = b"OM SYSTEM\0\0\0";
const TAG_CAMERA_SETTINGS: u16 = 0x2020;
const TYPE_IFD: u16 = 13;
const TAG_PREVIEW_VALID: u16 = 0x0100;
const TAG_PREVIEW_START: u16 = 0x0101;
const TAG_PREVIEW_LENGTH: u16 = 0x0102;

/// Parse an ORF (or a prefix of one). As in `nef::parse`, IFD0 or the Exif IFD
/// cut by the end of `buf` is an error, and so is a MakerNote whose header or
/// IFDs up to the CameraSettings one are cut, so the reader retries with the
/// whole file instead of taking the prefix for a file without a preview. The
/// preview itself may lie past `buf`. The individual Exif fields stay lenient.
pub fn parse(buf: &[u8]) -> Result<Arw> {
    let tiff = Tiff::new_with_magic(buf, 0, buf.len(), MAGIC)?;
    let ifd0_at = tiff.u32(4)? as usize;
    let ifd0 = tiff.ifd_entries(ifd0_at)?;
    let mut shot = Shot::default();
    let (orientation, exif_ifd) = exif::read_ifd0(&tiff, &ifd0, &mut shot);
    // Olympus pads Make and Model with spaces to a fixed length.
    for text in [&mut shot.make, &mut shot.model].into_iter().flatten() {
        text.truncate(text.trim_end().len());
    }
    let mut preview = None;
    if let Some(at) = exif_ifd {
        let entries = tiff.ifd_entries(at)?;
        // The next-IFD link ends the IFD, so the inline values are in `buf` too.
        tiff.u32(at + 2 + 12 * entries.len())?;
        exif::read_exif_ifd(&tiff, &entries, &mut shot);
        if let Some(e) = entries.iter().find(|e| e.tag == TAG_MAKER_NOTE) {
            preview = maker_note_preview(buf, e.count as usize, tiff.u32(e.value_field)? as usize)?;
        }
    }

    Ok(Arw {
        preview,
        full: preview,
        orientation,
        shot,
    })
}

/// The preview of the MakerNote of `len` bytes at `at`. A note with neither
/// header, or one that is wholly in `buf` but unreadable, has none; a note cut
/// by the end of `buf` before its CameraSettings IFD ends is an error.
fn maker_note_preview(buf: &[u8], len: usize, at: usize) -> Result<Option<Embedded>> {
    let cut = at.saturating_add(len) > buf.len();
    let end = at.saturating_add(len).min(buf.len());
    if at > end {
        bail!("Olympus MakerNote out of range");
    }
    let Some((ifd, little_endian)) = header(&buf[at..end]) else {
        if cut && end - at < OM_SYSTEM_HEADER.len() + 4 {
            bail!("Olympus MakerNote header out of range");
        }
        return Ok(None);
    };
    let note = Tiff::with_order(buf, at, end, little_endian)?;
    match camera_settings_preview(&note, ifd) {
        Err(e) if cut => Err(e),
        Err(_) => Ok(None),
        Ok(p) => Ok(p.map(|(start, length)| Embedded {
            offset: at + start,
            length,
        })),
    }
}

/// The note-relative offset of the note's IFD and the note's byte order.
fn header(note: &[u8]) -> Option<(usize, bool)> {
    let (order, ifd) = if note.starts_with(OLYMPUS_HEADER) {
        (note.get(8..10)?, 12)
    } else if note.starts_with(OM_SYSTEM_HEADER) {
        (note.get(12..14)?, 16)
    } else {
        return None;
    };
    match order {
        b"II" => Some((ifd, true)),
        b"MM" => Some((ifd, false)),
        _ => None,
    }
}

/// CameraSettings' `PreviewImageStart` (note-relative) and
/// `PreviewImageLength`, unless `PreviewImageValid` says there is none.
fn camera_settings_preview(note: &Tiff, ifd: usize) -> Result<Option<(usize, usize)>> {
    let main = note.ifd_entries(ifd)?;
    note.u32(ifd + 2 + 12 * main.len())?;
    let Some(e) = main.iter().find(|e| e.tag == TAG_CAMERA_SETTINGS) else {
        return Ok(None);
    };
    // Old bodies write the sub-IFD inline as `undefined` bytes; none of the
    // supported ones does.
    if !matches!(e.typ, TYPE_IFD | TYPE_LONG) || e.count != 1 {
        return Ok(None);
    }
    let at = note.u32(e.value_field)? as usize;
    let entries = note.ifd_entries(at)?;
    note.u32(at + 2 + 12 * entries.len())?;
    let int = |tag| {
        entries
            .iter()
            .find(|e| e.tag == tag)
            .and_then(|e| integer(note, e))
    };
    if int(TAG_PREVIEW_VALID).is_some_and(|v| v != 1) {
        return Ok(None);
    }
    let (Some(start), Some(length)) = (int(TAG_PREVIEW_START), int(TAG_PREVIEW_LENGTH)) else {
        return Ok(None);
    };
    Ok((length > 0).then_some((start as usize, length as usize)))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::arw::Rational;
    use crate::exif::*;
    use crate::jpeg::tests::{Field, W};

    const TYPE_UNDEFINED: u16 = 7;

    /// An ORF-shaped TIFF: `W::tiff` with the ORF magic in place of 42.
    pub(crate) fn orf(w: &W, ifd0: &[Field], exif: &[Field]) -> Vec<u8> {
        let mut out = w.tiff(ifd0, exif);
        out[2..4].copy_from_slice(&w.u16(0x4f52));
        out
    }

    /// An Olympus (`om` false) or OM System MakerNote in `w`'s byte order
    /// whose IFD points at a CameraSettings IFD holding `camera` (inline
    /// values only), `pad` bytes after the IFD.
    pub(crate) fn maker_note(w: &W, om: bool, pad: usize, camera: &[Field]) -> Field {
        let mut v = if om {
            OM_SYSTEM_HEADER.to_vec()
        } else {
            OLYMPUS_HEADER.to_vec()
        };
        v.extend_from_slice(if w.0 { b"II" } else { b"MM" });
        v.extend_from_slice(if om { b"\x04\x00" } else { b"\x03\x00" });
        let cs_at = v.len() + 2 + 12 + 4 + pad;
        v.extend_from_slice(&w.u16(1));
        v.extend_from_slice(&w.u16(TAG_CAMERA_SETTINGS));
        v.extend_from_slice(&w.u16(TYPE_IFD));
        v.extend_from_slice(&w.u32(1));
        v.extend_from_slice(&w.u32(cs_at as u32));
        v.extend_from_slice(&w.u32(0));
        v.resize(cs_at, 0);
        v.extend_from_slice(&w.u16(camera.len() as u16));
        for (tag, typ, count, bytes) in camera {
            assert!(bytes.len() <= 4);
            v.extend_from_slice(&w.u16(*tag));
            v.extend_from_slice(&w.u16(*typ));
            v.extend_from_slice(&w.u32(*count));
            let mut inline = bytes.clone();
            inline.resize(4, 0);
            v.extend_from_slice(&inline);
        }
        v.extend_from_slice(&w.u32(0));
        (TAG_MAKER_NOTE, TYPE_UNDEFINED, v.len() as u32, v)
    }

    pub(crate) fn preview_fields(w: &W, start: u32, length: u32) -> Vec<Field> {
        vec![
            w.long(TAG_PREVIEW_VALID, 1),
            w.long(TAG_PREVIEW_START, start),
            w.long(TAG_PREVIEW_LENGTH, length),
        ]
    }

    /// Where the MakerNote starts in `file`.
    pub(crate) fn note_at(file: &[u8]) -> usize {
        file.windows(OLYMPUS_HEADER.len())
            .position(|b| b == OLYMPUS_HEADER || b == &OM_SYSTEM_HEADER[..8])
            .unwrap()
    }

    fn at(e: Option<Embedded>) -> Option<(usize, usize)> {
        e.map(|e| (e.offset, e.length))
    }

    #[test]
    fn the_preview_is_found_relative_to_the_note_start() {
        for file_le in [true, false] {
            for note_le in [true, false] {
                for om in [true, false] {
                    let (file, note) = (W(file_le), W(note_le));
                    let buf = orf(
                        &file,
                        &[file.short(TAG_ORIENTATION, 1)],
                        &[maker_note(&note, om, 0, &preview_fields(&note, 5000, 1234))],
                    );
                    let a = parse(&buf).unwrap();
                    let expected = Some((note_at(&buf) + 5000, 1234));
                    assert_eq!(at(a.preview), expected, "{file_le} {note_le} {om}");
                    assert_eq!(at(a.full), expected);
                }
            }
        }
    }

    #[test]
    fn preview_image_valid_decides_whether_there_is_a_preview() {
        let w = W(true);
        let read = |camera: &[Field]| {
            let buf = orf(&w, &[], &[maker_note(&w, true, 0, camera)]);
            at(parse(&buf).unwrap().preview).map(|(_, length)| length)
        };
        assert_eq!(read(&preview_fields(&w, 5000, 1234)), Some(1234));
        let mut invalid = preview_fields(&w, 5000, 1234);
        invalid[0] = w.long(TAG_PREVIEW_VALID, 0);
        assert_eq!(read(&invalid), None);
        assert_eq!(read(&preview_fields(&w, 5000, 1234)[1..]), Some(1234));
        assert_eq!(read(&preview_fields(&w, 5000, 0)), None);
        assert_eq!(read(&preview_fields(&w, 5000, 1234)[..2]), None);
    }

    #[test]
    fn a_note_with_neither_header_has_no_preview() {
        let w = W(true);
        let (tag, typ, count, mut v) = maker_note(&w, false, 0, &preview_fields(&w, 5000, 1234));
        v[..8].copy_from_slice(b"SONY DSC");
        let buf = orf(&w, &[w.short(TAG_ORIENTATION, 6)], &[(tag, typ, count, v)]);
        let a = parse(&buf).unwrap();
        assert!(a.preview.is_none() && a.full.is_none());
        assert_eq!(a.orientation, 6);
    }

    #[test]
    fn a_note_cut_after_the_camera_settings_ifd_still_gives_the_preview() {
        let w = W(true);
        let camera = preview_fields(&w, 5000, 1234);
        let buf = orf(&w, &[], &[maker_note(&w, true, 64, &camera)]);
        let cs_end = buf.len();
        let mut long = buf.clone();
        long.resize(cs_end + 4096, 0);
        // Grow the declared note length past the buffer, as on every body:
        // the count of the Exif IFD's only entry, after IFD0's one entry.
        let count_at = 8 + (2 + 12 + 4) + 2 + 4;
        assert_eq!(
            long[count_at..count_at + 4],
            w.u32((cs_end - note_at(&buf)) as u32)
        );
        long[count_at..count_at + 4].copy_from_slice(&w.u32(1_500_000));
        let a = parse(&long[..cs_end]).unwrap();
        assert_eq!(at(a.preview), Some((note_at(&buf) + 5000, 1234)));

        let cs_at = cs_end - (2 + 12 * camera.len() + 4);
        for cut in [
            cs_end - 1,
            cs_at + 10,
            cs_at,
            note_at(&buf) + 20,
            note_at(&buf) + 4,
        ] {
            assert!(parse(&long[..cut]).is_err(), "cut at {cut}");
        }
    }

    #[test]
    fn reads_the_ifd0_and_exif_fields() {
        for le in [true, false] {
            let w = W(le);
            let buf = orf(
                &w,
                &[
                    w.ascii(TAG_MAKE, "OM Digital Solutions   "),
                    w.ascii(TAG_MODEL, "OM-1            "),
                    w.short(TAG_ORIENTATION, 8),
                ],
                &[
                    w.ascii(TAG_DATE_TIME_ORIGINAL, "2022:03:05 15:47:24"),
                    w.ascii(TAG_SUB_SEC_TIME_ORIGINAL, "12"),
                    w.rational(TAG_F_NUMBER, 28, 10),
                    w.rational(TAG_EXPOSURE_TIME, 1, 320),
                    w.short(TAG_ISO, 200),
                    maker_note(&w, true, 0, &preview_fields(&w, 5000, 1234)),
                ],
            );
            let a = parse(&buf).unwrap();
            let s = &a.shot;
            assert_eq!(a.orientation, 8, "le {le}");
            assert_eq!(s.make.as_deref(), Some("OM Digital Solutions"));
            assert_eq!(s.model.as_deref(), Some("OM-1"));
            assert_eq!(s.capture_time.as_deref(), Some("2022:03:05 15:47:24"));
            assert_eq!(s.subsec.as_deref(), Some("12"));
            assert_eq!(s.f_number, Some(Rational { num: 28, den: 10 }));
            assert_eq!(s.exposure_time, Some(Rational { num: 1, den: 320 }));
            assert_eq!(s.iso, Some(200));
            assert!(s.focus.is_none());
            assert!(a.preview.is_some());
        }
    }

    #[test]
    fn a_structure_past_the_buffer_is_an_error() {
        let w = W(false);
        let buf = orf(
            &w,
            &[w.short(TAG_ORIENTATION, 1)],
            &[w.ascii(TAG_DATE_TIME_ORIGINAL, "2022:03:05 15:47:24")],
        );
        assert!(parse(&buf).unwrap().preview.is_none());
        assert!(parse(&buf[..20]).is_err(), "a cut IFD0");
        assert!(
            parse(&buf[..8 + 2 + 12 * 2 + 4 + 10]).is_err(),
            "a cut Exif IFD"
        );
        assert!(parse(&w.tiff(&[], &[])).is_err(), "magic 42");
        assert!(parse(b"not a tiff").is_err());
    }
}
