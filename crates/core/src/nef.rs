//! Read a Nikon NEF into the `Arw` shape the scan, the index and the meta pane
//! already consume. A NEF is a plain TIFF, big-endian on older bodies and
//! little-endian on recent ones, read through the walker `sequence` uses.
//!
//! The embedded JPEGs are the SubIFDs with old-style JPEG compression: the
//! first is the full-size JpgFromRaw, and recent bodies add a later ~1620x1080
//! one, which is the preview. IFD0's own JPEG is a 160x120 thumbnail and the
//! MakerNote's `PreviewIFD` JPEG is at most 640 wide, so neither is used.
//!
//! The AF point comes from the MakerNote's `AFInfo2` on the Z bodies.

use anyhow::{anyhow, Result};

use crate::arw::{Arw, Embedded, FocusFrame, FocusLocation, Shot};
use crate::exif::{self, integer};
use crate::sequence::{Entry, Tiff};

const TAG_SUB_IFDS: u16 = 0x014a;
const TAG_COMPRESSION: u16 = 0x0103;
const TAG_JPEG_OFFSET: u16 = 0x0201;
const TAG_JPEG_LENGTH: u16 = 0x0202;
const COMPRESSION_OLD_JPEG: u32 = 6;
const TAG_MAKER_NOTE: u16 = 0x927c;
/// A Nikon MakerNote is `Nikon\0` plus four bytes, then a TIFF whose offsets
/// are relative to its own header.
const NIKON_HEADER: &[u8] = b"Nikon\0";
const NIKON_HEADER_LEN: usize = 10;
const TAG_AF_INFO2: u16 = 0x00b7;
/// `AFInfo2`'s `AFCoordinatesAvailable` byte: 1 when the AF area position is
/// filled in.
const AF_COORDINATES_AVAILABLE: usize = 7;

/// Parse a NEF (or a prefix of one). A structure that points past `buf` is an
/// error, so a prefix too short to hold IFD0, the Exif IFD (when IFD0 points
/// to one) and the SubIFDs is never taken for a file without embedded JPEGs
/// or with fields silently gone missing. The individual Exif fields stay
/// lenient: an unreadable one is just `None`.
pub fn parse(buf: &[u8]) -> Result<Arw> {
    let tiff = Tiff::new(buf, 0, buf.len())?;
    let ifd0_at = tiff.u32(4)? as usize;
    let ifd0 = tiff.ifd_entries(ifd0_at)?;
    let mut shot = Shot::default();
    let (orientation, exif_ifd) = exif::read_ifd0(&tiff, &ifd0, &mut shot);
    if let Some(at) = exif_ifd {
        let entries = tiff.ifd_entries(at)?;
        // The next-IFD link ends the IFD, so the inline values are in `buf` too.
        tiff.u32(at + 2 + 12 * entries.len())?;
        exif::read_exif_ifd(&tiff, &entries, &mut shot);
        if let Some((focus, frame)) = af_point(buf, &tiff, &entries)? {
            shot.focus = Some(focus);
            shot.focus_frame = frame;
        }
    }

    let mut jpegs = Vec::new();
    for at in sub_ifds(&tiff, &ifd0)? {
        let entries = tiff.ifd_entries(at)?;
        // The next-IFD link ends the IFD, so the inline values are in `buf` too.
        tiff.u32(at + 2 + 12 * entries.len())?;
        jpegs.extend(jpeg(&tiff, &entries));
    }
    let full = jpegs.first().copied();
    let preview = if jpegs.len() > 1 {
        jpegs.last().copied()
    } else {
        full
    };

    Ok(Arw {
        preview,
        full,
        orientation,
        shot,
        hevc: false,
    })
}

/// The SubIFD offsets. A single one rides in the entry itself; more are an
/// array at the offset the entry carries.
fn sub_ifds(tiff: &Tiff, ifd0: &[Entry]) -> Result<Vec<usize>> {
    let Some(e) = ifd0.iter().find(|e| e.tag == TAG_SUB_IFDS) else {
        return Ok(Vec::new());
    };
    if e.count == 1 {
        return Ok(vec![tiff.u32(e.value_field)? as usize]);
    }
    let array = tiff.u32(e.value_field)? as usize;
    (0..e.count as usize)
        .map(|i| Ok(tiff.u32(array + i * 4)? as usize))
        .collect()
}

/// The AF area of the MakerNote's `AFInfo2`, as written by the Z bodies
/// (versions `03xx` and `04xx`): its center and size in the `AFImage` frame,
/// which is the unrotated image with the origin at the top left. `None` when
/// the body wrote no position (auto-area before a focus lock, or an older
/// version, whose position is only a grid point name).
fn af_point(
    buf: &[u8],
    tiff: &Tiff,
    exif: &[Entry],
) -> Result<Option<(FocusLocation, Option<FocusFrame>)>> {
    let Some(e) = exif.iter().find(|e| e.tag == TAG_MAKER_NOTE) else {
        return Ok(None);
    };
    let len = e.count as usize;
    if len <= NIKON_HEADER_LEN {
        return Ok(None);
    }
    let Ok(at) = tiff.u32(e.value_field) else {
        return Ok(None);
    };
    let at = at as usize;
    // The MakerNote's declared length running past `buf` (including the
    // header itself, when `buf` is a prefix too short to hold it) is an
    // error, not a note to silently skip: the header can be a Nikon one that
    // is simply cut off, and treating a cut-off note as "no AF point" would
    // let the reader cache that miss under the current `EXTRACTOR_VERSION`
    // instead of retrying with the whole file.
    let end = at
        .checked_add(len)
        .filter(|&end| end <= buf.len())
        .ok_or_else(|| anyhow!("Nikon MakerNote out of range"))?;
    if &buf[at..at + NIKON_HEADER.len()] != NIKON_HEADER {
        return Ok(None);
    }
    let Ok(note) = Tiff::new(buf, at + NIKON_HEADER_LEN, end) else {
        return Ok(None);
    };
    let Ok(entries) = note.u32(4).and_then(|ifd0| note.ifd_entries(ifd0 as usize)) else {
        return Ok(None);
    };
    let Some(e) = entries.iter().find(|e| e.tag == TAG_AF_INFO2) else {
        return Ok(None);
    };
    let Ok(info) = note.u32(e.value_field) else {
        return Ok(None);
    };
    let info = info as usize;
    let Ok(version) = note.bytes(info, 4) else {
        return Ok(None);
    };
    let base = match version {
        b"0300" | b"0301" => 0x2a,
        b"0400" | b"0401" | b"0402" => 0x3e,
        _ => return Ok(None),
    };
    if (e.count as usize) < base + 12 {
        return Ok(None);
    }
    let Ok(available) = note.bytes(info + AF_COORDINATES_AVAILABLE, 1) else {
        return Ok(None);
    };
    if available != [1] {
        return Ok(None);
    }
    let v = |i: usize| note.u16(info + base + 2 * i).ok();
    let (Some(sensor_w), Some(sensor_h), Some(x), Some(y)) = (v(0), v(1), v(2), v(3)) else {
        return Ok(None);
    };
    if sensor_w == 0 || sensor_h == 0 || (x == 0 && y == 0) {
        return Ok(None);
    }
    let (width, height) = (v(4), v(5));
    Ok(Some((
        FocusLocation {
            sensor_w,
            sensor_h,
            x,
            y,
        },
        width
            .zip(height)
            .filter(|&(w, h)| w > 0 && h > 0)
            .map(|(width, height)| FocusFrame { width, height }),
    )))
}

fn jpeg(tiff: &Tiff, entries: &[Entry]) -> Option<Embedded> {
    let int = |tag| {
        entries
            .iter()
            .find(|e| e.tag == tag)
            .and_then(|e| integer(tiff, e))
    };
    if int(TAG_COMPRESSION)? != COMPRESSION_OLD_JPEG {
        return None;
    }
    let length = int(TAG_JPEG_LENGTH)? as usize;
    (length > 0).then_some(Embedded {
        offset: int(TAG_JPEG_OFFSET)? as usize,
        length,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::arw::Rational;
    use crate::exif::*;
    use crate::jpeg::tests::{Field, W};

    const TAG_NEW_SUBFILE_TYPE: u16 = 0x00fe;
    const COMPRESSION_NEF: u16 = 34713;
    const TYPE_UNDEFINED: u16 = 7;

    /// An `AFInfo2` of `version` with the AF image size, the area center and
    /// the area size at `base`, and `AFCoordinatesAvailable` set to `available`.
    pub(crate) fn af_info2(w: &W, version: &[u8; 4], available: u8, values: [u16; 6]) -> Field {
        let base = if version[1] == b'4' { 0x3e } else { 0x2a };
        let mut v = vec![0u8; base + 12 + 8];
        v[..4].copy_from_slice(version);
        v[4] = 2;
        v[AF_COORDINATES_AVAILABLE] = available;
        for (i, x) in values.iter().enumerate() {
            v[base + 2 * i..base + 2 * i + 2].copy_from_slice(&w.u16(*x));
        }
        (TAG_AF_INFO2, TYPE_UNDEFINED, v.len() as u32, v)
    }

    /// A Nikon MakerNote: the header, then a TIFF in `w`'s byte order holding
    /// `fields` in its IFD0.
    pub(crate) fn maker_note(w: &W, fields: &[Field]) -> Field {
        let mut v = NIKON_HEADER.to_vec();
        v.extend_from_slice(&[0x02, 0x10, 0, 0]);
        v.extend_from_slice(&w.tiff(fields, &[]));
        (TAG_MAKER_NOTE, TYPE_UNDEFINED, v.len() as u32, v)
    }

    fn af(file: &W, note: &W, fields: &[Field]) -> Shot {
        let buf = nef(
            file,
            &[file.short(TAG_ORIENTATION, 8)],
            &[maker_note(note, fields)],
            &[jpeg_sub(file, 1000, 900)],
        );
        parse(&buf).unwrap().shot
    }

    /// One IFD whose values all fit in their entries, with no next IFD.
    fn ifd(w: &W, fields: &[Field]) -> Vec<u8> {
        let mut out = w.u16(fields.len() as u16).to_vec();
        for (tag, typ, count, bytes) in fields {
            assert!(bytes.len() <= 4);
            out.extend_from_slice(&w.u16(*tag));
            out.extend_from_slice(&w.u16(*typ));
            out.extend_from_slice(&w.u32(*count));
            let mut inline = bytes.clone();
            inline.resize(4, 0);
            out.extend_from_slice(&inline);
        }
        out.extend_from_slice(&w.u32(0));
        out
    }

    /// A NEF-shaped TIFF: IFD0 plus a SubIFDs entry, the Exif IFD, then the
    /// SubIFDs themselves.
    pub(crate) fn nef(w: &W, ifd0: &[Field], exif: &[Field], subs: &[Vec<Field>]) -> Vec<u8> {
        let build = |at: usize| {
            let mut offsets = Vec::new();
            let mut next = at;
            for sub in subs {
                offsets.extend_from_slice(&w.u32(next as u32));
                next += 2 + 12 * sub.len() + 4;
            }
            let mut fields = ifd0.to_vec();
            fields.push((TAG_SUB_IFDS, TYPE_LONG, subs.len() as u32, offsets));
            w.tiff(&fields, exif)
        };
        let mut out = build(0);
        out = build(out.len());
        for sub in subs {
            out.extend_from_slice(&ifd(w, sub));
        }
        out
    }

    pub(crate) fn jpeg_sub(w: &W, offset: usize, length: usize) -> Vec<Field> {
        vec![
            w.long(TAG_NEW_SUBFILE_TYPE, 1),
            w.short(TAG_COMPRESSION, COMPRESSION_OLD_JPEG as u16),
            w.long(TAG_JPEG_OFFSET, offset as u32),
            w.long(TAG_JPEG_LENGTH, length as u32),
        ]
    }

    fn raw_sub(w: &W) -> Vec<Field> {
        vec![
            w.long(TAG_NEW_SUBFILE_TYPE, 0),
            w.short(TAG_COMPRESSION, COMPRESSION_NEF),
        ]
    }

    fn at(e: Option<Embedded>) -> Option<(usize, usize)> {
        e.map(|e| (e.offset, e.length))
    }

    #[test]
    fn the_first_jpeg_sub_ifd_is_full_and_a_later_one_the_preview() {
        for le in [true, false] {
            let w = W(le);
            let thumbnail = [w.long(TAG_JPEG_OFFSET, 3000), w.long(TAG_JPEG_LENGTH, 100)];
            let subs = [
                jpeg_sub(&w, 1000, 900),
                raw_sub(&w),
                jpeg_sub(&w, 2000, 500),
            ];
            let a = parse(&nef(&w, &thumbnail, &[], &subs)).unwrap();
            assert_eq!(at(a.full), Some((1000, 900)), "le {le}");
            assert_eq!(at(a.preview), Some((2000, 500)), "le {le}");
        }
    }

    #[test]
    fn a_lone_jpeg_sub_ifd_is_both_preview_and_full() {
        for le in [true, false] {
            let w = W(le);
            for subs in [
                vec![jpeg_sub(&w, 1000, 900)],
                vec![jpeg_sub(&w, 1000, 900), raw_sub(&w)],
            ] {
                let a = parse(&nef(&w, &[], &[], &subs)).unwrap();
                assert_eq!(at(a.full), Some((1000, 900)), "le {le}");
                assert_eq!(at(a.preview), Some((1000, 900)), "le {le}");
            }
        }
    }

    #[test]
    fn ifd0_s_thumbnail_is_never_the_preview() {
        let w = W(false);
        let thumbnail = [
            w.short(TAG_COMPRESSION, COMPRESSION_OLD_JPEG as u16),
            w.long(TAG_JPEG_OFFSET, 3000),
            w.long(TAG_JPEG_LENGTH, 100),
        ];
        let a = parse(&nef(&w, &thumbnail, &[], &[raw_sub(&w)])).unwrap();
        assert!(a.preview.is_none() && a.full.is_none());
    }

    #[test]
    fn reads_the_ifd0_and_exif_fields() {
        for le in [true, false] {
            let w = W(le);
            let buf = nef(
                &w,
                &[
                    w.ascii(TAG_MAKE, "NIKON CORPORATION"),
                    w.ascii(TAG_MODEL, "NIKON Z 9"),
                    w.short(TAG_ORIENTATION, 8),
                ],
                &[
                    w.ascii(TAG_DATE_TIME_ORIGINAL, "2022:02:05 13:48:11"),
                    w.ascii(TAG_SUB_SEC_TIME_ORIGINAL, "45"),
                    w.rational(TAG_F_NUMBER, 900, 100),
                    w.short(TAG_ISO, 64),
                ],
                &[jpeg_sub(&w, 1000, 900)],
            );
            let a = parse(&buf).unwrap();
            let s = &a.shot;
            assert_eq!(a.orientation, 8, "le {le}");
            assert_eq!(s.make.as_deref(), Some("NIKON CORPORATION"));
            assert_eq!(s.model.as_deref(), Some("NIKON Z 9"));
            assert_eq!(s.capture_time.as_deref(), Some("2022:02:05 13:48:11"));
            assert_eq!(s.subsec.as_deref(), Some("45"));
            assert_eq!(s.f_number, Some(Rational { num: 900, den: 100 }));
            assert_eq!(s.iso, Some(64));
            assert!(s.focus.is_none() && s.focus_frame.is_none());
        }
    }

    #[test]
    fn reads_the_af_area_of_a_z_body_s_af_info2() {
        for (file, note) in [(true, true), (true, false), (false, false)] {
            let (file, note) = (W(file), W(note));
            for version in [b"0300", b"0301", b"0400", b"0401", b"0402"] {
                let values = [5568, 3712, 3776, 1741, 552, 540];
                let s = af(&file, &note, &[af_info2(&note, version, 1, values)]);
                assert_eq!(
                    s.focus,
                    Some(FocusLocation {
                        sensor_w: 5568,
                        sensor_h: 3712,
                        x: 3776,
                        y: 1741,
                    }),
                    "{version:?} le {}/{}",
                    file.0,
                    note.0
                );
                assert_eq!(
                    s.focus_frame,
                    Some(FocusFrame {
                        width: 552,
                        height: 540,
                    })
                );
            }
        }
    }

    #[test]
    fn an_af_area_without_a_size_has_no_frame() {
        let w = W(true);
        let s = af(
            &w,
            &w,
            &[af_info2(&w, b"0301", 1, [6048, 4024, 3024, 1160, 0, 0])],
        );
        assert!(s.focus.is_some());
        assert!(s.focus_frame.is_none());
    }

    #[test]
    fn an_af_info2_without_a_position_gives_no_af_point() {
        let w = W(true);
        let values = [8256, 5504, 4905, 2461, 291, 323];
        for fields in [
            vec![af_info2(&w, b"0400", 0, values)],
            vec![af_info2(&w, b"0301", 1, [8256, 5504, 0, 0, 291, 323])],
            vec![af_info2(&w, b"0301", 1, [0, 0, 4905, 2461, 291, 323])],
            vec![af_info2(&w, b"0101", 1, values)],
            vec![w.short(TAG_ORIENTATION, 1)],
        ] {
            let s = af(&w, &w, &fields);
            assert!(s.focus.is_none() && s.focus_frame.is_none());
        }

        let mut short = af_info2(&w, b"0301", 1, values);
        short.3.truncate(0x2a + 10);
        short.2 = short.3.len() as u32;
        assert!(af(&w, &w, &[short]).focus.is_none(), "a short AFInfo2");

        let (tag, typ, _, mut v) = maker_note(&w, &[af_info2(&w, b"0301", 1, values)]);
        v[..6].copy_from_slice(b"Other\0");
        let buf = nef(
            &w,
            &[],
            &[(tag, typ, v.len() as u32, v)],
            &[jpeg_sub(&w, 1000, 900)],
        );
        assert!(
            parse(&buf).unwrap().shot.focus.is_none(),
            "not a Nikon note"
        );
    }

    #[test]
    fn a_maker_note_with_a_bad_inner_tiff_header_has_no_af_point() {
        let w = W(true);
        let (tag, typ, count, mut v) = maker_note(&w, &[af_info2(&w, b"0301", 1, values())]);
        // Corrupt the inner TIFF's byte order marker, right after the
        // `Nikon\0` header and its four following bytes, so the note is
        // fully inside `buf` but `Tiff::new` fails to construct it.
        v[NIKON_HEADER_LEN] = b'X';
        v[NIKON_HEADER_LEN + 1] = b'X';
        let buf = nef(&w, &[], &[(tag, typ, count, v)], &[jpeg_sub(&w, 1000, 900)]);
        let a = parse(&buf).unwrap();
        assert!(a.shot.focus.is_none() && a.shot.focus_frame.is_none());
        assert_eq!(at(a.full), Some((1000, 900)));
        assert_eq!(at(a.preview), Some((1000, 900)));
    }

    fn values() -> [u16; 6] {
        [5568, 3712, 3776, 1741, 552, 540]
    }

    #[test]
    fn a_structure_past_the_buffer_is_an_error() {
        let w = W(false);
        let subs = [
            jpeg_sub(&w, 1000, 900),
            raw_sub(&w),
            jpeg_sub(&w, 2000, 500),
        ];
        let buf = nef(&w, &[], &[], &subs);
        assert!(parse(&buf[..buf.len() - 4]).is_err(), "a cut SubIFD");
        assert!(parse(&buf[..20]).is_err(), "a cut IFD0");

        let mut far = buf.clone();
        far[4..8].copy_from_slice(&w.u32(0xFFFF));
        assert!(parse(&far).is_err(), "IFD0 past the end");

        let one = nef(&w, &[], &[], &[jpeg_sub(&w, 1000, 900)]);
        let pointer = one.len() - (2 + 12 * 4 + 4);
        let entry = one
            .windows(4)
            .position(|b| b == w.u32(pointer as u32))
            .unwrap();
        let mut far = one.clone();
        far[entry..entry + 4].copy_from_slice(&w.u32(0xFFFF));
        assert!(parse(&far).is_err(), "a SubIFD past the end");

        let with_exif = nef(
            &w,
            &[],
            &[w.ascii(TAG_DATE_TIME_ORIGINAL, "2022:02:05 13:48:11")],
            &subs,
        );
        let mut prefix = w.u16(TAG_EXIF_IFD).to_vec();
        prefix.extend_from_slice(&w.u16(TYPE_LONG));
        prefix.extend_from_slice(&w.u32(1));
        let exif_entry = with_exif
            .windows(prefix.len())
            .position(|b| b == prefix)
            .unwrap();
        let mut far = with_exif.clone();
        far[exif_entry + 8..exif_entry + 12].copy_from_slice(&w.u32(0xFFFF));
        assert!(parse(&far).is_err(), "the Exif IFD past the end");

        assert!(parse(b"not a tiff").is_err());
    }
}
