# Parsing RAW metadata in `crates/core`

Read this before touching the TIFF / MakerNote parsing in
`crates/core/src/arw.rs`, which reads both ARW and DNG files (IFD0, the
SubIFDs, the ExifIFD, and the Sony, Leica and Sigma MakerNotes), or in
`crates/core/src/nef.rs`, which reads Nikon NEF files (see [NEF](#nef)), or in
`crates/core/src/cr3.rs`, which reads Canon CR3 files (see [CR3](#cr3)). It lists the
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

## NEF

`crates/core/src/nef.rs` parses a Nikon NEF over `sequence::Tiff`, the
byte-order-aware walker, and reads IFD0 and the Exif IFD through the shared
crate-private `crates/core/src/exif.rs` (also used by `jpeg.rs`); unlike
`arw.rs`, it keeps its own Exif fields lenient (an unreadable one is `None`)
but errors on any IFD or SubIFD that runs past the buffer, so a short prefix
triggers the reader's whole-file retry instead of a file with no JPEGs.

### The preview is a SubIFD JPEG, not IFD0's or the MakerNote's (Measured)

The embedded JPEGs are the SubIFDs (0x014a) with `Compression = 6` and
`JPEGInterchangeFormat` / `JPEGInterchangeFormatLength` (0x0201 / 0x0202).
`parse` takes the first as `full` (the JpgFromRaw) and, when there are more,
the last as `preview`; with one, it is both.

- Measured on raw.pixls.us NEFs of 24 bodies (Z 9 back to D70): the first
  JPEG SubIFD is always the full-size JPEG, and every body since about the
  D800 / Df adds a later one of 1620x1080 (1632x1080 on the D800). The
  MakerNote `PreviewIFD` (0x0011) JPEG is only 640x424 (570x375 on older
  bodies), below `PREVIEW_MIN_WIDTH`, so it is not read at all.
- IFD0 of the Z 9, Z 8 and Z f carries its own 0x0201 / 0x0202: a 160x120
  thumbnail. Taking IFD0's JPEG as the preview, as `arw.rs` does for ARW,
  would show a thumbnail; only SubIFDs are searched.
- Size or byte count cannot tell the two JPEGs apart: the JPEG SubIFDs carry
  no `ImageWidth`, and a Z 6 sample's full JPEG (3024x2016) is smaller in
  bytes than its 1620x1080 preview. Hence the order rule.
- The Z bodies are little-endian; older ones (D90, D3, D7000, Df, D800) are
  big-endian.
- Every Nikon MakerNote offset is relative to the TIFF header at note offset
  10 (after `Nikon `, two version bytes and two more). Nothing in the note
  is read today; a future reader (e.g. `AFInfo2`) must build a second
  `Tiff::new(buf, note + 10, note_end)`.

## CR3

`crates/core/src/cr3.rs` walks a Canon CR3's ISOBMFF boxes (big-endian sizes,
a 64-bit `largesize` after the type when the 32-bit size is 1) up to `mdat`,
and reads the two Exif TIFFs through `crates/core/src/exif.rs`: `CMT1`
(IFD0: `Make`, `Model`, `Orientation`) with `read_ifd0` and `CMT2` (whose
IFD0 *is* the Exif IFD) with `read_exif_ifd`, each over
`Tiff::new(buf, payload, box_end)`. Like `nef.rs`, it keeps the Exif fields
lenient but errors on any box that runs past the buffer or its parent.

### Where the JPEGs are (Measured)

- `preview`: the `PRVW` box inside the top-level `uuid`
  `eaf42b5e-1c98-4b88-b9fb-b7dc406e4d16`, after the 16 uuid bytes and 8
  more. Its payload is a 16-byte header (the data length is the big-endian
  u32 at 12) and then a 1620x1080 JPEG. Fallback: `THMB` in the Canon `uuid`
  (`85c0b687-820f-11e0-8111-f4ce462b6a48`, inside `moov`), 160x120, length
  at 8, data at 16; then `full`.
- `full`: the `trak` whose `stsd` sample entry (`CRAW`) carries a `JPEG`
  sub-box; the raw tracks carry `CMP1`. The sub-boxes start 82 bytes into the
  sample entry's payload. The size is `stsz`'s `sample_size` (or its first
  entry when that is 0) and the offset the first `co64` (or `stco`) entry.
  It is the full-resolution JPEG on every sample (8192x5464 on the R5).
- The Canon `CTBO` box lists the top-level offsets (index 2 is the preview
  `uuid`); it matched the walk on every sample and is not read.
- Measured on raw.pixls.us CR3s of 14 bodies (EOS R, RP, R3, R5, R5 Mark II,
  R6, R6 Mark II, R6 Mark III, R7, R8, R10, R50, R50 V, R100): `moov` ends by
  ~90 KB and the `PRVW` box by ~940 KB, so the 1 MiB `HEAD_LIMIT` prefix
  parses and slices the preview without a ranged read or a whole-file
  retry.

### HDR PQ (HEIF) files carry no JPEG (Measured)

With HDR PQ on, the body writes HEVC instead of JPEG in `PRVW`, `THMB` and
the first track (whose sample entry carries `HEVC` / `hvcC` instead of
`JPEG`). The `PRVW` / `THMB` headers differ too (`PRVW`'s first u32 is 1,
`THMB`'s version is 1). `parse` takes `PRVW` / `THMB` only when their data
starts with a JPEG SOI (`FF D8`), and a track only with a `JPEG` sub-box, so
such a file parses with no `preview` and no `full`, and the reader reports
"no embedded preview". Every R8 sample and two of the four R5 Mark II samples
on raw.pixls.us are like this.

## Related

- The detector that consumes these AF points runs on the upright preview and
  maps its results back to stored coordinates; see
  [Feed an upright image, map detections back to stored coordinates](./tract-onnx-inference.md#feed-an-upright-image-map-detections-back-to-stored-coordinates-hit).
