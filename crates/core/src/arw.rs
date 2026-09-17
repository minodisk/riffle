//! Minimal parser that locates the embedded JPEGs in an ARW (a TIFF variant).

use anyhow::{anyhow, bail, Result};

const TAG_MAKE: u16 = 0x010f;
const TAG_MODEL: u16 = 0x0110;
const TAG_ORIENTATION: u16 = 0x0112;
const TAG_JPEG_OFFSET: u16 = 0x0201;
const TAG_JPEG_LENGTH: u16 = 0x0202;
const TAG_SUB_IFDS: u16 = 0x014a;
const TAG_EXIF_IFD: u16 = 0x8769;
const TAG_EXPOSURE_TIME: u16 = 0x829a;
const TAG_F_NUMBER: u16 = 0x829d;
const TAG_ISO: u16 = 0x8827;
const TAG_EXPOSURE_BIAS: u16 = 0x9204;
const TAG_FOCAL_LENGTH: u16 = 0x920a;
const TAG_LENS_MODEL: u16 = 0xa434;
const TAG_DATE_TIME_ORIGINAL: u16 = 0x9003;
const TAG_SUB_SEC_TIME_ORIGINAL: u16 = 0x9291;
const TAG_MAKER_NOTE: u16 = 0x927c;
const TAG_FOCUS_LOCATION: u16 = 0x2027;

const TYPE_ASCII: u16 = 2;
const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
const TYPE_RATIONAL: u16 = 5;
const TYPE_SRATIONAL: u16 = 10;

/// Location of one embedded JPEG inside an ARW.
#[derive(Debug, Clone, Copy)]
pub struct Embedded {
    pub offset: usize,
    pub length: usize,
}

/// Sony `FocusLocation`: the focus point in unrotated sensor coordinates,
/// together with the sensor size those coordinates are in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusLocation {
    pub sensor_w: u16,
    pub sensor_h: u16,
    pub x: u16,
    pub y: u16,
}

/// A TIFF RATIONAL or SRATIONAL: numerator over denominator, kept unreduced
/// so the shutter speed can be shown the way the camera wrote it (1/250).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    pub num: i64,
    pub den: i64,
}

impl Rational {
    pub fn value(self) -> Option<f64> {
        (self.den != 0).then(|| self.num as f64 / self.den as f64)
    }
}

/// The shooting settings read out of IFD0 and the ExifIFD. Every field is
/// optional: a body that does not write a tag is not an error.
#[derive(Debug, Default)]
pub struct Shot {
    /// Raw `DateTimeOriginal`, `YYYY:MM:DD HH:MM:SS`.
    pub capture_time: Option<String>,
    /// Raw `SubSecTimeOriginal`.
    pub subsec: Option<String>,
    pub focus: Option<FocusLocation>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_model: Option<String>,
    pub exposure_time: Option<Rational>,
    pub f_number: Option<Rational>,
    pub focal_length: Option<Rational>,
    pub exposure_bias: Option<Rational>,
    pub iso: Option<u32>,
}

#[derive(Debug)]
pub struct Arw {
    /// The small preview in IFD0 (Sony 1616x1080).
    pub preview: Option<Embedded>,
    /// The full-resolution JPEG (JpgFromRaw).
    pub full: Option<Embedded>,
    pub orientation: u16,
    pub shot: Shot,
}

/// One IFD entry: (tag, value-or-offset, type, count).
type Entry = (u16, u32, u16, u32);

fn u16le(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32le(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

/// Read an IFD and return its entries along with the offset of the next IFD.
fn read_ifd(buf: &[u8], at: usize) -> Result<(Vec<Entry>, usize)> {
    if at + 2 > buf.len() {
        bail!("IFD offset out of range");
    }
    let count = u16le(buf, at) as usize;
    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let e = at + 2 + i * 12;
        if e + 12 > buf.len() {
            bail!("IFD entry out of range");
        }
        // (tag, value/offset, type, count)
        entries.push((
            u16le(buf, e),
            u32le(buf, e + 8),
            u16le(buf, e + 2),
            u32le(buf, e + 4),
        ));
    }
    let next = at + 2 + count * 12;
    if next + 4 > buf.len() {
        bail!("next IFD offset out of range");
    }
    let next_ifd = u32le(buf, next) as usize;
    Ok((entries, next_ifd))
}

fn find(entries: &[Entry], tag: u16) -> Option<(u32, u32)> {
    entries.iter().find(|e| e.0 == tag).map(|e| (e.1, e.3))
}

fn embedded(entries: &[Entry]) -> Option<Embedded> {
    let (offset, _) = find(entries, TAG_JPEG_OFFSET)?;
    let (length, _) = find(entries, TAG_JPEG_LENGTH)?;
    Some(Embedded {
        offset: offset as usize,
        length: length as usize,
    })
}

/// Read an ASCII entry's value. Values of up to 4 bytes sit in the entry
/// itself; longer ones live at the offset the entry carries.
fn ascii(buf: &[u8], entry: &Entry) -> Result<Option<String>> {
    let (_, value, typ, count) = *entry;
    if typ != TYPE_ASCII {
        return Ok(None);
    }
    let count = count as usize;
    let inline = value.to_le_bytes();
    let bytes: &[u8] = if count <= 4 {
        &inline[..count]
    } else {
        let at = value as usize;
        let end = at
            .checked_add(count)
            .filter(|&end| end <= buf.len())
            .ok_or_else(|| anyhow!("ASCII value out of range"))?;
        &buf[at..end]
    };
    let text = String::from_utf8_lossy(bytes);
    Ok(Some(text.trim_end_matches('\0').to_string()))
}

/// Read a `SHORT[4]` entry. Eight bytes never fit in an entry, so the value is
/// always at the offset.
fn shorts4(buf: &[u8], entry: &Entry) -> Result<Option<[u16; 4]>> {
    let (_, value, typ, count) = *entry;
    if typ != TYPE_SHORT || count != 4 {
        return Ok(None);
    }
    let at = value as usize;
    if at.checked_add(8).is_none_or(|end| end > buf.len()) {
        bail!("SHORT[4] value out of range");
    }
    let mut out = [0u16; 4];
    for (i, v) in out.iter_mut().enumerate() {
        *v = u16le(buf, at + i * 2);
    }
    Ok(Some(out))
}

/// Find the Sony MakerNote IFD behind tag 0x927c. Recent bodies (Sony5) start
/// the IFD right at the tag's data offset; older ones prefix it with a 12-byte
/// `SONY DSC \0\0\0` / `SONY CAM \0\0\0` header. Either way the value
/// offsets inside it are absolute to the TIFF start, so the IFD can be read
/// straight out of `buf`.
fn maker_note_ifd(buf: &[u8], entries: &[Entry]) -> Result<Option<Vec<Entry>>> {
    let Some(entry) = entries.iter().find(|e| e.0 == TAG_MAKER_NOTE) else {
        return Ok(None);
    };
    let (at, count) = (entry.1 as usize, entry.3 as usize);
    if count <= 4 {
        // Too short to hold an IFD, and such a value is inline anyway.
        return Ok(None);
    }
    let end = at
        .checked_add(count)
        .filter(|&end| end <= buf.len())
        .ok_or_else(|| anyhow!("MakerNote out of range"))?;
    let at = if count >= 14 && buf[at..end].starts_with(b"SONY") {
        at + 12
    } else {
        at
    };
    let (ifd, _) = read_ifd(buf, at)?;
    Ok(Some(ifd))
}

/// Read a RATIONAL/SRATIONAL entry. Eight bytes never fit in an entry, so the
/// value is always at the offset.
fn rational(buf: &[u8], entry: &Entry) -> Result<Option<Rational>> {
    let (_, value, typ, count) = *entry;
    if (typ != TYPE_RATIONAL && typ != TYPE_SRATIONAL) || count != 1 {
        return Ok(None);
    }
    let at = value as usize;
    if at.checked_add(8).is_none_or(|end| end > buf.len()) {
        bail!("RATIONAL value out of range");
    }
    let (num, den) = (u32le(buf, at), u32le(buf, at + 4));
    Ok(Some(if typ == TYPE_SRATIONAL {
        Rational {
            num: i64::from(num as i32),
            den: i64::from(den as i32),
        }
    } else {
        Rational {
            num: i64::from(num),
            den: i64::from(den),
        }
    }))
}

/// Read a single SHORT or LONG entry, whose value always rides inside the
/// entry itself.
fn integer(entry: &Entry) -> Option<u32> {
    let (_, value, typ, count) = *entry;
    ((typ == TYPE_SHORT || typ == TYPE_LONG) && count == 1).then_some(value)
}

/// Follow IFD0 -> ExifIFD (0x8769) -> MakerNote (0x927c) for the shooting
/// settings and the focus point. Any of the three may be absent, which is not
/// an error; an offset pointing outside `buf` is.
fn exif(buf: &[u8], ifd0: &[Entry]) -> Result<Shot> {
    let mut shot = Shot::default();
    for e in ifd0 {
        match e.0 {
            TAG_MAKE => shot.make = ascii(buf, e)?,
            TAG_MODEL => shot.model = ascii(buf, e)?,
            _ => {}
        }
    }

    let Some((at, _)) = find(ifd0, TAG_EXIF_IFD) else {
        return Ok(shot);
    };
    let (exif_ifd, _) = read_ifd(buf, at as usize)?;

    for e in &exif_ifd {
        match e.0 {
            TAG_DATE_TIME_ORIGINAL => shot.capture_time = ascii(buf, e)?,
            TAG_SUB_SEC_TIME_ORIGINAL => shot.subsec = ascii(buf, e)?,
            TAG_LENS_MODEL => shot.lens_model = ascii(buf, e)?,
            TAG_EXPOSURE_TIME => shot.exposure_time = rational(buf, e)?,
            TAG_F_NUMBER => shot.f_number = rational(buf, e)?,
            TAG_FOCAL_LENGTH => shot.focal_length = rational(buf, e)?,
            TAG_EXPOSURE_BIAS => shot.exposure_bias = rational(buf, e)?,
            TAG_ISO => shot.iso = integer(e),
            _ => {}
        }
    }

    shot.focus = match maker_note_ifd(buf, &exif_ifd)? {
        Some(maker) => match maker.iter().find(|e| e.0 == TAG_FOCUS_LOCATION) {
            Some(e) => shorts4(buf, e)?.map(|v| FocusLocation {
                sensor_w: v[0],
                sensor_h: v[1],
                x: v[2],
                y: v[3],
            }),
            None => None,
        },
        None => None,
    };

    Ok(shot)
}

pub fn parse(buf: &[u8]) -> Result<Arw> {
    if buf.len() < 8 || &buf[0..2] != b"II" {
        bail!("not a little-endian TIFF/ARW");
    }
    let (ifd0, next) = read_ifd(buf, u32le(buf, 4) as usize)?;

    let orientation = find(&ifd0, TAG_ORIENTATION)
        .map(|(v, _)| v as u16)
        .unwrap_or(1);
    let preview = embedded(&ifd0);
    let shot = exif(buf, &ifd0)?;

    // Walk the IFD chain (IFD1, IFD2, ...) and the SubIFDs, and take the
    // largest JPEG as the full-resolution one (JpgFromRaw).
    let mut targets: Vec<usize> = Vec::new();
    let mut next = next;
    while next != 0 {
        targets.push(next);
        let (_, n) = read_ifd(buf, next)?;
        next = n;
        if targets.len() > 16 {
            break; // do not spin forever on a broken chain
        }
    }
    if let Some((val, count)) = find(&ifd0, TAG_SUB_IFDS) {
        if count == 1 {
            targets.push(val as usize);
        } else {
            let count = count as usize;
            let end = (val as usize)
                .checked_add(count * 4)
                .filter(|&end| end <= buf.len());
            if end.is_none() {
                bail!("SubIFD offset array out of range");
            }
            targets.extend((0..count).map(|i| u32le(buf, val as usize + i * 4) as usize));
        }
    }

    let mut full: Option<Embedded> = None;
    for off in targets {
        let (ifd, _) = read_ifd(buf, off)?;
        if let Some(e) = embedded(&ifd) {
            if full.is_none_or(|f| e.length > f.length) {
                full = Some(e);
            }
        }
    }

    Ok(Arw {
        preview,
        full,
        orientation,
        shot,
    })
}

impl Arw {
    pub fn slice<'a>(&self, buf: &'a [u8], e: Embedded) -> &'a [u8] {
        &buf[e.offset..e.offset + e.length]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serialise one IFD: entry count, the entries, and a null next-IFD link.
    fn ifd(entries: &[(u16, u16, u32, u32)]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        for (tag, typ, count, value) in entries {
            buf.extend_from_slice(&tag.to_le_bytes());
            buf.extend_from_slice(&typ.to_le_bytes());
            buf.extend_from_slice(&count.to_le_bytes());
            buf.extend_from_slice(&value.to_le_bytes());
        }
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf
    }

    fn ifd_len(entries: usize) -> usize {
        2 + entries * 12 + 4
    }

    /// Build a minimal little-endian TIFF whose IFD0 carries the given entries.
    fn tiff(entries: &[(u16, u16, u32, u32)]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"II\x2a\x00");
        buf.extend_from_slice(&8u32.to_le_bytes());
        buf.extend_from_slice(&ifd(entries));
        buf
    }

    const DATE: &[u8] = b"2026:09:13 09:23:33\0";
    /// Four bytes, so it rides inside the entry instead of at an offset.
    const SUBSEC: &[u8; 4] = b"122\0";

    /// Build a TIFF whose IFD0 points at an ExifIFD holding DateTimeOriginal
    /// and SubSecTimeOriginal, optionally followed by a Sony MakerNote IFD
    /// that optionally holds FocusLocation.
    fn tiff_with_exif(maker: bool, focus: Option<[u16; 4]>, sony_header: bool) -> Vec<u8> {
        let exif_at = 8 + ifd_len(1);
        let exif_entries = if maker { 3 } else { 2 };
        let maker_at = exif_at + ifd_len(exif_entries);
        let header_len = if sony_header { 12 } else { 0 };
        let maker_entries = usize::from(focus.is_some());
        let maker_len = if maker {
            header_len + ifd_len(maker_entries)
        } else {
            0
        };
        let date_at = maker_at + maker_len;
        let focus_at = date_at + DATE.len();

        let mut exif = vec![
            (
                TAG_DATE_TIME_ORIGINAL,
                TYPE_ASCII,
                DATE.len() as u32,
                date_at as u32,
            ),
            (
                TAG_SUB_SEC_TIME_ORIGINAL,
                TYPE_ASCII,
                4,
                u32::from_le_bytes(*SUBSEC),
            ),
        ];
        if maker {
            exif.push((TAG_MAKER_NOTE, 7, maker_len as u32, maker_at as u32));
        }

        let mut buf = tiff(&[(TAG_EXIF_IFD, 4, 1, exif_at as u32)]);
        assert_eq!(buf.len(), exif_at);
        buf.extend_from_slice(&ifd(&exif));
        assert_eq!(buf.len(), maker_at);
        if maker {
            if sony_header {
                buf.extend_from_slice(b"SONY DSC \0\0\0");
            }
            let entries: Vec<_> = focus
                .map(|_| (TAG_FOCUS_LOCATION, TYPE_SHORT, 4, focus_at as u32))
                .into_iter()
                .collect();
            buf.extend_from_slice(&ifd(&entries));
        }
        assert_eq!(buf.len(), date_at);
        buf.extend_from_slice(DATE);
        if let Some(f) = focus {
            for v in f {
                buf.extend_from_slice(&v.to_le_bytes());
            }
        }
        buf
    }

    #[test]
    fn parses_ifd0_preview_and_orientation() {
        let buf = tiff(&[
            (TAG_ORIENTATION, 3, 1, 8),
            (TAG_JPEG_OFFSET, 4, 1, 1024),
            (TAG_JPEG_LENGTH, 4, 1, 2048),
        ]);
        let a = parse(&buf).unwrap();
        assert_eq!(a.orientation, 8);
        let p = a.preview.unwrap();
        assert_eq!((p.offset, p.length), (1024, 2048));
        assert!(a.full.is_none());
    }

    #[test]
    fn defaults_orientation_when_absent() {
        let buf = tiff(&[(TAG_JPEG_OFFSET, 4, 1, 16), (TAG_JPEG_LENGTH, 4, 1, 32)]);
        assert_eq!(parse(&buf).unwrap().orientation, 1);
    }

    #[test]
    fn rejects_big_endian_header() {
        let mut buf = tiff(&[(TAG_ORIENTATION, 3, 1, 1)]);
        buf[0..2].copy_from_slice(b"MM");
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn reads_capture_time_subsec_and_focus_location() {
        let buf = tiff_with_exif(true, Some([7008, 4672, 3613, 1732]), false);
        let a = parse(&buf).unwrap();
        assert_eq!(a.shot.capture_time.as_deref(), Some("2026:09:13 09:23:33"));
        assert_eq!(a.shot.subsec.as_deref(), Some("122"));
        assert_eq!(
            a.shot.focus,
            Some(FocusLocation {
                sensor_w: 7008,
                sensor_h: 4672,
                x: 3613,
                y: 1732,
            })
        );
    }

    #[test]
    fn skips_the_older_sony_maker_note_header() {
        let buf = tiff_with_exif(true, Some([6000, 4000, 100, 200]), true);
        let a = parse(&buf).unwrap();
        assert_eq!(a.shot.focus.unwrap().x, 100);
    }

    #[test]
    fn a_maker_note_without_focus_location_is_none() {
        let a = parse(&tiff_with_exif(true, None, false)).unwrap();
        assert!(a.shot.focus.is_none());
        assert_eq!(a.shot.capture_time.as_deref(), Some("2026:09:13 09:23:33"));
    }

    #[test]
    fn no_maker_note_is_none() {
        let a = parse(&tiff_with_exif(false, None, false)).unwrap();
        assert!(a.shot.focus.is_none());
        assert_eq!(a.shot.subsec.as_deref(), Some("122"));
    }

    #[test]
    fn reads_the_shooting_settings_out_of_the_exif_ifd() {
        let exif_at = 8 + ifd_len(1);
        let rationals_at = exif_at + ifd_len(3);
        let mut buf = tiff(&[(TAG_EXIF_IFD, TYPE_LONG, 1, exif_at as u32)]);
        buf.extend_from_slice(&ifd(&[
            (TAG_EXPOSURE_TIME, TYPE_RATIONAL, 1, rationals_at as u32),
            (TAG_F_NUMBER, TYPE_RATIONAL, 1, (rationals_at + 8) as u32),
            (TAG_ISO, TYPE_SHORT, 1, 6400),
        ]));
        assert_eq!(buf.len(), rationals_at);
        for v in [1u32, 250, 28, 10] {
            buf.extend_from_slice(&v.to_le_bytes());
        }

        let shot = parse(&buf).unwrap().shot;
        assert_eq!(shot.exposure_time, Some(Rational { num: 1, den: 250 }));
        assert_eq!(shot.f_number, Some(Rational { num: 28, den: 10 }));
        assert_eq!(shot.f_number.unwrap().value(), Some(2.8));
        assert_eq!(shot.iso, Some(6400));
        assert_eq!(shot.focal_length, None);
    }

    #[test]
    fn no_exif_ifd_leaves_every_field_none() {
        let a = parse(&tiff(&[(TAG_ORIENTATION, 3, 1, 1)])).unwrap();
        assert!(a.shot.capture_time.is_none() && a.shot.subsec.is_none() && a.shot.focus.is_none());
    }

    /// A prefix too short for what the offsets point at must be an error, not a
    /// silently missing value.
    #[test]
    fn offsets_past_the_buffer_are_errors() {
        assert!(parse(&tiff(&[(TAG_EXIF_IFD, 4, 1, 0xffff)])).is_err());

        let mut buf = tiff_with_exif(true, Some([1, 2, 3, 4]), false);
        buf.truncate(buf.len() - 4);
        assert!(parse(&buf).is_err());

        let mut buf = tiff_with_exif(true, Some([1, 2, 3, 4]), false);
        buf.truncate(buf.len() - DATE.len() - 8);
        assert!(parse(&buf).is_err());
    }
}
