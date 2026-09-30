//! Read a Canon CR3 into the `Arw` shape the scan, the index and the meta pane
//! already consume. A CR3 is an ISOBMFF file: `ftyp` (brand `crx `), `moov`,
//! a few top-level `uuid` boxes, then `mdat` with the image data.
//!
//! Inside `moov`, Canon's `uuid` box holds `CMT1` (a TIFF whose IFD0 has
//! `Make`, `Model` and `Orientation`), `CMT2` (a TIFF whose IFD0 is the Exif
//! IFD) and `THMB` (a 160x120 thumbnail). Each `trak` is one image in `mdat`;
//! the one whose sample entry carries a `JPEG` box is the full-size JPEG. The
//! top-level preview `uuid` holds `PRVW`, a 1620x1080 JPEG. Bodies shooting
//! HDR PQ (HEIF) store HEVC in `PRVW`, `THMB` and the first track instead,
//! behind a version-1 header: such a `PRVW` / `THMB` is taken whole, marked
//! `Codec::Hevc`, for `hevc::to_jpeg` to decode, and with no JPEG track the
//! HEVC `PRVW` stands in for `full`. Such a file is marked `hevc` when a
//! track's sample entry carries an `HEVC` box and none carries `JPEG`.
//!
//! The AF point comes from `AFInfo2` in `CMT3`, the Canon MakerNote TIFF.

use anyhow::{anyhow, bail, ensure, Result};

use crate::arw::{Arw, Codec, Embedded, FocusFrame, FocusLocation, Shot};
use crate::exif;
use crate::sequence::Tiff;

const CANON_UUID: [u8; 16] = [
    0x85, 0xc0, 0xb6, 0x87, 0x82, 0x0f, 0x11, 0xe0, 0x81, 0x11, 0xf4, 0xce, 0x46, 0x2b, 0x6a, 0x48,
];
const PREVIEW_UUID: [u8; 16] = [
    0xea, 0xf4, 0x2b, 0x5e, 0x1c, 0x98, 0x4b, 0x88, 0xb9, 0xfb, 0xb7, 0xdc, 0x40, 0x6e, 0x4d, 0x16,
];
/// The preview `uuid` holds 8 bytes before its `PRVW` box.
const PREVIEW_UUID_SKIP: usize = 8;
/// `PRVW` and `THMB` hold a 16-byte header before their image data. Its first
/// byte is a version: 0 before a JPEG, whose length is the big-endian u32 at
/// 12 in `PRVW` and at 8 in `THMB`; 1 before HEVC boxes, whose length is the
/// u32 at 12 in both, after the u16 width and height at 6 and 8.
pub(crate) const IMAGE_HEADER_LEN: usize = 16;
const HEVC_IMAGE_VERSION: u8 = 1;
const HEVC_LENGTH_AT: usize = 12;
/// The sub-boxes of a `CRAW` sample entry start 82 bytes into its payload.
const SAMPLE_ENTRY_BOXES: usize = 82;
const SOI: [u8; 2] = [0xFF, 0xD8];
const TAG_AF_INFO2: u16 = 0x0026;
/// `AFInfo2` starts with eight `int16u`: size, AF area mode, the number of AF
/// points, the number of valid ones, the image size and the `AFImage` size.
const AF_INFO2_HEADER: usize = 8;

/// One box: its type, where its payload starts and where it ends (which may
/// lie past the buffer for a top-level box).
#[derive(Clone, Copy)]
pub(crate) struct Bx {
    pub(crate) typ: [u8; 4],
    pub(crate) payload: usize,
    pub(crate) end: usize,
}

/// The box header at `at`, which must lie in `buf`.
fn header(buf: &[u8], at: usize) -> Result<Bx> {
    let short = |n| {
        at.checked_add(n)
            .filter(|&e| e <= buf.len())
            .ok_or_else(|| anyhow!("CR3 box header past the buffer"))
    };
    short(8)?;
    let size = u32_at(buf, at)? as u64;
    let typ = buf[at + 4..at + 8].try_into().unwrap();
    let (size, payload) = if size == 1 {
        short(16)?;
        (u64_at(buf, at + 8)?, at + 16)
    } else {
        (size, at + 8)
    };
    ensure!(size >= (payload - at) as u64, "CR3 box size too small");
    let end = usize::try_from(size)
        .ok()
        .and_then(|s| at.checked_add(s))
        .ok_or_else(|| anyhow!("CR3 box size out of range"))?;
    Ok(Bx { typ, payload, end })
}

/// The boxes from `start` to `end`, all of which must lie in `buf`.
pub(crate) fn children(buf: &[u8], start: usize, end: usize) -> Result<Vec<Bx>> {
    ensure!(end <= buf.len(), "CR3 box past the buffer");
    let mut out = Vec::new();
    let mut at = start;
    while at < end {
        let b = header(buf, at)?;
        ensure!(b.end <= end, "CR3 box past its parent");
        out.push(b);
        at = b.end;
    }
    Ok(out)
}

pub(crate) fn child(boxes: &[Bx], typ: &[u8; 4]) -> Option<Bx> {
    boxes.iter().find(|b| &b.typ == typ).copied()
}

/// The `uuid` box among `boxes` whose payload starts with `uuid`.
fn uuid(buf: &[u8], boxes: &[Bx], uuid: &[u8; 16]) -> Option<Bx> {
    boxes
        .iter()
        .find(|b| &b.typ == b"uuid" && buf.get(b.payload..b.payload + 16) == Some(uuid))
        .copied()
}

fn u32_at(buf: &[u8], at: usize) -> Result<u32> {
    let b = buf
        .get(at..at + 4)
        .ok_or_else(|| anyhow!("CR3 field past the buffer"))?;
    Ok(u32::from_be_bytes(b.try_into().unwrap()))
}

fn u64_at(buf: &[u8], at: usize) -> Result<u64> {
    let b = buf
        .get(at..at + 8)
        .ok_or_else(|| anyhow!("CR3 field past the buffer"))?;
    Ok(u64::from_be_bytes(b.try_into().unwrap()))
}

/// Parse a CR3 (or a prefix of one). `moov` and the `PRVW` header must lie in
/// `buf`, so a prefix cut before them is an error and the reader retries the
/// whole file; the `PRVW` JPEG itself may run past `buf`, for a ranged read.
/// The Exif fields stay lenient: an unreadable one is just `None`.
pub fn parse(buf: &[u8]) -> Result<Arw> {
    let ftyp = header(buf, 0)?;
    ensure!(
        &ftyp.typ == b"ftyp" && buf.get(ftyp.payload..ftyp.payload + 4) == Some(b"crx "),
        "not a CR3"
    );

    let mut moov = None;
    let mut prvw = None;
    let mut complete = false;
    let mut at = ftyp.end;
    while at < buf.len() {
        let b = header(buf, at)?;
        if &b.typ == b"mdat" {
            complete = true;
            break;
        }
        if &b.typ == b"moov" {
            ensure!(b.end <= buf.len(), "CR3 moov past the buffer");
            moov = Some(b);
        } else if uuid(buf, &[b], &PREVIEW_UUID).is_some() {
            prvw = Some(preview(buf, b)?);
            complete = moov.is_some();
        }
        at = b.end;
    }
    ensure!(complete, "CR3 cut before its preview or mdat");
    let moov = moov.ok_or_else(|| anyhow!("CR3 has no moov"))?;

    let boxes = children(buf, moov.payload, moov.end)?;
    let mut shot = Shot::default();
    let mut orientation = 1;
    let mut thumbnail = None;
    if let Some(canon) = uuid(buf, &boxes, &CANON_UUID) {
        let inner = children(buf, canon.payload + 16, canon.end)?;
        if let Some(cmt1) = child(&inner, b"CMT1") {
            let tiff = Tiff::new(buf, cmt1.payload, cmt1.end)?;
            let entries = tiff.ifd_entries(tiff.u32(4)? as usize)?;
            orientation = exif::read_ifd0(&tiff, &entries, &mut shot).0;
        }
        if let Some(cmt2) = child(&inner, b"CMT2") {
            let tiff = Tiff::new(buf, cmt2.payload, cmt2.end)?;
            let entries = tiff.ifd_entries(tiff.u32(4)? as usize)?;
            exif::read_exif_ifd(&tiff, &entries, &mut shot);
        }
        if let Some(cmt3) = child(&inner, b"CMT3") {
            if let Some((focus, frame)) = af_point(buf, cmt3, shot.model.as_deref()) {
                shot.focus = Some(focus);
                shot.focus_frame = Some(frame);
            }
        }
        if let Some(thmb) = child(&inner, b"THMB") {
            thumbnail = image(buf, thmb, 8)?;
        }
    }

    let mut full = None;
    let mut hevc = false;
    for trak in boxes.iter().filter(|b| &b.typ == b"trak") {
        let Some((subs, stbl)) = sample_entry(buf, *trak)? else {
            continue;
        };
        if child(&subs, b"JPEG").is_some() {
            full = Some(jpeg_track(buf, &stbl)?);
            break;
        }
        hevc |= child(&subs, b"HEVC").is_some();
    }

    let hevc = hevc && full.is_none();
    let prvw = prvw.flatten();
    let full = full.or(prvw.filter(|p| p.codec == Codec::Hevc));
    Ok(Arw {
        preview: prvw.or(thumbnail).or(full),
        full,
        orientation,
        shot,
        hevc,
    })
}

/// The AF area of `AFInfo2` in the MakerNote box `cmt3`: the bounding box of
/// the AF points in focus, else of the one selected point, converted from
/// `AFInfo2`'s center-origin, Y-up coordinates to the top-left origin of the
/// `AFImage` frame (the unrotated image). Only for EOS bodies: ExifTool notes
/// the Y axis points down on PowerShots. `None` when no point is in focus and
/// the selection is not a single point (manual focus, or an automatic area
/// that never locked).
fn af_point(buf: &[u8], cmt3: Bx, model: Option<&str>) -> Option<(FocusLocation, FocusFrame)> {
    if !model?.contains("EOS") {
        return None;
    }
    let tiff = Tiff::new(buf, cmt3.payload, cmt3.end).ok()?;
    let entries = tiff.ifd_entries(tiff.u32(4).ok()? as usize).ok()?;
    let e = entries.iter().find(|e| e.tag == TAG_AF_INFO2)?;
    let count = e.count as usize;
    if e.typ != exif::TYPE_SHORT || count < AF_INFO2_HEADER {
        return None;
    }
    let at = tiff.u32(e.value_field).ok()? as usize;
    let v = |i: usize| tiff.u16(at + 2 * i).ok();
    let n = v(2)? as usize;
    let masks = n.div_ceil(16);
    if count < AF_INFO2_HEADER + 4 * n + 2 * masks {
        return None;
    }
    let (sensor_w, sensor_h) = (v(6)?, v(7)?);
    let valid = n.min(v(3)? as usize);
    let bit = |mask: usize, i: usize| {
        v(AF_INFO2_HEADER + 4 * n + mask * masks + i / 16).map(|m| m >> (i % 16) & 1 == 1)
    };
    let indices = |mask: usize| -> Option<Vec<usize>> {
        let mut out = Vec::new();
        for i in 0..valid {
            if bit(mask, i)? {
                out.push(i);
            }
        }
        Some(out)
    };
    let mut points = indices(0)?;
    if points.is_empty() {
        points = indices(1)?;
        if points.len() != 1 {
            return None;
        }
    }
    if sensor_w == 0 || sensor_h == 0 {
        return None;
    }
    let s =
        |array: usize, i: usize| v(AF_INFO2_HEADER + array * n + i).map(|x| i32::from(x as i16));
    let (mut left, mut right, mut top, mut bottom) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
    for &i in &points {
        let (w, h, x, y) = (s(0, i)?, s(1, i)?, s(2, i)?, s(3, i)?);
        left = left.min(x - w / 2);
        right = right.max(x + w / 2);
        top = top.min(-y - h / 2);
        bottom = bottom.max(-y + h / 2);
    }
    let coord = |c: i32, size: u16| (i32::from(size) / 2 + c).clamp(0, i32::from(size) - 1) as u16;
    Some((
        FocusLocation {
            sensor_w,
            sensor_h,
            x: coord((left + right) / 2, sensor_w),
            y: coord((top + bottom) / 2, sensor_h),
        },
        FocusFrame {
            width: (right - left).clamp(0, i32::from(u16::MAX)) as u16,
            height: (bottom - top).clamp(0, i32::from(u16::MAX)) as u16,
        },
    ))
}

/// The `PRVW` image of the preview `uuid` box `b`, whose header must lie in
/// `buf`; `None` when the preview is neither a JPEG nor HEVC.
fn preview(buf: &[u8], b: Bx) -> Result<Option<Embedded>> {
    let prvw = header(buf, b.payload + 16 + PREVIEW_UUID_SKIP)?;
    ensure!(
        &prvw.typ == b"PRVW" && prvw.end <= b.end,
        "CR3 preview box malformed"
    );
    image(buf, prvw, 12)
}

/// The image of `PRVW` / `THMB` box `b`: the whole payload, header included,
/// behind a version-1 header, else the data after the header when it is a
/// JPEG whose length is the u32 at `jpeg_length_at`. The header (and a JPEG's
/// SOI) must lie in `buf`; the rest may not.
fn image(buf: &[u8], b: Bx, jpeg_length_at: usize) -> Result<Option<Embedded>> {
    let offset = b.payload + IMAGE_HEADER_LEN;
    if buf.get(b.payload) == Some(&HEVC_IMAGE_VERSION) {
        let length = u32_at(buf, b.payload + HEVC_LENGTH_AT)? as usize;
        ensure!(
            offset.checked_add(length).is_some_and(|e| e <= b.end),
            "CR3 image past its box"
        );
        return Ok(Some(Embedded {
            offset: b.payload,
            length: IMAGE_HEADER_LEN + length,
            codec: Codec::Hevc,
        }));
    }
    let length = u32_at(buf, b.payload + jpeg_length_at)? as usize;
    let soi = buf
        .get(offset..offset + 2)
        .ok_or_else(|| anyhow!("CR3 image header past the buffer"))?;
    if soi != SOI {
        return Ok(None);
    }
    ensure!(
        offset.checked_add(length).is_some_and(|e| e <= b.end),
        "CR3 image past its box"
    );
    Ok(Some(Embedded {
        offset,
        length,
        codec: Codec::Jpeg,
    }))
}

/// The sub-boxes of `trak`'s first sample entry and the boxes of its `stbl`.
fn sample_entry(buf: &[u8], trak: Bx) -> Result<Option<(Vec<Bx>, Vec<Bx>)>> {
    let path = [b"mdia", b"minf", b"stbl"];
    let mut boxes = children(buf, trak.payload, trak.end)?;
    for typ in path {
        let Some(b) = child(&boxes, typ) else {
            return Ok(None);
        };
        boxes = children(buf, b.payload, b.end)?;
    }
    let Some(stsd) = child(&boxes, b"stsd") else {
        return Ok(None);
    };
    // Version and flags, then the entry count, then the first sample entry.
    let entry = header(buf, stsd.payload + 8)?;
    ensure!(entry.end <= stsd.end, "CR3 sample entry past its box");
    let subs = children(buf, entry.payload + SAMPLE_ENTRY_BOXES, entry.end)?;
    Ok(Some((subs, boxes)))
}

/// The JPEG in `mdat` that a JPEG track with the `stbl` boxes `boxes`
/// describes.
fn jpeg_track(buf: &[u8], boxes: &[Bx]) -> Result<Embedded> {
    let stsz = child(boxes, b"stsz").ok_or_else(|| anyhow!("CR3 JPEG track without stsz"))?;
    let mut length = u32_at(buf, stsz.payload + 4)? as usize;
    if length == 0 {
        ensure!(u32_at(buf, stsz.payload + 8)? > 0, "CR3 JPEG track empty");
        length = u32_at(buf, stsz.payload + 12)? as usize;
    }
    let offset = if let Some(co64) = child(boxes, b"co64") {
        ensure!(u32_at(buf, co64.payload + 4)? > 0, "CR3 JPEG track empty");
        usize::try_from(u64_at(buf, co64.payload + 8)?)?
    } else if let Some(stco) = child(boxes, b"stco") {
        ensure!(u32_at(buf, stco.payload + 4)? > 0, "CR3 JPEG track empty");
        u32_at(buf, stco.payload + 8)? as usize
    } else {
        bail!("CR3 JPEG track without an offset");
    };
    Ok(Embedded {
        offset,
        length,
        codec: Codec::Jpeg,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::arw::Rational;
    use crate::exif::*;
    use crate::jpeg::tests::{Field, W};

    fn bx(typ: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut out = ((8 + payload.len()) as u32).to_be_bytes().to_vec();
        out.extend_from_slice(typ);
        out.extend_from_slice(payload);
        out
    }

    fn cat(parts: &[Vec<u8>]) -> Vec<u8> {
        parts.concat()
    }

    fn uuid_box(uuid: &[u8; 16], payload: &[u8]) -> Vec<u8> {
        bx(b"uuid", &cat(&[uuid.to_vec(), payload.to_vec()]))
    }

    /// A `PRVW` / `THMB` box: a 16-byte header with the data length at
    /// `length_at`, then the data.
    fn image_box(typ: &[u8; 4], length_at: usize, data: &[u8]) -> Vec<u8> {
        let mut header = [0u8; IMAGE_HEADER_LEN];
        header[length_at..length_at + 4].copy_from_slice(&(data.len() as u32).to_be_bytes());
        bx(typ, &cat(&[header.to_vec(), data.to_vec()]))
    }

    /// A version-1 `PRVW` / `THMB` box of `width` x `height`, as HDR PQ files
    /// write before their HEVC boxes: the data length at 12, then the data.
    fn hevc_box(typ: &[u8; 4], width: u16, height: u16, data: &[u8]) -> Vec<u8> {
        let mut header = [0u8; IMAGE_HEADER_LEN];
        header[0] = HEVC_IMAGE_VERSION;
        header[5] = 2;
        header[6..8].copy_from_slice(&width.to_be_bytes());
        header[8..10].copy_from_slice(&height.to_be_bytes());
        header[10..12].copy_from_slice(&[0xff, 0xff]);
        header[12..16].copy_from_slice(&(data.len() as u32).to_be_bytes());
        bx(typ, &cat(&[header.to_vec(), data.to_vec()]))
    }

    /// A `trak` whose sample entry carries `sub` (`JPEG` or `CMP1`) and whose
    /// one sample is `length` bytes at `offset`, in a `co64` or an `stco`.
    fn trak(sub: &[u8; 4], offset: u64, length: u32, wide: bool) -> Vec<u8> {
        let mut entry = vec![0u8; SAMPLE_ENTRY_BOXES];
        entry.extend_from_slice(&bx(sub, &[]));
        let stsd = bx(
            b"stsd",
            &cat(&[vec![0; 4], 1u32.to_be_bytes().to_vec(), bx(b"CRAW", &entry)]),
        );
        let stsz = bx(
            b"stsz",
            &cat(&[
                vec![0; 4],
                length.to_be_bytes().to_vec(),
                1u32.to_be_bytes().to_vec(),
            ]),
        );
        let chunk = if wide {
            bx(
                b"co64",
                &cat(&[
                    vec![0; 4],
                    1u32.to_be_bytes().to_vec(),
                    offset.to_be_bytes().to_vec(),
                ]),
            )
        } else {
            bx(
                b"stco",
                &cat(&[
                    vec![0; 4],
                    1u32.to_be_bytes().to_vec(),
                    (offset as u32).to_be_bytes().to_vec(),
                ]),
            )
        };
        let stbl = bx(b"stbl", &cat(&[stsd, stsz, chunk]));
        bx(b"trak", &bx(b"mdia", &bx(b"minf", &stbl)))
    }

    pub(crate) struct Cr3 {
        pub(crate) ifd0: Vec<Field>,
        pub(crate) exif: Vec<Field>,
        /// The IFD0 of `CMT3`, the Canon MakerNote.
        pub(crate) makernote: Vec<Field>,
        pub(crate) thumbnail: Option<Vec<u8>>,
        pub(crate) preview: Option<Vec<u8>>,
        pub(crate) full: Vec<u8>,
        /// The sample entry sub-box of the track holding `full`: `JPEG`, or
        /// `HEVC` as on a file shot with HDR PQ on.
        pub(crate) full_sub: [u8; 4],
        /// `thumbnail` and `preview` sit behind version-1 headers, as the
        /// HEVC of a file shot with HDR PQ on.
        pub(crate) hevc_images: bool,
        /// The payload size of the `free` box before `moov`.
        pub(crate) pad: usize,
    }

    pub(crate) fn jpeg_bytes(n: usize, fill: u8) -> Vec<u8> {
        let mut v = vec![fill; n];
        v[..2].copy_from_slice(&SOI);
        v
    }

    impl Cr3 {
        pub(crate) fn new() -> Self {
            let w = W(true);
            Cr3 {
                ifd0: vec![w.short(TAG_ORIENTATION, 1)],
                exif: Vec::new(),
                makernote: Vec::new(),
                thumbnail: Some(jpeg_bytes(16, 1)),
                preview: Some(jpeg_bytes(64, 2)),
                full: jpeg_bytes(256, 3),
                full_sub: *b"JPEG",
                hevc_images: false,
                pad: 0,
            }
        }

        /// The file bytes, with the full JPEG in `mdat`.
        pub(crate) fn build(&self) -> Vec<u8> {
            let w = W(true);
            let make = |full_at: u64| {
                let mut canon = cat(&[
                    CANON_UUID.to_vec(),
                    bx(b"CMT1", &w.tiff(&self.ifd0, &[])),
                    bx(b"CMT2", &w.tiff(&self.exif, &[])),
                    bx(b"CMT3", &w.tiff(&self.makernote, &[])),
                ]);
                if let Some(t) = &self.thumbnail {
                    canon.extend_from_slice(&if self.hevc_images {
                        hevc_box(b"THMB", 320, 214, t)
                    } else {
                        image_box(b"THMB", 8, t)
                    });
                }
                let moov = bx(
                    b"moov",
                    &cat(&[
                        bx(b"uuid", &canon),
                        trak(b"CMP1", 0, 16, false),
                        trak(&self.full_sub, full_at, self.full.len() as u32, true),
                        trak(b"CMP1", 0, 16, true),
                    ]),
                );
                // Largesize headers on `free` and `mdat`, as real files give
                // `mdat`.
                let mut out = cat(&[
                    bx(b"ftyp", b"crx \0\0\0\x01crx isom"),
                    1u32.to_be_bytes().to_vec(),
                    b"free".to_vec(),
                    (16 + self.pad as u64).to_be_bytes().to_vec(),
                    vec![0; self.pad],
                    moov,
                ]);
                if let Some(p) = &self.preview {
                    let prvw = if self.hevc_images {
                        hevc_box(b"PRVW", 1620, 1080, p)
                    } else {
                        image_box(b"PRVW", 12, p)
                    };
                    let payload = cat(&[vec![0; PREVIEW_UUID_SKIP], prvw]);
                    out.extend_from_slice(&uuid_box(&PREVIEW_UUID, &payload));
                }
                out
            };
            let head = make(0);
            let full_at = (head.len() + 16) as u64;
            let mut out = make(full_at);
            out.extend_from_slice(&1u32.to_be_bytes());
            out.extend_from_slice(b"mdat");
            out.extend_from_slice(&(16 + self.full.len() as u64).to_be_bytes());
            out.extend_from_slice(&self.full);
            out
        }
    }

    fn at(e: Option<Embedded>) -> Option<(usize, usize)> {
        e.map(|e| (e.offset, e.length))
    }

    fn find(buf: &[u8], data: &[u8]) -> usize {
        buf.windows(data.len()).position(|w| w == data).unwrap()
    }

    #[test]
    fn the_preview_is_prvw_and_full_the_jpeg_track() {
        let c = Cr3::new();
        let buf = c.build();
        let a = parse(&buf).unwrap();
        let (p, f) = (c.preview.as_ref().unwrap(), &c.full);
        assert_eq!(at(a.preview), Some((find(&buf, p), p.len())));
        assert_eq!(at(a.full), Some((find(&buf, f), f.len())));
        assert_eq!(a.slice(&buf, a.full.unwrap()), &f[..]);
    }

    #[test]
    fn without_a_jpeg_prvw_the_preview_is_thmb_then_full() {
        let mut c = Cr3::new();
        c.preview = Some(vec![0x43; 64]);
        let buf = c.build();
        let t = c.thumbnail.as_ref().unwrap();
        assert_eq!(
            at(parse(&buf).unwrap().preview),
            Some((find(&buf, t), t.len()))
        );

        c.preview = None;
        let buf = c.build();
        assert_eq!(
            at(parse(&buf).unwrap().preview),
            Some((find(&buf, t), t.len()))
        );

        c.thumbnail = Some(vec![0x43; 16]);
        let buf = c.build();
        let a = parse(&buf).unwrap();
        assert_eq!(at(a.preview), at(a.full));
        assert!(a.full.is_some());
    }

    #[test]
    fn a_track_that_is_not_a_jpeg_is_never_full() {
        let buf = cat(&[
            bx(b"ftyp", b"crx \0\0\0\x01"),
            bx(
                b"moov",
                &cat(&[trak(b"CMP1", 100, 16, true), trak(b"HEVC", 100, 16, false)]),
            ),
            bx(b"mdat", &[]),
        ]);
        let a = parse(&buf).unwrap();
        assert!(a.full.is_none() && a.preview.is_none());
        assert!(a.hevc);

        let buf = cat(&[
            bx(b"ftyp", b"crx \0\0\0\x01"),
            bx(
                b"moov",
                &cat(&[trak(b"CMP1", 100, 16, true), trak(b"CMP1", 100, 16, false)]),
            ),
            bx(b"mdat", &[]),
        ]);
        assert!(!parse(&buf).unwrap().hevc, "only CMP1 tracks");
    }

    #[test]
    fn an_hdr_pq_file_is_marked_hevc_and_keeps_its_metadata() {
        let w = W(true);
        let c = Cr3 {
            ifd0: vec![
                w.ascii(TAG_MODEL, "Canon EOS R8"),
                w.short(TAG_ORIENTATION, 6),
            ],
            exif: vec![w.ascii(TAG_DATE_TIME_ORIGINAL, "2026:09:30 10:00:00")],
            thumbnail: Some(vec![0; 16]),
            preview: Some(vec![0; 64]),
            full: vec![0; 256],
            full_sub: *b"HEVC",
            ..Cr3::new()
        };
        let a = parse(&c.build()).unwrap();
        assert!(a.hevc);
        assert!(a.preview.is_none() && a.full.is_none());
        assert_eq!(a.orientation, 6);
        assert_eq!(a.shot.model.as_deref(), Some("Canon EOS R8"));
        assert_eq!(a.shot.capture_time.as_deref(), Some("2026:09:30 10:00:00"));

        assert!(!parse(&Cr3::new().build()).unwrap().hevc, "a normal file");
    }

    /// An HDR PQ file whose `THMB` and `PRVW` hold `thumbnail` and `preview`
    /// behind version-1 headers.
    pub(crate) fn hdr_pq(thumbnail: Option<Vec<u8>>, preview: Option<Vec<u8>>) -> Cr3 {
        Cr3 {
            thumbnail,
            preview,
            full: vec![0; 256],
            full_sub: *b"HEVC",
            hevc_images: true,
            ..Cr3::new()
        }
    }

    /// The range of an HEVC image: its box payload, header included.
    fn hevc_at(buf: &[u8], data: &[u8]) -> Option<(usize, usize)> {
        let offset = find(buf, data) - IMAGE_HEADER_LEN;
        Some((offset, IMAGE_HEADER_LEN + data.len()))
    }

    #[test]
    fn an_hevc_prvw_is_the_preview_and_the_full_image() {
        let (t, p) = (vec![0x11; 24], vec![0x22; 96]);
        let buf = hdr_pq(Some(t.clone()), Some(p.clone())).build();
        let a = parse(&buf).unwrap();
        assert!(a.hevc);
        assert_eq!(at(a.preview), hevc_at(&buf, &p));
        assert_eq!(at(a.full), hevc_at(&buf, &p));
        for e in [a.preview, a.full] {
            assert_eq!(e.unwrap().codec, Codec::Hevc);
        }
        let payload = a.slice(&buf, a.preview.unwrap());
        assert_eq!(&payload[..4], &[1, 0, 0, 0]);
        assert_eq!(&payload[6..10], &[0x06, 0x54, 0x04, 0x38], "1620x1080");

        // Its length is the u32 at 12, as in `PRVW`; the one at 8 would run
        // past the box.
        let buf = hdr_pq(Some(t.clone()), None).build();
        let a = parse(&buf).unwrap();
        assert_eq!(at(a.preview), hevc_at(&buf, &t), "the THMB stands in");
        assert!(a.full.is_none(), "a THMB is never full");

        let c = Cr3 {
            full: jpeg_bytes(256, 3),
            full_sub: *b"JPEG",
            ..hdr_pq(None, Some(p.clone()))
        };
        let buf = c.build();
        let a = parse(&buf).unwrap();
        assert_eq!(at(a.preview), hevc_at(&buf, &p));
        assert_eq!(
            a.full.unwrap().codec,
            Codec::Jpeg,
            "a JPEG track stays full"
        );
        assert!(!a.hevc);
    }

    #[test]
    fn an_hevc_image_past_its_box_is_an_error_and_its_header_is_enough() {
        let p = vec![0x22; 4096];
        let buf = hdr_pq(None, Some(p.clone())).build();
        let data_at = find(&buf, &p);
        let a = parse(&buf[..data_at]).unwrap();
        assert_eq!(at(a.preview), hevc_at(&buf, &p));
        assert!(parse(&buf[..data_at - 4]).is_err(), "cut inside the header");

        let mut long = buf.clone();
        long[data_at - 4..data_at].copy_from_slice(&0xFFFFu32.to_be_bytes());
        assert!(parse(&long).is_err(), "the HEVC data past its box");
    }

    #[test]
    fn reads_orientation_and_the_exif_fields_from_cmt1_and_cmt2() {
        let w = W(true);
        let mut c = Cr3::new();
        c.ifd0 = vec![
            w.ascii(TAG_MAKE, "Canon"),
            w.ascii(TAG_MODEL, "Canon EOS R5"),
            w.short(TAG_ORIENTATION, 8),
        ];
        c.exif = vec![
            w.ascii(TAG_DATE_TIME_ORIGINAL, "2020:08:03 13:48:11"),
            w.ascii(TAG_SUB_SEC_TIME_ORIGINAL, "37"),
            w.rational(TAG_F_NUMBER, 28, 10),
            w.short(TAG_ISO, 100),
        ];
        let a = parse(&c.build()).unwrap();
        let s = &a.shot;
        assert_eq!(a.orientation, 8);
        assert_eq!(s.make.as_deref(), Some("Canon"));
        assert_eq!(s.model.as_deref(), Some("Canon EOS R5"));
        assert_eq!(s.capture_time.as_deref(), Some("2020:08:03 13:48:11"));
        assert_eq!(s.subsec.as_deref(), Some("37"));
        assert_eq!(s.f_number, Some(Rational { num: 28, den: 10 }));
        assert_eq!(s.iso, Some(100));
        assert!(s.focus.is_none() && s.focus_frame.is_none());
    }

    /// One AF point of `AFInfo2`: width, height, and the center relative to
    /// the image center with Y up.
    type Point = (i16, i16, i16, i16);

    /// An `AFInfo2` of `n` points on a 6000x4000 `AFImage`, `points` filling
    /// the first ones, with the points at `in_focus` and `selected` flagged.
    fn af_info2(n: usize, points: &[Point], in_focus: &[usize], selected: &[usize]) -> Field {
        let masks = n.div_ceil(16);
        let mut v = vec![0u16; AF_INFO2_HEADER + 4 * n + 2 * masks];
        v[..AF_INFO2_HEADER].copy_from_slice(&[
            0,
            9,
            n as u16,
            points.len() as u16,
            6000,
            4000,
            6000,
            4000,
        ]);
        v[0] = (v.len() * 2) as u16;
        for (i, (w, h, x, y)) in points.iter().enumerate() {
            for (array, value) in [w, h, x, y].into_iter().enumerate() {
                v[AF_INFO2_HEADER + array * n + i] = *value as u16;
            }
        }
        for (mask, flagged) in [in_focus, selected].into_iter().enumerate() {
            for i in flagged {
                v[AF_INFO2_HEADER + 4 * n + mask * masks + i / 16] |= 1 << (i % 16);
            }
        }
        let bytes = v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
        (TAG_AF_INFO2, TYPE_SHORT, v.len() as u32, bytes)
    }

    fn af(model: &str, orientation: u16, info: Field) -> Shot {
        let w = W(true);
        let mut c = Cr3::new();
        c.ifd0 = vec![
            w.ascii(TAG_MODEL, model),
            w.short(TAG_ORIENTATION, orientation),
        ];
        c.makernote = vec![info];
        parse(&c.build()).unwrap().shot
    }

    fn focus(x: u16, y: u16) -> Option<FocusLocation> {
        Some(FocusLocation {
            sensor_w: 6000,
            sensor_h: 4000,
            x,
            y,
        })
    }

    #[test]
    fn the_af_point_in_focus_is_taken_with_y_up_from_the_image_center() {
        let s = af(
            "Canon EOS R10",
            1,
            af_info2(651, &[(224, 224, -470, 30)], &[0], &[0]),
        );
        assert_eq!(s.focus, focus(2530, 1970));
        assert_eq!(
            s.focus_frame,
            Some(FocusFrame {
                width: 224,
                height: 224,
            })
        );

        let s = af(
            "Canon EOS R10",
            8,
            af_info2(651, &[(1790, 1793, 749, 640)], &[0], &[0]),
        );
        assert_eq!(
            s.focus,
            focus(3749, 1360),
            "a portrait frame stays unrotated"
        );
        assert_eq!(
            s.focus_frame,
            Some(FocusFrame {
                width: 1790,
                height: 1792,
            })
        );
    }

    #[test]
    fn several_points_in_focus_give_their_bounding_box() {
        let points = [
            (100, 100, -1000, 500),
            (100, 100, 0, 0),
            (100, 100, 1000, -300),
            (100, 100, 2000, 1500),
        ];
        let s = af(
            "Canon EOS R6m2",
            1,
            af_info2(143, &points, &[0, 2, 17], &[0, 1, 2, 3]),
        );
        assert_eq!(s.focus, focus(3000, 1900));
        assert_eq!(
            s.focus_frame,
            Some(FocusFrame {
                width: 2100,
                height: 900,
            })
        );
    }

    #[test]
    fn a_lone_selected_point_stands_in_when_none_is_in_focus() {
        let s = af(
            "Canon EOS RP",
            1,
            af_info2(143, &[(480, 480, 1302, -173)], &[], &[0]),
        );
        assert_eq!(s.focus, focus(4302, 2173));

        let points = [(100, 100, -1000, 500), (100, 100, 1000, -300)];
        let s = af("Canon EOS R100", 1, af_info2(143, &points, &[], &[0, 1]));
        assert!(
            s.focus.is_none() && s.focus_frame.is_none(),
            "an area that never locked"
        );
    }

    #[test]
    fn no_af_point_without_a_valid_one_or_on_a_powershot() {
        let s = af("Canon EOS R6", 1, af_info2(1053, &[], &[], &[]));
        assert!(s.focus.is_none(), "manual focus");

        let s = af(
            "Canon PowerShot G7 X Mark III",
            1,
            af_info2(9, &[(100, 100, 10, 10)], &[0], &[0]),
        );
        assert!(s.focus.is_none(), "PowerShot Y points down");

        let (tag, typ, _, mut bytes) = af_info2(143, &[(480, 480, 1302, -173)], &[0], &[0]);
        bytes.truncate(bytes.len() - 2);
        let s = af("Canon EOS RP", 1, (tag, typ, bytes.len() as u32 / 2, bytes));
        assert!(s.focus.is_none(), "an AFInfo2 shorter than its points");
    }

    #[test]
    fn a_largesize_box_is_skipped_by_its_64_bit_size() {
        let c = Cr3 {
            pad: 40,
            ..Cr3::new()
        };
        let buf = c.build();
        let a = parse(&buf).unwrap();
        assert_eq!(a.slice(&buf, a.full.unwrap()), &c.full[..]);

        let size_at = find(&buf, b"free") + 4;
        for size in [8u64, u64::MAX] {
            let mut bad = buf.clone();
            bad[size_at..size_at + 8].copy_from_slice(&size.to_be_bytes());
            assert!(parse(&bad).is_err(), "largesize {size}");
        }
    }

    #[test]
    fn box_sizes_out_of_range_are_errors() {
        let buf = Cr3::new().build();
        let moov_at = find(&buf, b"moov") - 4;
        let mut big = buf.clone();
        big[moov_at..moov_at + 4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(parse(&big).is_err(), "moov past the end");

        let mut tiny = buf.clone();
        tiny[moov_at..moov_at + 4].copy_from_slice(&4u32.to_be_bytes());
        assert!(parse(&tiny).is_err(), "moov smaller than its header");

        let cmt1_at = find(&buf, b"CMT1") - 4;
        let mut inner = buf.clone();
        inner[cmt1_at..cmt1_at + 4].copy_from_slice(&0xFFFFu32.to_be_bytes());
        assert!(parse(&inner).is_err(), "CMT1 past the Canon uuid");

        let prvw_at = find(&buf, b"PRVW") + 4;
        let mut long = buf.clone();
        long[prvw_at + 12..prvw_at + 16].copy_from_slice(&0xFFFFu32.to_be_bytes());
        assert!(parse(&long).is_err(), "the PRVW JPEG past its box");

        assert!(parse(&buf[..moov_at + 64]).is_err(), "cut inside moov");
        assert!(parse(b"\0\0\0\x0cftypmif1").is_err(), "not a CR3");
    }

    #[test]
    fn a_prefix_holding_the_prvw_header_parses() {
        let mut c = Cr3::new();
        c.preview = Some(jpeg_bytes(4096, 2));
        let buf = c.build();
        let p = find(&buf, c.preview.as_ref().unwrap());
        let a = parse(&buf[..p + 2]).unwrap();
        assert_eq!(at(a.preview), Some((p, 4096)));
        assert!(parse(&buf[..p - 4]).is_err(), "cut inside the PRVW header");
    }
}
