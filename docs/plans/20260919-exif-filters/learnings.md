# Learnings

## Step 1

- `scan::Entry` now carries `shot: Shot` in place of the three copied
  fields; `Shot` gained `Clone` because `Entry` derives it. Only
  `write_batch` and the index test fixture had to follow (`..Shot::default()`).
- `exif::Exif` / `Labelled` derive `serde::Serialize` already, ahead of
  Step 2's `IndexedFile.exif`; it also keeps the `value` fields from tripping
  `dead_code` while only the pane (which reads labels) consumes them.
- The estimated aperture's `value` is the raw estimate (e.g. 1.9999), while
  its label rounds to one decimal, so two estimates with the same label can
  differ in value. Step 3 should group by label, not value.
- `decimal` stays `pub` in `exif.rs` because `read_metadata` still formats
  the exposure bias with it.

## Step 2

- The v2/v3 -> v4 migration drops and recreates `files` rather than
  `DELETE` + one `ALTER TABLE` per new column: the result is the same (no
  rows, new layout), `CREATE TABLE IF NOT EXISTS` stays the single source
  of the layout, and a v2 database takes the same path. The whole `prepare`
  schema work now runs in one transaction.
- The stored columns are the raw `Shot` fields (rationals as num/den,
  the estimated f-number as `REAL`); `entries()` rebuilds a `Shot` and calls
  `exif::exif`, so the labels come from the single formatter. An error row
  is detected with `error IS NOT NULL` and yields `exif: None`.

## Step 3

- The EXIF items live in a `#filter-exif` container after the stars items and
  are rebuilt wholesale from `entries` on every landed `refreshEntries()`; one
  delegated `click` listener on the container handles them, so the static
  `filterItems` NodeList stays as it was.
- Selections are keyed by label (aperture estimates can share a label with
  different values); focal length is keyed by its range label, sorted by range
  index. Numeric groups sort by value with the label as a tiebreak.
- The EXIF sets are cleared in `openDirectory` before `files = found.filter(passes)`,
  so the first filtered list of a new folder is not narrowed by the old one.
- The manual GUI confirmation (ARW folder and M11-P DNG folder) is pending the
  user; Step 4 records the result in the README.
