# Candidate 2: an object detector

Run 2026-10-15 on the app dump `D:\Photos\tests\2026-10-12-rank-app\` with
the hand labels of `D:\Photos\tests\2026-10-14-ball-labels\labels.tsv` (240
frames: `ball` visible 137 / none 103 / unsure 0; `contest` yes 98 / no 99 /
na 42, one blank, excluded like na). The shortlist, its licenses, pinned
files and costs, and the choice of the detector are in
[detectors.md](detectors.md).

## How the columns were made

- `riffle-cli previews D:\Photos\tests\2026-10-14-ball-previews <the 27
  folders> --near 640 12` wrote the upright preview of all 32,459 frames at
  the 4/8 DCT scale (808 x 540 or 540 x 808, JPEG q92, 4.2 GB) in 247 s.
- The chosen detector, **NanoDet-Plus m 416**, ran on every preview with
  `detect_all.py` (onnxruntime, 12 one-thread workers, 478 s) into
  `D:\Photos\tests\2026-10-14-ball-labels\detections.tsv` (every `person`
  and `sports ball` detection at 0.05 or above after the model's NMS, in
  preview pixels; 5.6 M lines).
- `detect_features.py ... --ball 0.13574 --person 0.35` derived
  `D:\Photos\tests\2026-10-14-ball-labels\extras_c2.tsv`, the six columns
  of the criterion: `ball present` (the top `sports ball` confidence),
  `ball indicator` (that at or above 0.13574, the best-F1 confidence on the
  hand labels), `ln persons` (persons at or above the model's default
  0.35), `contested` (a second person box overlapping the carrier's or
  within one carrier-box width of it, the carrier being the person box
  containing the ball's center, else the nearest; 0 without a ball at
  0.13574), `ball size` (the top ball box's long side over the frame's
  long side, 0 without a ball at 0.13574) and `ln persons x ball present`.
  Over all frames the means are 0.219, 0.641, 1.922, 0.607, 0.042 and
  0.421.
- `python -I -B ballgate.py D:\Photos\tests\2026-10-12-rank-app <...>\extras_c2.tsv <...>\extras_c1.tsv,<...>\extras_c2.tsv --labels <...>\labels.tsv --curve c2`
  printed everything below (8 jobs, about 6 minutes).

## Result

**Candidate 2 fails** (bars 1-3 missed, bar 4 met) and **the combination
fails** (bars 1-3 missed, bar 4 met).

- `c2`: 0.709 weighted AUC / 55.0% Top-1 against the reference's 0.710 /
  54.9% (-0.001, +0.1 points), 10 of the 20 large folders at or above the
  reference. Its `ball present` column reads 0.759 against the `ball` label,
  just over bar 4's 0.75.
- `c1+c2`: 0.705 / 54.1% (-0.005, -0.8 points), 8 of 20. Bar 4 reads the
  first `ball` column of the first file, Candidate 1's `ball visible`
  (0.829); Candidate 2's `ball present` reads 0.759, so bar 4 is met
  either way.
- **The `contested` column does not measure the contest**: 0.519 against
  the `contest` label (chance), while Candidate 1's `contested carrier`
  reads 0.802. Without a team cue, "a second person near the carrier" is
  true of most basketball frames with a ball: it is 1 on 0.607 of all
  frames, on 71% of the frames labeled `yes` and on 68% of those labeled
  `no`. It reads 0.682 against the `ball` label only because it is
  0 without a ball.
- Reading aids: each column alone ranks a burst at 0.497-0.591 weighted AUC
  (`ball present` best); gated by the burst's sports probability `c2` reads
  0.711 (+0.001) / 55.0%; on the sports bursts `c2` is 0.731 against 0.729,
  on the other bursts 0.606 against 0.621; pick vs reject (3 bursts) 0.933
  against 0.926. The learning curve rises 0.683 / 0.698 / 0.703 / 0.709 at
  5 / 10 / 20 / 26 folders, the reference's shape. The fit on every folder
  gives the six columns small weights (+0.063, -0.016, +0.094, +0.136,
  +0.042, -0.045) against 1.365 for the PCA block.

As with Candidate 1, the cue is measurable (the ball is, the detector's
contest is not) but adds no within-burst ordering on top of the shared
model.

## Shippability checks (NanoDet-Plus m 416)

| Check | Bar | Result | Met |
| --- | --- | --- | --- |
| License of code and weights | Apache-2.0, MIT or BSD | Apache-2.0 both ([detectors.md](detectors.md)) | yes |
| tract 0.23, `default-features = false` | loads and runs | loads and runs (`detector_loads_and_runs_in_tract`, `RIFFLE_DETECTOR_ONNX` / `RIFFLE_DETECTOR_INPUT=1,3,416,416`; the initializers listed as graph inputs are taken as constants) | yes |
| Boxes against onnxruntime on the labeled frames | same boxes at the chosen confidence, IoU >= 0.9 | the 240 labeled frames' input tensors (`detect_eval.py --tensors`) run through tract (`RIFFLE_DETECTOR_TENSORS`) and decoded the same way (`--compare`): at 0.13574, 240 of 240 frames with the same boxes, 9,827 of 9,827 boxes matched (lowest IoU 1.0000); at 0.35, 240 of 240, 1,635 of 1,635 | yes |
| One-thread warm cost in tract at 416 x 416 | <= 132 ms | 73.5 ms (74.5 ms on a second run; release build, 20 warm runs) | yes |
| File size | <= 100 MB | 4,793,616 bytes (4.8 MB) | yes |

NanoDet-Plus m 416 is shippable; it is not adopted because Candidate 2 fails
bars 1-3.

## Ballgate output

### Data

| Count | Value |
| --- | ---: |
| Folders | 27 |
| Frames / picks / explicit rejects | 32459 / 4419 / 58 |
| Frames without a vector (dropped) | 0 |
| Scorable bursts (sports / other by burst argmax) | 1801 (1478 / 323) |
| Scorable bursts with an explicit reject | 3 |
| `c2` columns (frames without a row) | ball present, ball indicator, ln persons, contested, ball size, ln persons x ball present (0) |
| `c1+c2` columns (frames without a row) | ball visible, contested carrier, ball visible minus no ball, contested carrier minus single player, ball present, ball indicator, ln persons, contested, ball size, ln persons x ball present (0) |

### Held-out results (leave one folder out)

| Ranking | Bursts | Weighted pairwise AUC | Per-folder SD | Unweighted pairwise AUC | Top-1 | Per-folder SD | Top-3 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Reference (shared model) | 1801 | 0.710 | 0.054 | 0.763 | 54.9% | 8.2% | 87.7% |
| c2 | 1801 | 0.709 | 0.064 | 0.762 | 55.0% | 8.0% | 87.8% |
| c1+c2 | 1801 | 0.705 | 0.059 | 0.760 | 54.1% | 8.0% | 87.8% |

`C` chosen per fold (count of folds):

| Model | 0.01 | 0.1 | 1.0 | 10.0 |
| --- | ---: | ---: | ---: | ---: |
| Reference (shared model) | 6 | 7 | 5 | 8 |
| c2 | 3 | 8 | 7 | 8 |
| c1+c2 | 6 | 8 | 3 | 9 |

### Per folder (held out)

| Folder | Bursts | Reference AUC | Top-1 | `c2` AUC | Delta | Top-1 | `c1+c2` AUC | Delta | Top-1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | 121 | 0.726 | 49.6% | 0.730 | +0.003 | 47.9% | 0.728 | +0.002 | 48.8% |
| `2026-06-13` | 92 | 0.726 | 56.5% | 0.738 | +0.013 | 57.6% | 0.741 | +0.015 | 56.5% |
| `2026-06-14` | 65 | 0.773 | 44.6% | 0.792 | +0.020 | 50.8% | 0.786 | +0.013 | 50.8% |
| `2026-06-20` | 98 | 0.667 | 43.9% | 0.644 | -0.023 | 40.8% | 0.652 | -0.015 | 41.8% |
| `2026-07-02` | 7 | 0.486 | 28.6% | 0.489 | +0.002 | 28.6% | 0.507 | +0.020 | 28.6% |
| `2026-07-05` | 75 | 0.682 | 56.0% | 0.693 | +0.011 | 56.0% | 0.690 | +0.008 | 56.0% |
| `2026-07-11` | 29 | 0.761 | 69.0% | 0.716 | -0.046 | 62.1% | 0.718 | -0.043 | 62.1% |
| `2026-07-18` | 109 | 0.801 | 75.2% | 0.796 | -0.005 | 73.4% | 0.788 | -0.013 | 71.6% |
| `2026-07-21` | 3 | 0.667 | 66.7% | 0.667 | +0.000 | 66.7% | 0.667 | +0.000 | 66.7% |
| `2026-07-22` | 17 | 0.561 | 41.2% | 0.553 | -0.008 | 41.2% | 0.511 | -0.050 | 35.3% |
| `2026-07-24` | 15 | 0.517 | 46.7% | 0.533 | +0.017 | 53.3% | 0.533 | +0.017 | 53.3% |
| `2026-07-26` | 6 | 0.719 | 33.3% | 0.760 | +0.042 | 50.0% | 0.750 | +0.031 | 50.0% |
| `2026-07-30` | 57 | 0.628 | 50.9% | 0.605 | -0.023 | 49.1% | 0.606 | -0.021 | 49.1% |
| `2026-07-31` | 32 | 0.646 | 53.1% | 0.590 | -0.056 | 40.6% | 0.614 | -0.032 | 37.5% |
| `2026-08-01` | 176 | 0.662 | 50.0% | 0.678 | +0.016 | 51.1% | 0.654 | -0.007 | 47.7% |
| `2026-08-02` | 20 | 0.725 | 60.0% | 0.735 | +0.010 | 60.0% | 0.735 | +0.010 | 60.0% |
| `2026-08-08` | 52 | 0.647 | 51.9% | 0.633 | -0.014 | 50.0% | 0.630 | -0.017 | 50.0% |
| `2026-08-15` | 21 | 0.608 | 52.4% | 0.592 | -0.015 | 47.6% | 0.621 | +0.014 | 52.4% |
| `2026-08-16` | 5 | 0.608 | 40.0% | 0.595 | -0.013 | 40.0% | 0.555 | -0.053 | 40.0% |
| `2026-08-22` | 116 | 0.747 | 52.6% | 0.763 | +0.016 | 57.8% | 0.773 | +0.025 | 57.8% |
| `2026-08-29` | 133 | 0.761 | 69.2% | 0.767 | +0.007 | 67.7% | 0.748 | -0.013 | 66.9% |
| `2026-09-13-b` | 79 | 0.652 | 49.4% | 0.654 | +0.001 | 53.2% | 0.646 | -0.007 | 50.6% |
| `2026-09-19` | 107 | 0.729 | 56.1% | 0.732 | +0.003 | 58.9% | 0.732 | +0.003 | 57.0% |
| `2026-09-27-a` | 199 | 0.706 | 53.8% | 0.695 | -0.011 | 54.3% | 0.693 | -0.013 | 52.8% |
| `2026-09-27-b` | 56 | 0.739 | 64.3% | 0.730 | -0.010 | 58.9% | 0.725 | -0.014 | 60.7% |
| `2026-10-03` | 111 | 0.757 | 53.2% | 0.754 | -0.003 | 54.1% | 0.754 | -0.003 | 53.2% |

Folders with 20 or more scorable bursts: 20. At or above the reference there: `c2` 10, `c1+c2` 8.

### Columns against the hand labels (frame-level ROC AUC)

240 labeled frames (240 with a `ball` label). `unsure` / `na` excluded.

| Candidate | Column | AUC vs `ball` | Visible / none | AUC vs `contest` | Yes / no |
| --- | ---: | ---: | ---: | ---: | ---: |
| `c2` | ball present | 0.759 | 137 / 103 | 0.455 | 98 / 99 |
| `c2` | ball indicator | 0.671 | 137 / 103 | 0.499 | 98 / 99 |
| `c2` | ln persons | 0.566 | 137 / 103 | 0.480 | 98 / 99 |
| `c2` | contested | 0.682 | 137 / 103 | 0.519 | 98 / 99 |
| `c2` | ball size | 0.735 | 137 / 103 | 0.440 | 98 / 99 |
| `c2` | ln persons x ball present | 0.769 | 137 / 103 | 0.449 | 98 / 99 |
| `c1+c2` | ball visible | 0.829 | 137 / 103 | 0.382 | 98 / 99 |
| `c1+c2` | contested carrier | 0.772 | 137 / 103 | 0.802 | 98 / 99 |
| `c1+c2` | ball visible minus no ball | 0.593 | 137 / 103 | 0.376 | 98 / 99 |
| `c1+c2` | contested carrier minus single player | 0.766 | 137 / 103 | 0.814 | 98 / 99 |
| `c1+c2` | ball present | 0.759 | 137 / 103 | 0.455 | 98 / 99 |
| `c1+c2` | ball indicator | 0.671 | 137 / 103 | 0.499 | 98 / 99 |
| `c1+c2` | ln persons | 0.566 | 137 / 103 | 0.480 | 98 / 99 |
| `c1+c2` | contested | 0.682 | 137 / 103 | 0.519 | 98 / 99 |
| `c1+c2` | ball size | 0.735 | 137 / 103 | 0.440 | 98 / 99 |
| `c1+c2` | ln persons x ball present | 0.769 | 137 / 103 | 0.449 | 98 / 99 |

### Criterion

Reference: weighted AUC 0.710, Top-1 54.9% (archived ranking.md: 0.710 / 54.9%).

| Candidate | 1. AUC minus reference (>= 0.010) | Met | 2. Top-1 minus reference, points (>= 2.0) | Met | 3. Folders at or above (>= 13) | Met | 4. `ball` column AUC vs label (>= 0.75) | Met | Verdict |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `c2` | -0.001 | no | +0.1 | no | 10 of 20 | no | 0.759 (`ball present`) | yes | fail |
| `c1+c2` | -0.005 | no | -0.8 | no | 8 of 20 | no | 0.829 (`ball visible`) | yes | fail |

### Reading aids (not in the criterion)

#### Each column alone as a within-burst ranking

| Candidate | Column | Weighted AUC | Top-1 | Top-3 |
| --- | ---: | ---: | ---: | ---: |
| `c2` | ball present | 0.591 | 42.4% | 80.4% |
| `c2` | ball indicator | 0.548 | 33.2% | 75.9% |
| `c2` | ln persons | 0.497 | 29.7% | 69.0% |
| `c2` | contested | 0.549 | 33.3% | 76.3% |
| `c2` | ball size | 0.575 | 37.9% | 78.7% |
| `c2` | ln persons x ball present | 0.586 | 41.9% | 79.8% |
| `c1+c2` | ball visible | 0.538 | 32.9% | 73.0% |
| `c1+c2` | contested carrier | 0.521 | 30.8% | 72.4% |
| `c1+c2` | ball visible minus no ball | 0.583 | 39.8% | 77.8% |
| `c1+c2` | contested carrier minus single player | 0.579 | 36.3% | 76.8% |
| `c1+c2` | ball present | 0.591 | 42.4% | 80.4% |
| `c1+c2` | ball indicator | 0.548 | 33.2% | 75.9% |
| `c1+c2` | ln persons | 0.497 | 29.7% | 69.0% |
| `c1+c2` | contested | 0.549 | 33.3% | 76.3% |
| `c1+c2` | ball size | 0.575 | 37.9% | 78.7% |
| `c1+c2` | ln persons x ball present | 0.586 | 41.9% | 79.8% |

#### Columns gated by the burst's sports probability

| Model | Weighted AUC | Minus reference | Top-1 | Minus reference, points |
| --- | ---: | ---: | ---: | ---: |
| `c2` x P(sports) | 0.711 | +0.001 | 55.0% | +0.2 |
| `c1+c2` x P(sports) | 0.709 | -0.001 | 55.1% | +0.3 |

#### Sports and non-sports bursts

| Model | Sports AUC (1478) | Top-1 | Other AUC (323) | Top-1 |
| --- | ---: | ---: | ---: | ---: |
| Reference (shared model) | 0.729 | 56.4% | 0.621 | 48.0% |
| `c2` | 0.731 | 56.9% | 0.606 | 46.1% |
| `c1+c2` | 0.727 | 56.0% | 0.605 | 45.2% |

#### Pick vs explicit reject

| Model | Bursts | Weighted AUC | Unweighted AUC |
| --- | ---: | ---: | ---: |
| Reference (shared model) | 3 | 0.926 | 0.963 |
| `c2` | 3 | 0.933 | 0.981 |
| `c1+c2` | 3 | 0.926 | 0.963 |

### Learning curve (`c2`)

| Training folders | Weighted AUC | Top-1 |
| --- | ---: | ---: |
| 5 | 0.683 | 51.1% |
| 10 | 0.698 | 53.8% |
| 20 | 0.703 | 54.0% |
| 26 | 0.709 | 55.0% |

### Weights of each candidate fit on every folder

`c2`: `C` = 1.0 (27 folders); standardized columns.

| Column | Weight |
| --- | ---: |
| ln rel sharpness | +0.229 |
| `eye_focus` logit | +0.173 |
| eyes open | +0.222 |
| \|yaw\| | -0.025 |
| \|pitch\| | +0.032 |
| \|roll\| | +0.026 |
| `photoTier` good | +0.127 |
| no `eye_focus` | -0.048 |
| no eyes judgment | +0.380 |
| no pose | -0.346 |
| position | -0.376 |
| first frame | -0.158 |
| PCA dims (L2 norm of the 32) | 1.365 |
| ball present | +0.063 |
| ball indicator | -0.016 |
| ln persons | +0.094 |
| contested | +0.136 |
| ball size | +0.042 |
| ln persons x ball present | -0.045 |

`c1+c2`: `C` = 1.0 (27 folders); standardized columns.

| Column | Weight |
| --- | ---: |
| ln rel sharpness | +0.228 |
| `eye_focus` logit | +0.174 |
| eyes open | +0.222 |
| \|yaw\| | -0.026 |
| \|pitch\| | +0.032 |
| \|roll\| | +0.027 |
| `photoTier` good | +0.127 |
| no `eye_focus` | -0.050 |
| no eyes judgment | +0.391 |
| no pose | -0.355 |
| position | -0.376 |
| first frame | -0.158 |
| PCA dims (L2 norm of the 32) | 1.495 |
| ball visible | +0.032 |
| contested carrier | -0.536 |
| ball visible minus no ball | +0.058 |
| contested carrier minus single player | +0.195 |
| ball present | +0.056 |
| ball indicator | -0.018 |
| ln persons | +0.089 |
| contested | +0.138 |
| ball size | +0.044 |
| ln persons x ball present | -0.038 |

