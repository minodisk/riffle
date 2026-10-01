# Learnings: big-endian DNG

## Step 1

### Implementation

- `arw::parse` builds `sequence::Tiff::new(buf, 0, buf.len())` and every read
  goes through its `u16` / `u32` / `bytes`. The old range-check messages
  (`IFD offset out of range`, `IFD entry out of range`, `next IFD offset out of
  range`, `ASCII value out of range`, `RATIONAL value out of range`,
  `SHORT[N] value out of range`, `SubIFD offset array out of range`,
  `MakerNote out of range`) are kept by mapping the `Tiff` range error to
  them.
- The `Entry` value field stays a `u32` decoded in the file's order. Two
  helpers recover the inline bytes: `inline_bytes` (the four bytes in file
  order: ASCII of four bytes or fewer, BYTE as `[0]`, the Sigma `SHORT[2]`) and
  `inline_short` (`& 0xFFFF` for `II`, `>> 16` for `MM`; `integer()` and the
  IFD0 orientation use it).
- For an `II` file the orientation is still `value as u16` (= `& 0xFFFF`), so
  nothing changes there.
- The header check is now `Tiff::new`'s: `invalid TIFF byte order`,
  `invalid TIFF magic number` or `TIFF header too short`. The magic-42 check is
  new. On the samples it only changes the message of three clusterfuzz files
  that already failed (`IFD entry out of range` / `IFD offset out of range` →
  `invalid TIFF magic number`).
- Tests: `rejects_big_endian_header` became `reads_a_big_endian_container`
  (IFD0 orientation, Make / Model, an offset ASCII `DateTimeOriginal`, an
  inline 3-byte `SubSecTimeOriginal`, three RATIONALs, an inline ISO `SHORT`
  with nonzero padding in the low half), plus
  `reads_inline_bytes_and_shorts_of_a_big_endian_maker_note` (BYTE and SHORT
  in a headerless note without Make),
  `reads_the_leica_focus_distance_through_a_big_endian_note`,
  `picks_dng_strip_jpegs_out_of_a_big_endian_sub_ifd_array` (a local `ifd_be`
  that writes `MG` padding after each inline SHORT), and
  `rejects_an_unknown_byte_order` so the negative case is still covered.
  `reader.rs`'s `arw_with_both` now takes a `W`, so `inv.DNG` is a real `MM`
  container.
- `EXTRACTOR_VERSION` 11 → 12.

### The sample run

- The sample tree changed between the planning run and this step: another
  session was adding files to `D:\Photos\samples` while the first
  `check-before.txt` was being written (the DNG folder's mtime moved during
  the run). To compare on the same tree, the pre-change release binary was
  copied aside and the two binaries were run back to back. `check-before.txt`
  and `check-after.txt` are that pair (2417 files in both).
- Before: 58 files fail with `not a little-endian TIFF/ARW`, not 42. 52 of
  them start with `MM` (the plan's 42 plus files added since, such as the
  iPhone 13 Pro, 15 Pro and 17 Pro Max, iPhone XR and Samsung GX20
  `SG204732`). The other 6 are not TIFFs at all: 4 clusterfuzz files (`FUJI`,
  `\0MRM`) and 2 JPEGs named `.dng` (`Unknown_sample.dng`,
  `iPhone_12_Pro_Max_thing.dng`); they now fail with `invalid TIFF byte order`.
- After: no `not a little-endian TIFF/ARW` line remains. DNG 241 ok / 100
  failed → 275 ok / 66 failed. The arw / cr3 / jpg / jpeg / nef / orf / raf
  lines and tallies are identical. The only differing DNG lines from files
  that are not `MM` are the 6 non-TIFFs and 3 clusterfuzz files above. The
  three libjpeg stderr lines at the top come out in a different order; that
  is thread interleaving.

Of the 52 `MM` files, 34 now open at every stage (scan, preview, full):

- Pentax / Samsung: 645D, K-01, K-30, K-500, K-50 (3 files), K-5, K-5 II,
  K-5 II s, K-7, K-x, K10D, K200D, K20D, K-r, GX10, GX20 (2 files).
- Ricoh: GR, GR II, GXR.
- Leica: M (Typ 240) compressed and uncompressed, M Monochrom (Typ 246).
- Apple: iPhone 6s Plus, 7 Plus, SE, XS, 12 Pro, 13 Pro, 13 Pro Max,
  15 Pro (ProRAW dark thumbnail), 17 Pro Max.

18 still fail, each for a reason specific to the file:

- `no embedded preview` (scan and preview; `full` is skipped because there is
  none): GR DIGITAL 2, GX200, PENTAX K-5 32-bit float HDR, Blackmagic Pocket
  Cinema Camera, OpticFilm 8100 and 8200i (4 scans), One `IMG_20150729_201116`,
  SHIELD Tablet K1 `-hdr`, `Unknown_5omBHhwSqrjN4GX7LPVgwbpA486`, iPhone 8,
  iPhone XR. These carry no YCbCr JPEG strip, so the camera-support rule
  applies, not this plan.
- `preview out of range` / `JpgFromRaw out of range`: the 4 RICOH GR DIGITAL 4
  samples. The files are truncated. ExifTool puts `_08`'s preview at 15269888
  with length 115200, which ends at 15385088, past the 15357447-byte file.
- `panic while encoding the thumbnail` / `panic while decoding the JPEG`:
  `Canon_EOS_350D_DIGITAL_DNG.dng`, a 13 KB minimized sample whose YCbCr
  strips are 26 bytes (ExifTool `PreviewImageLength = 26`).

### ExifTool cross-check (13.59)

Riffle's `Shot` (dumped with a throwaway test, since removed) against
`exiftool -n`:

| File | Field | Riffle | ExifTool |
| --- | --- | --- | --- |
| `K-5_II_IMGP2032.DNG` | Orientation | 1 | 1 |
| | DateTimeOriginal | 2017:01:14 20:17:46 | same |
| | FNumber | 40/10 | 4 |
| | ExposureTime | 1/60 | 0.01667 |
| | ISO | 400 | ExifIFD 400 (Pentax note index 13) |
| | FocalLength | 3500/100 | 35 |
| | Make / Model | `PENTAX` / `PENTAX K-5 II` (space-padded) | same |
| `iPhone_12_Pro_IMG_1361.DNG` | Orientation | 6 | 6 |
| | DateTimeOriginal / SubSec | 2020:12:29 14:24:45 / 700 | same |
| | FNumber | 8/5 | 1.6 |
| | ExposureTime | 1/5556 | 0.00017999 |
| | ISO | 32 | 32 |
| | FocalLength | 21/5 | 4.2 |
| | Make / Model | Apple / iPhone 12 Pro | same |
| `M_(Typ_240)_..._L1006373.DNG` | Orientation | 1 | 1 |
| | DateTimeOriginal | 2018:02:07 17:00:57 | same |
| | FNumber | none, estimated 8.0 from ApertureValue | no FNumber, ApertureValue 8 |
| | ExposureTime | 1/250 | 0.004 |
| | ISO | 200 | 200 |
| | FocalLength | 0/1 | 0 |
| | Make / Model | Leica Camera AG / LEICA M (Typ 240) | same |
| | Leica FocusDistance | 0 | 0 |
| `M_Monochrom_(Typ_246)_M2462362.DNG` | Leica FocusDistance | 3310 | 3310 |
| | ExposureTime / ISO / FocalLength | 1/500 / 1600 / 50 | 0.002 / 1600 / 50 |
| | estimated FNumber | 12.996 | ApertureValue 12.996 |

The M240 sample records FocusDistance 0 (lens `not selected`), so the M246
gives the non-zero check of the big-endian Leica note. Both notes start with
`4c45 4943 4100 02ff 001f 0300 0007` (ExifTool `MakerNoteLeica6` for the M240,
`MakerNoteLeica7` for the M246).

## Deferred issues (todo candidates)

- Pending manual check: open a folder of big-endian DNGs (for example the
  Pentax K-5 II, iPhone 12 Pro and Leica M (Typ 240) samples under
  `D:\Photos\samples\DNG\`) in the Riffle app on Windows, on an index written
  by `EXTRACTOR_VERSION` 11. Expected: the rows that cached `not a
  little-endian TIFF/ARW` are re-extracted, and the thumbnails, the preview
  and the 1:1 view show. The step's checkbox was ticked on the automated
  criteria (`check` run, unit tests, `mise run ci`). Basis: the version bump
  in `crates/app/src/index.rs`; this step did not open the app.
