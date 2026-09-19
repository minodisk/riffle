//! DxO PhotoLab `.dop` sidecar reading and writing for ratings.
//!
//! A `.dop` is a Lua table literal. PhotoLab keeps the judgement in two keys
//! directly under `Sidecar.Source.Items[0]`: `ShouldProcess` (`0` pick, `1`
//! reject, `2` unflagged) and `Rating` (`0..=5`). Riffle's judgement maps
//! onto them as follows: a reject is `ShouldProcess = 1` with `Rating` left as
//! it is; stars or a clear write `Rating` and set `ShouldProcess` to `0` for a
//! pick and `2` otherwise. A pick and stars coexist, as they do in PhotoLab.
//!
//! An existing sidecar is patched by splicing the bytes of those values and
//! of the two timestamps `Sidecar.Date` and `Items[0].ModificationDate`, so
//! PhotoLab's develop settings stay byte-for-byte; a sidecar is never
//! regenerated from a parse. The colour label is the `ColorLabel = "Red",`
//! line of the same item, absent for "no label"; [`write_label`] splices,
//! inserts or removes that line the same way.
//!
//! There is no Lua parser: a scanner tracks brace depth (skipping over
//! double-quoted strings) and matches `Key = value,` lines at the depth of the
//! table they belong to, which is what keeps nested keys such as the `Label`
//! entries inside `HSLHueSlices` from matching.

use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The sidecar path of an ARW: `.dop` appended to the full file name, as
/// PhotoLab names it (`_DSC0001.ARW` -> `_DSC0001.ARW.dop`).
///
/// Pure by design: when the directory already holds a sidecar differing only
/// in case, the caller is the one that lists the directory and prefers the
/// existing name.
pub fn sidecar_path(arw: &Path) -> PathBuf {
    let mut name = arw.as_os_str().to_os_string();
    name.push(".dop");
    PathBuf::from(name)
}

/// The rating of `Items[0]`: `-1` when `ShouldProcess = 1` whatever the
/// `Rating`, otherwise `Rating` when it is in `0..=5`.
///
/// `None` when neither applies; `Err` when the bytes are not a `.dop` the
/// scanner can follow.
pub fn read_rating(bytes: &[u8]) -> Result<Option<i8>, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| format!("not UTF-8: {e}"))?;
    let doc = locate(text)?;
    if doc.should_process.map(|r| &text[r.0..r.1]) == Some("1") {
        return Ok(Some(-1));
    }
    Ok(doc.rating.and_then(|(s, e)| {
        text[s..e]
            .parse::<i8>()
            .ok()
            .filter(|n| (0..=5).contains(n))
    }))
}

/// Whether `Items[0]` is picked (`ShouldProcess = 0`); `Err` as for
/// [`read_rating`].
pub fn read_pick(bytes: &[u8]) -> Result<bool, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| format!("not UTF-8: {e}"))?;
    let doc = locate(text)?;
    Ok(doc.should_process.map(|r| &text[r.0..r.1]) == Some("0"))
}

/// The sidecar bytes carrying `rating` and `pick`, written at `now` (a
/// [`timestamp`]).
///
/// With `existing` `None` this is a fresh minimal template for the file
/// `name`; otherwise the existing bytes with only `Rating`, `ShouldProcess`,
/// `Date` and `ModificationDate` spliced, each inserted as its own line when
/// absent. `name` is only used by the template.
///
/// `ShouldProcess` becomes `1` for a reject (whatever `pick` says), `0` for a
/// pick and `2` otherwise, so the caller passes the pick it wants kept.
///
/// `rating` `None` means unrated and writes `Rating = 0`. On a file that has
/// no sidecar yet, the caller must not write anything at all for a judgement
/// that is both unrated (`rating` `None`) and unpicked (`pick` `false`), as
/// with [`crate::xmp::write_rating`].
pub fn write_rating(
    existing: Option<&[u8]>,
    rating: Option<i8>,
    pick: bool,
    name: &str,
    now: &str,
) -> Result<Vec<u8>, String> {
    let value = rating.unwrap_or(0);
    if !(-1..=5).contains(&value) {
        return Err(format!("rating {value} is outside -1..=5"));
    }
    let Some(existing) = existing else {
        return Ok(template(value, pick, name, now, [uuid(), uuid()]).into_bytes());
    };
    let text = std::str::from_utf8(existing).map_err(|e| format!("not UTF-8: {e}"))?;
    let doc = locate(text)?;
    let stamp = format!("\"{now}\"");
    let mut edits = vec![
        doc.edit(text, doc.date, doc.sidecar_close, "Date", &stamp)?,
        doc.edit(
            text,
            doc.modification_date,
            doc.item_close,
            "ModificationDate",
            &stamp,
        )?,
    ];
    if value == -1 {
        edits.push(doc.edit(
            text,
            doc.should_process,
            doc.item_close,
            "ShouldProcess",
            "1",
        )?);
    } else {
        let flag = if pick { "0" } else { "2" };
        edits.push(doc.edit(
            text,
            doc.rating,
            doc.item_close,
            "Rating",
            &value.to_string(),
        )?);
        edits.push(doc.edit(
            text,
            doc.should_process,
            doc.item_close,
            "ShouldProcess",
            flag,
        )?);
    }
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut out = text.to_string();
    for (start, end, replacement) in edits {
        out.replace_range(start..end, &replacement);
    }
    Ok(out.into_bytes())
}

/// The colour label of `Items[0]`: the string inside the quotes of
/// `ColorLabel`, `None` when the key is absent, empty or not a double-quoted
/// string; `Err` as for [`read_rating`].
pub fn read_label(bytes: &[u8]) -> Result<Option<String>, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| format!("not UTF-8: {e}"))?;
    let doc = locate(text)?;
    Ok(doc.color_label.and_then(|(s, e)| {
        text[s..e]
            .strip_prefix('"')?
            .strip_suffix('"')
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    }))
}

/// The sidecar bytes carrying the colour label `label`, written at `now`.
///
/// `Some(label)` splices the quoted value in place or inserts a
/// `ColorLabel = "...",` line before the item's closing brace; `None` removes
/// the line, and leaves the label-less bytes untouched but for the
/// timestamps. The value is written as-is, unescaped. `Sidecar.Date` and
/// `Items[0].ModificationDate` are updated as by [`write_rating`].
///
/// With `existing` `None`, `Some(label)` is the fresh template for `name`
/// (unrated, unflagged) plus the label; `None` is an error, as the caller
/// never mints a sidecar for "no label".
pub fn write_label(
    existing: Option<&[u8]>,
    label: Option<&str>,
    name: &str,
    now: &str,
) -> Result<Vec<u8>, String> {
    let owned;
    let existing = match (existing, label) {
        (Some(existing), _) => existing,
        (None, Some(_)) => {
            owned = template(0, false, name, now, [uuid(), uuid()]);
            owned.as_bytes()
        }
        (None, None) => return Err("no sidecar to clear a label from".to_string()),
    };
    let text = std::str::from_utf8(existing).map_err(|e| format!("not UTF-8: {e}"))?;
    let doc = locate(text)?;
    let stamp = format!("\"{now}\"");
    let mut edits = vec![
        doc.edit(text, doc.date, doc.sidecar_close, "Date", &stamp)?,
        doc.edit(
            text,
            doc.modification_date,
            doc.item_close,
            "ModificationDate",
            &stamp,
        )?,
    ];
    match (label, doc.color_label) {
        (Some(label), found) => edits.push(doc.edit(
            text,
            found,
            doc.item_close,
            "ColorLabel",
            &format!("\"{label}\""),
        )?),
        (None, Some(found)) => edits.push(Doc::remove_line(text, found)),
        (None, None) => {}
    }
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut out = text.to_string();
    for (start, end, replacement) in edits {
        out.replace_range(start..end, &replacement);
    }
    Ok(out.into_bytes())
}

/// `t` in PhotoLab's format: UTC with seven fractional digits,
/// `2026-09-18T10:20:50.0471837Z`.
pub fn timestamp(t: SystemTime) -> String {
    let d = t.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    let (y, m, day) = civil_from_days((secs / 86_400) as i64);
    let rem = secs % 86_400;
    format!(
        "{y:04}-{m:02}-{day:02}T{:02}:{:02}:{:02}.{:07}Z",
        rem / 3600,
        rem / 60 % 60,
        rem % 60,
        d.subsec_nanos() / 100
    )
}

/// Howard Hinnant's `civil_from_days`: days since 1970-01-01 to a
/// proleptic Gregorian date.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// A random (version 4) UUID in PhotoLab's upper-case form.
fn uuid() -> String {
    let state = RandomState::new();
    let mut n = (u128::from(state.hash_one(0u8)) << 64) | u128::from(state.hash_one(1u8));
    n = (n & !(0xF << 76)) | (0x4 << 76);
    n = (n & !(0x3 << 62)) | (0x2 << 62);
    format!(
        "{:08X}-{:04X}-{:04X}-{:04X}-{:012X}",
        n >> 96,
        (n >> 80) & 0xFFFF,
        (n >> 64) & 0xFFFF,
        (n >> 48) & 0xFFFF,
        n & 0xFFFF_FFFF_FFFF
    )
}

fn template(value: i8, pick: bool, name: &str, now: &str, uuids: [String; 2]) -> String {
    let (rating, flag) = match (value, pick) {
        (-1, _) => (0, 1),
        (_, true) => (value, 0),
        _ => (value, 2),
    };
    let name = name.replace('\\', "\\\\").replace('"', "\\\"");
    let [item_uuid, source_uuid] = uuids;
    format!(
        "Sidecar = {{\n\
         Date = \"{now}\",\n\
         Software = \"riffle\",\n\
         Source = {{\n\
         Items = {{\n\
         {{\n\
         CreationDate = \"{now}\",\n\
         ModificationDate = \"{now}\",\n\
         Name = \"{name}\",\n\
         Rating = {rating},\n\
         ShouldProcess = {flag},\n\
         Uuid = \"{item_uuid}\",\n\
         }}\n\
         ,\n\
         }}\n\
         ,\n\
         Uuid = \"{source_uuid}\",\n\
         }}\n\
         ,\n\
         Version = \"21.0\",\n\
         }}\n"
    )
}

type Range = (usize, usize);

struct Line {
    start: usize,
    end: usize,
    depth: usize,
}

struct Doc {
    sidecar_close: usize,
    item_close: usize,
    date: Option<Range>,
    rating: Option<Range>,
    should_process: Option<Range>,
    modification_date: Option<Range>,
    color_label: Option<Range>,
}

impl Doc {
    /// A splice of `found`, or an insertion of a `key = value,` line before
    /// the table closing at `close` when the key is absent.
    fn edit(
        &self,
        text: &str,
        found: Option<Range>,
        close: usize,
        key: &str,
        value: &str,
    ) -> Result<(usize, usize, String), String> {
        if let Some((start, end)) = found {
            return Ok((start, end, value.to_string()));
        }
        let at = text[..close].rfind('\n').map_or(0, |i| i + 1);
        if !text[at..close].trim().is_empty() {
            return Err(format!(
                "cannot insert {key}: the table does not close on its own line"
            ));
        }
        let indent = &text[at..at + text[at..close].len() - text[at..close].trim_start().len()];
        Ok((at, at, format!("{indent}{key} = {value},\n")))
    }

    /// The removal of the whole line holding `found`, its newline included.
    fn remove_line(text: &str, found: Range) -> (usize, usize, String) {
        let start = text[..found.0].rfind('\n').map_or(0, |i| i + 1);
        let end = text[found.1..]
            .find('\n')
            .map_or(text.len(), |i| found.1 + i + 1);
        (start, end, String::new())
    }
}

fn locate(text: &str) -> Result<Doc, String> {
    let (lines, closes) = scan(text)?;
    let table = |range: Range, depth: usize, key: &str| -> Result<Range, String> {
        let (start, end) =
            field(text, &lines, range, depth, key).ok_or_else(|| format!("no {key} table"))?;
        if &text[start..end] != "{" {
            return Err(format!("{key} is not a table"));
        }
        Ok((start, closes[&start]))
    };
    let sidecar = table((0, text.len()), 0, "Sidecar")?;
    let source = table(sidecar, 1, "Source")?;
    let items = table(source, 2, "Items")?;
    let item_open = lines
        .iter()
        .filter(|l| l.start > items.0 && l.start < items.1 && l.depth == 3)
        .find_map(|l| {
            let line = &text[l.start..l.end];
            let trimmed = line.trim_start();
            trimmed
                .starts_with('{')
                .then(|| l.start + line.len() - trimmed.len())
        })
        .ok_or("Items has no item")?;
    let item = (item_open, closes[&item_open]);
    let in_item = |key| field(text, &lines, item, 4, key);
    Ok(Doc {
        sidecar_close: sidecar.1,
        item_close: item.1,
        date: field(text, &lines, sidecar, 1, "Date"),
        rating: in_item("Rating"),
        should_process: in_item("ShouldProcess"),
        modification_date: in_item("ModificationDate"),
        color_label: in_item("ColorLabel"),
    })
}

/// The lines with the brace depth at their start, and each `{`'s matching
/// `}`, skipping braces inside double-quoted strings.
fn scan(text: &str) -> Result<(Vec<Line>, HashMap<usize, usize>), String> {
    let bytes = text.as_bytes();
    let mut lines = Vec::new();
    let mut closes = HashMap::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut line_start = 0usize;
    let mut line_depth = 0usize;
    let mut in_string = false;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if in_string => i += 1,
            b'"' => in_string = !in_string,
            b'\n' if in_string => return Err("a string spans a line".to_string()),
            b'\n' => {
                lines.push(Line {
                    start: line_start,
                    end: i,
                    depth: line_depth,
                });
                line_start = i + 1;
                line_depth = stack.len();
            }
            b'{' if !in_string => stack.push(i),
            b'}' if !in_string => {
                let open = stack.pop().ok_or("unbalanced braces")?;
                closes.insert(open, i);
            }
            _ => {}
        }
        i += 1;
    }
    if in_string || !stack.is_empty() {
        return Err("unbalanced braces or an unterminated string".to_string());
    }
    lines.push(Line {
        start: line_start,
        end: text.len(),
        depth: line_depth,
    });
    Ok((lines, closes))
}

/// The value range of the first `key = value,` line at `depth` starting
/// inside `range`, the trailing `,` and whitespace excluded.
fn field(text: &str, lines: &[Line], range: Range, depth: usize, key: &str) -> Option<Range> {
    lines
        .iter()
        .filter(|l| l.start >= range.0 && l.start < range.1 && l.depth == depth)
        .find_map(|l| {
            let line = &text[l.start..l.end];
            let rest = line.trim_start().strip_prefix(key)?;
            let rest = rest.trim_start().strip_prefix('=')?.trim_start();
            let value = rest.trim_end();
            let value = value.strip_suffix(',').unwrap_or(value).trim_end();
            let start = l.start + line.len() - rest.len();
            Some((start, start + value.len()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const PICK: &str = include_str!("fixtures/dop/_DSC0001.ARW.dop");
    const REJECT: &str = include_str!("fixtures/dop/_DSC0002.ARW.dop");
    const THREE: &str = include_str!("fixtures/dop/_DSC0003.ARW.dop");
    const RED: &str = include_str!("fixtures/dop/_DSC0004.ARW.dop");
    const CLEARED: &str = include_str!("fixtures/dop/_DSC0005.ARW.dop");

    const LABELLED: [(&str, &str, i8); 7] = [
        (include_str!("fixtures/dop/_DSC0009.ARW.dop"), "Red", 0),
        (include_str!("fixtures/dop/_DSC0010.ARW.dop"), "Orange", 1),
        (include_str!("fixtures/dop/_DSC0011.ARW.dop"), "Yellow", 0),
        (include_str!("fixtures/dop/_DSC0012.ARW.dop"), "Green", 4),
        (include_str!("fixtures/dop/_DSC0013.ARW.dop"), "Blue", 0),
        (include_str!("fixtures/dop/_DSC0014.ARW.dop"), "Pink", 0),
        (include_str!("fixtures/dop/_DSC0015.ARW.dop"), "Purple", 0),
    ];
    const TABBED: &str = LABELLED[0].0;

    const NOW: &str = "2026-09-18T11:00:00.0000000Z";

    fn patched(source: &str, rating: Option<i8>, pick: bool) -> String {
        String::from_utf8(write_rating(Some(source.as_bytes()), rating, pick, "x", NOW).unwrap())
            .unwrap()
    }

    fn with_now(source: &str, date: &str, modified: &str) -> String {
        source
            .replace(
                &format!("Date = \"{date}\","),
                &format!("Date = \"{NOW}\","),
            )
            .replace(
                &format!("ModificationDate = \"{modified}\","),
                &format!("ModificationDate = \"{NOW}\","),
            )
    }

    #[test]
    fn sidecar_path_appends_to_the_full_name() {
        assert_eq!(
            sidecar_path(Path::new("/photos/_DSC0001.ARW")),
            PathBuf::from("/photos/_DSC0001.ARW.dop")
        );
    }

    #[test]
    fn reads_the_photolab_samples() {
        assert_eq!(read_rating(PICK.as_bytes()).unwrap(), Some(0));
        assert_eq!(read_rating(REJECT.as_bytes()).unwrap(), Some(-1));
        assert_eq!(read_rating(THREE.as_bytes()).unwrap(), Some(3));
        assert_eq!(read_rating(RED.as_bytes()).unwrap(), Some(0));
        assert_eq!(read_rating(CLEARED.as_bytes()).unwrap(), Some(0));
    }

    #[test]
    fn rating_a_pick_keeps_the_pick() {
        let out = patched(PICK, Some(3), true);
        let expected = with_now(
            PICK,
            "2026-09-18T10:20:50.0471837Z",
            "2026-09-18T10:20:50.0431829Z",
        )
        .replace("Rating = 0,", "Rating = 3,");
        assert_eq!(out, expected);
        assert!(out.contains("ShouldProcess = 0,"));
        assert_eq!(read_rating(out.as_bytes()).unwrap(), Some(3));
    }

    #[test]
    fn a_reject_keeps_the_stars() {
        let out = patched(THREE, Some(-1), false);
        let expected = with_now(
            THREE,
            "2026-09-18T10:21:52.2415388Z",
            "2026-09-18T10:21:52.2405327Z",
        )
        .replace("ShouldProcess = 2,", "ShouldProcess = 1,");
        assert_eq!(out, expected);
        assert!(out.contains("Rating = 3,"));
        assert_eq!(read_rating(out.as_bytes()).unwrap(), Some(-1));
    }

    #[test]
    fn clearing_a_reject_unflags_it() {
        let out = patched(REJECT, None, false);
        let expected = with_now(
            REJECT,
            "2026-09-18T10:21:52.2455450Z",
            "2026-09-18T10:21:52.2405327Z",
        )
        .replace("ShouldProcess = 1,", "ShouldProcess = 2,");
        assert_eq!(out, expected);
        assert_eq!(read_rating(out.as_bytes()).unwrap(), Some(0));
    }

    #[test]
    fn patching_keeps_the_colour_label_and_the_label_decoys() {
        let out = patched(RED, Some(5), false);
        let expected = with_now(
            RED,
            "2026-09-18T10:21:52.2425322Z",
            "2026-09-18T10:21:52.2405327Z",
        )
        .replace("\nRating = 0,", "\nRating = 5,");
        assert_eq!(out, expected);
        assert!(out.contains("ColorLabel = \"Red\",\n"));
        assert_eq!(out.matches("\nLabel = \"").count(), 8);
        assert!(out.ends_with("}\n\r\n"));
    }

    #[test]
    fn the_trailing_crlf_survives() {
        for source in [PICK, REJECT, THREE, CLEARED] {
            assert!(patched(source, Some(2), false).ends_with("}\n\r\n"));
        }
    }

    #[test]
    fn inserts_a_missing_should_process_in_the_item() {
        let source = THREE.replace("ShouldProcess = 2,\n", "");
        assert_eq!(read_rating(source.as_bytes()).unwrap(), Some(3));
        let out = patched(&source, Some(-1), false);
        assert!(out.contains(
            "Uuid = \"B638CACC-6B37-4480-8A40-9377C1C2783A\",\nShouldProcess = 1,\n}\n,\n}\n"
        ));
        assert_eq!(read_rating(out.as_bytes()).unwrap(), Some(-1));
    }

    #[test]
    fn inserts_a_missing_rating_in_the_item() {
        let source = PICK.replace("Rating = 0,\n", "");
        assert_eq!(read_rating(source.as_bytes()).unwrap(), None);
        let out = patched(&source, Some(4), true);
        assert!(out
            .contains("Uuid = \"B2A2E6B3-CECB-427D-84C3-49A1351D8E65\",\nRating = 4,\n}\n,\n}\n"));
        assert!(out.contains("ShouldProcess = 0,"));
        assert_eq!(read_rating(out.as_bytes()).unwrap(), Some(4));
    }

    #[test]
    fn reads_the_pick_of_the_photolab_samples() {
        assert!(read_pick(PICK.as_bytes()).unwrap());
        for source in [REJECT, THREE, RED, CLEARED] {
            assert!(!read_pick(source.as_bytes()).unwrap());
        }
    }

    #[test]
    fn a_pick_replaces_a_reject_and_back() {
        let picked = patched(REJECT, None, true);
        assert!(picked.contains("ShouldProcess = 0,"));
        assert!(read_pick(picked.as_bytes()).unwrap());
        assert_eq!(read_rating(picked.as_bytes()).unwrap(), Some(0));

        let rejected = patched(&picked, Some(-1), false);
        assert!(rejected.contains("ShouldProcess = 1,"));
        assert!(!read_pick(rejected.as_bytes()).unwrap());
        assert_eq!(read_rating(rejected.as_bytes()).unwrap(), Some(-1));
    }

    #[test]
    fn unpicking_keeps_the_stars_and_clearing_keeps_the_pick() {
        let unpicked = patched(PICK, Some(0), false);
        assert!(unpicked.contains("ShouldProcess = 2,"));
        assert!(!read_pick(unpicked.as_bytes()).unwrap());

        let starred = patched(PICK, Some(4), true);
        let cleared = patched(&starred, None, true);
        assert!(read_pick(cleared.as_bytes()).unwrap());
        assert_eq!(read_rating(cleared.as_bytes()).unwrap(), Some(0));
    }

    #[test]
    fn a_fresh_pick_is_should_process_zero() {
        let bytes = write_rating(None, Some(2), true, "a", NOW).unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("Rating = 2,\nShouldProcess = 0,\n"));
        assert!(read_pick(&bytes).unwrap());
        let reject = write_rating(None, Some(-1), true, "a", NOW).unwrap();
        assert!(!read_pick(&reject).unwrap());
    }

    #[test]
    fn an_out_of_range_rating_reads_as_absent() {
        let source = THREE.replace("Rating = 3,", "Rating = 9,");
        assert_eq!(read_rating(source.as_bytes()).unwrap(), None);
    }

    #[test]
    fn braces_inside_strings_are_skipped() {
        let source = THREE.replace("Albums = \"\",", "Albums = \"a } \\\" { b\",");
        assert_eq!(read_rating(source.as_bytes()).unwrap(), Some(3));
    }

    #[test]
    fn unparseable_files_are_errors() {
        let truncated = &THREE[..THREE.len() / 2];
        let unbalanced = format!("{THREE}}}\n");
        let no_items = THREE.replace("Items = {", "Things = {");
        let cases: [&[u8]; 5] = [
            b"hello",
            truncated.as_bytes(),
            unbalanced.as_bytes(),
            no_items.as_bytes(),
            b"Sidecar = {\n\xff\n}\n",
        ];
        for bytes in cases {
            assert!(read_rating(bytes).is_err());
            assert!(read_pick(bytes).is_err());
            assert!(write_rating(Some(bytes), Some(1), false, "x", NOW).is_err());
        }
    }

    #[test]
    fn a_fresh_template_round_trips() {
        for n in -1..=5 {
            let bytes = write_rating(None, Some(n), false, "_DSC0001.ARW", NOW).unwrap();
            assert_eq!(read_rating(&bytes).unwrap(), Some(n));
        }
    }

    #[test]
    fn the_fresh_template_is_the_documented_one() {
        let uuids = ["A".to_string(), "B".to_string()];
        assert_eq!(
            template(3, false, "_DSC0001.ARW", NOW, uuids),
            concat!(
                "Sidecar = {\n",
                "Date = \"2026-09-18T11:00:00.0000000Z\",\n",
                "Software = \"riffle\",\n",
                "Source = {\n",
                "Items = {\n",
                "{\n",
                "CreationDate = \"2026-09-18T11:00:00.0000000Z\",\n",
                "ModificationDate = \"2026-09-18T11:00:00.0000000Z\",\n",
                "Name = \"_DSC0001.ARW\",\n",
                "Rating = 3,\n",
                "ShouldProcess = 2,\n",
                "Uuid = \"A\",\n",
                "}\n",
                ",\n",
                "}\n",
                ",\n",
                "Uuid = \"B\",\n",
                "}\n",
                ",\n",
                "Version = \"21.0\",\n",
                "}\n",
            )
        );
    }

    #[test]
    fn a_fresh_reject_is_should_process_one() {
        let out =
            String::from_utf8(write_rating(None, Some(-1), false, "a", NOW).unwrap()).unwrap();
        assert!(out.contains("Rating = 0,\nShouldProcess = 1,\n"));
    }

    #[test]
    fn uuids_are_random_version_4() {
        let a = uuid();
        assert_eq!(a.len(), 36);
        assert_eq!(&a[14..15], "4");
        assert!(matches!(&a[19..20], "8" | "9" | "A" | "B"));
        assert_ne!(a, uuid());
    }

    #[test]
    fn a_rating_outside_the_range_is_an_error() {
        assert!(write_rating(None, Some(6), false, "a", NOW).is_err());
        assert!(write_rating(None, Some(-2), false, "a", NOW).is_err());
    }

    #[test]
    fn timestamps_are_photolab_formatted() {
        let t = UNIX_EPOCH + Duration::new(1_789_726_850, 47_183_700);
        assert_eq!(timestamp(t), "2026-09-18T10:20:50.0471837Z");
        let leap = UNIX_EPOCH + Duration::from_secs(1_709_164_800);
        assert_eq!(timestamp(leap), "2024-02-29T00:00:00.0000000Z");
        assert_eq!(timestamp(UNIX_EPOCH), "1970-01-01T00:00:00.0000000Z");
    }

    fn labelled(source: &str, label: Option<&str>) -> String {
        String::from_utf8(write_label(Some(source.as_bytes()), label, "x", NOW).unwrap()).unwrap()
    }

    fn tabbed_with_now(source: &str) -> String {
        with_now(
            source,
            "2026-09-18T23:03:06.0160000Z",
            "2026-09-18T12:50:44.9840000Z",
        )
    }

    #[test]
    fn reads_the_labels_of_the_photolab_10_0_2_samples() {
        for (source, label, rating) in LABELLED {
            assert_eq!(
                read_label(source.as_bytes()).unwrap().as_deref(),
                Some(label)
            );
            assert_eq!(read_rating(source.as_bytes()).unwrap(), Some(rating));
            assert!(!read_pick(source.as_bytes()).unwrap());
        }
    }

    #[test]
    fn reads_the_labels_of_the_photolab_10_0_1_samples() {
        assert_eq!(read_label(RED.as_bytes()).unwrap().as_deref(), Some("Red"));
        for source in [PICK, REJECT, THREE, CLEARED] {
            assert_eq!(read_label(source.as_bytes()).unwrap(), None);
        }
    }

    #[test]
    fn empty_or_unquoted_labels_read_as_absent() {
        let empty = RED.replace("ColorLabel = \"Red\",", "ColorLabel = \"\",");
        assert_eq!(read_label(empty.as_bytes()).unwrap(), None);
        let bare = RED.replace("ColorLabel = \"Red\",", "ColorLabel = 3,");
        assert_eq!(read_label(bare.as_bytes()).unwrap(), None);
    }

    #[test]
    fn setting_a_label_splices_only_the_value_and_the_timestamps() {
        let expected = with_now(
            RED,
            "2026-09-18T10:21:52.2425322Z",
            "2026-09-18T10:21:52.2405327Z",
        )
        .replace("ColorLabel = \"Red\",", "ColorLabel = \"Blue\",");
        assert_eq!(labelled(RED, Some("Blue")), expected);

        let expected =
            tabbed_with_now(TABBED).replace("ColorLabel = \"Red\",", "ColorLabel = \"Blue\",");
        assert_eq!(labelled(TABBED, Some("Blue")), expected);
    }

    #[test]
    fn inserts_a_label_line_in_an_unindented_item() {
        let out = labelled(THREE, Some("Blue"));
        assert!(out.contains(
            "Uuid = \"B638CACC-6B37-4480-8A40-9377C1C2783A\",\nColorLabel = \"Blue\",\n}\n,\n}\n"
        ));
        assert_eq!(read_label(out.as_bytes()).unwrap().as_deref(), Some("Blue"));
        assert_eq!(read_rating(out.as_bytes()).unwrap(), Some(3));
    }

    #[test]
    fn inserts_a_label_line_in_a_tab_indented_item() {
        let source = TABBED.replace("\t\t\tColorLabel = \"Red\",\n", "");
        assert_eq!(read_label(source.as_bytes()).unwrap(), None);
        let out = labelled(&source, Some("Blue"));
        assert!(out.contains(
            "\t\t\tUuid = \"7C7CB8FF-24DD-43A1-83CC-19ACD6642103\",\n\t\t\tColorLabel = \"Blue\",\n\t\t\t},\n"
        ));
        assert_eq!(read_label(out.as_bytes()).unwrap().as_deref(), Some("Blue"));
    }

    #[test]
    fn clearing_removes_exactly_the_label_line() {
        let out = labelled(RED, None);
        let expected = with_now(
            RED,
            "2026-09-18T10:21:52.2425322Z",
            "2026-09-18T10:21:52.2405327Z",
        )
        .replace("ColorLabel = \"Red\",\n", "");
        assert_eq!(out, expected);
        assert_eq!(out.matches("\nLabel = \"").count(), 8);
        assert!(out.ends_with("}\n\r\n"));

        let out = labelled(TABBED, None);
        let expected = tabbed_with_now(TABBED).replace("\t\t\tColorLabel = \"Red\",\n", "");
        assert_eq!(out, expected);
        assert_eq!(out.matches("\tLabel = \"").count(), 8);
        assert!(out.ends_with("}\n"));
        assert_eq!(read_label(out.as_bytes()).unwrap(), None);
    }

    #[test]
    fn clearing_an_absent_label_only_touches_the_timestamps() {
        let expected = with_now(
            THREE,
            "2026-09-18T10:21:52.2415388Z",
            "2026-09-18T10:21:52.2405327Z",
        );
        assert_eq!(labelled(THREE, None), expected);
    }

    #[test]
    fn labels_and_ratings_keep_each_other() {
        let rated = patched(&labelled(THREE, Some("Green")), Some(5), false);
        assert_eq!(
            read_label(rated.as_bytes()).unwrap().as_deref(),
            Some("Green")
        );
        assert_eq!(read_rating(rated.as_bytes()).unwrap(), Some(5));

        let labelled = labelled(&patched(TABBED, Some(2), true), Some("Pink"));
        assert_eq!(
            read_label(labelled.as_bytes()).unwrap().as_deref(),
            Some("Pink")
        );
        assert_eq!(read_rating(labelled.as_bytes()).unwrap(), Some(2));
        assert!(read_pick(labelled.as_bytes()).unwrap());
    }

    #[test]
    fn a_fresh_label_is_the_template_plus_the_line() {
        let bytes = write_label(None, Some("Red"), "a", NOW).unwrap();
        assert!(
            String::from_utf8_lossy(&bytes).contains("Rating = 0,\nShouldProcess = 2,\nUuid = ")
        );
        assert_eq!(read_label(&bytes).unwrap().as_deref(), Some("Red"));
        assert_eq!(read_rating(&bytes).unwrap(), Some(0));
        assert!(write_label(None, None, "a", NOW).is_err());
    }
}
