# Discard retarget evaluation

## Pre-registration

Written 2026-10-20, after Step 1 wrote `burst_features.tsv` (label-free) and
**before `retargetfit.py` exists and before any score of `G0+B`, `D0` or
`D0+B` has been computed or read**: when this section was written no model of
this plan had been fit, no new score existed anywhere, and the only outputs
of this plan in `D:\Photos\tests\2026-10-19-discard-retarget\` were
`burst_features.tsv` and `burstfeat.out.md`, which read the dump alone. It is
merged on its own, before `retargetfit.py` or `retargetread.py` exists; the
merge commit of the PR that adds this section, without results, is the
record, as for the archived
[discard filter evaluation](../20261018-discard-labels/discard.md)
before it. Nothing in this section changes after the new scores exist; a
deviation found while implementing Steps 3–5 is recorded in `learnings.md`
with its reason and does not touch the bars.

The question: on the 600 blind **"OK to discard?"** labels of discard-labels,
does training on those labels instead of on the dump's picks (effect **A**),
and does adding a small fixed set of burst-relative columns (effect **B**),
give a per-folder bottom cut that removes frames the user would not miss,
better than `G0`, at which depth and at what cost in rescued and lost frames?
Which follow-up, if any, deserves its own plan?

### What was known before this section

This pre-registration is **not blind** to `G0`'s reading of the same labels.
On the same dump of 27 folders and 32,459 frames and the same 600 labels,
the archived [discard.md](../20261018-discard-labels/discard.md)
"Results" read:

- **Discardable base rate 48.45% [42.03, 55.11]** of the population's frames
  (folder-cluster [40.74, 53.87]; unweighted 64.83%).
- **`G0`** at the per-folder full-dump cuts (weighted, stratified 95%
  interval):

  | Cut | `DP` % | `RR` % | `PL` % (full dump) | `DR` % | `DO` % | Removed % (population) |
  | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
  | q5 | 82.7 [76.0, 88.7] | 17.3 [11.3, 24.0] | 0.68 | 8.4 [7.2, 9.8] | 46.7 [39.9, 53.8] | 4.92 |
  | q10 | 76.3 [71.3, 81.3] | 23.7 [18.7, 28.7] | 2.10 | 15.5 [13.6, 18.1] | 45.4 [38.3, 52.9] | 9.87 |
  | q20 | 72.1 [67.5, 76.7] | 27.9 [23.3, 32.5] | 5.18 | 29.5 [25.9, 33.9] | 42.6 [34.8, 50.7] | 19.80 |

  Folder-cluster 97.5% quantile of `RR`: q5 25.7, q10 32.8, q20 36.3.
  Weighted AUC 0.710 [0.646, 0.771]. `DP` minus random (seed 0): q5 +30.7
  [-3.8, +63.9] (random q5 removes only 27 sampled frames), q10 +14.8
  [-8.7, +38.6], q20 +16.1 [+0.3, +31.7].
- **`eye_focus` (missing last)**: AUC 0.777 [0.716, 0.841], the highest of
  every score read; q5 `RR` 16.8 [1.2, 43.3] at `PL` 4.44%; q10 `RR` 14.3
  [2.9, 29.9] at `PL` 9.14%; q20 `RR` 28.3 at `PL` 15.59%.
- **Random (seed 0)**: AUC 0.528 [0.455, 0.604]; `DP` q5 52.0 [19.8, 85.5],
  q10 61.5 [37.9, 84.8], q20 56.0 [40.5, 71.5].
- **The target finding**: 51.7% [44.9, 59.0] of the dump's non-picks and
  24.4% [8.6, 42.8] of its sampled picks (12 of 39) are labeled `discard`,
  so "non-pick" is a poor stand-in for "OK to discard". By the discard-labels
  follow-up rule (no usable cut, `RR@5` 17.3% <= 30%, `DO@5` 46.7% >= 20%)
  its Decision recommended a plan that trains on the rescue labels with the
  controls as negatives: this one.

Also known, from Step 1 (label-free; `burstfeat.out.md` in the data folder
and `learnings.md`): the 7 B columns on all 32,459 frames (6,969 bursts,
1,943 singles); `nn_cos`, `sharp_z`, `sharp_rank` have a value on 30,516
frames (94.0%, NaN exactly on the singles), `ef_gap`, `ef_rank` on 26,523
(81.7%), `ear_gap`, `ear_rank` on 19,718 (60.7%). Against `G0`'s three
burst-relative base columns, the sharpness pair largely restates
`tech[:, 0]` (Spearman +0.694 for `sharp_z`, -0.652 for `sharp_rank`);
`nn_cos`, `ef_rank` and `ear_rank` are nearly independent of the base
columns (|rho| <= 0.145); the two gaps fall with burst size (rho -0.467 and
-0.461). Every Step 1 check passed (one row per dump frame; `sharp_rank` 0
exactly where `tech[:, 0]` is 0; `nn_cos` NaN exactly on the singles).

Not known before this section, with no prior number: any score of `G0+B`,
`D0` or `D0+B`, their label-free cuts, and any effect A or B.

**Selection bias.** The design of this plan (the retarget, the cue-like B
columns, the bar) was chosen **after** reading `G0`'s, `eye_focus`'s and
random's numbers on these same 600 labels, and `D0` / `D0+B` are trained on
them (held out by folder). No hyperparameter and no feature is selected on
them (Cells), so the leave-one-folder-out read is an honest estimate of the
fixed recipe, but the choice of the recipe itself is not blind. The Decision
weighs this; a pass leads to a fresh blind round before any app work
(Follow-up rule).

### Choices

Decided by the user on 2026-10-19, before any new score existed (plan.md,
"Choices"):

1. **The bar: looser**, `RR` <= **25%** with its stratified 97.5% quantile <=
   **35%**. Rejected: keep discard-labels' `RR` <= 15% with the quantile <=
   25%. Both keep `PL` <= 5% (full dump), removed share >= 4.5%, beat random
   and beat `G0` paired.
2. **A fresh blind evaluation sample: none now.** Leave-one-folder-out on the
   600 labels decides; a pass triggers a confirmatory blind round as the
   first step of the in-app plan. Rejected: draw one in this plan.
3. **The 1,200 strict labels in training: out.** They stay the secondary
   coherence check. Rejected: in, as positives only.
4. **The "beat random" form: the discardable base rate.** A cut beats random
   when its `DP`'s stratified 2.5% quantile is above the base rate's point
   estimate, **48.45%**. Rejected: the paired difference against the seed-0
   random cut at the same `q`, which removes only 27 sampled frames at q5.
   The seed-0 random rows are printed either way.

### Labels, population and weights

- The 600 rows of `D:\Photos\tests\2026-10-18-discard-labels\labels.tsv`
  (211 `keep`, 389 `discard`); **`discard` is the positive class**.
- **Population**: the dump minus the 1,200 frames of the strict sample,
  **31,259 frames**; `1/π` from `inclusion_prob` (sums to 31,259).
- Every estimate is **Horvitz–Thompson weighted** by `1/π` (a share is the
  ratio of the `1/π` sums over its numerator and denominator sets), with the
  **unweighted sample value** beside it, as discard.md.
- **The strict labels do not enter training** (choice 3). They stay the
  secondary coherence check: the strict-bad rate of each new cell's own
  bottom bands (per-folder q5, 5–10%, 10–20%, above 20%) beside its
  `discard` rate, weighted by the strict sample's own `π`. Expected: the
  strict-bad rate below the `discard` rate in each band; a check, not a bar.

### Cells and their training

Four cells on the same features of the same dump, every score **higher
keeps**, every frame scored by a model that did not see its folder:

- **`G0`**: the stored column of
  `D:\Photos\tests\2026-10-16-reject-filter-eval\scores_reject.tsv`,
  unchanged (the known reference).
- **`G0+B`**: `rejectfilter.g0_fold` with the 7 B columns of
  `burst_features.tsv` appended **raw (NaN kept)** to the 46-column
  `folder_gate.design(d, train_mask, "full")` (the NaN of the technical
  columns restored as `g0_fold` does); target `pick`; the dump's per-folder
  class weights rescaled to mean 1 per fold; `rf.HGB`; one fit per held-out
  folder on **every dump frame** of the other 26 folders; scored by
  `decision_function` on every frame of the held-out folder.
- **`D0`**: `rf.HGB` on the same 46 columns (NaN of the technical columns
  restored), target **`discard`**, fit on the **other folders' labeled
  frames** only (`labels.tsv` joined by (`folder`, `file`)) with `1/π`
  weights rescaled to mean 1 per fold (the archived `badmodel.fold` `B0`
  recipe); scored on **every dump frame** of the held-out folder by
  **`-decision_function`**, so higher keeps; 27 folds. A fold trains on
  504–597 labeled frames (the held-out folder holds 3 to 96 of the 600).
- **`D0+B`**: `D0` with the 7 B columns appended raw.
- **No hyperparameter or feature selection**: `rf.HGB` as is (200
  iterations, learning rate 0.05, 15 leaves, l2 1.0, seed 0) for all four
  cells, so effect A is the target (and its training set) alone and effect B
  the 7 columns alone; the B set is exactly Step 1's. No nested CV is
  therefore needed and the leave-one-folder-out read is the estimate.
- Reported label-free per fold: the training frames, positives and the
  **Kish effective sample size** `(Σw)² / Σw²` of the training weights (the
  `control` weights reach 238 per frame).
- **References**, not candidates: `eye_focus` (missing last) and random
  (seed-0 `tb`), as discard.md.

### Cuts

- Every cell and reference is cut **per folder on the full dump**: remove the
  bottom `floor(q * n / 100)` of the folder's frames by the score, ties by
  the seed-0 draw (`bf.graded_cut`), **`q` = 5 / 10 / 20**. No
  `quantile@95` for the new cells (a pick-recall device).
- The cut is computed on the full dump and then read on the sampled frames;
  removed share and `PL` are exact counts. **The label-free `PL` (full dump,
  population beside it) and removed share (population, full dump beside it)
  of each cell's cut are reported first, in Step 3, and decide those two
  conditions before any label is read against a new score.** The cut size
  per folder does not depend on the score, so the removed share differs
  between scores only by how many strict-sample frames fall inside (4.83–
  4.95% at q5 for the scores discard.md read).

### Primary metrics

Per cell and cut, exactly discard.md's, weighted with the unweighted value
and the sampled counts beside them:

1. **Discard precision `DP@q`**: the share of removed frames labeled
   `discard`.
2. **Rescue rate `RR@q`** = 1 − `DP@q`.
3. **Pick loss `PL@q`**: the share of the dump's picks inside the cut,
   label-free, **on the full dump** (32,459 frames, 4,419 picks) for the bar,
   the population value beside it.
4. **Discard recall `DR@q`** and **miss rate `MR@q`** = 1 − `DR@q`.
5. **Discardable share outside the cut `DO@q`**.

Plus the **removed share** (population, the bar's; full dump beside it) and,
threshold-free, the **weighted AUC** of `discard` vs `keep` per cell,
`roc_auc_score(discard, -score, sample_weight=1/π)`, a missing value as the
folder's lowest finite score minus 1 (only `eye_focus` has any). The
discardable base rate, with its interval, is printed first as the check
against discard.md.

### Effects

Paired, on the same bootstrap draws, in points of `DP` and of `DR` at each
`q` and in AUC:

- **A (retarget)**: `D0` − `G0` and `D0+B` − `G0+B`.
- **B (burst-relative columns)**: `G0+B` − `G0` and `D0+B` − `D0`.
- **Interaction**: (`D0+B` − `D0`) − (`G0+B` − `G0`).
- **A+B**: `D0+B` − `G0`.
- **Each new cell against the references**: `G0+B`, `D0`, `D0+B` each minus
  `eye_focus` and minus random (seed 0) at the same `q`.

The bootstrap resamples the labeled frames with the models and the cuts held
fixed: the training variance of `D0` / `D0+B` (which would change with the
resampled labels) is not captured. The folder-cluster bootstrap is the
sensitivity reading for it.

### Intervals

- A 1,000-draw **stratified bootstrap** over the sampled frames: resample
  with replacement within `b05`, `b10`, `b20` and each (`control`, folder)
  cell of `strata.tsv`, its drawn count kept; seed 0; **one count matrix
  reused across every quantity and every cell**, so paired effects share
  draws. Reported as the 2.5% / 97.5% quantiles.
- A **folder-cluster bootstrap** (the 27 folders with replacement, seed 0,
  1,000 draws) beside it for the base rate, the bars' quantities and the
  paired effects. It does not decide; a pass whose cluster interval crosses
  the bar is reported as **fragile**.
- The base rate in the "beat random" condition is the fixed point estimate
  48.45% (choice 4), not resampled.

### Bars

"Hide / sort last" semantics, as discard-labels: the cut's frames are hidden
from what the user picks from, or sorted last, and stay recoverable.

- **Usable discard cut** (the chosen looser bar, choice 1): a cut of a
  **new cell** (`G0+B`, `D0` or `D0+B` at q5, q10 or q20) is usable when all
  of these hold:
  1. weighted `RR` <= **25%**, with the stratified 97.5% quantile of `RR` <=
     **35%**;
  2. `PL` <= **5%** (full dump, label-free);
  3. it removes **>= 4.5%** of the population's frames (label-free);
  4. **beats random**: its `DP`'s stratified 2.5% quantile is above
     **48.45%** (choice 4);
  5. **beats `G0`**: its weighted `DP` minus `G0`'s at the same `q` has the
     paired stratified **2.5% quantile above zero**.
- **The rejected bar** (written out, read in Step 4 as a second table marked
  as a reading only): conditions 2–5 the same, condition 1 `RR` <= **15%**
  with the 97.5% quantile <= **25%**.
- **Best cut**: among the passing cuts, the cell with the largest weighted
  `DP` gain over `G0` at the deepest passing `q`; its deepest passing cut.
- `G0`, `eye_focus` and random are **not candidates**; they are read
  through the effects only.

**What `G0` itself already meets under the chosen bar**, from discard.md's
numbers (condition 5 does not apply to `G0`):

| `G0` cut | `RR` <= 25% | `RR` 97.5% quantile <= 35% | `PL` <= 5% | Removed >= 4.5% | `DP` 2.5% quantile > 48.45% | Conditions 1–4 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| q5 | yes (17.3) | yes (24.0) | yes (0.68) | yes (4.92) | yes (76.0) | all met |
| q10 | yes (23.7) | yes (28.7) | yes (2.10) | yes (9.87) | yes (71.3) | all met |
| q20 | no (27.9) | yes (32.5) | no (5.18) | yes (19.80) | yes (67.5) | not met |

So under the chosen looser bar and the base-rate form of "beat random",
**`G0` q5 and q10 would already pass conditions 1–4**; under the rejected
bar `G0` q5 fails `RR` <= 15% (17.3) and q10 fails both `RR` conditions. A
pass of a new cell at q5 or q10 therefore adds exactly one thing over `G0`:
condition 5, a `DP` gain over `G0` at the same depth with its paired 2.5%
quantile above zero. At q20 a new cell must also bring `RR` under 25% and
`PL` under 5%, which `G0` does not. The Decision states that `G0` itself
would have met the rescue and base-rate conditions under this bar, and does
not read a new cell's pass of conditions 1–4 as an improvement.

For reference only (not candidates): `eye_focus` q5 fails the `RR` 97.5%
quantile (43.3) and q10 fails `PL` (9.14); random's `DP` 2.5% quantile is
below 48.45% at every `q` (19.8, 37.9, 40.5).

### Follow-up rule

Applied in this order:

1. **A usable cut exists** → the Decision recommends a later plan for an
   in-app **discard candidates sorted last** view with a rescue loop at the
   best cut's depth, whose **first step is a fresh blind confirmatory
   round** (candidates and controls mixed and unmarked, as discard-labels
   Step 1), because the pass was read on the labels the recipe was chosen
   after and trained on, and because of the automation-bias finding (the
   unblinded 70 of 71 against the blind `RR@5` 17.3%).
2. **No usable cut, but effect A or A+B positive at q5** (`D0` − `G0` or
   `D0+B` − `G0` at q5 in `DP` with its paired stratified 2.5% quantile above
   zero) → **more labels**: a later plan for a second blind round sized at
   about 3 s per frame (600 frames about 30 minutes, 1,000 about 50 minutes)
   to retrain and re-read.
3. **Otherwise** → **nothing**.

### Multiplicity and population

- **Multiplicity**: 3 new cells, 3 cuts, 2 references and about 10 paired
  effects per `q`, read without correction, so a chance pass is possible;
  the Decision reads a lone marginal pass, or one that is fragile on the
  folder-cluster bootstrap, as such.
- **Population caveat**: one photographer's 27 folders, mostly basketball,
  minus the 1,200 strict-sample frames; 600 labels. The Decision speaks of
  this dump.

### What happens after

No step of this plan changes the app. Step 3 fits the three new cells and
prints only label-free readings; Step 4 reads every quantity above against
the labels; the Decision (Step 5) records the outcome against these bars and
names the follow-up by the rule above, as a recommendation only.

## Results

Run 2026-10-20 by `retargetread.py` (`python -I -B retargetread.py
D:\Photos\tests\2026-10-12-rank-app D:\Photos\tests\2026-10-15-folder-wide-eval
D:\Photos\tests\2026-10-16-reject-filter-eval
D:\Photos\tests\2026-10-18-discard-labels D:\Photos\tests\2026-10-17-bad-labels
D:\Photos\tests\2026-10-19-discard-retarget`), its stdout saved as
`retargetread.out.md` in the data folder. On the 600 labels (211 `keep`, 389
`discard`) and the scores of Step 3 (`scores_retarget.tsv`). Every interval
is the stratified bootstrap's 95% (1,000 draws, seed 0, one count matrix for
every quantity and cell) unless marked cluster (the folder-cluster bootstrap,
1,000 draws, seed 0); values are weighted by `1/π` unless marked unweighted.
No cut leaves a frame out (every score is finite on every frame;
`eye_focus`'s missing values sort last). Wall time 16 s (load 4 s; cuts,
metrics, AUCs and bootstraps 12 s).

### Reference checks

Every check passes: the stored `G0`, `eye_focus` and random reproduce
discard.md's Results digit for digit on this code path.

| Check | Expected | This run |
| --- | ---: | ---: |
| `strata.tsv` rows whose population, drawn count or `π` differ when recomputed from the dump | 0 of 30 | 0 of 30 |
| Sampled frames whose recomputed `G0` band / `π` differ from `labels.tsv` | 0 / 0 | 0 / 0 |
| Sampled frames in the strict sample | 0 | 0 |
| Sum of `1/π` over the sampled frames | 31,259 | 31,259.0 |
| Discardable base rate % | 48.45 [42.03, 55.11] | 48.45 [42.03, 55.11] (cluster [40.74, 53.87]; unweighted 64.83) |
| `G0` `RR` % q5 / q10 / q20 | 17.3 [11.3, 24.0] / 23.7 [18.7, 28.7] / 27.9 [23.3, 32.5] | the same |
| `G0` `PL` % q5 / q10 / q20; cluster `RR` 97.5% quantile | 0.68 / 2.10 / 5.18; 25.7 / 32.8 / 36.3 | the same |
| `G0` AUC | 0.710 [0.646, 0.771] | 0.710 [0.646, 0.771] |
| `eye_focus` `RR` % q5 / q10 / q20; `PL` % | 16.8 [1.2, 43.3] / 14.3 [2.9, 29.9] / 28.3 [15.7, 40.2]; 4.44 / 9.14 / 15.59 | the same |
| `eye_focus` AUC | 0.777 [0.716, 0.841] | 0.777 [0.716, 0.841] |
| Random `DP` % q5 / q10 / q20 | 52.0 [19.8, 85.5] / 61.5 [37.9, 84.8] / 56.0 [40.5, 71.5] | the same |
| Random AUC | 0.528 [0.455, 0.604] | 0.528 [0.455, 0.604] |
| `G0` q5 − random q5, `DP` points | +30.7 [-3.8, +63.9] | +30.7 [-3.8, +63.9] |

### Main table

`DP` / `RR` / `DR` / `MR` / `DO` weighted with the stratified interval and
the unweighted value in brackets; `PL` on the full dump (population after
the slash), label-free; Removed: the population's share (the full dump's
after the slash), label-free; Sampled: the sampled frames the cut removes /
of them `discard`; Picks: the sampled dump picks the cut removes / of them
`discard`.

| Score | Cut | Sampled / discard | Picks / discard | `DP` % | `RR` % | `PL` % | `DR` % | `MR` % | `DO` % | Removed % |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `G0` | q5 | 150 / 124 | 7 / 5 | 82.7 [76.0, 88.7] (82.7) | 17.3 [11.3, 24.0] (17.3) | 0.68 / 0.68 | 8.4 [7.2, 9.8] (31.9) | 91.6 [90.2, 92.8] (68.1) | 46.7 [39.9, 53.8] (58.9) | 4.92 / 4.96 |
| `G0` | q10 | 300 / 229 | 11 / 6 | 76.3 [71.3, 81.3] (76.3) | 23.7 [18.7, 28.7] (23.7) | 2.10 / 2.02 | 15.5 [13.6, 18.1] (58.9) | 84.5 [81.9, 86.4] (41.1) | 45.4 [38.3, 52.9] (53.3) | 9.87 / 9.96 |
| `G0` | q20 | 450 / 331 | 16 / 8 | 72.1 [67.5, 76.7] (73.6) | 27.9 [23.3, 32.5] (26.4) | 5.18 / 5.03 | 29.5 [25.9, 33.9] (85.1) | 70.5 [66.1, 74.1] (14.9) | 42.6 [34.8, 50.7] (38.7) | 19.80 / 19.97 |
| `G0+B` | q5 | 147 / 119 | 6 / 4 | 80.8 [74.2, 86.9] (81.0) | 19.2 [13.1, 25.8] (19.0) | 0.77 / 0.75 | 8.3 [6.9, 9.9] (30.6) | 91.7 [90.1, 93.1] (69.4) | 46.8 [40.0, 53.8] (59.6) | 4.91 / 4.96 |
| `G0+B` | q10 | 249 / 196 | 9 / 5 | 77.8 [72.1, 83.1] (78.7) | 22.2 [16.9, 27.9] (21.3) | 2.04 / 2.00 | 14.3 [12.0, 16.9] (50.4) | 85.7 [83.1, 88.0] (49.6) | 45.6 [38.7, 53.0] (55.0) | 9.87 / 9.96 |
| `G0+B` | q20 | 411 / 309 | 14 / 8 | 72.5 [64.1, 80.1] (75.2) | 27.5 [19.9, 35.9] (24.8) | 4.93 / 4.87 | 31.6 [26.0, 37.8] (79.4) | 68.4 [62.2, 74.0] (20.6) | 42.0 [34.3, 50.3] (42.3) | 19.83 / 19.97 |
| `D0` | q5 | 58 / 50 | 1 / 0 | 59.5 [35.0, 93.3] (86.2) | 40.5 [6.7, 65.0] (13.8) | 2.29 / 2.26 | 5.3 [3.2, 8.5] (12.9) | 94.7 [91.5, 96.8] (87.1) | 48.0 [41.6, 55.0] (62.5) | 4.97 / 4.96 |
| `D0` | q10 | 113 / 94 | 2 / 1 | 65.3 [46.6, 85.9] (83.2) | 34.7 [14.1, 53.4] (16.8) | 4.84 / 4.87 | 11.0 [7.2, 15.8] (24.2) | 89.0 [84.2, 92.8] (75.8) | 47.0 [40.3, 53.8] (60.6) | 9.97 / 9.96 |
| `D0` | q20 | 222 / 187 | 7 / 3 | 66.5 [53.7, 79.7] (84.2) | 33.5 [20.3, 46.3] (15.8) | 11.59 / 11.64 | 26.2 [19.7, 33.7] (48.1) | 73.8 [66.3, 80.3] (51.9) | 44.2 [37.2, 51.7] (53.4) | 20.02 / 19.97 |
| `D0+B` | q5 | 58 / 50 | 0 / 0 | 67.3 [40.5, 94.3] (86.2) | 32.7 [5.7, 59.5] (13.8) | 2.26 / 2.26 | 7.5 [3.8, 11.8] (12.9) | 92.5 [88.2, 96.2] (87.1) | 47.4 [40.8, 54.1] (62.5) | 4.96 / 4.96 |
| `D0+B` | q10 | 125 / 102 | 2 / 0 | 57.3 [41.0, 76.1] (81.6) | 42.7 [23.9, 59.0] (18.4) | 5.09 / 5.15 | 15.5 [9.9, 21.6] (26.2) | 84.5 [78.4, 90.1] (73.8) | 47.1 [40.4, 54.4] (60.4) | 10.00 / 9.96 |
| `D0+B` | q20 | 231 / 191 | 10 / 5 | 64.7 [52.3, 77.7] (82.7) | 35.3 [22.3, 47.7] (17.3) | 11.38 / 11.52 | 28.2 [21.4, 36.0] (49.1) | 71.8 [64.0, 78.6] (50.9) | 44.1 [36.9, 51.6] (53.7) | 20.02 / 19.97 |
| `eye_focus` (missing last) | q5 | 38 / 33 | 2 / 1 | 83.2 [56.7, 98.8] (86.8) | 16.8 [1.2, 43.3] (13.2) | 4.44 / 4.44 | 8.1 [3.7, 13.2] (8.5) | 91.9 [86.8, 96.3] (91.5) | 46.7 [40.4, 53.4] (63.3) | 4.83 / 4.96 |
| `eye_focus` (missing last) | q10 | 82 / 72 | 3 / 2 | 85.7 [70.1, 97.1] (87.8) | 14.3 [2.9, 29.9] (12.2) | 9.14 / 8.98 | 15.2 [9.4, 22.0] (18.5) | 84.8 [78.0, 90.6] (81.5) | 45.0 [38.6, 51.9] (61.2) | 9.78 / 9.96 |
| `eye_focus` (missing last) | q20 | 189 / 160 | 8 / 5 | 71.7 [59.8, 84.3] (84.7) | 28.3 [15.7, 40.2] (15.3) | 15.59 / 15.50 | 31.7 [24.1, 40.0] (41.1) | 68.3 [60.0, 75.9] (58.9) | 42.1 [35.1, 49.9] (55.7) | 19.87 / 19.97 |
| Random (seed 0) | q5 | 27 / 16 | 0 / 0 | 52.0 [19.8, 85.5] (59.3) | 48.0 [14.5, 80.2] (40.7) | 4.71 / 4.70 | 2.6 [0.8, 5.4] (4.1) | 97.4 [94.6, 99.2] (95.9) | 48.4 [41.9, 55.2] (65.1) | 4.95 / 4.96 |
| Random (seed 0) | q10 | 52 / 36 | 1 / 1 | 61.5 [37.9, 84.8] (69.2) | 38.5 [15.2, 62.1] (30.8) | 9.48 / 9.48 | 10.6 [5.4, 16.7] (9.3) | 89.4 [83.3, 94.6] (90.7) | 47.3 [40.7, 54.0] (64.4) | 9.99 / 9.96 |
| Random (seed 0) | q20 | 121 / 77 | 6 / 2 | 56.0 [40.5, 71.5] (63.6) | 44.0 [28.5, 59.5] (36.4) | 19.26 / 19.36 | 21.0 [13.8, 28.2] (19.8) | 79.0 [71.8, 86.2] (80.2) | 46.8 [39.9, 54.4] (65.1) | 19.98 / 19.97 |

The folder-cluster intervals of every row are in `retargetread.out.md`; the
bars' quantities carry theirs in the bars tables below.

### Threshold-free: weighted AUC of `discard` vs `keep`

| Score | AUC | Cluster | Unweighted |
| --- | ---: | ---: | ---: |
| `G0` | 0.710 [0.646, 0.771] | [0.655, 0.759] | 0.699 |
| `G0+B` | 0.689 [0.622, 0.755] | [0.635, 0.740] | 0.699 |
| `D0` | 0.761 [0.697, 0.827] | [0.683, 0.820] | 0.791 |
| `D0+B` | 0.731 [0.667, 0.798] | [0.658, 0.793] | 0.786 |
| `eye_focus` (missing last) | 0.777 [0.716, 0.841] | [0.719, 0.832] | 0.803 |
| Random (seed 0) | 0.528 [0.455, 0.604] | [0.445, 0.627] | 0.502 |

### Effects

Paired on the same draws (labeled frames resampled, models and cuts fixed):
the point, the stratified interval and the cluster interval, in points of
weighted `DP` and `DR` and in AUC.

`DP`, points:

| Effect | q5 | Cluster | q10 | Cluster | q20 | Cluster |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A: `D0` − `G0` | -23.2 [-48.1, +10.2] | [-45.6, +9.0] | -11.0 [-30.4, +8.9] | [-26.3, +12.1] | -5.6 [-18.6, +7.6] | [-18.8, +5.8] |
| A: `D0+B` − `G0+B` | -13.5 [-41.4, +14.2] | [-36.8, +16.0] | -20.5 [-38.0, -2.2] | [-36.0, -4.0] | -7.8 [-22.2, +7.7] | [-21.6, +5.7] |
| B: `G0+B` − `G0` | -1.9 [-6.5, +2.9] | [-6.7, +2.1] | +1.4 [-2.9, +5.9] | [-2.2, +5.7] | +0.3 [-7.8, +6.7] | [-6.6, +7.4] |
| B: `D0+B` − `D0` | +7.9 [-3.1, +19.6] | [-2.4, +19.2] | -8.0 [-25.2, +5.6] | [-25.0, +1.8] | -1.9 [-7.9, +4.2] | [-9.9, +5.6] |
| Interaction | +9.7 [-2.7, +22.0] | [+2.3, +18.8] | -9.5 [-26.8, +4.9] | [-29.5, +2.0] | -2.2 [-11.4, +8.3] | [-13.3, +8.6] |
| A+B: `D0+B` − `G0` | -15.3 [-43.9, +11.8] | [-39.5, +12.3] | -19.0 [-36.7, -0.8] | [-33.0, -3.5] | -7.5 [-20.1, +5.7] | [-19.3, +3.7] |
| `G0+B` − `eye_focus` | -2.4 [-20.6, +24.2] | [-21.4, +29.5] | -8.0 [-20.3, +8.7] | [-19.9, +6.9] | +0.7 [-13.1, +14.2] | [-10.8, +12.6] |
| `G0+B` − random | +28.8 [-5.7, +61.1] | [-3.3, +65.0] | +16.3 [-7.8, +40.9] | [-17.0, +41.9] | +16.4 [+2.0, +31.7] | [-1.1, +34.8] |
| `D0` − `eye_focus` | -23.7 [-57.3, +15.7] | [-57.9, +30.4] | -20.4 [-43.4, +3.0] | [-39.1, +6.8] | -5.2 [-16.1, +5.4] | [-15.2, +3.9] |
| `D0` − random | +7.5 [-32.7, +52.6] | [-36.8, +55.2] | +3.8 [-25.6, +33.2] | [-29.6, +39.7] | +10.5 [-7.5, +29.8] | [-15.3, +34.4] |
| `D0+B` − `eye_focus` | -15.8 [-48.6, +19.1] | [-47.5, +31.3] | -28.4 [-48.6, -7.6] | [-46.3, -6.7] | -7.0 [-17.3, +3.4] | [-17.0, +2.2] |
| `D0+B` − random | +15.4 [-27.2, +59.0] | [-27.6, +59.5] | -4.2 [-32.7, +23.7] | [-41.5, +29.3] | +8.7 [-9.2, +27.9] | [-14.8, +32.4] |

`DR`, points, and AUC:

| Effect | `DR` q5 | Cluster | `DR` q10 | Cluster | `DR` q20 | Cluster | AUC | Cluster |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| A: `D0` − `G0` | -3.1 [-5.6, +0.1] | [-6.5, +0.2] | -4.5 [-8.8, +0.3] | [-8.9, -1.0] | -3.3 [-10.9, +4.8] | [-11.9, +4.0] | +0.050 [-0.017, +0.126] | [-0.034, +0.125] |
| A: `D0+B` − `G0+B` | -0.8 [-4.9, +4.1] | [-5.2, +3.0] | +1.2 [-4.6, +7.7] | [-3.7, +5.5] | -3.4 [-11.9, +5.6] | [-11.4, +2.6] | +0.042 [-0.027, +0.120] | [-0.040, +0.114] |
| B: `G0+B` − `G0` | -0.1 [-1.1, +1.0] | [-1.3, +0.6] | -1.3 [-2.5, +0.1] | [-2.7, +0.1] | +2.1 [-2.5, +7.6] | [-2.0, +6.7] | -0.021 [-0.046, +0.002] | [-0.043, +0.006] |
| B: `D0+B` − `D0` | +2.2 [-0.7, +6.0] | [-0.5, +5.4] | +4.5 [+0.4, +9.4] | [+0.4, +9.4] | +2.0 [-2.5, +6.7] | [-2.6, +7.6] | -0.029 [-0.052, -0.007] | [-0.054, -0.006] |
| Interaction | +2.3 [-1.0, +6.2] | [-0.2, +5.5] | +5.8 [+1.5, +10.9] | [+2.3, +9.8] | -0.1 [-7.4, +7.1] | [-4.7, +4.2] | -0.008 [-0.043, +0.025] | [-0.039, +0.018] |
| A+B: `D0+B` − `G0` | -0.9 [-4.7, +3.7] | [-5.9, +3.1] | -0.1 [-5.9, +6.2] | [-5.7, +4.6] | -1.3 [-9.3, +7.2] | [-8.4, +4.2] | +0.021 [-0.046, +0.098] | [-0.066, +0.095] |
| `G0+B` − `eye_focus` | +0.2 [-5.3, +5.1] | [-4.3, +4.8] | -0.9 [-7.7, +5.3] | [-6.9, +6.2] | -0.1 [-9.5, +8.7] | [-8.6, +9.5] | -0.088 [-0.166, -0.020] | [-0.164, -0.017] |
| `G0+B` − random | +5.6 [+2.4, +8.4] | [+2.3, +8.7] | +3.6 [-2.6, +10.0] | [-1.3, +9.5] | +10.5 [+1.1, +21.1] | [-0.3, +22.3] | +0.161 [+0.056, +0.259] | [+0.073, +0.223] |
| `D0` − `eye_focus` | -2.7 [-8.6, +2.8] | [-8.1, +3.2] | -4.2 [-10.2, +0.3] | [-8.1, +0.6] | -5.5 [-11.3, +0.2] | [-11.1, -1.3] | -0.017 [-0.059, +0.026] | [-0.054, +0.019] |
| `D0` − random | +2.7 [-0.8, +6.7] | [-1.7, +6.6] | +0.4 [-5.0, +5.9] | [-5.1, +5.9] | +5.2 [-4.2, +16.7] | [-9.1, +17.4] | +0.233 [+0.139, +0.330] | [+0.086, +0.345] |
| `D0+B` − `eye_focus` | -0.6 [-5.4, +4.1] | [-4.4, +3.7] | +0.3 [-6.8, +7.5] | [-5.6, +7.4] | -3.5 [-10.1, +2.8] | [-10.0, +2.9] | -0.046 [-0.085, -0.006] | [-0.086, -0.011] |
| `D0+B` − random | +4.8 [+0.3, +10.0] | [-0.5, +9.9] | +4.9 [-2.0, +12.0] | [-2.3, +12.1] | +7.2 [-1.6, +18.0] | [-7.2, +19.7] | +0.203 [+0.111, +0.308] | [+0.059, +0.311] |

The interaction is (`D0+B` − `D0`) − (`G0+B` − `G0`).

### The coherence check on the strict labels

Each cell's own per-folder bands on the full dump: the `discard` rate on the
600 labels beside the strict-bad rate on the strict sample (its own `π` and
strata). `G0`'s rows reproduce discard.md's per-band table.

| Score | Band | Sampled / discard | `discard` rate % | Strict frames / strict bad | Strict-bad rate % | Strict below discard |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `G0` | bottom 5% | 150 / 124 | 82.7 [76.0, 88.7] | 71 / 22 | 27.3 [16.1, 41.4] | yes |
| `G0` | 5–10% | 150 / 105 | 70.0 [62.7, 78.0] | 77 / 10 | 12.5 [3.9, 25.3] | yes |
| `G0` | 10–20% | 150 / 102 | 68.0 [60.7, 75.3] | 146 / 17 | 10.1 [5.2, 16.9] | yes |
| `G0` | above 20% | 150 / 58 | 42.6 [34.8, 50.7] | 906 / 31 | 3.7 [2.2, 5.2] | yes |
| `G0+B` | bottom 5% | 147 / 119 | 80.8 [74.2, 86.9] | 74 / 17 | 17.4 [9.9, 26.6] | yes |
| `G0+B` | 5–10% | 102 / 77 | 73.9 [64.6, 83.5] | 75 / 11 | 15.2 [5.4, 28.4] | yes |
| `G0+B` | 10–20% | 162 / 113 | 68.6 [55.5, 81.3] | 135 / 19 | 12.4 [6.4, 20.5] | yes |
| `G0+B` | above 20% | 189 / 80 | 42.0 [34.3, 50.3] | 916 / 33 | 3.7 [2.3, 5.3] | yes |
| `D0` | bottom 5% | 58 / 50 | 59.5 [35.0, 93.3] | 56 / 10 | 10.6 [3.4, 21.5] | yes |
| `D0` | 5–10% | 55 / 44 | 71.9 [44.3, 93.2] | 61 / 10 | 13.9 [3.9, 25.6] | yes |
| `D0` | 10–20% | 109 / 93 | 67.5 [50.3, 85.1] | 106 / 11 | 7.1 [2.8, 13.3] | yes |
| `D0` | above 20% | 378 / 202 | 44.2 [37.2, 51.7] | 977 / 49 | 4.5 [3.1, 6.1] | yes |
| `D0+B` | bottom 5% | 58 / 50 | 67.3 [40.5, 94.3] | 57 / 11 | 12.4 [4.4, 23.9] | yes |
| `D0+B` | 5–10% | 67 / 52 | 50.3 [29.7, 74.8] | 50 / 7 | 7.4 [2.7, 14.7] | yes |
| `D0+B` | 10–20% | 106 / 89 | 76.7 [60.2, 93.0] | 117 / 16 | 14.2 [6.4, 23.1] | yes |
| `D0+B` | above 20% | 369 / 198 | 44.1 [36.9, 51.6] | 976 / 46 | 4.0 [2.7, 5.5] | yes |

### The bars applied

Under the **chosen bar** (`RR` <= 25%, 97.5% quantile <= 35%). Conditions in
order: (1) `RR`, (1) its quantile, (2) `PL` <= 5%, (3) removed >= 4.5%, (4)
`DP` 2.5% quantile > 48.45%, (5) `DP` − `G0` > 0 with its paired 2.5%
quantile > 0. Cluster values in parentheses.

| Cell | Cut | `RR` % / 97.5% quantile (cluster) | `PL` % | Removed % | `DP` 2.5% quantile (cluster) | `DP` − `G0`, points / 2.5% quantile (cluster) | Conditions 1 / 1 / 2 / 3 / 4 / 5 | Verdict |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `G0+B` | q5 | 19.2 / 25.8 (28.1) | 0.77 | 4.91 | 74.2 (71.9) | -1.9 / -6.5 (-6.7) | yes / yes / yes / yes / yes / no | not usable |
| `G0+B` | q10 | 22.2 / 27.9 (30.7) | 2.04 | 9.87 | 72.1 (69.3) | +1.4 / -2.9 (-2.2) | yes / yes / yes / yes / yes / no | not usable |
| `G0+B` | q20 | 27.5 / 35.9 (35.3) | 4.93 | 19.83 | 64.1 (64.7) | +0.3 / -7.8 (-6.6) | no / no / yes / yes / yes / no | not usable |
| `D0` | q5 | 40.5 / 65.0 (65.7) | 2.29 | 4.97 | 35.0 (34.3) | -23.2 / -48.1 (-45.6) | no / no / yes / yes / no / no | not usable |
| `D0` | q10 | 34.7 / 53.4 (51.2) | 4.84 | 9.97 | 46.6 (48.8) | -11.0 / -30.4 (-26.3) | no / no / yes / yes / no / no | not usable |
| `D0` | q20 | 33.5 / 46.3 (49.4) | 11.59 | 20.02 | 53.7 (50.6) | -5.6 / -18.6 (-18.8) | no / no / no / yes / yes / no | not usable |
| `D0+B` | q5 | 32.7 / 59.5 (59.0) | 2.26 | 4.96 | 40.5 (41.0) | -15.3 / -43.9 (-39.5) | no / no / yes / yes / no / no | not usable |
| `D0+B` | q10 | 42.7 / 59.0 (60.1) | 5.09 | 10.00 | 41.0 (39.9) | -19.0 / -36.7 (-33.0) | no / no / no / yes / no / no | not usable |
| `D0+B` | q20 | 35.3 / 47.7 (50.0) | 11.38 | 20.02 | 52.3 (50.0) | -7.5 / -20.1 (-19.3) | no / no / no / yes / yes / no | not usable |

**No usable discard cut under the chosen bar; no best cut.**

Under the **rejected bar** (`RR` <= 15%, 97.5% quantile <= 25%), **a reading
only**; conditions 2–5 and every quantity as above.

| Cell | Cut | `RR` <= 15% | 97.5% quantile <= 25% | Conditions 2 / 3 / 4 / 5 | Verdict |
| --- | ---: | ---: | ---: | ---: | ---: |
| `G0+B` | q5 | no (19.2) | no (25.8) | yes / yes / yes / no | not usable |
| `G0+B` | q10 | no (22.2) | no (27.9) | yes / yes / yes / no | not usable |
| `G0+B` | q20 | no (27.5) | no (35.9) | yes / yes / yes / no | not usable |
| `D0` | q5 | no (40.5) | no (65.0) | yes / yes / no / no | not usable |
| `D0` | q10 | no (34.7) | no (53.4) | yes / yes / no / no | not usable |
| `D0` | q20 | no (33.5) | no (46.3) | no / yes / yes / no | not usable |
| `D0+B` | q5 | no (32.7) | no (59.5) | yes / yes / no / no | not usable |
| `D0+B` | q10 | no (42.7) | no (59.0) | no / yes / no / no | not usable |
| `D0+B` | q20 | no (35.3) | no (47.7) | no / yes / yes / no | not usable |

No usable cut under the rejected bar either; no best cut.

### The follow-up rule's inputs

| Input | Value |
| --- | ---: |
| A usable cut under the chosen bar | no |
| A at q5: `D0` − `G0`, `DP` points (cluster) | -23.2 [-48.1, +10.2] ([-45.6, +9.0]) |
| A+B at q5: `D0+B` − `G0`, `DP` points (cluster) | -15.3 [-43.9, +11.8] ([-39.5, +12.3]) |
| The branch these inputs select | 3 (otherwise: nothing) |

Neither paired 2.5% quantile is above zero, so branch 2 does not apply; the
Decision (Step 5) reads this.

### Reading

The stored `G0`, `eye_focus` and random reproduce discard.md exactly, and
the strata, `π` and base rate (48.45%) check. No cut of `G0+B`, `D0` or
`D0+B` meets the chosen bar, and none meets the rejected one. `G0+B` q5 and
q10 meet conditions 1–4 (`RR` 19.2 [13.1, 25.8] and 22.2 [16.9, 27.9], as
`G0`'s 17.3 and 23.7 do) and fail condition 5: their `DP` differs from
`G0`'s by -1.9 [-6.5, +2.9] and +1.4 [-2.9, +5.9] points. The discard
target's cells rescue 32.7–42.7% of what they remove at every cut (weighted),
their `DP` 2.5% quantiles are below 48.45% at q5 and q10, and they put
2.26–2.29% of the picks into the q5 cut against `G0`'s 0.68%; their
unweighted `RR` is 13.8–18.4%, so the frames marked keep inside their cuts
carry larger `1/π` than the ones marked discard. Threshold-free, `D0` has
the highest AUC of the three new cells, 0.761 [0.697, 0.827], against `G0`'s
0.710 (effect A +0.050 [-0.017, +0.126]) and `eye_focus`'s 0.777 (`D0` −
`eye_focus` -0.017 [-0.059, +0.026]). The B columns lower the AUC in both
targets (`G0+B` − `G0` -0.021 [-0.046, +0.002], `D0+B` − `D0` -0.029
[-0.052, -0.007]). At the cuts, effect A in `DP` is negative at every `q`
with intervals spanning zero except with B at q10 (`D0+B` − `G0+B` -20.5
[-38.0, -2.2]; A+B -19.0 [-36.7, -0.8]); B on the discard target raises
`DR` at q10 (+4.5 [+0.4, +9.4], interaction +5.8 [+1.5, +10.9]). Against
`eye_focus`, every new cell's AUC difference is negative, with intervals
below zero for `G0+B` (-0.088) and `D0+B` (-0.046); against random, every
AUC difference is above zero with intervals above zero. These are a few of
about 84 paired intervals read without correction. The strict-bad rate is
below the `discard` rate in every band of every cell. The follow-up rule's
inputs (no usable cut; A and A+B at q5 with 2.5% quantiles of -48.1 and
-43.9) are those of its third branch; the Decision (Step 5) reads them.
