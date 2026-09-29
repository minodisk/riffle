//! Read the standard Exif shooting tags of IFD0 and the Exif IFD into the
//! orientation and a `Shot`, over the byte-order-aware walker `sequence` uses.
//! Shared by the containers whose Exif is a plain TIFF: a JPEG's APP1 segment
//! holds IFD0 and the Exif IFD in one TIFF, while a CR3 keeps them in two, so
//! the IFD0 and Exif IFD parts can also be fed separately.
//!
//! Lenient throughout: an entry that cannot be read leaves just its own field
//! `None`, and nothing is an error. Nothing from a MakerNote is read.

use crate::arw::{Rational, Shot};
use crate::sequence::{Entry, Tiff};

pub(crate) const TAG_MAKE: u16 = 0x010f;
pub(crate) const TAG_MODEL: u16 = 0x0110;
pub(crate) const TAG_ORIENTATION: u16 = 0x0112;
pub(crate) const TAG_EXIF_IFD: u16 = 0x8769;
pub(crate) const TAG_EXPOSURE_TIME: u16 = 0x829a;
pub(crate) const TAG_F_NUMBER: u16 = 0x829d;
pub(crate) const TAG_ISO: u16 = 0x8827;
pub(crate) const TAG_DATE_TIME_ORIGINAL: u16 = 0x9003;
pub(crate) const TAG_APERTURE_VALUE: u16 = 0x9202;
pub(crate) const TAG_EXPOSURE_BIAS: u16 = 0x9204;
pub(crate) const TAG_FOCAL_LENGTH: u16 = 0x920a;
pub(crate) const TAG_SUB_SEC_TIME_ORIGINAL: u16 = 0x9291;
pub(crate) const TAG_LENS_MODEL: u16 = 0xa434;

pub(crate) const TYPE_ASCII: u16 = 2;
pub(crate) const TYPE_SHORT: u16 = 3;
pub(crate) const TYPE_LONG: u16 = 4;
pub(crate) const TYPE_RATIONAL: u16 = 5;
pub(crate) const TYPE_SRATIONAL: u16 = 10;

/// The orientation and `Shot` of the IFD0 at `ifd0` and the Exif IFD it
/// points to. An unreadable IFD0 is orientation 1 and a default `Shot`.
pub(crate) fn read(tiff: &Tiff, ifd0: usize) -> (u16, Shot) {
    let mut shot = Shot::default();
    let Ok(entries) = tiff.ifd_entries(ifd0) else {
        return (1, shot);
    };
    let (orientation, exif_ifd) = read_ifd0(tiff, &entries, &mut shot);
    let entries = exif_ifd
        .and_then(|at| tiff.ifd_entries(at).ok())
        .unwrap_or_default();
    read_exif_ifd(tiff, &entries, &mut shot);
    (orientation, shot)
}

/// Fill `make` and `model` from IFD0 entries, returning the orientation (1
/// when absent) and the Exif IFD offset, if any.
pub(crate) fn read_ifd0(tiff: &Tiff, entries: &[Entry], shot: &mut Shot) -> (u16, Option<usize>) {
    let mut orientation = 1;
    let mut exif_ifd = None;
    for e in entries {
        match e.tag {
            TAG_ORIENTATION => orientation = integer(tiff, e).map_or(1, |v| v as u16),
            TAG_MAKE => shot.make = ascii(tiff, e),
            TAG_MODEL => shot.model = ascii(tiff, e),
            TAG_EXIF_IFD => exif_ifd = tiff.u32(e.value_field).ok().map(|at| at as usize),
            _ => {}
        }
    }
    (orientation, exif_ifd)
}

/// Fill the shooting fields from Exif IFD entries. `estimated_f_number` is
/// cleared when `f_number` is present.
pub(crate) fn read_exif_ifd(tiff: &Tiff, entries: &[Entry], shot: &mut Shot) {
    for e in entries {
        match e.tag {
            TAG_DATE_TIME_ORIGINAL => shot.capture_time = ascii(tiff, e),
            TAG_SUB_SEC_TIME_ORIGINAL => shot.subsec = ascii(tiff, e),
            TAG_LENS_MODEL => shot.lens_model = ascii(tiff, e),
            TAG_EXPOSURE_TIME => shot.exposure_time = rational(tiff, e),
            TAG_F_NUMBER => shot.f_number = rational(tiff, e),
            TAG_APERTURE_VALUE => {
                shot.estimated_f_number = rational(tiff, e)
                    .and_then(Rational::value)
                    .map(|av| 2f64.powf(av / 2.0))
            }
            TAG_FOCAL_LENGTH => shot.focal_length = rational(tiff, e),
            TAG_EXPOSURE_BIAS => shot.exposure_bias = rational(tiff, e),
            TAG_ISO => shot.iso = integer(tiff, e),
            _ => {}
        }
    }
    if shot.f_number.is_some() {
        shot.estimated_f_number = None;
    }
}

/// An ASCII entry's text without its NUL terminator. Values of up to 4 bytes
/// sit in the entry itself; longer ones live at the offset it carries.
pub(crate) fn ascii(tiff: &Tiff, e: &Entry) -> Option<String> {
    if e.typ != TYPE_ASCII {
        return None;
    }
    let count = e.count as usize;
    let bytes = if count <= 4 {
        tiff.bytes(e.value_field, count).ok()?
    } else {
        tiff.bytes(tiff.u32(e.value_field).ok()? as usize, count)
            .ok()?
    };
    Some(
        String::from_utf8_lossy(bytes)
            .trim_end_matches('\0')
            .to_string(),
    )
}

/// A single RATIONAL / SRATIONAL, always at the offset the entry carries.
pub(crate) fn rational(tiff: &Tiff, e: &Entry) -> Option<Rational> {
    if (e.typ != TYPE_RATIONAL && e.typ != TYPE_SRATIONAL) || e.count != 1 {
        return None;
    }
    let at = tiff.u32(e.value_field).ok()? as usize;
    let (num, den) = (tiff.u32(at).ok()?, tiff.u32(at + 4).ok()?);
    Some(if e.typ == TYPE_SRATIONAL {
        Rational {
            num: i64::from(num as i32),
            den: i64::from(den as i32),
        }
    } else {
        Rational {
            num: i64::from(num),
            den: i64::from(den),
        }
    })
}

/// A single SHORT or LONG, which always rides inside the entry itself.
pub(crate) fn integer(tiff: &Tiff, e: &Entry) -> Option<u32> {
    match (e.typ, e.count) {
        (TYPE_SHORT, 1) => tiff.u16(e.value_field).ok().map(u32::from),
        (TYPE_LONG, 1) => tiff.u32(e.value_field).ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jpeg::tests::W;

    #[test]
    fn ifd0_and_exif_ifd_fed_from_two_tiffs() {
        for le in [true, false] {
            let w = W(le);
            let first = w.tiff(
                &[
                    w.ascii(TAG_MAKE, "Canon"),
                    w.ascii(TAG_MODEL, "Canon EOS R5"),
                    w.short(TAG_ORIENTATION, 8),
                ],
                &[],
            );
            let second = w.tiff(
                &[
                    w.ascii(TAG_DATE_TIME_ORIGINAL, "2026:09:29 08:00:00"),
                    w.ascii(TAG_SUB_SEC_TIME_ORIGINAL, "42"),
                    w.rational(TAG_F_NUMBER, 4, 1),
                    w.rational(TAG_APERTURE_VALUE, 4, 1),
                    w.short(TAG_ISO, 400),
                ],
                &[],
            );
            let first = Tiff::new(&first, 0, first.len()).unwrap();
            let second = Tiff::new(&second, 0, second.len()).unwrap();

            let mut shot = Shot::default();
            let ifd0 = first.ifd_entries(8).unwrap();
            let (orientation, _) = read_ifd0(&first, &ifd0, &mut shot);
            let exif_ifd = second.ifd_entries(8).unwrap();
            read_exif_ifd(&second, &exif_ifd, &mut shot);

            assert_eq!(orientation, 8, "le {le}");
            assert_eq!(shot.make.as_deref(), Some("Canon"));
            assert_eq!(shot.model.as_deref(), Some("Canon EOS R5"));
            assert_eq!(shot.capture_time.as_deref(), Some("2026:09:29 08:00:00"));
            assert_eq!(shot.subsec.as_deref(), Some("42"));
            assert_eq!(shot.f_number, Some(Rational { num: 4, den: 1 }));
            assert_eq!(shot.estimated_f_number, None);
            assert_eq!(shot.iso, Some(400));
        }
    }
}
