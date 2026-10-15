# Learnings: ball-game-ranking-features

The plan with the pre-registered criteria landed on `main` as
`6138670662d69713559bf0df76b3f517dbe5adf8` (PR #804, 2026-10-14T12:50:57+09:00),
before any prompt vector, detector output or fit existed.

## Step 1

- **Prompts written blind.** `ball_prompts.json` was written from the four
  definitions in it before any text vector or cosine was computed and
  without opening a frame. It has the archived `genre_prompts.json` shape
  (`label`, `parent`, `prompts`, plus a `definition`), so the archived
  `genre_text.py` tokenizes it unchanged; Step 2's `ball_text.py` can be
  that thin.
- **`ballgate.py` extends by import.** It puts
  `../_archived/20261012-image-embeddings` on `sys.path` with
  `sys.dont_write_bytecode` (as `rank_prior.py` does) and uses `gate.load`,
  `build`, `scorable`, `design(..., "shared")`, `choose_c`, `fit`,
  `burst_metrics`, `summarize`, `per_folder`, `spread`, and `gate.lofo` for
  the reference itself. A candidate's design is `gate.design`'s shared
  matrix with the candidate's columns appended, imputed with the training
  mean and standardized on the training folders (column-wise, so it equals
  standardizing the stacked matrix). Its own `run_fold` / `lofo` / `curve`
  mirror gate.py's (same folds, same curve seeds) only because gate.py
  hard-codes the design variant; joblib's loky workers re-import `gate`
  fine from the inserted path. They do not inherit
  `sys.dont_write_bytecode`, so a run without `-B` leaves
  `_archived/20261012-image-embeddings/__pycache__/gate.cpython-313.pyc`
  behind (untracked, not ignored; removed before the commit); run it as
  `python -I -B ballgate.py`.
- **Bar 4's column rule.** The criterion says "its `ball` column"; the
  script reads the candidate's first column whose name starts with `ball`
  (for the combination, the first file's, i.e. Candidate 1's when given as
  `extras_c1.tsv,extras_c2.tsv`). Every column's AUC against both labels is
  printed anyway, so Step 3 can quote Candidate 2's for the combination if
  the order matters. Name the columns accordingly in Steps 2 and 3
  (`ball visible ...`, `ball present ...`).
- **Sports subset.** By the burst's argmax of the mean top-level softmax
  (`d["genre"]`, the burst average of `genres.tsv`'s softmax columns), as
  gate.py's provisional-genre table does: 1,478 sports / 323 other scorable
  bursts. The sports-gated reading aid multiplies each candidate column
  (imputed with its overall mean) by that burst's sports probability before
  the standardization and runs the same LOFO.
- **The sample.** `ball_sample.py` (seed 20261014) drew 240 frames from 24
  folders into `D:\Photos\tests\2026-10-14-ball-labels\`: `frames.tsv`,
  `img/` (240 upright previews at 1600 px through the archived
  `genre_sample.previews`, its `EDGE` raised from 800 so a small ball is
  visible; the ARW preview is about that size), `labels.tsv` (the empty
  template) and `label.html`. The page's UI text is Japanese, as the
  archived `genre_sample.py`'s was, since the user labels in Japanese.
  Label values: `ball` `visible` / `none` / `unsure`, `contest` `yes` /
  `no` / `na`.
- **Runtime.** The dry run (reference, one 2-column candidate, the gated
  aid and the learning curve) took 2 min 44 s with 8 jobs, so the full
  Step 2 / 3 runs fit in one background job without trouble.
- **The `--labels` path was exercised on random labels** (a scratch copy
  of `frames.tsv`), with a combination spec, and the missing-row guard
  (460 of 32,459 missing) stops the run as intended. Those numbers are
  not results and are not recorded.

- **Labeling keys changed by the user (recorded in Step 2).** At the user's
  request the labeling page's keys were changed from 1/2/3 and Q/W/E to
  **Z/X/C** for `ball` (visible / none / unsure) and **A/S/D** for
  `contest` (yes / no / na). The user edited
  `D:\Photos\tests\2026-10-14-ball-labels\label.html` by hand; Step 2's
  commit makes `ball_sample.py`'s `PAGE` generate the same six keys (the
  only difference between the hand-edited page and the template, checked
  by a diff). The sample was not regenerated.

### Dry run (random noise)

A two-column extras file of uniform noise (`ball noise`, `noise 2`, seed 7)
for all 32,459 frames, run with `--curve noise`. The reference reproduces
`ranking.md` exactly, **0.710 weighted AUC / 54.9% Top-1** (unweighted
0.763, Top-3 87.7%, `C` per fold 6 / 7 / 5 / 8), and the noise candidate
fails every bar:

| Ranking | Bursts | Weighted pairwise AUC | Per-folder SD | Unweighted pairwise AUC | Top-1 | Per-folder SD | Top-3 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Reference (shared model) | 1801 | 0.710 | 0.054 | 0.763 | 54.9% | 8.2% | 87.7% |
| noise | 1801 | 0.709 | 0.061 | 0.763 | 54.0% | 8.4% | 87.5% |

| Candidate | 1. AUC minus reference (>= 0.010) | Met | 2. Top-1 minus reference, points (>= 2.0) | Met | 3. Folders at or above (>= 13) | Met | 4. `ball` column AUC vs label (>= 0.75) | Met | Verdict |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `noise` | -0.001 | no | -0.9 | no | 9 of 20 | no | no `--labels` | - | fail |

Reading aids: each noise column alone 0.497 / 0.515 weighted AUC; gated by
P(sports) 0.708 (-0.002), 53.9%; sports bursts 0.729 -> 0.731, other 0.621
-> 0.610; pick vs reject (3 bursts) 0.926 -> 0.933; learning curve 0.690 /
0.698 / 0.703 / 0.709 at 5 / 10 / 20 / 26 folders; the fit on every folder
gives the noise columns +0.007 and +0.060 at `C` = 0.1. Folders with 20 or
more scorable bursts: 20, as the criterion assumed.

## Step 2

- **Labels in.** The user labeled the sample (Step 1's pending manual
  step, done, so it is no longer under the deferred issues): 240 rows,
  `ball` visible 137 / none 103 / unsure 0, `contest` yes 98 / no 99 /
  na 42 and one blank, which `ballgate.py`'s `label_auc` drops like `na`.
- **Scripts.** `ball_text.py` imports the archived `genre_text.py` and calls
  its `main` on `ball_prompts.json` (the format and the vector rule are
  unchanged); `ball_scores.py` uses `gate.load_embed` / `gate.load_text`, so
  the keys are `gate.py`'s `(folder, file)`. Outputs under
  `D:\Photos\tests\2026-10-14-ball-labels\` (`ball_text.tsv`,
  `extras_c1.tsv`, next to the labels), not in the app dump. Column names:
  `ball visible` first, so bar 4 reads it; the contrasts are
  `... minus no ball` / `... minus single player`.
- **The raw cosines sit in the expected band** (`ball visible` mean 0.088,
  `contested carrier` 0.097); the ball contrast is tiny (mean -0.007, SD
  0.007) and reads only 0.593 against the `ball` label, while the raw
  `ball visible` reads 0.829. The contest contrast reads 0.814 against
  `contest`, the raw 0.802.
- **No prompt revision.** The first set's scores were seen before any
  revision could be written, so a revision could not be the one the
  criterion reads; with bars 1-3 missed (0.012 AUC and 2.4 Top-1 points
  short of the bars) it was not run.
- **Result: Candidate 1 fails** (0.708 / 54.4% against 0.710 / 54.9%, 10
  of 20 folders; bar 4 met at 0.829). The gate run (reference, `c1`, the
  gated aid and the curve) took about 3 minutes with 8 jobs, `-I -B`; no
  `__pycache__` appeared.
- **Backslashes through the Bash tool's heredoc.** Even a quoted heredoc
  (`<<'EOF'`) collapsed `\\` to `\` here, so a docstring's `\\t` became a
  tab escape and a Windows path in a Python string turned into control
  characters. Write files with backslashes through the Edit / Write tools,
  or build them with `chr(92)`.

## Step 3

- **Previews through a new `riffle-cli previews` subcommand**, not by
  letting `embed --previews` take `--near`: `embed` runs the embedding model
  on every file, which the detector export does not need. `previews
  <out-dir> <dir|file>... --near <px> [threads]` writes the
  `decode_upright_near` image (the embedding's input path) as a q92 JPEG to
  `<out-dir>/<folder>/<stem>.jpg` plus `list.tsv`; the option parsing has a
  unit test. `--near 640` gives 808 x 540: the 3/8 scale of a 1616-px
  preview is 606, under 640, so the rule takes 4/8. All 32,459 frames in
  247 s with 12 threads, 4.2 GB, none failed. The command was run from a
  copy of the exe in the scratchpad so a rebuild could not hit a locked
  file.
- **`GridSample` loads in tract 0.23 with `default-features = false`.** The
  plan expected D-FINE / RT-DETRv2 to fail the load; the onnx-community
  exports of D-FINE N / S and RT-DETRv2 R18 load and run, but at 1.0 / 2.4 /
  2.8 s per image on one thread (onnxruntime: 85 / 254 / 464 ms), so they
  are no-ship on cost, not on ops. tract runs the transformer heads 9-12x
  slower than onnxruntime, the conv detectors 2-4x (NanoDet 73.5 against
  34.9 ms, YOLOX Nano 53.1 against 14.5).
- **Deviation: RTMDet and DEIM were not measured.** Neither publishes an
  ONNX (RTMDet needs an mmdeploy export with PyTorch and mmcv, not set up
  here; onnx-community has no DEIM COCO export), and DEIM shares D-FINE's
  decoder, whose tract cost already rules that family out. `detectors.md`
  tables them as not measured.
- **NanoDet-Plus's ONNX lists its initializers as graph inputs.**
  onnxruntime warns once per weight (silenced with `log_severity_level = 3`)
  and cannot fold them as constants, so its ms is a little pessimistic;
  tract pins only input 0 and takes the rest as constants, and the boxes
  match onnxruntime's exactly on the 240 labeled frames.
- **Preprocessing mirrored, not imported.** OpenCV is not in the venv;
  `detect_eval.py`'s `resize_linear` reimplements `cv2.resize`
  INTER_LINEAR (half-pixel centers, no antialias, rounded to uint8) for
  YOLOX and the half-pixel-free sampling of NanoDet's `warpPerspective`.
  The boxes were checked by eye on three labeled frames before the runs.
- **Feature rules the pre-registration left open**, fixed in
  `detect_features.py` before the gate run: the ball for `contested` and
  `ball size` is the top-confidence `sports ball` box and counts only at or
  above the best-F1 confidence (0.13574, the exact value from
  `detections.tsv`, which matches `detect_eval.py`'s 0.136); the carrier is
  the person box containing the ball's center, else the nearest by the
  point-to-box distance, ties by the box center; "within one carrier-box
  width" is the Euclidean gap between the two boxes against the carrier's
  width, an overlap being gap 0; persons are counted at NanoDet's default
  0.35.
- **`detections.tsv` is 385 MB** (5.6 M lines) because every detection at
  0.05 or above is kept, so a threshold change needs no re-run; the run took
  478 s with 12 one-thread onnxruntime workers.
- **Result: Candidate 2 and the combination fail** bars 1-3 (0.709 / 55.0%
  and 0.705 / 54.1% against 0.710 / 54.9%; 10 and 8 of 20 folders), bar 4 met
  (0.759 for `ball present`; the combination's bar 4 reads Candidate 1's
  `ball visible`, 0.829, as Step 1 noted). The detector's `contested` reads
  0.519 against the `contest` label. Decision 1: none ships, Steps 4-6
  dropped. The gate run took about 6 minutes; no `__pycache__` appeared.
- **The heredoc backslash collapse hit again**, in a Python heredoc that
  patched `crates/cli/src/main.rs`: `\\t` / `\\n` in a Rust string became a
  real tab and newline. Fixed with the Edit tool; the Step 2 note stands.

## Deferred issues (todo candidates)

- **Keep or remove `crates/core/tests/detector_tract.rs`.** Step 3 added it
  as the shippability check (an `#[ignore]` test reading
  `RIFFLE_DETECTOR_ONNX`, `RIFFLE_DETECTOR_INPUT` and
  `RIFFLE_DETECTOR_TENSORS`); the plan said Step 4 could remove it or keep it
  as its seed, and Step 4 is dropped (Decision 1). It costs nothing in CI
  (ignored) and is the loading / cost / box-agreement harness a later
  detector would reuse; decide whether to keep it. Related:
  `crates/core/tests/detector_tract.rs`,
  `docs/plans/20261014-ball-game-ranking-features/detect_eval.py`
  (`--tensors` / `--compare`).
