//! Read a JPEG file's Exif (APP1) into the `Arw` shape the scan, the index and
//! the meta pane already consume, so a JPEG needs no type of its own. Both TIFF
//! byte orders are read, through the walker `sequence` uses.

use anyhow::{ensure, Result};

use crate::arw::{Arw, Shot};
use crate::sequence::{find_exif_tiff, Tiff};

/// Whether `buf` holds a complete Exif segment, i.e. a prefix of a file this
/// long already parses the same as the whole file.
pub fn has_exif(buf: &[u8]) -> bool {
    find_exif_tiff(buf).is_ok()
}

/// Parse a JPEG file (or a prefix of one) into its orientation and `Shot`.
///
/// Only a missing SOI is an error. A JPEG without an Exif segment, or with a
/// malformed one, is orientation 1 and a default `Shot`, and an entry that
/// cannot be read leaves just its own field `None`. Nothing from a MakerNote
/// is read. `preview` and `full` are `None`: the whole file is both, and
/// `reader` hands it out whole, its Orientation rewritten to 1.
pub fn parse(buf: &[u8]) -> Result<Arw> {
    ensure!(
        buf.starts_with(&[0xFF, 0xD8]),
        "not a JPEG file (missing SOI marker)"
    );
    let (orientation, shot) = read_exif(buf).unwrap_or((1, Shot::default()));
    Ok(Arw {
        preview: None,
        full: None,
        orientation,
        shot,
        hevc: false,
    })
}

/// The orientation and `Shot` of the Exif segment of the JPEG in `buf`. An
/// error when `buf` holds no complete Exif segment (none at all, or one cut
/// off by the end of `buf`); a complete segment whose TIFF is malformed is
/// orientation 1 and a default `Shot`.
pub(crate) fn read_exif(buf: &[u8]) -> Result<(u16, Shot)> {
    let (base, end) = find_exif_tiff(buf)?;
    Ok(Tiff::new(buf, base, end)
        .ok()
        .and_then(|tiff| {
            let ifd0 = tiff.u32(4).ok()?;
            Some(crate::exif::read(&tiff, ifd0 as usize))
        })
        .unwrap_or((1, Shot::default())))
}

/// Rewrite the IFD0 Orientation of the JPEG in `buf` to 1 in place, in the
/// TIFF's own byte order, so a decoder that honors Exif draws it unrotated and
/// `Arw.orientation` stays the only rotation applied. A JPEG without Exif, with
/// a malformed one or without a SHORT Orientation entry is left untouched.
pub fn neutralize_orientation(buf: &mut [u8]) {
    let Some((at, little_endian)) = orientation_at(buf) else {
        return;
    };
    let one = if little_endian {
        1u16.to_le_bytes()
    } else {
        1u16.to_be_bytes()
    };
    buf[at..at + 2].copy_from_slice(&one);
}

/// The absolute offset of the IFD0 Orientation value and the TIFF's byte order.
fn orientation_at(buf: &[u8]) -> Option<(usize, bool)> {
    use crate::exif::{TAG_ORIENTATION, TYPE_SHORT};
    let (base, end) = find_exif_tiff(buf).ok()?;
    let tiff = Tiff::new(buf, base, end).ok()?;
    let ifd0 = tiff.u32(4).ok()? as usize;
    let e = tiff
        .ifd_entries(ifd0)
        .ok()?
        .into_iter()
        .find(|e| e.tag == TAG_ORIENTATION && e.typ == TYPE_SHORT && e.count == 1)?;
    tiff.u16(e.value_field).ok()?;
    Some((base + e.value_field, tiff.little_endian()))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::arw::Rational;
    use crate::exif::*;

    /// A TIFF field: tag, type, count and its value bytes, already in the
    /// file's byte order.
    pub(crate) type Field = (u16, u16, u32, Vec<u8>);

    /// Writes TIFF values in one byte order.
    pub(crate) struct W(pub(crate) bool);

    impl W {
        pub(crate) fn u16(&self, v: u16) -> [u8; 2] {
            if self.0 {
                v.to_le_bytes()
            } else {
                v.to_be_bytes()
            }
        }

        pub(crate) fn u32(&self, v: u32) -> [u8; 4] {
            if self.0 {
                v.to_le_bytes()
            } else {
                v.to_be_bytes()
            }
        }

        pub(crate) fn ascii(&self, tag: u16, s: &str) -> Field {
            let mut v = s.as_bytes().to_vec();
            v.push(0);
            (tag, TYPE_ASCII, v.len() as u32, v)
        }

        pub(crate) fn short(&self, tag: u16, v: u16) -> Field {
            (tag, TYPE_SHORT, 1, self.u16(v).to_vec())
        }

        pub(crate) fn long(&self, tag: u16, v: u32) -> Field {
            (tag, TYPE_LONG, 1, self.u32(v).to_vec())
        }

        pub(crate) fn rational(&self, tag: u16, num: u32, den: u32) -> Field {
            let mut v = self.u32(num).to_vec();
            v.extend_from_slice(&self.u32(den));
            (tag, TYPE_RATIONAL, 1, v)
        }

        pub(crate) fn srational(&self, tag: u16, num: i32, den: i32) -> Field {
            let mut v = self.u32(num as u32).to_vec();
            v.extend_from_slice(&self.u32(den as u32));
            (tag, TYPE_SRATIONAL, 1, v)
        }

        /// A TIFF with `ifd0` plus a pointer to an Exif IFD holding `exif`.
        pub(crate) fn tiff(&self, ifd0: &[Field], exif: &[Field]) -> Vec<u8> {
            let ifd_len = |n: usize| 2 + 12 * n + 4;
            let exif_at = 8 + ifd_len(ifd0.len() + 1);
            let data_at = exif_at + ifd_len(exif.len());
            let mut ifd0 = ifd0.to_vec();
            ifd0.push(self.long(TAG_EXIF_IFD, exif_at as u32));

            let mut out = Vec::new();
            out.extend_from_slice(if self.0 { b"II" } else { b"MM" });
            out.extend_from_slice(&self.u16(42));
            out.extend_from_slice(&self.u32(8));
            let mut data = Vec::new();
            for ifd in [&ifd0[..], exif] {
                out.extend_from_slice(&self.u16(ifd.len() as u16));
                for (tag, typ, count, bytes) in ifd {
                    out.extend_from_slice(&self.u16(*tag));
                    out.extend_from_slice(&self.u16(*typ));
                    out.extend_from_slice(&self.u32(*count));
                    if bytes.len() <= 4 {
                        let mut inline = bytes.clone();
                        inline.resize(4, 0);
                        out.extend_from_slice(&inline);
                    } else {
                        out.extend_from_slice(&self.u32((data_at + data.len()) as u32));
                        data.extend_from_slice(bytes);
                    }
                }
                out.extend_from_slice(&self.u32(0));
            }
            assert_eq!(out.len(), data_at);
            out.extend_from_slice(&data);
            out
        }
    }

    pub(crate) fn plain_jpeg(w: usize, h: usize) -> Vec<u8> {
        let rgb = vec![128u8; w * h * 3];
        let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        c.set_size(w, h);
        c.set_quality(80.0);
        let mut c = c.start_compress(Vec::new()).unwrap();
        c.write_scanlines(&rgb).unwrap();
        c.finish().unwrap()
    }

    /// `jpeg` with an APP1 Exif segment carrying `tiff` spliced in after SOI.
    pub(crate) fn with_exif(jpeg: &[u8], tiff: &[u8]) -> Vec<u8> {
        let mut out = jpeg[..2].to_vec();
        out.extend_from_slice(&[0xFF, 0xE1]);
        out.extend_from_slice(&((2 + 6 + tiff.len()) as u16).to_be_bytes());
        out.extend_from_slice(b"Exif\0\0");
        out.extend_from_slice(tiff);
        out.extend_from_slice(&jpeg[2..]);
        out
    }

    fn full(w: &W, subsec: &str) -> Vec<u8> {
        let tiff = w.tiff(
            &[
                w.ascii(TAG_MAKE, "SONY"),
                w.ascii(TAG_MODEL, "ILCE-7M5"),
                w.short(TAG_ORIENTATION, 6),
            ],
            &[
                w.ascii(TAG_DATE_TIME_ORIGINAL, "2026:09:27 12:34:56"),
                w.ascii(TAG_SUB_SEC_TIME_ORIGINAL, subsec),
                w.ascii(TAG_LENS_MODEL, "FE 50mm F1.2 GM"),
                w.rational(TAG_EXPOSURE_TIME, 1, 250),
                w.rational(TAG_F_NUMBER, 28, 10),
                w.rational(TAG_APERTURE_VALUE, 3, 1),
                w.rational(TAG_FOCAL_LENGTH, 50, 1),
                w.srational(TAG_EXPOSURE_BIAS, -7, 10),
                w.short(TAG_ISO, 800),
            ],
        );
        with_exif(&plain_jpeg(16, 16), &tiff)
    }

    #[test]
    fn reads_every_field_in_both_byte_orders() {
        for le in [true, false] {
            let a = parse(&full(&W(le), "123456")).unwrap();
            let s = &a.shot;
            assert_eq!(a.orientation, 6, "le {le}");
            assert_eq!(s.make.as_deref(), Some("SONY"));
            assert_eq!(s.model.as_deref(), Some("ILCE-7M5"));
            assert_eq!(s.capture_time.as_deref(), Some("2026:09:27 12:34:56"));
            assert_eq!(s.subsec.as_deref(), Some("123456"));
            assert_eq!(s.lens_model.as_deref(), Some("FE 50mm F1.2 GM"));
            assert_eq!(s.exposure_time, Some(Rational { num: 1, den: 250 }));
            assert_eq!(s.f_number, Some(Rational { num: 28, den: 10 }));
            assert_eq!(s.estimated_f_number, None, "cleared by FNumber");
            assert_eq!(s.focal_length, Some(Rational { num: 50, den: 1 }));
            assert_eq!(s.exposure_bias, Some(Rational { num: -7, den: 10 }));
            assert_eq!(s.iso, Some(800));
            assert!(s.focus.is_none() && s.focus_frame.is_none());
            assert!(s.focus_distance_mm.is_none() && s.focus_mode.is_none());
            assert!(s.af_tracking.is_none() && s.creative_style.is_none());
            assert!(a.preview.is_none() && a.full.is_none());
        }
    }

    #[test]
    fn an_inline_subsec_is_read_in_both_byte_orders() {
        for le in [true, false] {
            let a = parse(&full(&W(le), "12")).unwrap();
            assert_eq!(a.shot.subsec.as_deref(), Some("12"), "le {le}");
        }
    }

    #[test]
    fn aperture_value_estimates_the_f_number_without_one() {
        for le in [true, false] {
            let w = W(le);
            let tiff = w.tiff(
                &[],
                &[w.rational(TAG_APERTURE_VALUE, 4, 1), w.long(TAG_ISO, 12800)],
            );
            let a = parse(&with_exif(&plain_jpeg(16, 16), &tiff)).unwrap();
            assert_eq!(a.shot.f_number, None);
            assert_eq!(a.shot.estimated_f_number, Some(4.0));
            assert_eq!(a.shot.iso, Some(12800));
            assert_eq!(a.orientation, 1);
        }
    }

    #[test]
    fn a_jpeg_without_exif_is_the_defaults() {
        let a = parse(&plain_jpeg(16, 16)).unwrap();
        assert_eq!(a.orientation, 1);
        assert!(a.shot.capture_time.is_none() && a.shot.make.is_none());
        assert!(!has_exif(&plain_jpeg(16, 16)));
        assert!(has_exif(&full(&W(true), "1")));
    }

    #[test]
    fn neutralizing_rewrites_only_the_orientation_in_both_byte_orders() {
        for le in [true, false] {
            let w = W(le);
            let tiff = w.tiff(
                &[w.ascii(TAG_MAKE, "SONY"), w.short(TAG_ORIENTATION, 8)],
                &[w.ascii(TAG_DATE_TIME_ORIGINAL, "2026:09:27 12:34:56")],
            );
            let file = with_exif(&plain_jpeg(16, 16), &tiff);
            assert_eq!(parse(&file).unwrap().orientation, 8);
            let mut out = file.clone();
            neutralize_orientation(&mut out);
            let a = parse(&out).unwrap();
            assert_eq!(a.orientation, 1, "le {le}");
            assert_eq!(a.shot.make.as_deref(), Some("SONY"));
            assert_eq!(a.shot.capture_time.as_deref(), Some("2026:09:27 12:34:56"));
            assert_eq!(out.len(), file.len());
            let changed = file.iter().zip(&out).filter(|(a, b)| a != b).count();
            assert_eq!(changed, 1, "le {le}: one byte of the value");
        }
    }

    #[test]
    fn neutralizing_leaves_a_jpeg_without_an_orientation_untouched() {
        let w = W(true);
        let without = with_exif(
            &plain_jpeg(16, 16),
            &w.tiff(&[w.ascii(TAG_MAKE, "SONY")], &[w.short(TAG_ISO, 100)]),
        );
        let long = with_exif(
            &plain_jpeg(16, 16),
            &w.tiff(&[w.long(TAG_ORIENTATION, 8)], &[]),
        );
        let mut cut = full(&w, "1");
        cut.truncate(40);
        for file in [plain_jpeg(16, 16), without, long, cut, Vec::new()] {
            let mut out = file.clone();
            neutralize_orientation(&mut out);
            assert!(out == file);
        }
    }

    #[test]
    fn a_file_that_is_not_a_jpeg_is_an_error() {
        assert!(parse(b"II\x2a\x00not a jpeg").is_err());
        assert!(parse(&[]).is_err());
    }

    #[test]
    fn a_truncated_or_malformed_segment_is_the_defaults() {
        let buf = full(&W(false), "123456");
        for cut in [3, 6, 12, 20, 40, 80, 120, 200] {
            let a = parse(&buf[..cut]).unwrap();
            assert_eq!(a.orientation, 1, "cut at {cut}");
            assert!(a.shot.capture_time.is_none(), "cut at {cut}");
        }

        // An IFD0 offset past the end of the segment.
        let w = W(true);
        let mut tiff = w.tiff(&[w.short(TAG_ORIENTATION, 8)], &[]);
        tiff[4..8].copy_from_slice(&w.u32(0xFFFF));
        let a = parse(&with_exif(&plain_jpeg(16, 16), &tiff)).unwrap();
        assert_eq!(a.orientation, 1);

        // A value pointing out of the segment costs that field only.
        let mut tiff = w.tiff(
            &[w.short(TAG_ORIENTATION, 8)],
            &[
                w.ascii(TAG_DATE_TIME_ORIGINAL, "2026:09:27 12:34:56"),
                w.rational(TAG_EXPOSURE_TIME, 1, 250),
            ],
        );
        let exposure = tiff.len() - 8;
        let pointer = tiff
            .windows(4)
            .position(|b| b == w.u32(exposure as u32))
            .unwrap();
        tiff[pointer..pointer + 4].copy_from_slice(&w.u32(0xFFFF));
        let a = parse(&with_exif(&plain_jpeg(16, 16), &tiff)).unwrap();
        assert_eq!(a.orientation, 8);
        assert_eq!(a.shot.capture_time.as_deref(), Some("2026:09:27 12:34:56"));
        assert_eq!(a.shot.exposure_time, None);
    }
}
