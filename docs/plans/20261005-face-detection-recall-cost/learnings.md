# Learnings: face-detection-recall-cost

## Step 1: `riffle-cli detect` and the baseline (2026-10-06)

### The command

- `riffle-cli detect <dir|file>... [threads]` reads each preview, takes
  `sharpness::trusted_focus`, times `decode::decode_rgb` and then
  `faces::detect_around_rgb` separately (together they are exactly
  `detect_around`), and prints one line per file: name, `crop` / `whole`,
  the face count with each face's box side (the longer of width and height,
  stored preview pixels) and score, the decode and the detection time. Then
  files, errors, files with a face, faces, and the `stats` rows of both
  times. No `riffle-core` change was needed.
- It defaults to **one thread**, unlike `scan` / `candidates`: the per-file
  times are the point, and 24 threads inflate them (the 2134-ARW folder at 24
  threads: detection mean 34.9 ms against 23-24 ms on one thread).
- The "detection" time includes `upright_rgb` (a full copy of the RGB
  preview), the crop, the box-average into the 320 input and the tract run;
  it is not the bare inference the old `bench` row 4 timed. This is the
  number Step 3's budget is about, since Step 3 removes the full-size work.
- The model is built before the timing (`faces::detect` on a 1x1 image).
- `stats` was split into `summary` (pure: mean, median, p95, max) and the
  printing, so the statistics are unit-tested; `detect_line` (the per-file
  line) is tested too.

### Machine

Windows 11 Home 10.0.26200, Intel Core i7-13700 (24 hardware threads),
32 GB, release `riffle-cli` built from `main` at `6da10d28` plus this
command (detection unchanged), files read from the local NTFS drive `D:`,
warm page cache (each set read once before timing), runs alternated. The
earlier tables in `docs/humans/performance.md` are Linux WSL2 on the same CPU.
The raw outputs are saved in `D:\Photos\tests\2026-10-06-face-recall\baseline\`
(`detect-dng37.txt`, `detect-arw40.txt`, `candidates-*.txt`, the two sample
lists), so later steps can diff per-file lines against them.

### `detect` totals (one thread)

| Set | Path | Files | With a face | Faces | Decode mean / median / p95 | Detection mean / median / p95 |
|---|---|---|---|---|---|---|
| 36 M11-P DNGs + `L1005161` (2112x1408) | whole | 37 | 9 | 12 | 14.3 / 13.6 / 18.5 ms | 24.8 / 24.4 / 27.4 ms |
| same, second run | whole | 37 | 9 | 12 | 14.2 / 13.6 / 18.0 ms | 24.5 / 24.1 / 26.8 ms |
| same, third run | whole | 37 | 9 | 12 | 14.4 / 13.6 / 17.8 ms | 24.4 / 23.9 / 27.2 ms |
| 40 α7 V ARWs of `2026-09-19`, every 53rd-ish file (1616x1080) | crop | 40 | 39 | 79 | 9.9 / 9.9 / 11.2 ms | 24.2 / 24.3 / 25.6 ms |
| same, second run | crop | 40 | 39 | 79 | 9.9 / 9.9 / 11.5 ms | 24.1 / 23.8 / 27.1 ms |
| same, third run | crop | 40 | 39 | 79 | 9.4 / 9.4 / 9.9 ms | 23.3 / 23.3 / 24.4 ms |

- Whole-image path baseline per file (decode + detection): about **38.5 ms**
  on this machine (14.3 + 24.5). The user's budget (at most about 2x after
  Step 3) is therefore about 77 ms decode + detection per file on the
  whole-image path, measured with `riffle-cli detect` on these 37 DNGs.
- The crop path's detection costs the same as the whole path (both run one
  320 input); the whole path is only dearer through the larger DNG decode.
- Every ARW in `2026-09-19` has a trusted AF point: `detect` on the whole
  folder at 24 threads printed 2134 `crop`, 0 `whole` (1952 with a face,
  3787 faces).

### Recall against the ground truth

See [`faces-truth.md`](./faces-truth.md). On `main`: files with a face found
8 / 34 files that contain one (23.5%), faces 10 / 71 (14.1%), `L1005161.DNG`
2 / 7. Of the 28 sampled files with no face found, 26 contain a face; only 2
are truly empty. No false positive in the sample.

### `scan` and `candidates` wall times

`scan` is pass 1 (thumbnail and metadata; it no longer detects). The
whole-image detection of the DNGs runs in `candidates` (pass 2,
`extract_analysis`: `whole_image_faces` + `score_preview`), which on the
no-AF-point DNGs does no cue and no eye measures.

146-DNG folder `D:\photos\2026\2026-02-01`, three alternated runs:

| Command | Threads | Run 1 | Run 2 | Run 3 |
|---|---|---|---|---|
| `scan` | 24 | 0.17 s (per file on a worker 26.8 ms, p95 39.2) | 0.18 s (27.1, 41.8) | 0.18 s (27.6, 42.6) |
| `candidates` | 24 | 1.09 s | 1.02 s | 1.04 s |
| `scan` | 1 | 1.96 s (13.4 ms, p95 19.8) | 1.93 s (13.2, 20.0) | 1.95 s (13.4, 19.8) |
| `candidates` | 1 | 7.39 s (50.6 ms/file) | 7.63 s (52.3) | 7.70 s (52.7) |

`candidates` on one thread, ~51 ms per DNG, is the whole-image path end to
end: read + full RGB decode + upright + detection (~38.5 ms of it per the
`detect` table) + the grayscale decode and score of `score_preview`.

2134-ARW folder `D:\photos\2026\2026-09-19`, 24 threads, three alternated
runs:

| Command | Run 1 | Run 2 | Run 3 |
|---|---|---|---|
| `scan` | 1.61 s (per file 18.0 ms, p95 23.2) | 1.94 s (21.7, 27.5) | 1.94 s (21.8, 28.4) |
| `candidates` | 12.16 s | 12.86 s | 12.52 s |

`candidates` states on this folder: 1700 Candidate, 252 NotCandidate, 182
Unknown; the per-file lines are identical across the three runs. (The
folder also carries 309 labeled frames: candidates 273, in focus 246
(90.1%); coverage 92.5%; AUC lap 0.630 / combined 0.759.) The 2026-09-26
Windows measurement in `performance.md` had 10.1-10.3 s for the same command
on the same folder; today's 12.2-12.9 s is on a later `main` and is the
baseline Steps 2 and 3 compare against, not that one.

### `candidates` on the labeled folders (24 threads)

| Set | Files | Labeled with a face | Candidates | In focus | Precision | Coverage | AUC lap | AUC combined | Wall |
|---|---|---|---|---|---|---|---|---|---|
| Training (`2026-06-05`, `2026-07-31`, `2026-09-13-a`, `2026-09-19`, `2026-09-19-focus-sample-2`) | 500 | 406 (339 in focus) | 333 | 310 | 93.1% | 91.4% | 0.816 | 0.852 | 3.72 s |
| Held-out (`2026-06-14`, `2026-07-18`, `2026-08-01`, `2026-08-22`) | 400 | 400 (342 in focus) | 366 | 326 | 89.1% | 95.3% | 0.635 | 0.754 | 2.85 s |

Both match the reference numbers of the af-eye-in-focus-probability plan
exactly. The per-file lines are saved for the Step 2 / 3 identity check.

### Notes for Steps 2 and 3

- Most missed faces are 55-120 preview px (8-18 px at the 320 input), so a
  640 input (16-36 px) should recover most of them. A handful of misses are
  large faces under goggles / neck warmers or with motion blur; those are a
  model limit, not geometry, and set the recall ceiling.
- The 2 truly empty files (`L1005393`, `L1005473`) and the 14 marginal faces
  are where a new false positive or a borderline hit would show; check them
  by eye on the Step 2 PNGs.
- The `faces` PNGs of the 37 files are in `D:\Photos\tests\2026-10-06-face-recall\`;
  regenerate them with the new detector into a sibling folder for Step 2.

### CI

- The first `mise run ci` failed on clippy's `type_complexity` for
  `Vec<Result<(bool, Vec<Face>, f64, f64)>>`; the tuple became the small
  `Detected` struct.

## Step 2: a larger model input for the whole-image path (2026-10-06)

### What landed

- `faces::WHOLE_INPUT = (640, 448)`: the whole-image search (`detect_around`
  with no AF point, through the new `faces::detect_whole`) runs YuNet at
  640x448, or 448x640 for a portrait upright image. `detect` and the
  `CATCH_CROP` path keep the 320x320 `INPUT`.
- One optimized plan per input (`faces::Input`: square, landscape,
  portrait), each in its own `OnceLock` of a `static` array, built on first
  use. `input_tensor` and `decode_stride` take the input's width and height
  (the stride grid is `width / stride` cells per row); the scale is `fit`,
  the smaller of the two axis ratios, so a 4:3 image fits the 640x448 input
  at 597 px.
- `SCORE_THRESHOLD` / `CATCH_CONFIDENCE` unchanged on both paths.
- `riffle-cli detect` builds the landscape and portrait plans before the
  timing too (without that, the first landscape and the first portrait file
  each paid ~35-40 ms of plan build); `bench` gains row
  `5. whole-image detection`.
- `FACES_VERSION` 3 -> 4.

### Measurements (Windows 11 machine of Step 1, release, warm cache)

The outputs are in `D:\Photos\tests\2026-10-06-face-recall\step2\`
(`before-*` / `after-*` per command, `d*-030.txt` / `r*.txt` for the
experiments, `png\` the `riffle-cli faces` images of the 37 files with the
new detector, `score.py` the scoring against `faces-truth.md`). The before
binary is `main` at `d03356d7`; its `detect` lines are identical to the
Step 1 baseline.

Option (c), the score threshold alone (320 input, threshold lowered for
the experiment and the found faces counted at each score):

| Threshold | Files (of 34) | Faces (of 71) | `L1005161` | Files / faces at >= 0.8 |
|---|---|---|---|---|
| 0.6 (main) | 8 | 10 | 2 | 4 / 6 |
| 0.5 | 11 | 13 | 3 | 4 / 6 |
| 0.4 | 13 | 18 | 4 | 4 / 6 |
| 0.3 | 16 | 25 | 5 | 4 / 6 |

It adds low-scoring boxes only; the sharpness score's `FACE_CONFIDENCE` 0.8
sees nothing new, so (c) was dropped. Larger inputs, at the 0.6 threshold
(files, faces, `L1005161`, files / faces >= 0.8, and the bare tract run per
file on one thread):

| Input | Files | Faces | `L1005161` | >= 0.8 | Inference |
|---|---|---|---|---|---|
| 320x320 | 8 | 10 | 2 | 4 / 6 | 20.1 ms |
| 416x416 | 14 | 19 | 4 | 5 / 6 | 33.9 ms |
| 480x480 | 18 | 26 | 6 | 13 / 16 | 45.2 ms |
| 512x512 | 19 | 27 | 6 | 15 / 17 | 57.9 ms |
| 576x576 | 24 | 32 | 7 | 18 / 23 | 66.8 ms |
| 576x384 | 24 | 32 | 7 | 18 / 23 | 45.7 ms |
| 640x640 | 23 | 35 | 6 | 19 / 25 | 84.7-87.4 ms |
| 640x448 | 23 | 36 | 6 | 19 / 25 | 59.6 ms |
| 704x480 | 26 | 44 | 7 | 22 / 30 | 68.9 ms |
| 736x736 | 26 | 39 | 7 | 23 / 32 | (detection 117 ms) |

- A rectangular input with the preview's 3:2 shape gives the square's
  recall at about 2/3 of its cost (the square leaves a third as padding).
  Tiling (b) was not tried: a 640x448 single run already costs less than four
  320 tiles would (4 x 20 ms plus the overlap), with no border merge.
- Chosen: 640x448. It clears the plan's targets (more files with a face:
  8 -> 23 of 34; `L1005161` 6 of 7, at least 5 required) and stays under the
  budget after Step 3 with margin. 704x480 recalls more (26 / 44 / 7) but
  would land at ~75 ms after Step 3, at the budget's edge; 576x384 is cheaper
  (~52 ms after Step 3) with 18 / 23 at >= 0.8 against 19 / 25.
- False faces: checked by eye on the new PNGs. Every box is on a face; the
  two files with no countable face (`L1005393`, `L1005473`) get none.
  Over-counts against the truth table are marginal faces the counting rule
  left out (e.g. the ~50 px goggled skier of `L1005294`, found at 0.66).
  `L1005161`: the six found are all real; the one missed is the woman in
  the middle, half behind the toddler's head.
- The model-input scores move a little on the large faces too:
  `L1005568`'s 372 px face went from 0.80 to 0.75 (its other face, 0.88, is
  still the one the sharpness window takes).

`riffle-cli detect` on the 37 DNGs, one thread, alternated, three runs:
before decode 14.0-14.2 ms, detection mean 24.2-25.0 ms (median 24.1-24.3,
p95 25.5-30.4); after decode 14.0-14.1 ms, detection 63.9-64.0 ms (median
63.1-63.8, p95 67.7-69.9). 24 files with a face, 42 faces (12 before).

Budget: whole-image decode + detection is now ~78 ms per file against the
Step 1 baseline of 38.5 ms (2.0x, the user's limit of ~77 ms, before
Step 3). The DCT-scaled decode the 640 input needs is 3/8 (2112 -> 792 px),
which took 4.6 ms on these files (2/8 4.5 ms, 4/8 6.9 ms, full 14 ms;
measured with a temporary helper, removed), and the upright copy and
box-average shrink with it (~4.5 ms of the 64 ms today at full size). Expected
after Step 3: ~4.6 + ~1 + 59.6 = **~65 ms**, about 1.7x the baseline.

`riffle-cli candidates` on the 146-DNG folder (before / after, three runs
each): 24 threads 1.21 / 1.18 / 1.03 s -> 2.14 / 2.23 / 2.24 s; one thread
7.40 / 7.60 / 7.54 s -> 13.76 / 14.19 / 15.43 s (51.6 -> 98.7 ms per
file). `scan` unchanged (24 threads 0.17-0.18 s; 1 thread 1.92-2.01 s, the
first before-run at 24 threads, 0.91 s, was a cold outlier).

Labeled folders: `candidates` on the five training and four held-out
folders at 24 threads prints exactly the Step 1 baseline lines (diff empty
apart from the wall-time line): training 333 / 310 / 93.1% / 91.4% / AUC
0.816 / 0.852, held-out 366 / 326 / 89.1% / 95.3% / 0.635 / 0.754. The
2134-ARW folder's per-file lines are identical too, and its time did not
rise: 15.24 / 13.54 / 13.82 s before, 13.68 / 13.90 / 11.81 s after
(alternated; the spread is the machine's).

Peak working set of `candidates <dir> 24` (polled from PowerShell, two runs
each): 2134 ARWs 393 / 396 MB before, 393 / 399 MB after (no file there
takes the whole-image path, so the 640x448 plans are never built); 146 DNGs
509 / 514 MB before, 646 / 646 MB after.

### Notes

- The `#[ignore]`d real-image test still passes (run with a JPEG made from
  the `L1005553` PNG; it uses `detect`, the 320 input).
- `bench` row 5 was noisier than `detect` on a first try (mean 81.7 ms on
  10 DNGs, p95 119 ms); `detect` is the number to compare.
- A heredoc containing Python with nested `'''` strings once failed in the
  Bash tool with "unexpected EOF while looking for matching"; the Edit tool
  was used for those edits instead.

## Deferred issues (todo candidates)

- If Step 3 lands well under the budget, consider 704x480 for the
  whole-image input (files 26 / faces 44 / `L1005161` 7 of 7, 22 / 30 at
  >= 0.8, against 23 / 36 / 6 and 19 / 25 now; inference 69 ms against
  59.6 ms). Basis: Step 2 measurements in
  `docs/plans/20261005-face-detection-recall-cost/learnings.md`; files
  `crates/core/src/faces.rs` (`WHOLE_INPUT`), `crates/app/src/index.rs`
  (`FACES_VERSION`).
- Pending manual check (app, Windows or macOS): open
  `D:\photos\2026\2026-02-01` in the app after updating, let the second pass
  re-run (`FACES_VERSION` 4), and press `f` on `L1005161.DNG`: six face boxes
  should be drawn (all but the woman in the middle), and on `L1005233.DNG`
  (portrait) none, as `riffle-cli faces` shows. The step's checkbox was
  ticked on the automated criteria (`riffle-cli detect` / `candidates` and
  the PNGs); the GUI was not run.
