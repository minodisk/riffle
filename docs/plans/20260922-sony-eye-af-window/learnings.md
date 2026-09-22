# Learnings

## Step 1

- Real α7 V files store `FocusFrameSize` (0x2037) as `UNDEFINED[6]` (type 7,
  count 6), not `SHORT[3]`; exiftool only reinterprets it as `int16u[3]`. A
  reader accepting just `SHORT[3]` returned `None` on `_DSC3590.ARW`. The
  parser now treats `UNDEFINED[6]` as `SHORT[3]` (value at offset either way)
  and still accepts `SHORT[3]`. Verified on `_DSC3590.ARW`: `AFTracking` 0,
  frame 832x740, validity bytes `01 01` (257).
- `shorts4` was generalised to `shorts::<N>` for the 3-SHORT read.

## Step 2

- `score_preview` took a new `frame: Option<FocusFrame>` parameter (used only
  together with `focus`) rather than a subject enum; the routing stays in
  `sharpness.rs` and `scan::extract` passes `eye_af_frame(..).map(|(_, f)| f)`.
- Scan before/after of the skip, 2026-09-22, Linux WSL2 (24 hardware
  threads), release `riffle-cli scan /mnt/d/Photos/2026/2026-07-11 <threads>`
  (468 α7 V ARWs), warm page cache (one untimed scan first), runs alternated
  main / branch, two runs each. `before` = build of `origin/main` at `e62fb18`,
  `after` = this branch.

  | threads | per-file mean before -> after | p95 before -> after | files/s before -> after |
  |---|---|---|---|
  | 12 | 74.6 / 80.8 -> 33.9 / 33.9 ms | 90.3 / 97.5 -> 63.3 / 59.7 ms | 160 / 147 -> 345 / 350 (2.93 / 3.18 s -> 1.36 / 1.34 s) |
  | 1 | 43.0 / 43.1 -> 19.6 / 19.8 ms | 50.4 / 49.9 -> 40.6 / 39.3 ms | 23 / 23 -> 51 / 51 |

- Eye-AF path count (temporary `eprintln!` in `scan::extract`, not
  committed): 441 of 468 files took it; 25 had `AFTracking` 0; 2 had
  `AFTracking` 1 with the focus point at the exact sensor centre (3504, 2336)
  and frames 153x156 and 3307x2317, so they fell back to detection. Neither
  is the 832x740 not-engaged frame, so an engaged frame dead centre does
  occur, but it is rare (2/468) and only costs the detector path; kept
  Option A.
- The p95 stays higher than the pre-detection baseline (35.9 ms at 12
  threads in `measurements.md`) because the 27 fallback files still run
  detection.
- The burst spot-check of the ranking change (camera frame vs YuNet eye
  midpoint) was not done: `riffle-cli scan` prints no per-file score.

## Deferred issues (todo candidates)

- Spot-check whether the sharpness ranking within a burst changes now that
  Sony frames with face tracking are scored on the camera's AF frame instead
  of YuNet's eye midpoint (plan "Trade-offs and risks"). Needs a per-file
  score output; basis: Step 2 implementation. Files:
  `crates/core/src/sharpness.rs`, `crates/core/src/scan.rs`,
  `crates/cli/src/main.rs`.
