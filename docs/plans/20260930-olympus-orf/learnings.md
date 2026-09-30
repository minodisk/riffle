# Learnings: olympus-orf

## Step 1

- `Tiff::new` now delegates to `new_with_magic(buf, base, end, &[42])`, which
  reads the byte order and then builds the walker through `with_order`. Routing
  both new constructors through the existing one keeps them in use before
  Step 2's `orf.rs` calls them, so no `allow(dead_code)` was needed.
- `with_order` checks only `base <= end <= buf.len()`; it asks for no minimum
  length, since a MakerNote segment is bounded by the prefix and every read
  is range-checked against `end` anyway.
- `cargo` is not on the Git Bash `PATH` here; run it through
  `mise exec -- cargo ...`.

## Step 2

- Per-model survey on raw.pixls.us (CC0) samples, all saved flat in
  `D:\photos\samples\ORF\` as `<Model>_<original name>`. "Preview end" is the
  absolute offset where the preview JPEG ends (`HEAD_LIMIT` = 1,048,576).
  Every file: `IIRO`, IFD0 without 0x0201, the four MakerNote sub-IFDs
  (0x2010 / 0x2020 / 0x2040 / 0x2050; the XZ-10 lacks 0x2050) type 13 count
  1, `PreviewImageValid` 1, a 3200x2400 preview that is the only JPEG at least
  1000 px wide (SOI + SOF scan), no `SubSecTimeOriginal`. Preview and 1:1
  (the same JPEG) decoded through `riffle-cli bench`; `focusbox` stops at
  "no FocusLocation" after decoding the preview, as expected for Step 2;
  `read_metadata` on a 1 MiB truncated copy returned the same orientation,
  capture time, model and preview `Embedded` on every file (the prefix alone
  parses); `riffle-cli scan` of the folder: 14 files, 0 errors. The meta pane
  fields (`Make`, `Model`, `LensModel`, `ExposureTime`, `FNumber`, `ISO`,
  `FocalLength`, `ExposureBiasValue`, `DateTimeOriginal`) matched the Python
  TIFF dump on every file.

  | File | Model (EXIF) | Orientation | Capture time | Note header / version | Note length | Preview end | Past prefix | Preview decode |
  | --- | --- | --- | --- | --- | --- | --- | --- | --- |
  | `OM-1_om1.orf` | OM-1 | 1 | 2022:03:05 15:47:24 | `OM SYSTEM` `04 00` | 1,812,088 | 1,014,705 | no | 52 ms |
  | `OM-1MarkII_20mp-12bit.orf` | OM-1MarkII | 1 | 2024:02:28 10:23:46 | `OM SYSTEM` `04 00` | 1,812,088 | 979,703 | no | 72 ms |
  | `OM-5_C1291096.ORF` | OM-5 | 1 | 2023:01:29 12:59:21 | `OM SYSTEM` `04 00` | 1,518,216 | 1,054,885 | yes | 41 ms |
  | `OM-3_P5048182.ORF` | OM-3 | 1 | 2025:05:04 08:36:50 | `OM SYSTEM` `04 00` | 1,812,088 | 971,900 | no | 44 ms |
  | `OM-5MarkII_OM_System_OM-5_2_regular.ORF` | OM-5MarkII | 1 | 2025:09:05 16:56:31 | `OM SYSTEM` `04 00` | 1,518,216 | 1,089,998 | yes | 41 ms |
  | `E-M1MarkIII_em1iii.orf` | E-M1MarkIII | 1 | 2020:03:16 14:47:28 | `OLYMPUS` `03 00` | 1,520,544 | 1,295,242 | yes | 53 ms |
  | `E-M1X_P3240916.ORF` | E-M1X | 1 | 2019:03:24 18:32:12 | `OLYMPUS` `03 00` | 1,514,400 | 1,031,300 | no | 49 ms |
  | `E-M1MarkII_Olympus_EM1mk2_Standard_20MP.ORF` | E-M1MarkII | 1 | 2017:11:30 20:21:49 | `OLYMPUS` `03 00` | 1,497,128 | 1,114,150 | yes | 47 ms |
  | `E-M5MarkIII_PB240241.ORF` | E-M5MarkIII | 1 | 2019:11:24 19:20:00 | `OLYMPUS` `03 00` | 1,520,592 | 1,128,922 | yes | 45 ms |
  | `E-M10MarkIV_PA040002.ORF` | E-M10MarkIV | 1 | 2020:10:04 19:08:37 | `OLYMPUS` `03 00` | 1,520,624 | 1,284,988 | yes | 46 ms |
  | `E-P7_P8162689.ORF` | E-P7 | 1 | 2022:08:16 12:41:10 | `OLYMPUS` `03 00` | 1,543,664 | 975,812 | no | 47 ms |
  | `PEN-F_PenFNormal.orf` | PEN-F | 1 | 2019:01:24 12:29:49 | `OLYMPUS` `03 00` | 1,497,612 | 1,118,475 | yes | 42 ms |
  | `XZ-10_P1240016.ORF` | XZ-10 | 6 | 2016:01:24 15:48:29 | `OLYMPUS` `03 00` | 145,950 | 1,271,490 | yes | 43 ms |
  | `E-30_RAW_OLYMPUS_E30.ORF` | E-30 | 8 | 2008:12:20 16:35:34 | `OLYMPUS` `03 00` | 1,452,144 | 1,150,523 | yes | 52 ms |

- Portrait: no candidate-body sample on raw.pixls.us is portrait (the IFD0
  `Orientation` of every ORF there was read from its first 4 KB; only the
  C5050Z, the E-30 and the XZ-10 are not 1). The XZ-10 (6) and E-30 (8) were
  downloaded instead: their previews, viewed after rotating per the tag, come
  out upright, so ORF bodies write a correct IFD0 `Orientation`.
- The XZ-10's MakerNote declares only 145,950 bytes and its preview lies past
  the note's end (note-relative start 194,758). Resolving the preview
  relative to the note start is still right; the "preview past the declared
  note is fine" rule in the plan was needed for it.
- AF values seen (for Step 4): `AFTargetInfo` on the OM bodies: OM-1
  (640x480, focus area 0, selected 312,231,16,18), OM-1 Mark II (selected
  296,213,48,54), OM-5 Mark II (focus and selected 297,224,47,33), OM-5 and
  OM-3 all zero after the frame size. `AFPointSelected` is nonzero on the
  PEN-F, E-M1X, E-M5 Mark III, E-M10 Mark IV, E-P7 and OM-5 Mark II (for
  example E-P7 `320/640, 305/480`), zero on the rest. All near the center, so
  none confirms the origin; Step 4 still needs an off-center sample.
- Olympus pads IFD0's `Make` / `Model` with spaces (`"OM-1            "`),
  which the meta pane would join into "OM Digital Solutions    OM-1    ";
  `orf::parse` trims trailing spaces of those two fields. Step 3 should list
  the models normalized ("OM System OM-1", "Olympus E-M1 Mark III").
- raw.pixls.us answers `/data/...` file URLs with a 301 to `/download/...`;
  `curl -L` is needed, or the file comes back empty. Ranged requests do not
  work, but `curl ... | head -c 4096` reads a header cheaply.
- Old bodies: the SP-350 and the C5050Z write magic `RS` (accepted); no
  old-style (type 7) sub-IFD was met, so it is not supported.
