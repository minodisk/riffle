# Measurements taken before this plan (2026-09-22)

Machine: Linux WSL2, 24 hardware threads (`nproc`), release builds of
`riffle-cli`. Files read from `/mnt/d/Photos/2026/...` (a Windows NTFS drive
mounted in WSL). Page cache warm (each folder scanned once before timing).

## Face detection latency on real previews

`origin/main` at `ff21c25`. For each file: `riffle-cli bench <file>` (row
`4. face detection`, one thread) and `riffle-cli faces <file> <out.png>`.
Files sampled evenly from the folder listing (about 40 per folder).

| Body / folder | n | mean | median | p95 | preview |
|---|---|---|---|---|---|
| α7 V ARW, `2026-08-29` | 39 | 15.8 ms | 15.4 ms | 17.1 ms | 1080x1616 |
| M11-P DNG, `2026-02-01` | 36 | 16.1 ms | 16.0 ms | 17.1 ms | 2112x1408 |

First call (model build): ~42-44 ms.

Faces found per file (same samples):

- ARW (39): 0 faces 9, 1 face 6, 2 faces 11, 3 faces 4, 4 faces 6, 5-7 faces 3
- DNG (36): 0 faces 29, 1 face 2, 2 faces 5

Visual check of `faces` output: boxes and eye points sit on the faces,
portrait (orientation 8) frames included. Misses: `L1005161.DNG`, a group of
7 people on a swing, found 2; `_DSC3590.ARW`, basketball, missed a player
in three-quarter profile.

## Scan before/after adding face detection

`before` = build of `c496d48` (Step 2 of face-aware-sharpness merged; the scan
passed no faces), `after` = `ff21c25`. `riffle-cli scan <dir> <threads>`, runs
alternated, two runs each (numbers were stable within ~2 ms except the first
`after` 1-thread run).

| Folder | files | threads | per-file mean before -> after | p95 before -> after | wall |
|---|---|---|---|---|---|
| `2026-07-11` (α7 V ARW) | 468 | 1 | 17.8 -> 43.3-47.2 ms | 21.9 -> 50.7-54.4 ms | |
| `2026-07-11` (α7 V ARW) | 468 | 12 | 29.3 -> 67-68 ms | 35.9 -> 81-83 ms | 1.13 s (414 files/s) -> 2.63 s (178 files/s) |
| `2026-02-01` (M11-P DNG) | 146 | 1 | 27.8 -> 57.9 ms | 39.2 -> 73.9 ms | |
| `2026-02-01` (M11-P DNG) | 146 | 12 | 45 -> 91-92 ms | 63 -> 135-139 ms | |

## MakerNote face/eye-AF check

exiftool 13.59 (perl, from the GitHub source tree).

- M11-P DNG (`L1005161.DNG`): no AF position tags; only `FocusDistance`.
- α7 V (ILCE-7M5) ARW: no face-count or per-face position tags decode.
  83 files sampled (every 150th file of `2026-08-29`, `2026-08-01`,
  `2026-09-13-a`, `2026-07-11`, `2026-06-14`):
  - `AFTracking`: Face tracking 50, Lock On AF 12, Off 18 (plus 3 in
    combinations counted above; see the raw grouping below)
  - Raw `AFAreaMode` / `AFTracking` grouping: Human Eye Tracking / Face
    tracking 43; Expanded Flexible Spot / Off 7; Unknown (22) / Lock On AF 6;
    Human Eye Tracking / Off 6; Human Eye Tracking / Lock On AF 4;
    Unknown (22) / Off 3; Unknown (22) / Face tracking 3; Multi / Off 2;
    Expanded Flexible Spot / Lock On AF 2; Custom AF Area / Lock On AF 2
  - Of 47 files with Face tracking, 43 had `FocusLocation` moved off the
    sensor centre (3504, 2336 on 7008x4672) with a square `FocusFrameSize`
    between 153x154 and 438x439; 4 stayed at the centre with 832x740.
