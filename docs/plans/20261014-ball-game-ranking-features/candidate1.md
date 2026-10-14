# Candidate 1: SigLIP 2 text prompts

Run 2026-10-15 on the app dump `D:\Photos\tests\2026-10-12-rank-app\` with
the hand labels of `D:\Photos\tests\2026-10-14-ball-labels\labels.tsv` (240
frames: `ball` visible 137 / none 103 / unsure 0; `contest` yes 98 / no 99 /
na 42, one blank, excluded like na).

## How the columns were made

- `ball_text.py` ran the four ensembles of `ball_prompts.json` (the Step 1
  version, written blind) through the SigLIP 2 text tower as the archived
  `genre_text.py` does (each prompt L2-normalized, the mean normalized
  again) into `D:\Photos\tests\2026-10-14-ball-labels\ball_text.tsv`.
- `ball_scores.py` wrote `D:\Photos\tests\2026-10-14-ball-labels\extras_c1.tsv`
  for all 32,459 frames of the dump's `embed.tsv`: the cosine with
  `ball visible`, with `contested carrier`, and each minus the cosine with
  its negative (`no ball`, `single player`). Over all frames: `ball visible`
  mean 0.088 (SD 0.023), `contested carrier` 0.097 (0.037), the ball
  contrast -0.007 (0.007), the contest contrast 0.016 (0.025), inside the
  expected 0.05-0.15 band for the raw cosines.
- `python -I -B ballgate.py D:\Photos\tests\2026-10-12-rank-app <...>\extras_c1.tsv --labels <...>\labels.tsv --curve c1`
  printed everything below (8 jobs).

**Prompt revision: none.** The criterion reads the first (and only) prompt
set. A revision written now would not be blind (the scores on the pairs
and the labels have been seen), so it could not be the one the criterion
reads, and with bars 1-3 missed by a wide margin it would not change the
outcome.

## Result

**Candidate 1 fails** (bars 1-3 missed, bar 4 met). Its `ball visible`
column does measure the ball (frame-level AUC 0.829 against the hand
label), and `contested carrier` measures the contest (0.802; 0.814 as the
contrast), but adding the four columns to the shared model does not improve
the within-burst ranking: 0.708 weighted AUC / 54.4% Top-1 against the
reference's 0.710 / 54.9%, 10 of the 20 large folders at or above the
reference. A likely reading (not tested here): the frames of a burst share
the scene, and what of the ball and the contest varies within it the 32
PCA dimensions of the same embedding already carry, so the cosines add no
ordering.

Reading aids: each column alone ranks a burst at 0.521-0.583 weighted AUC
(the two contrasts best, 0.583 and 0.579); gated by the burst's sports
probability the candidate reads 0.709 (-0.001) / 54.6% (-0.2 points); on
the sports bursts 0.727 against 0.729 and on the other bursts 0.617
against 0.621; pick vs reject (3 bursts) unchanged at 0.926. The learning
curve rises 0.689 / 0.699 / 0.706 / 0.708 at 5 / 10 / 20 / 26 folders,
the reference's shape. The fit on every folder gives the four columns small
weights (-0.024, -0.109, +0.035, +0.139) against 1.196 for the PCA block.
The ball contrast (`ball visible` minus `no ball`) reads only 0.593 against
the `ball` label (the `no ball` prompts probably also respond to a ball
in the picture), so the
contrast column would not be the one to name the ball.

The reference reproduces the archived `ranking.md` exactly (0.710 / 54.9%).

## Ballgate output

### Data

| Count | Value |
| --- | ---: |
| Folders | 27 |
| Frames / picks / explicit rejects | 32459 / 4419 / 58 |
| Frames without a vector (dropped) | 0 |
| Scorable bursts (sports / other by burst argmax) | 1801 (1478 / 323) |
| Scorable bursts with an explicit reject | 3 |
| `c1` columns (frames without a row) | ball visible, contested carrier, ball visible minus no ball, contested carrier minus single player (0) |

### Held-out results (leave one folder out)

| Ranking | Bursts | Weighted pairwise AUC | Per-folder SD | Unweighted pairwise AUC | Top-1 | Per-folder SD | Top-3 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Reference (shared model) | 1801 | 0.710 | 0.054 | 0.763 | 54.9% | 8.2% | 87.7% |
| c1 | 1801 | 0.708 | 0.059 | 0.762 | 54.4% | 7.8% | 87.8% |

`C` chosen per fold (count of folds):

| Model | 0.01 | 0.1 | 1.0 | 10.0 |
| --- | ---: | ---: | ---: | ---: |
| Reference (shared model) | 6 | 7 | 5 | 8 |
| c1 | 10 | 7 | 2 | 7 |

### Per folder (held out)

| Folder | Bursts | Reference AUC | Top-1 | `c1` AUC | Delta | Top-1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | 121 | 0.726 | 49.6% | 0.725 | -0.001 | 50.4% |
| `2026-06-13` | 92 | 0.726 | 56.5% | 0.712 | -0.014 | 55.4% |
| `2026-06-14` | 65 | 0.773 | 44.6% | 0.765 | -0.007 | 44.6% |
| `2026-06-20` | 98 | 0.667 | 43.9% | 0.672 | +0.004 | 44.9% |
| `2026-07-02` | 7 | 0.486 | 28.6% | 0.506 | +0.019 | 28.6% |
| `2026-07-05` | 75 | 0.682 | 56.0% | 0.687 | +0.005 | 56.0% |
| `2026-07-11` | 29 | 0.761 | 69.0% | 0.759 | -0.003 | 69.0% |
| `2026-07-18` | 109 | 0.801 | 75.2% | 0.807 | +0.005 | 74.3% |
| `2026-07-21` | 3 | 0.667 | 66.7% | 0.667 | +0.000 | 66.7% |
| `2026-07-22` | 17 | 0.561 | 41.2% | 0.550 | -0.011 | 35.3% |
| `2026-07-24` | 15 | 0.517 | 46.7% | 0.517 | +0.000 | 46.7% |
| `2026-07-26` | 6 | 0.719 | 33.3% | 0.708 | -0.010 | 33.3% |
| `2026-07-30` | 57 | 0.628 | 50.9% | 0.628 | +0.000 | 50.9% |
| `2026-07-31` | 32 | 0.646 | 53.1% | 0.653 | +0.006 | 50.0% |
| `2026-08-01` | 176 | 0.662 | 50.0% | 0.658 | -0.004 | 49.4% |
| `2026-08-02` | 20 | 0.725 | 60.0% | 0.725 | +0.000 | 60.0% |
| `2026-08-08` | 52 | 0.647 | 51.9% | 0.645 | -0.002 | 51.9% |
| `2026-08-15` | 21 | 0.608 | 52.4% | 0.553 | -0.054 | 47.6% |
| `2026-08-16` | 5 | 0.608 | 40.0% | 0.608 | +0.000 | 40.0% |
| `2026-08-22` | 116 | 0.747 | 52.6% | 0.759 | +0.012 | 53.4% |
| `2026-08-29` | 133 | 0.761 | 69.2% | 0.739 | -0.022 | 63.9% |
| `2026-09-13-b` | 79 | 0.652 | 49.4% | 0.653 | +0.001 | 49.4% |
| `2026-09-19` | 107 | 0.729 | 56.1% | 0.719 | -0.010 | 54.2% |
| `2026-09-27-a` | 199 | 0.706 | 53.8% | 0.712 | +0.006 | 55.3% |
| `2026-09-27-b` | 56 | 0.739 | 64.3% | 0.734 | -0.006 | 64.3% |
| `2026-10-03` | 111 | 0.757 | 53.2% | 0.759 | +0.002 | 54.1% |

Folders with 20 or more scorable bursts: 20. At or above the reference there: `c1` 10.

### Columns against the hand labels (frame-level ROC AUC)

240 labeled frames (240 with a `ball` label). `unsure` / `na` excluded.

| Candidate | Column | AUC vs `ball` | Visible / none | AUC vs `contest` | Yes / no |
| --- | ---: | ---: | ---: | ---: | ---: |
| `c1` | ball visible | 0.829 | 137 / 103 | 0.382 | 98 / 99 |
| `c1` | contested carrier | 0.772 | 137 / 103 | 0.802 | 98 / 99 |
| `c1` | ball visible minus no ball | 0.593 | 137 / 103 | 0.376 | 98 / 99 |
| `c1` | contested carrier minus single player | 0.766 | 137 / 103 | 0.814 | 98 / 99 |

### Criterion

Reference: weighted AUC 0.710, Top-1 54.9% (archived ranking.md: 0.710 / 54.9%).

| Candidate | 1. AUC minus reference (>= 0.010) | Met | 2. Top-1 minus reference, points (>= 2.0) | Met | 3. Folders at or above (>= 13) | Met | 4. `ball` column AUC vs label (>= 0.75) | Met | Verdict |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `c1` | -0.002 | no | -0.4 | no | 10 of 20 | no | 0.829 (`ball visible`) | yes | fail |

### Reading aids (not in the criterion)

#### Each column alone as a within-burst ranking

| Candidate | Column | Weighted AUC | Top-1 | Top-3 |
| --- | ---: | ---: | ---: | ---: |
| `c1` | ball visible | 0.538 | 32.9% | 73.0% |
| `c1` | contested carrier | 0.521 | 30.8% | 72.4% |
| `c1` | ball visible minus no ball | 0.583 | 39.8% | 77.8% |
| `c1` | contested carrier minus single player | 0.579 | 36.3% | 76.8% |

#### Columns gated by the burst's sports probability

| Model | Weighted AUC | Minus reference | Top-1 | Minus reference, points |
| --- | ---: | ---: | ---: | ---: |
| `c1` x P(sports) | 0.709 | -0.001 | 54.6% | -0.2 |

#### Sports and non-sports bursts

| Model | Sports AUC (1478) | Top-1 | Other AUC (323) | Top-1 |
| --- | ---: | ---: | ---: | ---: |
| Reference (shared model) | 0.729 | 56.4% | 0.621 | 48.0% |
| `c1` | 0.727 | 56.0% | 0.617 | 47.4% |

#### Pick vs explicit reject

| Model | Bursts | Weighted AUC | Unweighted AUC |
| --- | ---: | ---: | ---: |
| Reference (shared model) | 3 | 0.926 | 0.963 |
| `c1` | 3 | 0.926 | 0.963 |

### Learning curve (`c1`)

| Training folders | Weighted AUC | Top-1 |
| --- | ---: | ---: |
| 5 | 0.689 | 51.2% |
| 10 | 0.699 | 53.6% |
| 20 | 0.706 | 54.5% |
| 26 | 0.708 | 54.4% |

### Weights of each candidate fit on every folder

`c1`: `C` = 0.1 (27 folders); standardized columns.

| Column | Weight |
| --- | ---: |
| ln rel sharpness | +0.230 |
| `eye_focus` logit | +0.171 |
| eyes open | +0.212 |
| \|yaw\| | -0.024 |
| \|pitch\| | +0.029 |
| \|roll\| | +0.026 |
| `photoTier` good | +0.125 |
| no `eye_focus` | -0.049 |
| no eyes judgment | +0.212 |
| no pose | -0.179 |
| position | -0.372 |
| first frame | -0.157 |
| PCA dims (L2 norm of the 32) | 1.196 |
| ball visible | -0.024 |
| contested carrier | -0.109 |
| ball visible minus no ball | +0.035 |
| contested carrier minus single player | +0.139 |

