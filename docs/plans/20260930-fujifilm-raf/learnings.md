# Learnings: Fujifilm RAF support

## Step 1: RAF parser

- `jpeg.rs` gained `pub(crate) fn read_exif(buf) -> Result<(u16, Shot)>`: an
  error when `buf` holds no complete Exif segment, the defaults when the
  segment is complete but its TIFF is malformed. `jpeg::parse` calls it with
  `unwrap_or((1, Shot::default()))`, so its behavior and tests are unchanged.
  `raf::parse` treats a `read_exif` error as lenient (orientation 1) only
  when the whole JPEG lies inside the buffer; otherwise it is an error, so the
  reader's whole-file retry runs.
- `raf::parse` does not error when the JPEG's *end* runs past the buffer:
  that is the normal prefix case (every sample's JPEG runs past 1 MiB). The
  parser cannot know the file size; `reader::embedded_from` checks the end
  against it, as for NEF / CR3.
- Thumbnail decision: the embedded JPEG is 4416x2944 (X) / 4000x3000 (GFX),
  so the fixed 2/8 scale gave 1104x736 / 1000x750 thumbnails of ~111 KB mean
  (`riffle-cli scan` over the 46 samples). `scan::extract_unless` now uses
  `thumbnail_jpeg_near(.., JPEG_THUMBNAIL_LONG_EDGE)` for `.raf` as it does
  for JPEG files: 404 px, 19.4 KB mean, and the thumbnail encode drops from
  ~37-107 ms to ~16-60 ms per file. ARW / NEF / CR3 / DNG keep 2/8.
- The 1:1 view on RAF is that embedded JPEG: 4416x2944 against sensors of
  6240x4160 (26 MP) and 7728x5152 (40 MP), 4000x3000 against 8256x6192
  (GFX 50) / 11648x8736 (GFX 100). Step 2 states it in the README.
- A scratch `crates/core/examples/rafdump.rs` (deleted before committing)
  dumped each sample's `Shot` and timed the reads; Python scripts (in the
  session scratchpad) read the header, the JPEG's segments and the Fujifilm
  MakerNote independently.

### Sample verification (2026-09-30, local only, not committed)

46 raw.pixls.us RAFs of 22 bodies, saved under `D:\photos\samples\RAF\` (the
smallest compressed and the smallest uncompressed sample per body, plus the
X-T5 and X-M5 "16:9" samples; files named `<body>_<cmp|unc|16x9>_<id>.RAF`,
with the raw.pixls.us exiv2 `exifdata` text of all 77 RAFs of these bodies
under `exif\`).

- Every file parses from the 1 MiB prefix (`raf::parse(head)` ok) and
  through the reader; preview and 1:1 (the same JPEG) decode on all 46.
- Header: version `0201` on all; JPEG at `0x94` on all; M-RAW words
  (`0x48` / `0x4c`) zero on all, so no M-RAW sample exists among them. One
  JPEG per file, no MPF (APP2) segment.
- The JPEG's Exif segment ends at byte 65,600 (65,602 on the X-T3 and GFX 100)
  on every sample, so metadata always comes from the prefix; the JPEG itself
  (1.27 to 5.51 MiB) never fits, so every preview / 1:1 open is one ranged
  read (1-4 ms warm).
- The "16:9" samples are not 16:9: both JPEGs are 3:2 4416x2944 with no
  letterbox (X-T5: a 6000x4000 crop in `RawImageCroppedSize`; X-M5: a
  4992x3328 crop, the "1.25x" crop mode per the file name).
- Portrait: X-H2 (both samples, Orientation 6) and X100VI (both,
  Orientation 8). Riffle reads the same orientation as exiv2.
- Meta rows (Model, Orientation, DateTimeOriginal, SubSecTimeOriginal,
  LensModel, ExposureTime, FNumber, FocalLength, ExposureBiasValue, ISO)
  cross-checked against the raw.pixls.us exiv2 `exifdata` for all 46 files:
  460 values, no real mismatch. The two diffs are presentation only: exiv2
  trims the X-M5's leading spaces in `LensModel` (`  55mmF1.4      `), and
  prints the X-S10's `ExposureTime` 10/28 as `1/3 s`. The X100V / X100VI /
  GFX100RF (fixed lens) write no `LensModel`.
- Sub-second: present on GFX100 II, GFX100S II, GFX100RF, X100VI, X-T50,
  X-M5, X-E5 and X-T30 III (two digits); absent on the X-T3, X-T4, X-Pro3,
  X100V, X-S10, X-E4, X-T30 II, GFX 100, GFX100S, GFX50S II, X-H2, X-H2S,
  X-S20 and X-T5.
- `riffle-cli bench` over the 46: preview decode mean 90.6 ms (p95 120.7),
  full decode 92.1 ms, partial 512 px 15.6 ms, face detection 31.6 ms.
  `riffle-cli scan` (after the thumbnail change): 1 thread 10.20 s, 5 files/s,
  per file mean 221.8 ms / p95 290.2 ms; 24 threads 1.28 s, 36 files/s.
  `riffle-cli focusbox` reports "no FocusLocation" on every sample (no AF
  point is read yet), as on the Leica M11-P / SIGMA fp L.

| Sample | JPEG px | JPEG bytes | Exif ends at | Preview decode | Orientation | SubSec | FocusMode / AFMode / FocusPixel |
|---|---|---|---|---|---|---|---|
| `GFX100RF_cmp_8094.RAF` | 4000x3000 | 3,115,491 (2.97 MiB) | 65,600 | 64.3 ms | 1 | 02 | 0 / 1 / 1999, 1499 |
| `GFX100RF_unc_8093.RAF` | 4000x3000 | 3,143,340 (3.00 MiB) | 65,600 | 64.5 ms | 1 | 93 | 0 / 1 / 1999, 1499 |
| `GFX100S_II_cmp_7777.RAF` | 4000x3000 | 2,550,040 (2.43 MiB) | 65,600 | 58.2 ms | 1 | 22 | 0 / 1 / 1999, 2336 |
| `GFX100S_II_unc_7775.RAF` | 4000x3000 | 2,557,562 (2.44 MiB) | 65,600 | 58.9 ms | 1 | 89 | 0 / 1 / 1999, 2336 |
| `GFX100S_cmp_4497.RAF` | 4000x3000 | 3,428,976 (3.27 MiB) | 65,600 | 63.4 ms | 1 | – | 0 / 1 / 2001, 1501 |
| `GFX100S_unc_4494.RAF` | 4000x3000 | 3,300,757 (3.15 MiB) | 65,600 | 62.2 ms | 1 | – | 0 / 1 / 2001, 1501 |
| `GFX100_II_cmp_7176.RAF` | 4000x3000 | 1,381,550 (1.32 MiB) | 65,600 | 48.4 ms | 1 | 01 | 0 / 256 / 2278, 1500 |
| `GFX100_II_unc_7174.RAF` | 4000x3000 | 1,327,373 (1.27 MiB) | 65,600 | 48.5 ms | 1 | 42 | 0 / 256 / 2000, 1500 |
| `GFX50S_II_cmp_5004.RAF` | 4000x3000 | 3,266,957 (3.12 MiB) | 65,600 | 66.9 ms | 1 | – | 0 / 1 / 2001, 1501 |
| `GFX50S_II_unc_5002.RAF` | 4000x3000 | 3,238,675 (3.09 MiB) | 65,600 | 69.5 ms | 1 | – | 0 / 1 / 2001, 1501 |
| `GFX_100_cmp_3775.RAF` | 4000x3000 | 3,056,164 (2.91 MiB) | 65,602 | 60.2 ms | 1 | – | 1 / 0 / 1441, 1221 |
| `GFX_100_unc_3772.RAF` | 4000x3000 | 3,122,523 (2.98 MiB) | 65,602 | 62.2 ms | 1 | – | 1 / 0 / 1441, 1221 |
| `X-E4_cmp_4448.RAF` | 4416x2944 | 4,050,554 (3.86 MiB) | 65,600 | 78.2 ms | 1 | – | 0 / 1 / 2208, 1472 |
| `X-E4_unc_4449.RAF` | 4416x2944 | 4,083,760 (3.89 MiB) | 65,600 | 147.4 ms | 1 | – | 0 / 1 / 2208, 1472 |
| `X-E5_cmp_8509.RAF` | 4416x2944 | 5,610,122 (5.35 MiB) | 65,600 | 139.4 ms | 1 | 54 | 0 / 1 / 1284, 1471 |
| `X-E5_unc_8507.RAF` | 4416x2944 | 5,670,423 (5.41 MiB) | 65,600 | 110.3 ms | 1 | 38 | 0 / 1 / 1284, 1471 |
| `X-H2S_cmp_6009.RAF` | 4416x2944 | 5,265,240 (5.02 MiB) | 65,600 | 83.6 ms | 1 | – | 0 / 1 / 2207, 1472 |
| `X-H2S_unc_6008.RAF` | 4416x2944 | 5,268,586 (5.02 MiB) | 65,600 | 83.3 ms | 1 | – | 0 / 1 / 2207, 1472 |
| `X-H2_cmp_6001.RAF` | 4416x2944 | 4,801,080 (4.58 MiB) | 65,600 | 86.3 ms | 6 | – | 0 / 1 / 1592, 1472 |
| `X-H2_unc_6003.RAF` | 4416x2944 | 4,886,902 (4.66 MiB) | 65,600 | 83.8 ms | 6 | – | 0 / 1 / 1592, 1472 |
| `X-M5_16x9_7750.RAF` | 4416x2944 | 5,544,792 (5.29 MiB) | 65,600 | 84.8 ms | 1 | 59 | 0 / 1 / 2399, 1471 |
| `X-M5_cmp_7749.RAF` | 4416x2944 | 5,572,638 (5.31 MiB) | 65,600 | 86.0 ms | 1 | 48 | 0 / 1 / 2360, 1471 |
| `X-M5_unc_7747.RAF` | 4416x2944 | 5,757,280 (5.49 MiB) | 65,600 | 89.3 ms | 1 | 24 | 0 / 1 / 2360, 1471 |
| `X-Pro3_cmp_3701.RAF` | 4416x2944 | 3,299,252 (3.15 MiB) | 65,600 | 72.7 ms | 1 | – | 0 / 1 / 2362, 1781 |
| `X-Pro3_unc_3700.RAF` | 4416x2944 | 3,147,711 (3.00 MiB) | 65,600 | 76.0 ms | 1 | – | 0 / 1 / 2362, 1317 |
| `X-S10_cmp_4190.RAF` | 4416x2944 | 2,816,149 (2.69 MiB) | 65,600 | 71.6 ms | 1 | – | 0 / 0 / 2826, 1781 |
| `X-S10_unc_4188.RAF` | 4416x2944 | 2,809,878 (2.68 MiB) | 65,600 | 73.2 ms | 1 | – | 0 / 0 / 1281, 1472 |
| `X-S20_cmp_6668.RAF` | 4416x2944 | 5,504,730 (5.25 MiB) | 65,600 | 82.3 ms | 1 | – | 0 / 512 / 1329, 1065 |
| `X-S20_unc_6667.RAF` | 4416x2944 | 5,496,324 (5.24 MiB) | 65,600 | 80.0 ms | 1 | – | 0 / 512 / 1260, 1223 |
| `X-T30_III_cmp_8771.RAF` | 4416x2944 | 5,558,990 (5.30 MiB) | 65,600 | 84.6 ms | 1 | 05 | 0 / 1 / 2206, 1470 |
| `X-T30_III_unc_8770.RAF` | 4416x2944 | 5,464,852 (5.21 MiB) | 65,600 | 83.4 ms | 1 | 17 | 0 / 1 / 2206, 1470 |
| `X-T30_II_cmp_5064.RAF` | 4416x2944 | 3,577,036 (3.41 MiB) | 65,600 | 72.2 ms | 1 | – | 0 / 1 / 1899, 1472 |
| `X-T30_II_unc_5066.RAF` | 4416x2944 | 3,692,158 (3.52 MiB) | 65,600 | 73.6 ms | 1 | – | 0 / 1 / 1899, 1472 |
| `X-T3_cmp_2783.RAF` | 4416x2944 | 2,776,772 (2.65 MiB) | 65,602 | 69.9 ms | 1 | – | 1 / 0 / 2517, 1472 |
| `X-T3_unc_2784.RAF` | 4416x2944 | 2,667,229 (2.54 MiB) | 65,602 | 70.9 ms | 1 | – | 1 / 0 / 2517, 1472 |
| `X-T4_cmp_3918.RAF` | 4416x2944 | 4,239,974 (4.04 MiB) | 65,600 | 79.3 ms | 1 | – | 0 / 1 / 2517, 1472 |
| `X-T4_unc_3913.RAF` | 4416x2944 | 4,174,032 (3.98 MiB) | 65,600 | 77.3 ms | 1 | – | 0 / 1 / 2517, 1472 |
| `X-T50_cmp_7807.RAF` | 4416x2944 | 5,449,742 (5.20 MiB) | 65,600 | 80.3 ms | 1 | 38 | 0 / 256 / 1591, 2086 |
| `X-T50_unc_7805.RAF` | 4416x2944 | 5,728,689 (5.46 MiB) | 65,600 | 82.5 ms | 1 | 48 | 0 / 256 / 1898, 2086 |
| `X-T5_16x9_7271.RAF` | 4416x2944 | 3,687,013 (3.52 MiB) | 65,600 | 76.8 ms | 1 | – | 1 / 0 / 2207, 1472 |
| `X-T5_cmp_6123.RAF` | 4416x2944 | 3,847,648 (3.67 MiB) | 65,600 | 75.1 ms | 1 | – | 0 / 1 / 2207, 1472 |
| `X-T5_unc_6124.RAF` | 4416x2944 | 3,875,074 (3.70 MiB) | 65,600 | 75.9 ms | 1 | – | 0 / 1 / 2207, 1472 |
| `X100VI_cmp_7302.RAF` | 4416x2944 | 5,593,033 (5.33 MiB) | 65,600 | 85.4 ms | 8 | 17 | 0 / 256 / 2514, 855 |
| `X100VI_unc_7300.RAF` | 4416x2944 | 5,753,197 (5.49 MiB) | 65,600 | 85.5 ms | 8 | 84 | 0 / 256 / 2206, 1163 |
| `X100V_cmp_3812.RAF` | 4416x2944 | 5,618,448 (5.36 MiB) | 65,600 | 83.9 ms | 1 | – | 0 / 0 / 1590, 1781 |
| `X100V_unc_3813.RAF` | 4416x2944 | 5,781,050 (5.51 MiB) | 65,600 | 87.7 ms | 1 | – | 0 / 0 / 1281, 1163 |

### FocusPixel survey (for the Step 3 decision)

Read with an independent Python walk of the Fujifilm MakerNote (`FUJIFILM`,
IFD offset at 8, little-endian, note-relative offsets). Every sample writes
`0x1023 FocusPixel`, `0x1021 FocusMode` and `0x1022 AFMode` (table above; the
exiv2 dumps do not decode `FocusPixel`, so they cannot serve as the
reference here).

- The frame looks like the embedded JPEG's: centered points are
  (2207..2208, 1472) on 4416x2944 X bodies and (1999..2001, 1499..1501) on
  4000x3000 GFX bodies, i.e. the JPEG's center, not the sensor's
  (`RawImageCroppedSize` 6240x4160 / 7728x5152 / 11648x8736). So Step 3's
  `sensor_w` / `sensor_h` is most likely `PixelXDimension` /
  `PixelYDimension` (equal to the JPEG size on every sample), to be confirmed
  by drawing.
- Off-center landscape points exist: X-E5 (1284, 1471), X-S10 (2826, 1781)
  and (1281, 1472), X-S20 (1329, 1065) / (1260, 1223), X-T50 (1591, 2086) /
  (1898, 2086), X-Pro3 (2362, 1781) / (2362, 1317), X100V (1590, 1781) /
  (1281, 1163), GFX100 II (2278, 1500), GFX100S II (1999, 2336), X-M5 (2360,
  1471), X-T30 II (1899, 1472).
- Portrait: X-H2 (Orientation 6) at (1592, 1472) and X100VI (Orientation 8)
  at (2514, 855) / (2206, 1163), off-center in the stored frame, so the
  unrotated-coordinates question can be settled.
- "16:9" / crop: X-M5 crop sample (2399, 1471) in the same 4416x2944 frame;
  the X-T5 16:9 sample is manual focus.
- Manual focus (`FocusMode` 1, `AFMode` 0) on the X-T3, X-T5 16:9 and GFX 100
  samples still writes a `FocusPixel` (the X-T3's at (2517, 1472), the GFX
  100's at (1441, 1221)), so Step 3 must drop it on `FocusMode` 1 as planned.
  `AFMode` 0 with `FocusMode` 0 (X-S10, X100V) is autofocus with an area
  mode ExifTool does not name.
- Verdict: Step 3 looks viable on these samples (off-center landscapes and
  two portrait bodies), pending the drawn-point check.

## Deferred issues (todo candidates)

- RAF scan cost per file: `riffle-cli scan` over the 46 samples takes
  ~222 ms per file on one thread (the α7 V ARW: ~68 ms on 12 threads, `docs/performance.md`). The
  embedded JPEG is 4416x2944 / 4000x3000 (vs ARW's 1616x1080), it is decoded
  whole for the sharpness score, and with no AF point read the face search
  runs on the whole image. Step 3 (an AF point) removes the face search for
  AF frames; a scaled decode for the score would be a separate change.
  Basis: Step 1 sample survey. Files: `crates/core/src/scan.rs`,
  `crates/core/src/sharpness.rs`, `crates/core/src/raf.rs`.
- No M-RAW RAF among the raw.pixls.us samples of the target bodies (`0x48`
  zero on all 46); M-RAW parsing (the second header) stays unverified.
  Basis: Step 1 sample survey. Files: `crates/core/src/raf.rs`.
