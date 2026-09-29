# Learnings: canon-nikon-raw

## Step 1: shared Exif reader

- The IFD0 + Exif IFD reading moved from `jpeg.rs` to the crate-private
  `crates/core/src/exif.rs` (`mod exif;` in `lib.rs`, not `pub`). It exposes
  `read(tiff, ifd0)` for a single TIFF and `read_ifd0` / `read_exif_ifd` for
  a container that keeps the two IFDs in separate TIFFs (CR3's `CMT1` /
  `CMT2`); `read_ifd0` returns the orientation and the Exif IFD offset.
  `read_exif_ifd` clears `estimated_f_number` itself, so feed all the Exif
  IFD entries in one call.
- The tag and type constants (`TAG_*`, `TYPE_*`) and `ascii` / `rational` /
  `integer` are `pub(crate)` in `exif.rs`. `jpeg.rs`'s tests reach them with
  `use crate::exif::*;` (the only change to that test module besides
  visibility), so their bodies are unchanged.
- The synthetic TIFF writer stays in `jpeg::tests`: `W(little_endian)`, the
  `Field` type and all of `W`'s methods (`u16`, `u32`, `ascii`, `short`,
  `long`, `rational`, `srational`, `tiff`) are now `pub(crate)`, alongside
  `plain_jpeg` and `with_exif`. `W::tiff` always appends an 0x8769 pointer to
  IFD0; a caller that wants a lone IFD (e.g. CR3's `CMT2`) can pass its
  entries as `ifd0` and ignore that pointer, as `exif::tests` does. NEF's
  SubIFDs / MakerNote will need a more general builder or hand-built bytes.

## Step 2: NEF parser

- The plan assumed the preview would be the Nikon MakerNote `PreviewIFD`
  JPEG. On every sample it is 640x424 (570x375 on D3 / D40 / D90 / D7000 /
  D800 / Df, 564x372 on the D70), far below `PREVIEW_MIN_WIDTH`. Instead,
  every body from about 2012 on (D800, Df, D750, D5, D500, D850, all Z)
  writes a second JPEG SubIFD of 1620x1080 (1632x1080 on the D800) after the
  raw SubIFD. `nef::parse` takes the first JPEG SubIFD as `full` and the last
  as `preview`, else `full`; the MakerNote is not read at all, so there is no
  MakerNote preview test and no ISO fallback (ExifIFD 0x8827 is present on
  every sample).
- Byte count cannot pick the full JPEG: the Z 6 sample's full JPEG is
  3024x2016 in 432 KB while its 1620x1080 preview is 737 KB. The JPEG
  SubIFDs carry no width, and their SOF is often past the 1 MiB prefix, so
  the rule is by order.
- IFD0's 0x0201 / 0x0202 is a 160x120 thumbnail on the Z 9, Z 8 and Z f
  (absent on the others); only SubIFDs are searched.
- All Z bodies and the D500 / D750 / D850 / D5 / D7500 / D3500 are
  little-endian; D3, D40, D70, D90, D7000, D800 and Df are big-endian.
- A truncated SubIFD must be an error, not "no JPEG": `Tiff::ifd_entries`
  reads each entry's tag / type / count but not its value, so a buffer cut
  inside the last entry's value parsed fine and the lenient `integer` just
  dropped the JPEG. `parse` now reads each SubIFD's next-IFD link to prove
  the whole IFD is in the buffer.
- The Step 1 `W::tiff` writer always adds an 0x8769 pointer, which is fine
  for a NEF; `nef::tests::nef` builds the file twice (once to learn the
  TIFF's length, once with the SubIFD offsets pointing past it) and appends
  the inline-only SubIFDs. `reader` tests reuse it (`pub(crate)`).

### Sample verification (2026-09-29, local only, not committed)

Downloaded from raw.pixls.us (CC0) into the system temp dir. exiftool is
not installed here, so the EXIF rows were cross-checked against an
independent Python TIFF dump (DateTimeOriginal, SubSecTimeOriginal,
Orientation, ExposureTime, FNumber, ISO, FocalLength, LensModel), and
`riffle-cli info` / `bench` / `scan` / `faces` were run on each file.
`read_metadata` succeeded on a 1 MiB truncated copy of every file (IFD0,
the SubIFDs and the Exif IFD end by ~300 KB; the MakerNote ends by ~260 KB),
so no whole-file retry is needed for metadata. `scan` over the 12 whole
files: 0 errors, thumbnails ~21.8 KB mean.

Whole files (preview decoded, 1:1 decoded, capture time with sub-second):

| Body   | Preview   | Full      | Preview offset | Full decode |
| ------ | --------- | --------- | -------------- | ----------- |
| Z 9    | 1620x1080 | 8256x5504 | 330 KB         | 260 ms      |
| Z 8    | 1620x1080 | 8256x5504 | 360 KB         | 248 ms      |
| Z 7II  | 1620x1080 | 8256x5504 | 288 KB         | 232 ms      |
| Z 6II  | 1620x1080 | 6048x4024 | 291 KB         | 139 ms      |
| Z 6    | 1620x1080 | 3024x2016 | 1.43 MB        | 34 ms       |
| Z 5    | 1620x1080 | 6016x4016 | 283 KB         | 129 ms      |
| Z f    | 1620x1080 | 6048x4032 | 370 KB         | 122 ms      |
| Z fc   | 1620x1080 | 5568x3712 | 293 KB         | 102 ms      |
| Z 50   | 1620x1080 | 5568x3712 | 2.22 MB        | 185 ms      |
| Z 30   | 1620x1080 | 5568x3712 | 257 KB         | 101 ms      |
| D850   | 1620x1080 | 4128x2752 | 262 KB         | 61 ms       |
| D500   | 1620x1080 | 2784x1856 | 256 KB         | 27 ms       |

- The Z 30 sample is a portrait frame (Orientation 8); `riffle-cli faces`
  wrote it upright at 1080x1620. The Df (1 MiB prefix only) is Orientation 6.
- The Z 6 and Z 50 samples (and the Z 7 prefix) place the preview past the
  1 MiB prefix, so their preview costs the ranged read; the others' preview
  lies inside it. Preview decode is 14-26 ms on every body, so the 2/8
  thumbnail scale gives ~405x270 thumbnails as on ARW.
- The D850 / D500 samples are reduced RAW sizes, hence their small full JPEG.
- Z 6III is not on raw.pixls.us, so it is not verified.
- 1 MiB prefix only (metadata, preview / full offsets, sub-second; JPEGs not
  decoded): Z 7, D5, D750, D800, D7500, D3500, Df (two JPEG SubIFDs), and
  D3, D7000, D90, D40, D70 (one JPEG SubIFD, so the preview is the full
  JPEG). Only the 12 whole-file bodies above count as verified.

## Step 3: CR3 parser

- Layout confirmed on every sample: `ftyp` (`crx `), `moov`, the XMP `uuid`
  (`be7acfcb...`), the preview `uuid` (`eaf42b5e...`, 16 uuid bytes + 8
  bytes, then `PRVW`), sometimes `free`, then `mdat` with a 64-bit
  `largesize` header. `CMT1` / `CMT2` are little-endian TIFFs on every body.
- `PRVW` payload: u32 0, u16 1, u16 1620, u16 1080, u16, u32 JPEG length,
  then the JPEG (so the JPEG starts 24 bytes into the box). `THMB`: version
  byte + 3 flag bytes, u16 160, u16 120, u32 length, 4 bytes, then the JPEG.
- The `CRAW` sample entry in `stsd` is a VisualSampleEntry plus Canon
  fields; its sub-boxes (`JPEG`, or `CMP1` / `CDI1` / `IAD1` on the raw
  tracks) start 82 bytes into its payload. `stsz` gives the JPEG size in
  its `sample_size` field (count 1); `co64` the offset, which is the start
  of `mdat`'s data on every sample (the JPEG track is first).
- HDR PQ (HEIF) files: `PRVW` starts `01 00 00 00 00 02` and holds HEVC
  (`CISZ`...), `THMB` is version 1 with HEVC, and the first track's entry
  carries `HEVC` / `hvcC` / `GRID` instead of `JPEG`. All four R8 samples
  and R5 Mark II `CRAW.CR3` / `RAW.CR3` are like this; `parse` returns no
  preview and no full JPEG for them (the SOI check), so the reader errors
  "no embedded preview". R5 Mark II's `APS-C_CRAW.CR3` is a JPEG file.
- `CTBO` index 2 (the preview `uuid`'s offset) matched the box walk on every
  sample; it is not read.
- Top-level walk rule: `moov` must lie wholly in the buffer, and the walk is
  complete once it reaches `mdat` or has seen both `moov` and the `PRVW`
  header (its JPEG may run past a prefix). Anything else that runs out of
  buffer is an error, so a short prefix gets the reader's whole-file retry
  instead of a silent `THMB` fallback.
- Test fixture: `cr3::tests::Cr3` builds a whole file (largesize `free` pad
  before `moov` and a largesize `mdat`); `reader` tests reuse it, with a
  `HEAD_LIMIT`-sized preview for the ranged read and a `HEAD_LIMIT` pad for
  the whole-file retry.
- exiftool is still not installed; the EXIF rows were cross-checked against
  an independent Python dump of `CMT1` / `CMT2` (Model, Orientation,
  DateTimeOriginal, SubSecTimeOriginal, LensModel, ExposureTime, FNumber,
  FocalLength, ExposureBiasValue, ISO): all matched `parse`'s `Shot`.

### Sample verification (2026-09-29, local only, not committed)

Downloaded from raw.pixls.us (CC0) into the system temp dir. Checked with
`riffle-cli info` / `faces` / `bench` / `scan` and a scratch binary calling
`reader::read_metadata` / `read_preview` / `read_full` and decoding both
JPEGs. `read_metadata` also succeeded on a 1 MiB truncated copy of each
file. `scan` over the files: thumbnails ~17 KB mean, the only errors the two
HEIF files.

Whole files (preview decoded, 1:1 decoded, capture time with sub-second):

| Body              | Preview   | Full      | moov end | PRVW end | Full decode |
| ----------------- | --------- | --------- | -------- | -------- | ----------- |
| EOS R             | 1620x1080 | 6720x4480 | 31 KB    | 331 KB   | 422 ms      |
| EOS RP            | 1620x1080 | 6240x4160 | 26 KB    | 377 KB   | 370 ms      |
| EOS R3            | 1620x1080 | 6000x4000 | 39 KB    | 684 KB   | 364 ms      |
| EOS R5            | 1620x1080 | 8192x5464 | 35 KB    | 393 KB   | 607 ms      |
| EOS R5 Mark II \* | 1620x1080 | 5088x3392 | 55 KB    | 321 KB   | 178 ms      |
| EOS R6            | 1620x1080 | 5472x3648 | 37 KB    | 544 KB   | 294 ms      |
| EOS R6 Mark II    | 1620x1080 | 6000x4000 | 59 KB    | 703 KB   | 381 ms      |
| EOS R6 Mark III   | 1620x1080 | 6960x4640 | 49 KB    | 657 KB   | 506 ms      |
| EOS R7            | 1620x1080 | 6960x4640 | 38 KB    | 374 KB   | 415 ms      |
| EOS R10           | 1620x1080 | 6000x4000 | 52 KB    | 597 KB   | 380 ms      |
| EOS R50           | 1620x1080 | 6000x4000 | 37 KB    | 877 KB   | 437 ms      |
| EOS R50 V         | 1620x1080 | 6000x4000 | 44 KB    | 625 KB   | 386 ms      |
| EOS R100          | 1620x1080 | 6000x4000 | 28 KB    | 331 KB   | 330 ms      |

\* The R5 Mark II row is `APS-C_CRAW.CR3` (an APS-C crop, hence the smaller
full JPEG); its full-frame samples are HEIF and fail as described above.

- Portrait: the R10 `IMG_4361.CR3` sample is Orientation 8; `riffle-cli
  faces` wrote it upright at 1080x1620 and found the face.
- Preview decode is 20-30 ms on every body, so thumbnails come out ~405x270
  as on ARW.
- `moov` ends by 90 KB and the `PRVW` box by 941 KB (the HEIF R8 sample; the
  JPEG ones by 877 KB), so every sample's metadata and preview come from the
  1 MiB prefix. `HEAD_LIMIT` stays 1 MiB; the full JPEG is always a ranged
  read (it starts at `mdat`).
- Not verified: EOS R8 (every raw.pixls.us sample is HEIF), EOS R1 (not on
  raw.pixls.us). The Model string of the R5 Mark II and R6 Mark II is
  `Canon EOS R5m2` / `Canon EOS R6m2`.

## Step 4: strings, docs and the compatibility lists

- The README lists 13 CR3 and 12 NEF bodies, exactly the whole-file rows of
  the Step 2 / 3 tables. The EOS R5 Mark II is listed on the strength of its
  APS-C crop JPEG sample; the HEIF limitation is a README sentence rather
  than a per-body caveat, since it follows the HDR PQ setting, not the body.
  The EOS R8 stays unlisted (HEIF samples only).
- `docs/cameras.md` rows show AF point `–` for Canon / Nikon although the
  bodies record one; a sentence under the table says Riffle does not read it
  yet, since the page is titled "What the camera records".
- `NO_FILES_TEXT` became "This folder has no RAW or JPEG files.": a
  JPEG-only folder lists its JPEGs, so the empty state only shows when both
  are absent.
- The literal NUL in `docs/agents/raw-metadata-parsing.md` was fixed here
  (now `Nikon\0`). Neither Python's `bytes.replace` nor `perl -pe
  's/\x00/.../'` passed through the Bash tool changed it (the `\x00` escape
  seems to be mangled on the way); `perl -pi -e 'my $z=chr(0); ...'` worked.
- The two deferred Step 2 / 3 items (pre-2012 NEF preview fallback, HEIF
  CR3) were added to `todo.md` as their own sections in this step, as the
  caller asked, alongside the Tier 2 / Tier 3 update.
- `docs/performance.md` gained a short NEF / CR3 section from the Step 2 / 3
  numbers; it does not link the plan's learnings, since the plan folder moves
  under `_archived/` at wrap-up.

## Deferred issues (todo candidates)

The three items below were handled in Step 4 (the first two added to
`todo.md`, the NUL byte fixed directly); they are kept here for the record.

- NEFs from bodies older than about 2012 (D3, D40, D70, D90, D7000 on the
  samples) carry one JPEG SubIFD, so `preview` falls back to the full-size
  JpgFromRaw: each page turn decodes a 3000-5000 px JPEG and the fixed 2/8
  thumbnail scale gives large thumbnails, the same issue recorded for the
  SIGMA fp L. Basis: Step 2 sample survey. Files: `crates/core/src/nef.rs`,
  `crates/core/src/scan.rs` (`thumbnail_jpeg`).
- CR3 files shot with HDR PQ (HEIF) carry only HEVC images (`PRVW`, `THMB`
  and the first track), so Riffle cannot show them: the reader errors "no
  embedded preview" and the strip shows the file as failed. Every EOS R8
  sample and the full-frame EOS R5 Mark II samples on raw.pixls.us are
  like this. Supporting them needs an HEVC decoder (or the camera's JPEG
  track, which these files do not have). Basis: Step 3 sample survey.
  Files: `crates/core/src/cr3.rs`, `crates/core/src/decode.rs`.
- `docs/agents/raw-metadata-parsing.md` line "10 (after `Nikon `, ..." in
  the NEF section (from Step 2) contains a literal NUL byte inside the
  backticks (meant as `Nikon\0`), which makes grep treat the file as
  binary. Basis: noticed while editing the CR3 section in Step 3. File:
  `docs/agents/raw-metadata-parsing.md`.
