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

## Deferred issues (todo candidates)

- NEFs from bodies older than about 2012 (D3, D40, D70, D90, D7000 on the
  samples) carry one JPEG SubIFD, so `preview` falls back to the full-size
  JpgFromRaw: each page turn decodes a 3000-5000 px JPEG and the fixed 2/8
  thumbnail scale gives large thumbnails, the same issue recorded for the
  SIGMA fp L. Basis: Step 2 sample survey. Files: `crates/core/src/nef.rs`,
  `crates/core/src/scan.rs` (`thumbnail_jpeg`).
