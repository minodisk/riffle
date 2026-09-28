# Parsing RAW metadata in `crates/core`

Read this before touching the TIFF / MakerNote parsing in
`crates/core/src/arw.rs`, which reads both ARW and DNG files (IFD0, the
SubIFDs, the ExifIFD, and the Sony, Leica and Sigma MakerNotes). It lists the
pitfalls this repository has already hit, each with the reason it happens.
`crates/core/src/reader.rs` only picks which bytes to read; no maker-specific
parsing lives there. For a new Sony MakerNote field, also read
[Sony MakerNote fields: verify each tag's type and model `Condition` in Sony.pm directly](./tauri-app.md#sony-makernote-fields-verify-each-tags-type-and-model-condition-in-sonypm-directly-hit).

The tags follow [`tauri-app.md`](./tauri-app.md): **Hit** broke something
here, **Measured** steered a design decision, **Inferred** comes from sources
or docs only.

## TIFF structure

### A single `SubIFDs` entry holds the IFD offset inline (Hit)

`SubIFDs` (0x014a) with `count == 1` carries the one SubIFD's offset in the
entry's value field; only with `count > 1` is the value an offset to an array
of IFD offsets. The `TAG_SUB_IFDS` branch of `parse` pushes the value itself
for `count == 1` and reads the array (range-checked) otherwise.

- Why: a LONG[1] fits in the entry's four value bytes, so TIFF stores it
  inline, like any value of four bytes or fewer.
- Testing: a synthetic-TIFF fixture with one SubIFD only exercises the inline
  path. Give it at least two SubIFDs (as `tiff_with_sub_ifds` in the tests
  does) to cover the array path.
- Source: [leica-dng-support learnings, Step 1](../plans/_archived/20260918-leica-dng-support/learnings.md#step-1).

### A count-1 `SHORT` entry's padding can be nonzero (Hit)

A `SHORT` with `count == 1` occupies the low two bytes of the entry's value
field; the other two bytes are padding, and some writers leave per-file
garbage there (SIGMA fp L DNGs do, on `Compression`, `PhotometricInterpretation`
and the like). `integer()` masks a count-1 `SHORT` with `& 0xFFFF`.

- What broke: without the mask, `strip_jpeg` compared a padded `Compression`
  / `PhotometricInterpretation` against the JPEG / YCbCr values and rejected
  the strip JPEGs, so a SIGMA fp L DNG got the wrong full-size JPEG or no
  embedded JPEG at all.
- Rule: read any single `SHORT` / `LONG` through `integer()` rather than
  using the raw value field. The regression test
  `short_entries_ignore_the_padding_in_their_high_half` ORs garbage into the
  high half.
- Source: [tiff-short-padding learnings, Step 1](../plans/_archived/20260924-tiff-short-padding/learnings.md#step-1).

## MakerNotes

### The Sony MakerNote gate skips only a known non-Sony `Make` (Hit)

`maker_note_ifd` reads the MakerNote (0x927c) as a Sony IFD unless the note
lacks the `SONY` header **and** `Make` is present and does not start with
`SONY`. A missing `Make` still gets the Sony parse.

- Why: recent Sony bodies (Sony5) start the IFD right at the tag's offset
  with no header, so "parse only when the note starts with `SONY` or `Make`
  starts with `SONY`" broke the existing synthetic fixtures, whose
  headerless Sony5-style notes carry no `Make`. Reading another maker's note
  as a Sony IFD walks garbage, hence the gate.
- Testing: a synthetic non-Sony note must set `Make` (the tests use
  `Leica Camera AG`); without it the Sony parse reads the note's first bytes
  (`LE`) as an entry count and errors.
- Source: [leica-dng-support learnings, Steps 1 and 3](../plans/_archived/20260918-leica-dng-support/learnings.md#step-1).

### Leica: `LEICA\0` header, an IFD at offset 8, `FocusDistance` in millimeters (Hit)

A Leica MakerNote (exiftool's `MakerNoteLeica9`) is `LEICA\0` plus two bytes
(`02 00`), then a plain little-endian IFD at note offset 8
(`LEICA_HEADER_LEN`). `leica_focus_distance` reads only `FocusDistance`,
tag 0x0304, a LONG count 1 (value inline), taken as millimeters. The note's
own `FNumber` is 1.0 on M-mount lenses and is not read.

- The aperture comes from the ExifIFD instead: `ApertureValue` (0x9202, APEX,
  RATIONAL) converts as `2^(AV/2)` into `estimated_f_number`, which `exif`
  clears when `FNumber` is present, so it is only used when `FNumber` is
  absent.
- Testing: a "non-Sony note is skipped" fixture needs a valid empty Leica IFD
  (`leica_note(&[])`). A `0xffff` entry count after the header, which the
  Sony-only parser used to ignore, is an out-of-range IFD error once the
  Leica parse reads it.
- Source: [leica-dng-support learnings, Step 3](../plans/_archived/20260918-leica-dng-support/learnings.md#step-3)
  (the exiftool comparison over the samples).

### Sony `FocusFrameSize` is `UNDEFINED[6]` on real files (Hit)

ExifTool documents `FocusFrameSize` (0x2037) as `int16u[3]` (width, height,
validity), but real α7 V files store it as `UNDEFINED[6]`; exiftool only
reinterprets the bytes. The `focus_frame` match in `exif` remaps
`(TYPE_UNDEFINED, 6)` to `(TYPE_SHORT, 3)` before `shorts::<3>` and still
accepts a genuine `SHORT[3]`. A zero validity SHORT gives `None`.

- What broke: a reader accepting only `SHORT[3]` returned `None` on a real
  file, so the eye-AF frame was never used.
- Rule: check a tag's type on a real file, not only in ExifTool's table; the
  table can describe the decoded value rather than the stored type.
- Source: [sony-eye-af-window learnings, Step 1](../plans/_archived/20260922-sony-eye-af-window/learnings.md#step-1).

### Sigma: inline values first, and `Make` differs by body (Hit)

`sigma_af_point` reads the Sigma BF's AF point, tag 0x0147, a `SHORT[2]`
that fits in the entry and so is read from the value field
(`value.to_le_bytes()`), never from `buf`.

- An entry whose value fits in the entry is inline, not an offset: with
  `count <= 4` the MakerNote tag's value field holds the note bytes
  themselves, so reading it as an offset lands anywhere (the test uses
  `0xffff_fff0`, past the buffer). And a note of `count <= header_len` has no
  room for an IFD after its header. So the size check comes before the
  offset / range check: `count <= SIGMA_HEADER_LEN` returns `None` first, and
  only a longer note whose range runs past the buffer bails. This is the
  same order `leica_focus_distance` uses with `LEICA_HEADER_LEN`. Test:
  `a_sigma_bf_maker_note_with_an_inline_value_is_none`.
- The same vendor writes `Make` differently by body: the BF writes `Sigma` /
  `Sigma BF`, the fp L `SIGMA` / `SIGMA fp L`. So the gate is a
  case-insensitive `SIGMA` prefix on `Make` plus an exact `Model` of
  `Sigma BF` (`SIGMA_BF_MODEL`).
- Source: [sigma-bf-af-point learnings, Step 1](../plans/_archived/20260924-sigma-bf-af-point/learnings.md#step-1)
  (the exiftool comparison and the grid check).

## Related

- The detector that consumes these AF points runs on the upright preview and
  maps its results back to stored coordinates; see
  [Feed an upright image, map detections back to stored coordinates](./tract-onnx-inference.md#feed-an-upright-image-map-detections-back-to-stored-coordinates-hit).
