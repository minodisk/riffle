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
