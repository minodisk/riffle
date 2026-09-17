//! Read only as much of an ARW as the metadata and the preview need.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::arw::{self, Arw};

/// How much of a file the bounded read takes.
///
/// On the α7 V files measured here every IFD, the ExifIFD, the MakerNote and
/// the end of the IFD0 preview sit within the first ~542 KB of a 48 MB ARW.
/// 1 MiB leaves close to twice that as margin while still reading ~1/48th of
/// the file; bodies that lay their preview out further in are covered by the
/// ranged re-read in `read_preview`.
pub const HEAD_LIMIT: usize = 1 << 20;

/// Read at most `limit` bytes from the start of `path`.
pub fn read_head(path: &Path, limit: usize) -> Result<Vec<u8>> {
    head_of(&mut File::open(path)?, limit)
}

fn head_of(file: &mut File, limit: usize) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    file.take(limit as u64).read_to_end(&mut buf)?;
    Ok(buf)
}

/// Parse a file's metadata and return its IFD0 preview JPEG, reading as little
/// as possible: a bounded prefix, plus a ranged read of the preview itself if
/// it lies beyond that prefix.
///
/// A prefix too short to parse is an error rather than a wrong answer, so in
/// that case the whole file is read and parsed instead.
pub fn read_preview(path: &Path) -> Result<(Arw, Vec<u8>)> {
    let mut file = File::open(path)?;
    let head = head_of(&mut file, HEAD_LIMIT)?;
    // Anything past the prefix exists only if the file is longer than it.
    let bounded = head.len() == HEAD_LIMIT;

    match preview_from(&head, &mut file, bounded) {
        Ok(found) => Ok(found),
        Err(_) if bounded => {
            let buf = std::fs::read(path)?;
            // The prefix error only explains a truncated read; once the whole
            // file is in memory, an error there describes what is actually
            // wrong with the file, so surface that one instead.
            preview_from(&buf, &mut file, false)
        }
        Err(e) => Err(e),
    }
}

/// Parse just a file's metadata, reading the same bounded prefix as
/// `read_preview` and falling back to the whole file when that prefix is too
/// short to parse.
pub fn read_metadata(path: &Path) -> Result<Arw> {
    let head = read_head(path, HEAD_LIMIT)?;
    let bounded = head.len() == HEAD_LIMIT;
    match arw::parse(&head) {
        Ok(arw) => Ok(arw),
        Err(_) if bounded => arw::parse(&std::fs::read(path)?),
        Err(e) => Err(e),
    }
}

fn preview_from(buf: &[u8], file: &mut File, bounded: bool) -> Result<(Arw, Vec<u8>)> {
    let arw = arw::parse(buf)?;
    let e = arw.preview.ok_or_else(|| anyhow!("no embedded preview"))?;
    let end = e
        .offset
        .checked_add(e.length)
        .ok_or_else(|| anyhow!("preview out of range"))?;
    if end <= buf.len() {
        let jpeg = arw.slice(buf, e).to_vec();
        return Ok((arw, jpeg));
    }
    if !bounded {
        return Err(anyhow!("preview out of range"));
    }
    if end as u64 > file.metadata()?.len() {
        return Err(anyhow!("preview out of range"));
    }
    let mut jpeg = vec![0u8; e.length];
    file.seek(SeekFrom::Start(e.offset as u64))?;
    file.read_exact(&mut jpeg)?;
    Ok((arw, jpeg))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path =
            std::env::temp_dir().join(format!("riffle-reader-{name}-{}", std::process::id()));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn reads_at_most_the_limit() {
        let path = temp_file("head", &[7u8; 100]);
        assert_eq!(read_head(&path, 10).unwrap(), [7u8; 10]);
        assert_eq!(read_head(&path, 1000).unwrap().len(), 100);
        std::fs::remove_file(&path).unwrap();
    }

    /// A TIFF whose IFD0 points at a preview that starts after `at`.
    fn arw_with_preview(at: usize, jpeg: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"II\x2a\x00");
        buf.extend_from_slice(&8u32.to_le_bytes());
        buf.extend_from_slice(&2u16.to_le_bytes());
        for (tag, value) in [(0x0201u16, at as u32), (0x0202, jpeg.len() as u32)] {
            buf.extend_from_slice(&tag.to_le_bytes());
            buf.extend_from_slice(&4u16.to_le_bytes());
            buf.extend_from_slice(&1u32.to_le_bytes());
            buf.extend_from_slice(&value.to_le_bytes());
        }
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.resize(at, 0);
        buf.extend_from_slice(jpeg);
        buf
    }

    #[test]
    fn preview_inside_the_prefix_is_sliced() {
        let jpeg = [1u8, 2, 3, 4];
        let path = temp_file("near", &arw_with_preview(64, &jpeg));
        let (_, out) = read_preview(&path).unwrap();
        assert_eq!(out, jpeg);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn preview_past_the_prefix_is_read_by_range() {
        let jpeg = [9u8; 32];
        let path = temp_file("far", &arw_with_preview(HEAD_LIMIT + 4096, &jpeg));
        let (_, out) = read_preview(&path).unwrap();
        assert_eq!(out, jpeg);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn a_preview_past_the_end_of_the_file_is_an_error() {
        let mut buf = arw_with_preview(HEAD_LIMIT + 4096, &[5u8; 16]);
        buf.truncate(HEAD_LIMIT + 4096);
        let path = temp_file("short", &buf);
        assert!(read_preview(&path).is_err());
        std::fs::remove_file(&path).unwrap();
    }
}
