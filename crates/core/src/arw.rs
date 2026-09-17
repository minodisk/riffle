//! Minimal parser that locates the embedded JPEGs in an ARW (a TIFF variant).

use anyhow::{bail, Result};

const TAG_ORIENTATION: u16 = 0x0112;
const TAG_JPEG_OFFSET: u16 = 0x0201;
const TAG_JPEG_LENGTH: u16 = 0x0202;
const TAG_SUB_IFDS: u16 = 0x014a;

/// Location of one embedded JPEG inside an ARW.
#[derive(Debug, Clone, Copy)]
pub struct Embedded {
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug)]
pub struct Arw {
    /// The small preview in IFD0 (Sony 1616x1080).
    pub preview: Option<Embedded>,
    /// The full-resolution JPEG (JpgFromRaw).
    pub full: Option<Embedded>,
    pub orientation: u16,
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
    let next_ifd = if next + 4 <= buf.len() {
        u32le(buf, next) as usize
    } else {
        0
    };
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

pub fn parse(buf: &[u8]) -> Result<Arw> {
    if buf.len() < 8 || &buf[0..2] != b"II" {
        bail!("not a little-endian TIFF/ARW");
    }
    let (ifd0, next) = read_ifd(buf, u32le(buf, 4) as usize)?;

    let orientation = find(&ifd0, TAG_ORIENTATION)
        .map(|(v, _)| v as u16)
        .unwrap_or(1);
    let preview = embedded(&ifd0);

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
            targets.extend((0..count as usize).map(|i| u32le(buf, val as usize + i * 4) as usize));
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

    /// Build a minimal little-endian TIFF whose IFD0 carries the given entries.
    fn tiff(entries: &[(u16, u16, u32, u32)]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"II\x2a\x00");
        buf.extend_from_slice(&8u32.to_le_bytes());
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
}
