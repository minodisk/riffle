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
[discard filter evaluation](../_archived/20261018-discard-labels/discard.md)
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
the archived [discard.md](../_archived/20261018-discard-labels/discard.md)
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
