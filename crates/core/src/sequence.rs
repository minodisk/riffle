//! Sequence the `DateTimeOriginal` of the JPEGs directly under a folder at
//! second granularity, to work around Google Photos ignoring sub-second
//! timestamps of burst photos and not preserving the order within the same
//! second. Derived from minodisk/lapse v0.4.0, MIT.
//!
//! Unlike lapse, the files are ordered by capture time (`DateTimeOriginal`,
//! then `SubSecTimeOriginal`, then natural filename order), so a folder
//! exported from two bodies interleaves by time, and the result is written as
//! a complete copy into the sibling folder [`output_dir`]; the source files
//! are never written.
//!
//! Assignment rule: new time = max(original time, previous file's new time + 1 second).
//! Only burst shots collapsed into the same second are shifted minimally; if
//! there is a time gap before another scene, that scene's original capture
//! time is preserved as is.
//!
//! # Crate selection notes
//!
//! The date-time tags are overwritten directly in place inside the JPEG's
//! APP1 (Exif) segment, without changing their length.
//!
//! - `kamadak-exif` is read-only and cannot write EXIF.
//! - Writing via `little_exif` or `img-parts` rebuilds the APP1 segment,
//!   which risks corrupting absolute offsets inside MakerNote and causing
//!   side effects on tags other than the date-times.
//! - An EXIF date-time is a fixed 19 bytes, "YYYY:MM:DD HH:MM:SS" (20 bytes
//!   including the NUL terminator), so its length never changes across a
//!   rewrite. Overwriting just the value area of the target tags therefore
//!   leaves every other part of the file — including the image scan data —
//!   completely untouched.
//!
//! For these reasons, instead of relying on external crates' write support,
//! we implement our own minimal TIFF/IFD walker that replaces the target
//! ASCII values inside APP1 in place. This lets tests guarantee that "the
//! differing bytes are confined to the target tags' value areas".

use std::cmp::Ordering as CmpOrdering;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use anyhow::{bail, ensure, Context, Error, Result};
use chrono::{Duration, NaiveDateTime};
use rayon::prelude::*;

/// EXIF date-time format (colon-separated, fixed 19 bytes)
pub const EXIF_DATETIME_FORMAT: &str = "%Y:%m:%d %H:%M:%S";

/// Newtype representing an EXIF date-time "YYYY:MM:DD HH:MM:SS".
///
/// A value of this type is guaranteed to be a valid EXIF date-time by parsing
/// at construction (Parse, don't validate). Validation is consolidated at the
/// single boundary (reading from a file), so no runtime validation is needed
/// in later computation and writing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExifDateTime(NaiveDateTime);

impl ExifDateTime {
    /// The date-time one second later. Minute/hour/day carry-over is handled
    /// by chrono.
    pub fn succ(self) -> Self {
        Self(self.0 + Duration::seconds(1))
    }

    /// The fixed 19-byte representation written into EXIF tags.
    pub fn to_exif_bytes(self) -> [u8; 19] {
        let s = self.0.format(EXIF_DATETIME_FORMAT).to_string();
        // Years with 5+ digits cannot be represented in EXIF and are
        // unreachable from parsed values
        s.into_bytes()
            .try_into()
            .expect("EXIF date-time is always 19 bytes")
    }

    /// Parses from the bytes of an EXIF tag value area.
    pub fn from_exif_bytes(bytes: &[u8]) -> Result<Self> {
        let s = std::str::from_utf8(bytes).context("date-time tag value is not ASCII")?;
        s.parse()
    }
}

impl FromStr for ExifDateTime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        NaiveDateTime::parse_from_str(s.trim(), EXIF_DATETIME_FORMAT)
            .map(Self)
            .with_context(|| format!("cannot parse as EXIF date-time (YYYY:MM:DD HH:MM:SS): {s:?}"))
    }
}

impl fmt::Display for ExifDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format(EXIF_DATETIME_FORMAT))
    }
}

/// Length of an EXIF date-time "YYYY:MM:DD HH:MM:SS" (excluding the NUL terminator)
pub const DATETIME_LEN: usize = 19;

/// Absolute byte offsets (from the start of the file) of the date-time tag
/// values to be rewritten.
#[derive(Debug)]
pub struct DateTimeOffsets {
    /// DateTimeOriginal (0x9003, Exif IFD). Required.
    pub datetime_original: usize,
    /// DateTimeDigitized / CreateDate (0x9004, Exif IFD). Synced when present.
    pub datetime_digitized: Option<usize>,
    /// DateTime / ModifyDate (0x0132, IFD0). Synced when present.
    pub datetime: Option<usize>,
}

impl DateTimeOffsets {
    /// All offsets to be rewritten
    pub fn all(&self) -> Vec<usize> {
        let mut v = vec![self.datetime_original];
        v.extend(self.datetime_digitized);
        v.extend(self.datetime);
        v
    }
}

/// Finds the value offsets of the date-time tags in a JPEG buffer.
/// Missing DateTimeOriginal is an error. 0x9004 / 0x0132 are None when absent
/// (the in-place approach never creates tags that do not exist).
pub fn find_datetime_offsets(buf: &[u8]) -> Result<DateTimeOffsets> {
    let (tiff_base, seg_end) = find_exif_tiff(buf)?;
    let tiff = Tiff::new(buf, tiff_base, seg_end)?;

    let ifd0 = tiff.u32(4)? as usize;
    let mut datetime = None;
    let mut exif_ifd_off = None;
    for e in tiff.ifd_entries(ifd0)? {
        match e.tag {
            0x0132 => datetime = tiff.ascii_value_abs(&e).ok(),
            0x8769 => exif_ifd_off = Some(tiff.u32(e.value_field)? as usize),
            _ => {}
        }
    }

    let exif_ifd_off = exif_ifd_off.context("missing Exif IFD (tag 0x8769)")?;
    let mut datetime_original = None;
    let mut datetime_digitized = None;
    for e in tiff.ifd_entries(exif_ifd_off)? {
        match e.tag {
            0x9003 => datetime_original = Some(tiff.ascii_value_abs(&e)?),
            0x9004 => datetime_digitized = tiff.ascii_value_abs(&e).ok(),
            _ => {}
        }
    }

    Ok(DateTimeOffsets {
        datetime_original: datetime_original.context("missing DateTimeOriginal (tag 0x9003)")?,
        datetime_digitized,
        datetime,
    })
}

/// Reads the current value of DateTimeOriginal.
pub fn read_datetime_original(buf: &[u8]) -> Result<ExifDateTime> {
    let offsets = find_datetime_offsets(buf)?;
    read_datetime_at(buf, offsets.datetime_original)
}

/// Reads the 19-byte date-time at the given offset.
pub fn read_datetime_at(buf: &[u8], offset: usize) -> Result<ExifDateTime> {
    ExifDateTime::from_exif_bytes(&buf[offset..offset + DATETIME_LEN])
}

/// Reads SubSecTimeOriginal (0x9291, Exif IFD, ASCII) as its decimal digits.
/// `None` when the tag is absent, not ASCII, or not all digits once the NUL
/// terminator and spaces are trimmed. The digits are the fraction of the
/// second after the decimal point, so `"5"` (0.5) is later than `"12"`
/// (0.12); [`cmp_subsec`] compares them that way.
pub fn read_subsec_original(buf: &[u8]) -> Result<Option<String>> {
    let (tiff_base, seg_end) = find_exif_tiff(buf)?;
    let tiff = Tiff::new(buf, tiff_base, seg_end)?;

    let ifd0 = tiff.u32(4)? as usize;
    let mut exif_ifd_off = None;
    for e in tiff.ifd_entries(ifd0)? {
        if e.tag == 0x8769 {
            exif_ifd_off = Some(tiff.u32(e.value_field)? as usize);
        }
    }
    let exif_ifd_off = exif_ifd_off.context("missing Exif IFD (tag 0x8769)")?;
    for e in tiff.ifd_entries(exif_ifd_off)? {
        if e.tag != 0x9291 || e.typ != 2 {
            continue;
        }
        // An ASCII value of up to 4 bytes lives in the value field itself;
        // a longer one is at the offset the field holds.
        let bytes = if e.count <= 4 {
            tiff.bytes(e.value_field, e.count as usize)?
        } else {
            tiff.bytes(tiff.u32(e.value_field)? as usize, e.count as usize)?
        };
        let digits = std::str::from_utf8(bytes)
            .unwrap_or("")
            .trim_matches(|c: char| c == '\0' || c == ' ');
        return Ok(
            (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
                .then(|| digits.to_string()),
        );
    }
    Ok(None)
}

/// Compares two SubSecTimeOriginal digit strings as fractions of a second:
/// left-aligned decimal digits, so `"5"` > `"12"` and `"5"` == `"50"`. A
/// missing value is 0.
pub fn cmp_subsec(a: Option<&str>, b: Option<&str>) -> CmpOrdering {
    let (a, b) = (a.unwrap_or(""), b.unwrap_or(""));
    let len = a.len().max(b.len());
    format!("{a:0<len$}").cmp(&format!("{b:0<len$}"))
}

/// Overwrites the value areas (19 bytes each) of the found date-time tags
/// with the new date-time. ExifDateTime is guaranteed by its type to always
/// have a valid 19-byte representation, so no validation is needed and the
/// buffer length never changes.
pub fn patch_datetimes(buf: &mut [u8], offsets: &DateTimeOffsets, new: ExifDateTime) {
    let bytes = new.to_exif_bytes();
    for off in offsets.all() {
        buf[off..off + DATETIME_LEN].copy_from_slice(&bytes);
    }
}

/// Walks the JPEG segments and returns the APP1 (Exif) segment's
/// (absolute offset of the TIFF header, absolute offset of the segment end).
fn find_exif_tiff(buf: &[u8]) -> Result<(usize, usize)> {
    ensure!(
        buf.len() >= 2 && buf[0] == 0xFF && buf[1] == 0xD8,
        "not a JPEG file (missing SOI marker)"
    );
    let mut pos = 2;
    while pos + 2 <= buf.len() {
        ensure!(
            buf[pos] == 0xFF,
            "invalid JPEG segment structure (offset {pos})"
        );
        let marker = buf[pos + 1];
        // Tolerate fill bytes (runs of 0xFF)
        if marker == 0xFF {
            pos += 1;
            continue;
        }
        // EXIF never appears after SOS (start of scan data)
        if marker == 0xDA || marker == 0xD9 {
            break;
        }
        // Standalone markers without a length field (TEM, RSTn)
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            pos += 2;
            continue;
        }
        ensure!(pos + 4 <= buf.len(), "segment header is truncated");
        let len = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]) as usize;
        ensure!(
            len >= 2 && pos + 2 + len <= buf.len(),
            "segment length exceeds file size"
        );
        if marker == 0xE1 && len >= 2 + 6 + 8 && &buf[pos + 4..pos + 10] == b"Exif\0\0" {
            // pos+10 is the start of the TIFF header, pos+2+len the segment end
            return Ok((pos + 10, pos + 2 + len));
        }
        pos += 2 + len;
    }
    bail!("APP1 (Exif) segment not found")
}

/// Helper for reading the TIFF structure. All offsets are relative to the
/// TIFF header (base).
struct Tiff<'a> {
    buf: &'a [u8],
    base: usize,
    end: usize,
    little_endian: bool,
}

/// An IFD entry. `value_field` is the base-relative offset of the 4-byte
/// value/offset field.
struct Entry {
    tag: u16,
    typ: u16,
    count: u32,
    value_field: usize,
}

impl<'a> Tiff<'a> {
    fn new(buf: &'a [u8], base: usize, end: usize) -> Result<Self> {
        ensure!(end <= buf.len() && end - base >= 8, "TIFF header too short");
        let little_endian = match &buf[base..base + 2] {
            b"II" => true,
            b"MM" => false,
            _ => bail!("invalid TIFF byte order"),
        };
        let tiff = Tiff {
            buf,
            base,
            end,
            little_endian,
        };
        ensure!(tiff.u16(2)? == 42, "invalid TIFF magic number");
        Ok(tiff)
    }

    fn bytes(&self, rel: usize, n: usize) -> Result<&[u8]> {
        let abs = self.base + rel;
        ensure!(
            abs + n <= self.end,
            "TIFF structure points outside the segment"
        );
        Ok(&self.buf[abs..abs + n])
    }

    fn u16(&self, rel: usize) -> Result<u16> {
        let b = self.bytes(rel, 2)?;
        Ok(if self.little_endian {
            u16::from_le_bytes([b[0], b[1]])
        } else {
            u16::from_be_bytes([b[0], b[1]])
        })
    }

    fn u32(&self, rel: usize) -> Result<u32> {
        let b = self.bytes(rel, 4)?;
        Ok(if self.little_endian {
            u32::from_le_bytes([b[0], b[1], b[2], b[3]])
        } else {
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        })
    }

    fn ifd_entries(&self, ifd: usize) -> Result<Vec<Entry>> {
        let n = self.u16(ifd)? as usize;
        let mut entries = Vec::with_capacity(n);
        for i in 0..n {
            let e = ifd + 2 + i * 12;
            entries.push(Entry {
                tag: self.u16(e)?,
                typ: self.u16(e + 2)?,
                count: self.u32(e + 4)?,
                value_field: e + 8,
            });
        }
        Ok(entries)
    }

    /// Returns the absolute offset of an ASCII date-time tag's value area.
    /// A date-time is 19 chars + NUL = 20 bytes, which exceeds 4 bytes, so the
    /// value is always an offset reference.
    fn ascii_value_abs(&self, e: &Entry) -> Result<usize> {
        ensure!(
            e.typ == 2,
            "date-time tag 0x{:04X} is not ASCII type",
            e.tag
        );
        ensure!(
            e.count as usize >= DATETIME_LEN,
            "date-time tag 0x{:04X} has invalid length",
            e.tag
        );
        let off = self.u32(e.value_field)? as usize;
        let abs = self.base + off;
        ensure!(
            abs + DATETIME_LEN <= self.end,
            "date-time tag 0x{:04X} value points outside the segment",
            e.tag
        );
        Ok(abs)
    }
}

/// One file's place in the computed order: its original time and sub-second,
/// and the time it is assigned.
#[derive(Debug, Clone)]
pub struct Planned {
    pub old: ExifDateTime,
    pub new: ExifDateTime,
    /// SubSecTimeOriginal digits, `None` when absent.
    pub subsec: Option<String>,
}

/// Result of processing one file (DateTimeOriginal before/after)
#[derive(Debug, Clone)]
pub struct FileOutcome {
    pub old: ExifDateTime,
    pub new: ExifDateTime,
    /// SubSecTimeOriginal digits, `None` when absent.
    pub subsec: Option<String>,
    /// True if the EXIF was rewritten (or would be rewritten in dry-run);
    /// false if every target tag already had the assigned time
    pub changed: bool,
}

/// Results of a [`run`], in the computed order.
#[derive(Debug)]
pub struct Summary {
    /// One entry per file handled; a file skipped because the run was
    /// canceled has none.
    pub results: Vec<(PathBuf, Result<FileOutcome, String>)>,
    /// Number of target JPEGs in the folder, handled or not.
    pub total: usize,
    pub canceled: bool,
}

fn is_jpeg(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e.to_ascii_lowercase().as_str(), "jpg" | "jpeg"))
        .unwrap_or(false)
}

/// Enumerates the JPEGs (.jpg / .jpeg, case-insensitive) directly under the
/// given directory in natural filename order. Other formats such as RAW are
/// ignored. Subdirectories are not recursed.
pub fn collect_jpegs(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("failed to read directory: {}: {e}", dir.display()))?;
    let mut named: Vec<(String, PathBuf)> = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if !path.is_file() || !is_jpeg(&path) {
            continue;
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        named.push((name, path));
    }
    // Natural sort: compare numeric parts as numbers (natord) so that
    // image_2.jpg comes before image_10.jpg. Not plain lexicographic order.
    named.sort_by(|a, b| natord::compare(&a.0, &b.0));
    Ok(named.into_iter().map(|(_, p)| p).collect())
}

/// The folder a [`run`] writes to: `<parent>/<name>-sequenced`, next to
/// `folder`. A folder already named `x-sequenced` gets `x-sequenced-sequenced`.
pub fn output_dir(folder: &Path) -> Result<PathBuf, String> {
    let name = folder
        .file_name()
        .ok_or_else(|| format!("cannot name an output folder for: {}", folder.display()))?;
    Ok(folder.with_file_name(format!("{}-sequenced", name.to_string_lossy())))
}

/// Reads every target JPEG's original time in parallel, sorts them by
/// (`DateTimeOriginal`, `SubSecTimeOriginal`, natural filename) and assigns
/// "new time = max(original time, previous new time + 1 second)"
/// sequentially in that order; the first file keeps its original time.
/// Files whose original time cannot be read are excluded from the assignment
/// and come last, as errors. No JPEG at all is an error.
#[allow(clippy::type_complexity)]
pub fn plan(dir: &Path) -> Result<Vec<(PathBuf, Result<Planned, String>)>, String> {
    let files = collect_jpegs(dir)?;
    if files.is_empty() {
        return Err(format!("no target JPEG files found in: {}", dir.display()));
    }

    let mut originals: Vec<(PathBuf, Result<(ExifDateTime, Option<String>), String>)> = files
        .into_par_iter()
        .map(|path| {
            let original = read_original(&path).map_err(|e| format!("{e:#}"));
            (path, original)
        })
        .collect();

    // `collect_jpegs` yields natural filename order and `sort_by` is stable,
    // so natural filename order is the last tie-break.
    originals.sort_by(|(_, a), (_, b)| match (a, b) {
        (Ok((ta, sa)), Ok((tb, sb))) => ta
            .cmp(tb)
            .then_with(|| cmp_subsec(sa.as_deref(), sb.as_deref())),
        (Ok(_), Err(_)) => CmpOrdering::Less,
        (Err(_), Ok(_)) => CmpOrdering::Greater,
        (Err(_), Err(_)) => CmpOrdering::Equal,
    });

    let mut prev: Option<ExifDateTime> = None;
    Ok(originals
        .into_iter()
        .map(|(path, original)| {
            let assigned = original.map(|(old, subsec)| {
                let new = match prev {
                    Some(p) => old.max(p.succ()),
                    None => old,
                };
                prev = Some(new);
                Planned { old, new, subsec }
            });
            (path, assigned)
        })
        .collect())
}

fn read_original(path: &Path) -> Result<(ExifDateTime, Option<String>)> {
    let buf = fs::read(path).with_context(|| format!("failed to read: {}", path.display()))?;
    // SubSec is only a tie-break: a malformed entry reads as absent rather
    // than failing the file.
    Ok((
        read_datetime_original(&buf)?,
        read_subsec_original(&buf).ok().flatten(),
    ))
}

/// Sequences the JPEGs in `dir` into [`output_dir`]`(dir)`.
///
/// 1. [`plan`]: read the original times in parallel, sort, and assign the
///    new times sequentially
/// 2. Unless `dry_run`: create the output folder if missing and delete the
///    `.jpg` / `.jpeg` files directly inside it (any other file or subfolder
///    is left alone)
/// 3. Patch each file in memory and write it into the output folder under
///    its own name in parallel, patched and unchanged files alike, each
///    through a temporary file in the output folder renamed into place, so no
///    half-written file can ever be observed. The source files are never
///    opened for writing. The copies do not carry the source mtime or
///    permissions.
///
/// `on_progress(done, total, path, outcome)` is called once per handled file,
/// with `done` counting up to `total`. `cancel` is checked before step 2 and
/// before each file's write; once it is set no further file is started, and
/// the output folder holds only the complete files written so far (the next
/// run rebuilds it). With `dry_run` nothing on disk is touched, not even the
/// output folder.
pub fn run<F>(
    dir: &Path,
    dry_run: bool,
    cancel: &AtomicBool,
    on_progress: F,
) -> Result<Summary, String>
where
    F: Fn(usize, usize, &Path, &Result<FileOutcome, String>) + Sync,
{
    let planned = plan(dir)?;
    let total = planned.len();
    let out = output_dir(dir)?;
    if !dry_run {
        if canceled(cancel) {
            return Ok(Summary {
                results: Vec::new(),
                total,
                canceled: true,
            });
        }
        clear_output(dir, &out)?;
    }

    let done = Mutex::new(0usize);
    let handled: Vec<Option<(PathBuf, Result<FileOutcome, String>)>> = planned
        .into_par_iter()
        .map(|(path, planned)| {
            if canceled(cancel) {
                return None;
            }
            let result = planned.and_then(|p| {
                process_file(&path, p, (!dry_run).then_some(out.as_path()))
                    .map_err(|e| format!("{e:#}"))
            });
            let mut done = done.lock().unwrap_or_else(|e| e.into_inner());
            *done += 1;
            on_progress(*done, total, &path, &result);
            Some((path, result))
        })
        .collect();

    let canceled = handled.iter().any(Option::is_none);
    Ok(Summary {
        results: handled.into_iter().flatten().collect(),
        total,
        canceled,
    })
}

fn canceled(cancel: &AtomicBool) -> bool {
    cancel.load(Ordering::Relaxed)
}

fn clear_output(dir: &Path, out: &Path) -> Result<(), String> {
    fs::create_dir_all(out).map_err(|e| format!("failed to create: {}: {e}", out.display()))?;
    // `out` may be a symlink or junction resolving to `dir` (or any other
    // ancestor holding source files); refuse to delete anything in that case.
    if let (Ok(dir_canon), Ok(out_canon)) = (fs::canonicalize(dir), fs::canonicalize(out)) {
        if dir_canon == out_canon {
            return Err(format!(
                "output folder resolves to the source folder: {}",
                out.display()
            ));
        }
    }
    let entries = fs::read_dir(out)
        .map_err(|e| format!("failed to read directory: {}: {e}", out.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_file() && is_jpeg(&path) {
            fs::remove_file(&path)
                .map_err(|e| format!("failed to delete: {}: {e}", path.display()))?;
        }
    }
    Ok(())
}

fn process_file(path: &Path, planned: Planned, out: Option<&Path>) -> Result<FileOutcome> {
    let mut buf = fs::read(path).with_context(|| format!("failed to read: {}", path.display()))?;
    let offsets = find_datetime_offsets(&buf)?;
    let old = read_datetime_at(&buf, offsets.datetime_original)?;
    let new = planned.new;
    let new_bytes = new.to_exif_bytes();
    let unchanged = offsets
        .all()
        .into_iter()
        .all(|o| buf[o..o + DATETIME_LEN] == new_bytes);
    if !unchanged {
        patch_datetimes(&mut buf, &offsets, new);
    }
    if let Some(out) = out {
        let name = path
            .file_name()
            .with_context(|| format!("no file name: {}", path.display()))?;
        write_atomic(out, &out.join(name), &buf)?;
    }
    Ok(FileOutcome {
        old,
        new,
        subsec: planned.subsec,
        changed: !unchanged,
    })
}

/// Writes the whole file to a temporary file in `dir` and then renames it to
/// `path`, so a crash or a cancel mid-write never leaves a half-written file.
fn write_atomic(dir: &Path, path: &Path, data: &[u8]) -> Result<()> {
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(data)?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)
        .map_err(|e| e.error)
        .with_context(|| format!("failed to write: {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_display_roundtrip() {
        let dt: ExifDateTime = "2024:01:02 03:04:59".parse().unwrap();
        assert_eq!(dt.to_string(), "2024:01:02 03:04:59");
        assert_eq!(&dt.to_exif_bytes(), b"2024:01:02 03:04:59");
    }

    #[test]
    fn parse_rejects_invalid_format() {
        assert!("2024-01-02 03:04:59".parse::<ExifDateTime>().is_err());
        assert!("not a datetime".parse::<ExifDateTime>().is_err());
        assert!("".parse::<ExifDateTime>().is_err());
    }

    #[test]
    fn succ_rolls_over_minute() {
        let dt: ExifDateTime = "2024:12:31 23:59:59".parse().unwrap();
        assert_eq!(dt.succ().to_string(), "2025:01:01 00:00:00");
    }

    #[test]
    fn ord_and_max() {
        let a: ExifDateTime = "2024:01:02 03:04:59".parse().unwrap();
        let b: ExifDateTime = "2024:01:02 03:05:10".parse().unwrap();
        assert_eq!(a.max(b), b);
        assert_eq!(b.max(a.succ()), b);
    }

    #[test]
    fn subsec_compares_as_a_fraction() {
        assert_eq!(cmp_subsec(Some("5"), Some("12")), CmpOrdering::Greater);
        assert_eq!(cmp_subsec(Some("5"), Some("50")), CmpOrdering::Equal);
        assert_eq!(cmp_subsec(None, Some("0")), CmpOrdering::Equal);
        assert_eq!(cmp_subsec(None, Some("01")), CmpOrdering::Less);
    }

    #[test]
    fn output_dir_is_a_sibling() {
        let dir = Path::new("photos").join("export");
        assert_eq!(
            output_dir(&dir).unwrap(),
            Path::new("photos").join("export-sequenced")
        );
        assert_eq!(
            output_dir(&Path::new("photos").join("x-sequenced")).unwrap(),
            Path::new("photos").join("x-sequenced-sequenced")
        );
    }
}
