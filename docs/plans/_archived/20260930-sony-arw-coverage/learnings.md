# Learnings: sony-arw-coverage

## Step 1: Sony ARW sample verification

### Method (2026-09-30, local only, not committed)

- All 66 `.ARW` files of the 16 candidate folders on raw.pixls.us (CC0) were
  downloaded with `curl -L` from the Apache listings into
  `D:\photos\samples\ARW\<MODEL>\` (one folder per model), which is how the
  verification below ran. The A9 II names need `(`, `)` and `:` URL-encoded.
  After the verification the files were flattened into
  `D:\photos\samples\ARW\` (next to the CR3 and NEF samples), each renamed
  `<Model>_<original name>` from the EXIF Model string (e.g.
  `ILCE-6700_DSC00002.ARW`), because default names such as `DSC00002.ARW`
  collide across bodies. The prefix is skipped when the original name already
  starts with `<Model>_` (the α7 IV files, e.g.
  `ILCE-7M4_DSC06673_FullFrame-Raw-Uncompressed.ARW`); the α7C
  `DSC00107%5b1%5d.ARW` is `ILCE-7C_DSC00107[1].ARW`.
- Each file went through `riffle-cli info`, `focusbox` and `bench` (per
  folder), `riffle-cli scan` ran over every folder, and `faces` over one
  file per listed body. exiftool is still not installed; an independent
  Python TIFF dump (IFD0, the IFD chain, the SubIFDs with each JPEG's SOF
  size, the Exif IFD and the Sony MakerNote) was compared against a scratch
  example printing `reader::read_metadata`'s `Shot` (deleted afterwards):
  DateTimeOriginal, SubSecTimeOriginal, ExposureTime, FNumber, ISO,
  FocalLength, ExposureBiasValue and LensModel matched on all 66 files.
- `read_metadata` succeeded on a 1 MiB truncated copy of every file. The
  MakerNote ends by 43 KB and the IFD0 preview by 926 KB (the α7S III
  `DSC01569.ARW`; every other file by 830 KB), so metadata and the preview
  always lie inside `HEAD_LIMIT`; the full-size JPEG starts at 491-950 KB
  and runs past it, which is the ranged read `read_full` already does.
- `scan`: 0 errors in every folder.

### Findings

- Every MakerNote is headerless (Sony5): `maker_note_ifd`'s 12-byte
  `SONY DSC` path is not taken on any of these bodies, the α6400 / α6600
  included.
- Older bodies write no full-size JPEG. On the α9 II, α7R IV, α7R IVA, α7C,
  α6400, α6600 and ZV-E10 the IFD chain holds only IFD0 (the 1616x1080
  preview) and IFD1 (a 160x120 thumbnail); the SubIFD is the raw data. So
  `parse` takes the thumbnail as `full` (the `bench` full decode is 0.2 ms)
  and the 1:1 view would crop a 160x120 image. These bodies are left off the
  README (see the deferred issue below). The recent bodies add IFD2, the
  full-size JPEG.
- The α6400, α6600, α7R IV and α7R IVA write no `SubSecTimeOriginal`; the
  α9 II, α7C and ZV-E10 do.
- `FocusFrameSize` (0x2037) exists only on the recent bodies, always as
  `UNDEFINED[6]`; the older ones have no 0x2037.
- `AFTracking` (0x2021) never reads 1 (face tracking) on these samples. It
  reads 2 (lock-on AF) on the α7 IV, α6400, α9 II and ZV-E10 samples, 0
  elsewhere. `docs/cameras.md` therefore marks Face tracking `–` for every
  new body, with a note that `–` there means "not observed on the sample".
- Crop mode reads as (a) in the plan: `FocusLocation` is already in the crop
  frame. The α7 IV APS-C files give 4608x3072, the size of their full JPEG
  (4608x3072), and the α7R V S35 files 6240x4160 (full JPEG 6240x4160; the
  S35 Small file 4752x3168, full JPEG 4752x3168). The drawn point lands near
  the frame center on the crop preview as on the full-frame shot of the same
  scene; the scenes are foliage and a lawn in wide-area AF-C, so the point
  moves between shots and "the same subject" can only be judged loosely. No
  parser change, no `EXTRACTOR_VERSION` bump.
- Reduced RAW sizes differ by body in `FocusLocation`'s frame: the α7 IV
  full-frame M / S keep 7008x4672 while the full JPEG is 4608x3072 /
  3504x2336, whereas the α7R V, α9 III and α7CR M / S write the reduced size
  (6240x4160, 4752x3168, 3936x2632, 3008x2000). Both work, as the consumers
  scale by `sensor_w` / `sensor_h` against the JPEG. The aspect ratio always
  matches the preview: the α7C 4:3 sample writes 5328x4000 with a 1440x1080
  preview, the ZV-E10 16:9 sample 6000x3376 with a 1920x1080 preview.
- Lossless-compressed and uncompressed files resolve IFD0 / IFD2 the same way
  as compressed ones on every body.
- Manual focus: every α9 III and α7CR sample is `FocusMode` 0 with the point
  at the exact center and `FocusFrameSize` validity 0; `trusted_focus`
  drops the point (`faces` prints no AF point), as designed. So both bodies
  get `–` for AF point and AF frame size, with a sentence under the table.
- The α1 lossless and uncompressed samples write `FocusFrameSize` validity 0
  while the compressed one writes 1026x912 valid, all in AF-S at the center.
- Off-center checks (`focusbox`): α7S III `DSC01568` (the point sits on a
  treetop's edge against the sky), α7C II `DSC02431` / `DSC02435` (the wide
  AF-area zone on the grass / the tree line), α7 IV large (a leaf cluster),
  α7R V (the lawn at the tree line), ZV-E1 (a zone on the house and trees).
  The α1, α6700 and α7C II `DSC02429` points are at the exact center, which
  proves nothing about the mapping beyond the shared tag layout.
- No sample is a portrait frame (Orientation 1 on all 66), so orientation is
  unverified on every body; `faces` found no face on any listed body.

### Per body (listed)

Preview 1616x1080 on all. Capture time, sub-seconds, exposure, F, ISO,
focal length and lens are all present and match the Python dump. Metadata
and the preview fit in 1 MiB on all.

| Body (Model) | Full JPEG | EXIF rows (one sample) | FocusLocation | FocusFrameSize | AFTracking | Full decode |
|---|---|---|---|---|---|---|
| α1 (`ILCE-1`) | 8640x5760 | 2021:04:05 15:29:13.943, 1/160, F8, ISO 500, 135 mm, FE 135mm F1.8 GM | 8640x5760 at the center, AF-S | 1026x912 (compressed); invalid on the other two | 0 | 309 ms |
| α9 III (`ILCE-9M3`) | 6000x4000, M 3936x2632, S 3008x2000 | 2024:03:26 12:20:54.724, 1/50, F1.8, ISO 1000, 55 mm, FE 55mm F1.8 ZA | center, MF on all, dropped | invalid | 0 | 77 ms |
| α7 IV (`ILCE-7M4`) | 7008x4672, M 4608x3072, S 3504x2336; APS-C 4608x3072, S 3504x2336 | 2023:10:19 16:12:37.961, 1/50, F5.6, ISO 250, 50 mm, FE 50mm F2.5 G | 7008x4672 off-center; APS-C 4608x3072 | 350x351; APS-C 230x231 | 2 (lock-on AF) | 114 ms |
| α7R V (`ILCE-7RM5`) | 9504x6336, M 6240x4160, S 4752x3168; S35 6240x4160, S 4752x3168 | 2022:12:17 16:03:31.062, 1/2000, F1.2, ISO 100, 50 mm, FE 50mm F1.2 GM | off-center, in the file's own frame | 208x211 (M 137x139, S 104x106) | 0 | 208 ms |
| α7S III (`ILCE-7SM3`) | 4240x2832 | 2022:04:10 17:25:40.849, 1/400, F4, ISO 80, 24 mm, FE 24-105mm F4 G OSS | off-center, on the subject | 92x94 | 0 | 62 ms |
| α7C II (`ILCE-7CM2`) | 7008x4672; APS-C 4608x3072 | 2023:10:02 18:18:37.146, 1/100, F7.1, ISO 100, 24 mm, FE 24-70mm F4 ZA OSS | off-center on 2 of 5 | 1095x769 (APS-C 101x102) | 0 | 144 ms |
| α7CR (`ILCE-7CR`) | 9504x6336, M 6240x4160, S 4752x3168 | 2023:11:28 11:10:19.823, 1/100, F5, ISO 125, 49.5 mm, 24-70mm F2.8 DG DN \| Art 019 | center, MF on all, dropped | invalid | 0 | 200 ms |
| α6700 (`ILCE-6700`) | 6192x4128 | 2023:08:01 17:00:23.272, 1/50, F11, ISO 100, 16 mm, E 16-55mm F2.8 G | center, AF-S | 135x138 | 0 | 158 ms |
| ZV-E1 (`ZV-E1`) | 4240x2832 | 2023:08:19 12:53:58.187, 1/800, F8, ISO 200, 20 mm, FE 20mm F1.8 G | off-center, DMF | 649x490 | 0 | 65 ms |

### Per body (not listed: no full-size JPEG)

| Body (Model) | Preview | FocusLocation | FocusFrameSize | AFTracking | Sub-second |
|---|---|---|---|---|---|
| α9 II (`ILCE-9M2`) | 1616x1080 | 6000x4000, off-center | absent | 2 | yes |
| α7R IV (`ILCE-7RM4`) | 1616x1080 | 9504x6336 | absent | 0 | no |
| α7R IVA (`ILCE-7RM4A`) | 1616x1080 | 9504x6336, center | absent | 0 | no |
| α7C (`ILCE-7C`) | 1616x1080 (1440x1080 at 4:3) | 6000x4000 / 5328x4000 | absent | 0 | yes |
| α6400 (`ILCE-6400`) | 1616x1080 | 6000x4000, off-center | absent | 2 | no |
| α6600 (`ILCE-6600`) | 1616x1080 | 6000x4000, off-center | absent | 0 | no |
| ZV-E10 (`ZV-E10`) | 1920x1080 (16:9) | 6000x3376 | absent | 2 | yes |

## Deferred issues (todo candidates)

- **Older ARW bodies take the 160x120 thumbnail as the full-size JPEG.**
  Basis: this step's sample verification (the `bench` full decode of 0.2 ms
  and the Python dump showing only IFD0 and IFD1 JPEGs) on the α9 II, α7R IV,
  α7R IVA, α7C, α6400, α6600 and ZV-E10. `arw::parse` picks the largest JPEG
  in the IFD chain and the SubIFDs as `full`, which on these files is the
  IFD1 thumbnail, so `read_focus_crop` (the 1:1 view) crops an upscaled
  160x120 image. A fix needs a decision on what 1:1 shows without a
  full-size JPEG (refuse, fall back to the 1616x1080 preview, or decode the
  raw data), then those bodies can be added to the README. Files:
  `crates/core/src/arw.rs` (`parse`), `crates/core/src/reader.rs`
  (`read_full`), `crates/app/src/commands.rs` (`read_focus_crop`),
  `README.md`, `README.ja.md`, `docs/cameras.md`, `docs/raw-formats.md`.
- **AF point, face tracking and portrait orientation unconfirmed on some
  Sony bodies for lack of samples.** Basis: this step's verification. Every
  α9 III and α7CR sample is manual focus, no sample records `AFTracking` 1,
  and no sample is a portrait frame. An AF-C sample of a person (portrait
  orientation) from those bodies would let `docs/cameras.md` turn their `–`
  into `✓`. Files: `docs/cameras.md`.
