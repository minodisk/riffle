# Learnings

## Step 1: Read the new Sony MakerNote tags in core

- Observed tag types on the ILCE-7M5 (`_DSC0001.ARW` in the `sony flags`
  samples, dumped from the MakerNote IFD; the other four files read the same
  values through `reader::read_metadata`):
  - `AFAreaModeSetting` 0x201c: BYTE, count 1, value 13 (Custom AF Area), so
    `byte()` is right here, unlike 0x201a which turned out to be a LONG.
  - `RAWFileType` 0x2029, `MeteringMode2` 0x202c, `ExposureMode` 0xb041,
    `ReleaseMode` 0xb049, `SequenceNumber` 0xb04a: SHORT, count 1.
  - `DynamicRangeOptimizer` 0xb025, `ImageStabilization` 0xb026: LONG,
    count 1.
  - `CreativeStyle` 0xb020: ASCII, count 16 (`"Standard"` padded with NULs),
    out of line at a TIFF-relative offset.
- Values on all five files: af_area_mode 13, release_mode 2,
  sequence_number 2, 1, 2, 3, 1, image_stabilization 1, exposure_mode 15,
  metering_mode 256, creative_style "Standard", dynamic_range_optimizer 3,
  raw_file_type 2. All five were shot in Custom AF Area (13), so
  only that AF area value has been seen on a real file.
- The MakerNote sits at offset 5358 (length 38412), well inside the 1 MiB
  `HEAD_LIMIT` prefix; `arw::parse` on the prefix alone yields the same
  values, so no full-file fallback is needed.
- The `DynamicRangeOptimizer` table in Sony.pm (checked): 0 Off, 1 Standard,
  2 Advanced Auto, 3 Auto, 8-12 Advanced Lv1-Lv5, 16-23 Lv1-Lv8.
- Writing `b"Standard\0"` through a bash heredoc into a Python script
  turned the `\0` into a real NUL byte in the Rust source; fixed by hand.

## Step 2: Format the AF fields in `exif.rs` and show them in the meta pane

- `Exif` is also serialized into every `IndexedFile` of the folder listing
  (`index.rs` rebuilds it from the cached columns with `..Shot::default()`),
  so the three new fields travel there as `null` for every file. The
  frontend's `Exif` interface in `crates/app/ui/src/exif.ts` does not mirror
  them and does not need to; the meta pane reads them from `Metadata`.
- The Sony fixture in `meta.test.ts` now carries sample labels for the three
  rows, so the existing "splits Sony-like input" test asserts their order
  after `Shutter type`.

## Deferred issues (todo candidates)

- The folder listing payload carries `focus_mode` / `af_tracking` / `af_area`
  (and, after Step 3, seven more Maker note fields) as always-`null` members
  of `Exif`, since the index does not cache them. Basis: Step 2
  implementation (the plan puts the fields on `Exif`). Options: move the
  Maker note formatting out of `Exif` into a separate struct used only by
  `read_metadata`, or skip serializing `None`. Files:
  `crates/app/src/exif.rs`, `crates/app/src/index.rs`,
  `crates/app/ui/src/exif.ts`.
- `crates/core/src/sharpness.rs`'s `shot.focus_mode == Some(MANUAL_FOCUS)`
  read (used to decide the sharpness-score strategy) has no model gate, so on
  the older `DSC-` bodies ExifTool excludes from `FocusMode` (value always
  0), it is misread as manual focus. Basis: review feedback, Round 1 item 1
  (`docs/plans/review-history/sony-af-meta-step-2/review-20260925-2138.md`).
  Pre-existing, out of scope for this plan. File: `crates/core/src/sharpness.rs`.
