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
