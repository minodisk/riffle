# Model search: pick and discard suggestion

## Pre-registration of the exploration

Written 2026-10-20, **before `searchfit.py` exists and before any score of a
candidate has been computed or read**: when this section was written no
model of this plan had been fit, no new score existed anywhere, and the only
outputs of this plan in `D:\Photos\tests\2026-10-20-model-search\` were
`lockbox.tsv` and `lockbox.out.md`, written by this plan's `lockbox.py`
from each folder's frame and pick counts alone (no label, no model score).
It is merged on its own, before `searchfit.py` or `searchread.py` exists; the
merge commit of the PR that adds this section, without results, is the
record, as for the archived
[discard retarget evaluation](../_archived/20261019-discard-retarget/retarget.md)
before it. Nothing in this section changes after the new scores exist; a
deviation found while implementing Steps 2–3 is recorded in `learnings.md`
with its reason and does not touch the selection rules.

The question: within a bounded, fixed search over feature sets, model forms,
targets and score combinations, which candidate gives the best **pick
suggestion** (the folder's top q% against the dump's picks) and which the
best **discard suggestion** (the folder's bottom q% against the blind
"OK to discard?" labels)? This section fixes the exploration (Steps 2–3)
and the guard of its confirmation; the confirmation's bars are fixed in
Step 4's "Pre-registration of the confirmation", before any lockbox pick or
fresh label is read.

### What was known before this section

This pre-registration is **not blind** to the existing models' readings of
the same labels and picks. On the same dump of 27 folders and 32,459 frames
(4,419 picks):

- **On the 600 blind discard labels**, from the archived
  [discard.md](../_archived/20261018-discard-labels/discard.md) "Results"
  and reproduced digit for digit by
  [retarget.md](../_archived/20261019-discard-retarget/retarget.md)
  "Reference checks" (weighted by `1/π`, stratified 95% interval):
  - **Discardable base rate 48.45% [42.03, 55.11]** of the population's
    frames (folder-cluster [40.74, 53.87]; unweighted 64.83%).
  - **`G0`**:

    | Cut | `DP` % | `RR` % | `PL` % (full dump) | `DR` % | `DO` % | Removed % (population) |
    | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
    | q5 | 82.7 [76.0, 88.7] | 17.3 [11.3, 24.0] | 0.68 | 8.4 [7.2, 9.8] | 46.7 [39.9, 53.8] | 4.92 |
    | q10 | 76.3 [71.3, 81.3] | 23.7 [18.7, 28.7] | 2.10 | 15.5 [13.6, 18.1] | 45.4 [38.3, 52.9] | 9.87 |
    | q20 | 72.1 [67.5, 76.7] | 27.9 [23.3, 32.5] | 5.18 | 29.5 [25.9, 33.9] | 42.6 [34.8, 50.7] | 19.80 |

    Weighted discard AUC **0.710** [0.646, 0.771].
  - **`eye_focus` (missing last)**: AUC **0.777** [0.716, 0.841], the
    highest of every score read; `RR` q5 16.8 [1.2, 43.3] at `PL` 4.44%,
    q10 14.3 [2.9, 29.9] at `PL` 9.14%, q20 28.3 [15.7, 40.2] at `PL`
    15.59%.
  - **Random (seed 0)**: AUC 0.528 [0.455, 0.604]; `DP` q5 52.0, q10 61.5,
    q20 56.0.
  - **`D0` / `D0+B`** (the discard target on `G0`'s features, retarget.md
    "Results"): AUC 0.761 [0.697, 0.827] / 0.731 [0.667, 0.798]; `RR` q5
    40.5 / 32.7, q10 34.7 / 42.7; `PL` q5 2.29 / 2.26%; the folds' Kish ESS
    of `1/π` 166.1–195.0 on 504–597 labeled frames (the archived
    discard-retarget `learnings.md`, Step 3). `G0+B` (the burst-relative `B`
    columns added) moved nothing: AUC 0.689, `DP` − `G0` within ±2 points.
  - Under the looser bar (`RR` <= 25%, 97.5% quantile <= 35%, `PL` <= 5%,
    removed >= 4.5%, `DP` 2.5% quantile above 48.45%) **`G0` q5 and q10
    already meet every condition except "beats `G0`"** (retarget.md, "Bars").
- **On the dump's picks, folder-wide**, from the folder-wide-ranking-eval
  data folder's `folderwide.out.md` (the 25 qualifying folders, >= 50 frames
  and >= 5 picks; mean, per-folder SD in brackets): **`W0`** frame AUC
  **0.666** (0.095), R-precision **28.6** (10.8) % (chance, the mean pick
  rate, 13.8%), lift at the top tenth 2.32 (0.82); `F0+ctx` frame AUC 0.666,
  R-precision 27.0%. **`G0`** frame AUC **0.670** (the reject-filter-eval
  `rejectfilter.out.md`). `P@10` (the pick share of a folder's top 10%) has
  not been read for any score; the lift at the top tenth is the nearest
  known number.
- **Label-free, from `lockbox.py`** (below): the per-folder frame and pick
  counts, which equal `folderwide.out.md`'s "Per folder" columns.

Not known before this section, with no prior number: any score of a
candidate of this plan, any `P@q` / `R@q`, and any reading of the lockbox
folders' picks against a candidate. (`folderwide.out.md` already holds the
stored `W0` / `F0+ctx` per-folder frame AUC and R-precision of every folder,
the lockbox ones included; those are references, not candidates, and the
draw below does not use them.)

**Selection bias.** This plan's design (the search space, the two tasks,
the bars) was chosen **after** reading `G0`'s, `eye_focus`'s and `D0`'s
numbers on these same 600 labels and `W0`'s, `F0+ctx`'s and `G0`'s on these
same picks, and the discard-target candidates are trained on those labels
(held out by folder). The exploration then reads 69 candidates on them. The
mechanical selection rules below, fixed before any score, and the single
confirmatory read on data no candidate was selected on (the pick lockbox and
the fresh discard round) are the protections. The Decision weighs this and
reports the winners' exploration-to-confirmation shrinkage as its measure.

### Choices

Decided by the user on 2026-10-20, before Step 1 (plan.md, "Choices"):

1. **Overfitting guard: the hybrid form.** The discard task explores freely
   on all 27 folders and 600 labels and is confirmed by a **fresh blind
   round** of 600 frames; the pick task is confirmed on a **lockbox** of
   about 7 folders drawn here, unread by any pick candidate until Step 5.
   Rejected: (a) a lockbox for both tasks; (b) a fresh round with no pick
   guard.
2. **Pick bar: relative** to `W0` (mean `P@10` >= +2.0 points, the
   folder-bootstrap 2.5% quantile above zero, and 16 of 25 folders scaled
   at or above `W0`). Rejected: an absolute `P@10` >= 40%.
3. **Discard bar: the looser one** (`RR` <= 25%, its 97.5% quantile <=
   35%), plus "beats `G0`" and "beats `eye_focus`-alone", the latter **in
   weighted discard AUC with the paired 2.5% quantile above zero**.
   Rejected: the strict `RR` <= 15% / 25%; "beats `eye_focus`" as a point
   above.
4. **Search space: full, 69 candidates.** Rejected: a reduced 25.
5. Taken as proposed: the discard selection depth is **q10**; the weight
   cap is **5 × the median sampled `1/π`** (about 50).

### Data, labels and weights

- **Dump**: `D:\Photos\tests\2026-10-12-rank-app\`, read by the archived
  `gate.load` / `gate.build` / `folder_gate.extend` into `d` (32,459 frames,
  27 folders, 6,969 bursts; `emb` 768-d, `tech` 11 columns, `context` 3,
  `pick`, `weight` the per-folder class weights).
- **Discard labels**: the 600 rows of
  `D:\Photos\tests\2026-10-18-discard-labels\labels.tsv` (211 `keep`, 389
  `discard`; **`discard` the positive class**), with `strata.tsv`; the
  population the dump minus the 1,200 strict-sample frames, 31,259 frames;
  every estimate Horvitz–Thompson weighted by `1/π`, the unweighted value
  beside it, as discard.md.
- **Clipped training weights** (the discard target only): `w = min(1/π,
  cap)` with **`cap` = 5 × the median of the 600 sampled `1/π`**, one value
  computed in Step 2 from `inclusion_prob` alone (label-free, about 50) and
  used in every fold of both fit sets, then rescaled to mean 1 over the
  fold's training frames. Kish ESS `(Σw)² / Σw²` reported per fold with and
  without the clip. The clip is for training only; every read keeps the
  unclipped `1/π`.
- **Strict labels** (`D:\Photos\tests\2026-10-17-bad-labels\`): not opened
  by Step 2; the secondary coherence check on the discard winner only in
  Step 3.

### The pick lockbox

Drawn by `lockbox.py` in this plan folder (`python -I -B lockbox.py
D:\Photos\tests\2026-10-12-rank-app D:\Photos\tests\2026-10-20-model-search`,
6 s), which reads only `gate.load`'s frames and counts each folder's frames
and picks. The rule: over the 27 folders sorted by name,
`numpy.random.default_rng(0).choice(27, 7, replace=False)`, repeated on the
same generator until a draw holds **20–30% of the frames** and **>= 5
qualifying folders** (>= 50 frames, >= 5 picks); the first such draw is the
lockbox. It was accepted at **draw 2**. Its output is `lockbox.tsv` and
`lockbox.out.md` in the data folder.

| Set | Folders | Qualifying | Frames | Share of frames | Picks | Share of picks |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Lockbox | 7 | 7 | 9,591 | 29.5% | 1,301 | 29.4% |
| Exploration | 20 | 18 | 22,868 | 70.5% | 3,118 | 70.6% |

The **lockbox folders**:

| Folder | Frames | Picks | Qualifying |
| --- | ---: | ---: | ---: |
| `2026-07-26` | 64 | 7 | yes |
| `2026-07-30` | 936 | 105 | yes |
| `2026-07-31` | 349 | 52 | yes |
| `2026-08-01` | 3,401 | 463 | yes |
| `2026-08-02` | 150 | 28 | yes |
| `2026-08-29` | 3,451 | 475 | yes |
| `2026-09-27-b` | 1,240 | 171 | yes |

The other 20 folders are the **exploration folders**; 18 of them qualify
(`2026-07-21` and `2026-07-23` do not).

### Two fit sets

Every candidate is fit twice, leave one folder out, every frame scored by a
model that did not see its folder:

- **Pick fit set** (`scores_pick.tsv`): the 20 exploration folders only, as
  if the lockbox did not exist. A fold holds out one exploration folder and
  trains on the other 19 (the pick target on their frames, the discard
  target on their labeled frames); the PCA, the mean fill and the
  standardization are fit on those 19; the combinations are built from
  this set's `All@32 / HGB / pick` (`G0`'s recipe without the lockbox) in
  place of the stored `G0`, and `stack` is trained on the exploration
  folders' labels. Lockbox rows are `nan`. **No lockbox folder's frame,
  pick or label enters any fit of this set.**
- **Discard fit set** (`scores_discard.tsv`): all 27 folders, the 600
  labels and every pick; 27 folds; the combinations built from the stored
  `G0`.

Which fit each read uses:

| Read | Step | Fit |
| --- | ---: | --- |
| Pick table and the pick selection rule (the 18 qualifying exploration folders) | 3 | pick fit set |
| Discard table, AUCs, label-free conditions and the discard selection rule (27 folders, 600 labels) | 3 | discard fit set |
| Reading aids (dimension curve, forms, targets): pick-side rows / discard-side rows | 3 | pick fit set / discard fit set |
| Discard confirmation on the fresh labels | 5 | discard fit set, the winner's stored score (the fresh labels never enter a fit) |
| Pick confirmation on the 7 lockbox folders | 5 | the pick winner's recipe fit **once on all 20 exploration folders** (PCA, fill, standardization and inner `C` choice on them), scored on every lockbox frame; a combination winner from this fit's `All@32 / HGB / pick` as `G0`; a `stack` winner is trained on the 20 exploration folders' labeled frames with `G0` taken from the pick fit set's held-out scores (option (a), as in the exploration), its `eye_focus` logit / `ln sharpness` mean fill and standardization fit on those 20 folders, then applied to the lockbox frames |

The references are read from their stored scores in both tasks (`W0`,
`F0+ctx` from the folder-wide-eval `scores.tsv`; `G0` from
`scores_reject.tsv`; `D0` from `scores_retarget.tsv`), each fit leave one
folder out on all 27 folders, so on the exploration folders and on the
lockbox they were trained with more folders than the pick candidates. That
favors the references; a pick pass against `W0` is conservative for it.

The lockbox guards the **pick selection**. The discard task's `PL` (full
dump) counts the lockbox folders' picks inside a bottom cut; that is a
discard-task read and does not enter the pick rule.

### The search space (69 candidates)

Named `<features> / <form> / <target>`; every score **higher keeps** (a
discard-target model scored by `-decision_function`).

- **Feature sets (7)**: `T+C` (the 11 technical + 3 context columns, no
  embedding); `All@4`, `All@8`, `All@16`, `All@32` (technical + context +
  `k` whitened PCA dims of `emb`); `Emb@8`, `Emb@32` (PCA dims only).
  `All@32` is `G0`'s design. One local `design(d, train_mask, tech, dims)`
  with `folder_gate.design`'s recipe: the technical NaN mean-filled on the
  training folders, `PCA(k, whiten=True, random_state=0)` fit on the
  training folders, every column standardized on them. For the HGB forms
  the technical NaN are restored after it, as `rejectfilter.g0_fold` does;
  the logistic forms use the mean-filled design.
- **Model forms (5)**:
  - **`HGB`**: `rf.HGB` as is, `HistGradientBoostingClassifier(max_iter=200,
    learning_rate=0.05, max_leaf_nodes=15, l2_regularization=1.0,
    early_stopping=False, random_state=0)`.
  - **`HGB-reg`**, fixed here and not tuned:
    `HistGradientBoostingClassifier(max_iter=100, learning_rate=0.05,
    max_leaf_nodes=7, l2_regularization=10.0, min_samples_leaf=50,
    early_stopping=False, random_state=0)`.
  - **`HGB-mono`**: `rf.HGB` plus `monotonic_cst` over the design's columns:
    for the pick target **+1** on `ln rel sharpness`, `` `eye_focus` logit``,
    `eyes open`, `` `photoTier` good`` and `ln sharpness`, **−1** on
    `|yaw|`, `|pitch|`, `|roll|`, **0** on `no eye_focus`, `no eyes
    judgment`, `no pose`, the 3 context columns and every PCA column. The
    constraints hold over the positive class's probability, so for the
    discard target (positive = `discard`) every sign is negated; the keep
    direction is the same. Not defined on `Emb@*`.
  - **`LR-L2`**: `LogisticRegression(C=c, max_iter=2000)` on the
    standardized design, `c` chosen per fold by an inner `GroupKFold`
    (`k = min(5, training folders)`) over `gate.CS` = (0.01, 0.1, 1, 10).
  - **`LR-L1`**: the same with `l1_ratio=1.0, solver="liblinear"` (in
    sklearn 1.9.1 `penalty` is deprecated in favor of `l1_ratio`;
    `liblinear` supports a pure L1 penalty), the same grid and inner choice.
  - 7 × 5 − 2 = **33 designs**.
- **Targets (2)**:
  - **`pick`**: every frame of the training folders, label `pick`, weights
    the dump's per-folder class weights rescaled to mean 1 over the training
    frames (as `g0_fold`), for every form.
  - **`discard`**: the labeled frames of the training folders, label
    `discard`, the clipped weights above (as `retargetfit.d0_fold` with the
    clip). A discard fold of the discard fit set trains on 504–597 labeled
    frames.
  - The inner `C` criterion: for `pick` `folder_gate.choose_c`'s (the mean
    per-folder pick AUC over the inner validation folds); for `discard` the
    weighted (clipped) discard AUC over each inner validation fold's labeled
    frames, averaged over the inner folds.
  - **`C` fallback** (the discard target only): if no value of `C` is
    chosen by more than half the folds of a fit set for a logistic
    discard-target candidate, that candidate is refit with `C = 1.0` (as
    `L0`) in every fold of that set, the chosen counts recorded.
  - 33 × 2 = **66 fitted candidates**.
- **Score combinations (3)**, built from the fit set's `G0` (see "Two fit
  sets"):
  - **`G0 ⊕ ef`**: the mean of the per-folder normalized ranks (average
    rank of ties, scaled to 0–1) of `G0` and `eye_focus`; a frame without
    `eye_focus` takes `G0`'s rank alone.
  - **`G0 | ef-rule`**: frames with an `eye_focus` below
    `gate.GOOD_EYE_FOCUS` (0.9) sorted below the rest of their folder, each
    part by `G0`; a frame without `eye_focus` stays in the upper part.
  - **`stack`**: `LogisticRegression(C=1.0, max_iter=2000)` on 3
    standardized columns, the fit set's held-out `G0` score, `` `eye_focus`
    logit`` and `ln sharpness` (both mean-filled on the training folders),
    trained on the training folders' labeled frames with the clipped
    weights, target `discard`, held out by folder, scored by
    `-decision_function`.
    Its `G0` column is each frame's score from the fit set's own
    leave-one-folder-out run (pick fit set: the held-out `G0` pick scores;
    discard fit set: the stored `G0`). That column is out of fold only for
    the frame it scores: when `stack` holds out folder i, its training rows'
    `G0` scores came from `G0` models that saw folder i (its picks in the
    pick fit set, the stored `G0` in the discard fit set), so folder i
    reaches `stack` second-hand. This is accepted and not nested; it is
    stated here so the "did not see its folder" claim above is read as exact
    for every candidate except `stack`, and the label-free combinations (no
    training step) are exact. The held-out folder's discard labels never
    enter this way.
  - The same two label-free combinations applied to the pick winner are
    printed in Step 3 as a reading only, not as candidates.
- **66 + 3 = 69 candidates.**
- **References, not candidates**: `G0`, `W0`, `F0+ctx`, `eye_focus`
  (missing last), random (seed-0 draw), and `D0` (unclipped weights,
  stored) as the clipped-weight sensitivity reading.
- Nothing else is searched: no hyperparameter beyond the two fixed HGB
  settings and the logistic `C`, no feature engineering beyond the PCA
  dimensionality.

### `monotonic_cst` with missing values on sklearn 1.9.1

Checked from the scikit-learn 1.9.1 documentation only (the
`HistGradientBoostingClassifier` API page and the User Guide's "Monotonic
Constraints" and "Missing values support" sections), no fit. The API page
says the constraints "are only valid for binary classifications and hold
over the probability of the positive class"; the User Guide adds that they
are not supported for multiclass and cannot apply to categorical features.
Missing-value support is native ("the tree grower learns at each split point
whether samples with missing values should go to the left or right child").
**Neither page states any restriction on combining `monotonic_cst` with
NaN**, and neither says where the missing values fall relative to the
constrained values. So `HGB-mono` keeps the NaN-restored technical columns,
as `HGB` does, and its constraint is read as holding among non-missing
values only. **Fallback**: if a Step 2 fit raises on that combination, or
its scores break the constraint among non-missing values on a spot check,
`HGB-mono` uses the mean-filled technical columns (`folder_gate.design`'s)
for every fold of both sets, recorded in `learnings.md`.

### The two tasks and their metrics

- **Pick suggestion (the top)**, label-free except for the dump's picks, on
  the **18 qualifying exploration folders** (pick fit set): per folder, the
  top `floor(q · n / 100)` frames by score, ties by the seed-0 draw, q = 5 /
  10 / 20; **`P@q`** the pick share among them, **`R@q`** the share of the
  folder's picks among them, R-precision (`folderwide.r_precision`) and the
  frame AUC; the mean over the folders with the per-folder SD, the pooled
  value, and paired per-folder differences against the references with
  `folderwide.boot_folders` (1,000 draws, seed 0) and the one-sided Wilcoxon
  signed-rank p. **This task optimizes the top**, and is reported as such.
- **Discard suggestion (the bottom)**, exactly discard.md's, on all 27
  folders and 600 labels (discard fit set): per folder on the full dump, the
  bottom `floor(q · n / 100)` by score (`bf.graded_cut`), q = 5 / 10 / 20;
  `DP`, `RR`, `PL` (full dump, label-free; population beside it), `DR`,
  `MR`, `DO`, removed share (population, label-free; full dump beside it);
  the weighted discard AUC; the 1,000-draw stratified bootstrap (one count
  matrix for every candidate and reference, so differences are paired) and
  the folder-cluster bootstrap beside it; the strict-bad coherence check on
  the winner's bands only.

### Selection rules

Applied mechanically in Step 3; the references are not eligible; one winner
per task, and a winner may be the same candidate for both tasks.

- **Discard**: among the candidates whose **q10** cut passes the label-free
  conditions, **`PL` <= 5% on the full dump** and **removed >= 4.5% of the
  population**, the one with the **highest weighted `DP@10`**; ties by the
  weighted discard AUC. The q5 ranking under the same conditions is printed
  beside it as a reading; a candidate's rank at q5 does not change the
  winner. If no candidate passes at q10, the same rule at q5 chooses; if
  none passes there either, there is no discard winner and Step 4 records
  the discard task's follow-up as nothing.
- **Pick**: among all 69 candidates, the one with the **highest mean
  `P@10`** over the 18 qualifying exploration folders; ties by the mean
  frame AUC.
- Remaining exact ties: the earlier candidate in this section's order
  (feature set, then form, then target, then the combinations).
- The runner-up of each rule is named beside the winner.

### The guard

- **Pick**: the lockbox above. Before Step 5 no lockbox folder's frame,
  pick or label enters a pick fit and no pick read covers a lockbox folder.
  Step 5 reads the pick winner once on the 7 lockbox folders (all
  qualifying) against `W0`, with the bars Step 4 fixes; choice 2's "16 of
  25 scaled" is **at least 5 of 7** folders at or above `W0` (64% of 7,
  rounded up).
- **Discard**: the exploration reads all 27 folders and 600 labels, the
  lockbox folders' frames and labels included. Its confirmation is a
  **fresh blind round** drawn in Step 4: 600 frames (about 3 s per frame,
  about 30 minutes); population the dump minus the 1,800 frames already
  labeled (the 1,200 strict and 600 discard samples), 30,659 frames; the
  strata and their allocation fixed in Step 4, before the sample is drawn.

### Multiplicity and population

- **Multiplicity**: 69 candidates × 2 tasks × 3 cuts, plus the reading aids,
  read without correction on labels and picks the design was chosen after.
  The best of them is expected to look better than it is; this is why the
  selection rules are mechanical and fixed here, and why each task's
  confirmation is a single read of a single frozen winner on data no
  selection touched. The Decision reads the shrinkage from exploration to
  confirmation as the measure of the search's overfit.
- **Population caveat**: one photographer's 27 folders, mostly basketball;
  600 discard labels (and 600 fresh ones in Step 4); the pick confirmation
  on 7 folders, so its folder bootstrap is wide and a fail is weakly
  informative. The Decision speaks of this dump.

### What happens after

No step of this plan changes the app. Step 2 fits the 66 candidates and
builds the 3 combinations, both fit sets, and prints only label-free
readings; Step 3 reads them on both tasks and applies the rules above;
Step 4 fixes the confirmation's bars and draws the fresh sample; Step 5
reads once; Step 6 decides.

## Results of the exploration

Run 2026-10-21 by `searchread.py` (`python -I -B searchread.py
D:\Photos\tests\2026-10-12-rank-app D:\Photos\tests\2026-10-15-folder-wide-eval
D:\Photos\tests\2026-10-16-reject-filter-eval
D:\Photos\tests\2026-10-18-discard-labels D:\Photos\tests\2026-10-17-bad-labels
D:\Photos\tests\2026-10-19-discard-retarget
D:\Photos\tests\2026-10-20-model-search 8`), its stdout saved as
`searchread.out.md` in the data folder; the same command with `--check-all`
runs only the pick task's code path on all 27 folders for the stored
references and saves `searchread.check-all.out.md`. The discard task reads
`scores_discard.tsv` (all 27 folders, the 600 labels: 211 `keep`, 389
`discard`); the pick task reads `scores_pick.tsv` on the **18 qualifying
exploration folders only** (22,818 frames, 3,107 picks, chance 13.7%); no
lockbox folder's picks were read against a candidate. Discard intervals are
the stratified bootstrap's 95% (1,000 draws, seed 0, one count matrix for
every score) unless marked cluster; values weighted by `1/π`. Pick intervals
are the 1,000-draw folder bootstrap (seed 0) of the mean paired per-folder
difference, with the one-sided Wilcoxon p. Wall time 33 s (load 5 s; cuts
and metrics 1 s; the AUC bootstraps of 75 scores 26 s on 8 jobs); the
`--check-all` run 4 s. `searchread.out.md` holds every table in full (the
discard main table for every score and cut with cluster intervals, the AUC
table, every candidate's pick row and paired differences, and every reading
aid with its intervals); below are the parts the selection and the readings
rest on.

### Reference checks

Every check passes.

| Check | Expected | This run |
| --- | ---: | ---: |
| `strata.tsv` rows recomputed from the dump that differ / sampled frames whose band or `π` differ / in the strict sample | 0 of 30 / 0 / 0 / 0 | the same |
| Sum of `1/π`; discardable base rate % | 31,259.0; 48.45 [42.03, 55.11] | the same |
| `G0` `DP` % q5 / q10 / q20 | 82.7 [76.0, 88.7] / 76.3 [71.3, 81.3] / 72.1 [67.5, 76.7] | the same |
| `G0` `RR` % q5 / q10 / q20; `PL` %; cluster `RR` 97.5% quantile | 17.3 / 23.7 / 27.9; 0.68 / 2.10 / 5.18; 25.7 / 32.8 / 36.3 | the same |
| `G0` / `eye_focus` / random / `D0` AUC | 0.710 [0.646, 0.771] / 0.777 [0.716, 0.841] / 0.528 [0.455, 0.604] / 0.761 [0.697, 0.827] | the same |
| `eye_focus` `RR` % q5 / q10 / q20; `PL` % | 16.8 [1.2, 43.3] / 14.3 [2.9, 29.9] / 28.3 [15.7, 40.2]; 4.44 / 9.14 / 15.59 | the same |
| Random `DP` % q5 / q10 / q20; `G0` q5 − random q5 | 52.0 / 61.5 / 56.0; +30.7 [-3.8, +63.9] | the same |
| Frames left out of a cut over every candidate and `q` | 0 | 0 |
| Pick fit set finite on the exploration frames, `nan` on the lockbox frames; lockbox folders among the pick task's folders | yes; 0 | yes; 0 |
| `--check-all` (25 qualifying folders of the 27): `W0` frame AUC (SD) / pooled; R-precision % (SD) / pooled; lift at the top tenth | 0.666 (0.095) / 0.718; 28.6 (10.8) / 34.4; 2.32 (0.82) | the same |
| `--check-all`: `F0+ctx` frame AUC / pooled; R-precision / pooled; lift | 0.666 (0.102) / 0.727; 27.0 (11.5) / 33.5; 2.08 (0.98) | the same |
| `--check-all`: `G0` mean frame AUC | 0.670 | 0.670 |

### Discard task: the selection rule applied

43 of the 69 candidates pass the label-free conditions at q10 (`PL` <= 5%
on the full dump, removed >= 4.5% of the population), all 69 at q5. The
first 10 by weighted `DP@10` (ties by AUC; the q5 rank among the 69 q5
passers by `DP@5` beside it):

| Rank | Candidate | `DP@10` % | `RR@10` % | `PL@10` % | Removed q10 % | AUC | `DP@5` % | `PL@5` % | q5 rank |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | `G0 ⊕ ef` | 90.0 [86.1, 93.3] | 10.0 [6.7, 13.9] | 2.10 | 9.95 | 0.763 [0.703, 0.822] | 88.4 [82.9, 93.2] | 0.77 | 11 |
| 2 | `All@32 / LR-L2 / discard` | 85.9 [78.0, 92.6] | 14.1 [7.4, 22.0] | 2.83 | 9.92 | 0.807 [0.755, 0.857] | 93.5 [87.4, 97.5] | 1.13 | 4 |
| 3 | `stack` | 83.3 [71.9, 92.5] | 16.7 [7.5, 28.1] | 3.46 | 10.14 | 0.767 [0.703, 0.825] | 89.3 [83.0, 94.4] | 1.40 | 9 |
| 4 | `All@8 / HGB-mono / pick` | 82.7 [75.0, 88.0] | 17.3 [12.0, 25.0] | 2.96 | 9.97 | 0.673 [0.602, 0.739] | 72.1 [63.0, 81.2] | 1.38 | 47 |
| 5 | `All@32 / HGB-mono / pick` | 81.7 [76.4, 86.7] | 18.3 [13.3, 23.6] | 2.13 | 9.86 | 0.714 [0.646, 0.775] | 78.9 [72.1, 85.4] | 0.84 | 30 |
| 6 | `All@8 / LR-L2 / discard` | 80.8 [68.2, 91.9] | 19.2 [8.1, 31.8] | 4.39 | 9.97 | 0.801 [0.743, 0.853] | 74.7 [55.6, 91.9] | 1.90 | 42 |
| 7 | `All@32 / LR-L1 / discard` | 80.3 [68.4, 91.5] | 19.7 [8.5, 31.6] | 3.71 | 9.98 | 0.787 [0.727, 0.843] | 92.5 [85.8, 96.9] | 1.36 | 6 |
| 8 | `All@32 / HGB-reg / pick` | 80.3 [74.0, 86.3] | 19.7 [13.7, 26.0] | 2.04 | 9.88 | 0.718 [0.648, 0.777] | 78.6 [70.1, 86.5] | 0.75 | 32 |
| 9 | `All@8 / HGB-reg / pick` | 80.2 [71.8, 86.9] | 19.8 [13.1, 28.2] | 2.85 | 9.93 | 0.701 [0.630, 0.761] | 80.2 [68.9, 88.3] | 1.15 | 24 |
| 10 | `G0 \| ef-rule` | 79.7 [72.9, 86.0] | 20.3 [14.0, 27.1] | 2.67 | 10.20 | 0.753 [0.688, 0.813] | 83.8 [76.9, 90.3] | 1.13 | 15 |

The q5 ranking, a reading that does not change the winner (the first 5):
`All@4 / HGB-reg / discard` 94.9 [88.8, 98.6] (`PL@5` 2.40), `T+C / HGB /
discard` 94.1, `All@16 / HGB-reg / discard` 93.7, `All@32 / LR-L2 /
discard` 93.5 (q10 rank 2), `T+C / LR-L1 / discard` 93.1; of these only
`All@32 / LR-L2 / discard` passes the q10 conditions.

**Discard winner: `G0 ⊕ ef`; runner-up: `All@32 / LR-L2 / discard`.**

### Discard task: the winners beside the references

| Score | q5 `DP` % | q5 `RR` / `PL` % | q10 `DP` % | q10 `RR` / `PL` % | q20 `DP` / `RR` / `PL` % | AUC | AUC cluster |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `G0 ⊕ ef` (winner) | 88.4 [82.9, 93.2] | 11.6 / 0.77 | 90.0 [86.1, 93.3] | 10.0 / 2.10 | 80.1 / 19.9 / 5.73 | 0.763 [0.703, 0.822] | [0.707, 0.813] |
| `All@32 / LR-L2 / discard` (runner-up) | 93.5 [87.4, 97.5] | 6.5 / 1.13 | 85.9 [78.0, 92.6] | 14.1 / 2.83 | 74.9 / 25.1 / 8.42 | 0.807 [0.755, 0.857] | [0.745, 0.859] |
| `G0` | 82.7 [76.0, 88.7] | 17.3 / 0.68 | 76.3 [71.3, 81.3] | 23.7 / 2.10 | 72.1 / 27.9 / 5.18 | 0.710 [0.646, 0.771] | [0.655, 0.759] |
| `W0` (4 sampled frames out) | 76.9 [63.6, 88.6] | 23.1 / 1.68 | 66.6 [52.5, 80.2] | 33.4 / 4.05 | 64.1 / 35.9 / 7.34 | 0.671 [0.601, 0.737] | |
| `eye_focus` (missing last) | 83.2 [56.7, 98.8] | 16.8 / 4.44 | 85.7 [70.1, 97.1] | 14.3 / 9.14 | 71.7 / 28.3 / 15.59 | 0.777 [0.716, 0.841] | [0.719, 0.832] |
| Random (seed 0) | 52.0 [19.8, 85.5] | 48.0 / 4.71 | 61.5 [37.9, 84.8] | 38.5 / 9.48 | 56.0 / 44.0 / 19.26 | 0.528 [0.455, 0.604] | |
| `D0` (stored, unclipped) | 59.5 [35.0, 93.3] | 40.5 / 2.29 | 65.3 [46.6, 85.9] | 34.7 / 4.84 | 66.5 / 33.5 / 11.59 | 0.761 [0.697, 0.827] | |

Paired differences of the winner (stratified, cluster in parentheses):

| Difference | q5 | q10 | q20 |
| --- | ---: | ---: | ---: |
| `G0 ⊕ ef` − `G0`, `DP` points | +5.7 [+0.4, +11.4] ([+0.9, +10.7]) | +13.7 [+9.8, +17.7] ([+9.7, +19.4]) | +8.0 [-1.0, +15.8] ([-0.1, +17.0]) |
| `G0 ⊕ ef` − `eye_focus`, `DP` points | +5.2 [-11.4, +31.0] ([-11.7, +36.7]) | +4.3 [-7.0, +20.3] ([-6.8, +19.3]) | +8.4 [-4.0, +21.1] ([-2.2, +22.8]) |
| `G0 ⊕ ef` − random, `DP` points | +36.4 [+2.7, +69.8] ([+3.9, +75.6]) | +28.5 [+4.7, +52.3] ([-2.9, +54.6]) | +24.1 [+5.9, +41.4] ([+6.5, +43.4]) |
| `G0 ⊕ ef` − `G0`, AUC | +0.053 [+0.008, +0.099] ([+0.010, +0.091]) | | |
| `G0 ⊕ ef` − `eye_focus`, AUC | -0.015 [-0.062, +0.030] ([-0.071, +0.040]) | | |

The highest AUCs among the candidates, beside `eye_focus`'s (AUC −
`eye_focus` paired, stratified and cluster); the `All@4` and `All@8`
`LR-L1 / discard` rows fail the q10 `PL` condition:

| Score | AUC | AUC − `G0` | AUC − `eye_focus` |
| --- | ---: | ---: | ---: |
| `All@16 / LR-L1 / discard` | 0.809 [0.753, 0.860] | +0.099 [+0.036, +0.169] | +0.031 [+0.006, +0.056] ([+0.003, +0.064]) |
| `All@8 / LR-L1 / discard` | 0.808 [0.751, 0.861] | +0.098 [+0.033, +0.168] | +0.031 [+0.004, +0.057] ([+0.004, +0.063]) |
| `All@4 / LR-L1 / discard` | 0.808 [0.750, 0.862] | +0.098 [+0.034, +0.168] | +0.031 [+0.004, +0.056] ([+0.004, +0.063]) |
| `All@32 / LR-L2 / discard` | 0.807 [0.755, 0.857] | +0.097 [+0.039, +0.163] | +0.030 [-0.009, +0.068] ([-0.004, +0.071]) |
| `All@8 / LR-L2 / discard` | 0.801 [0.743, 0.853] | +0.091 [+0.026, +0.157] | +0.023 [-0.015, +0.062] ([-0.008, +0.065]) |
| `All@16 / LR-L2 / discard` | 0.800 [0.744, 0.850] | +0.089 [+0.027, +0.153] | +0.022 [-0.018, +0.060] ([-0.013, +0.061]) |
| `All@4 / LR-L2 / discard` | 0.796 [0.737, 0.849] | +0.086 [+0.021, +0.157] | +0.018 [-0.019, +0.053] ([-0.014, +0.058]) |
| `All@32 / LR-L1 / discard` | 0.787 [0.727, 0.843] | +0.077 [+0.012, +0.151] | +0.010 [-0.026, +0.045] ([-0.022, +0.056]) |
| `eye_focus` (missing last) | 0.777 [0.716, 0.841] | +0.067 [-0.000, +0.141] | - |

### Pick task: the selection rule applied

On the 18 qualifying exploration folders: mean (per-folder SD); pooled is
the frame-weighted mean of the per-folder values. References first, then
the first 10 candidates by mean `P@10` (ties by mean frame AUC).

| Score | `P@5` % | `P@10` % | `P@20` % | `R@10` % | R-precision % | Frame AUC | Pooled `P@10` | `P@10` − `W0`, points | p | Folders at or above `W0` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `W0` | 31.9 (16.4) | 31.2 (11.6) | 26.5 (9.6) | 22.9 (8.7) | 28.5 (11.5) | 0.669 (0.100) | 37.5 | - | - | - |
| `G0` (stored) | 36.1 (15.2) | 30.6 (13.2) | 26.1 (10.2) | 22.2 (9.2) | 28.4 (11.3) | 0.674 (0.115) | 37.1 | -0.7 [-3.0, +2.2] | 0.8794 | 9 of 18 |
| `F0+ctx` | 31.1 (17.4) | 27.0 (15.5) | 25.1 (8.8) | 19.9 (11.6) | 26.7 (12.3) | 0.669 (0.106) | 35.7 | -4.2 [-7.4, -1.7] | 0.9995 | 5 of 18 |
| `D0` | 20.9 (11.9) | 19.1 (10.2) | 19.6 (7.6) | 13.7 (7.0) | 19.1 (9.7) | 0.589 (0.071) | 24.2 | | | |
| `eye_focus` (missing last) | 18.5 (17.0) | 17.8 (7.9) | 19.5 (7.0) | 12.9 (5.5) | 19.2 (7.8) | 0.578 (0.050) | 19.3 | -13.4 [-18.8, -6.8] | 0.9987 | 2 of 18 |
| Random (seed 0) | 13.4 (9.5) | 15.3 (8.0) | 15.4 (6.3) | 11.0 (6.2) | 15.0 (6.2) | 0.519 (0.071) | 13.4 | -16.0 [-22.9, -8.2] | 0.9997 | 5 of 18 |
| 1. `All@32 / HGB-mono / pick` | 32.5 (17.2) | 30.4 (10.7) | 25.3 (8.8) | 22.3 (8.4) | 28.1 (9.6) | 0.671 (0.098) | 36.4 | -0.9 [-2.7, +0.5] | 0.7651 | 12 of 18 |
| 2. `G0 ⊕ ef` | 29.2 (15.4) | 29.6 (9.8) | 25.0 (7.5) | 21.4 (7.0) | 27.1 (8.7) | 0.664 (0.070) | 32.0 | -1.7 [-5.9, +3.4] | 0.8236 | 7 of 18 |
| 3. `All@32 / HGB / pick` | | 29.0 (12.7) | | 21.0 (9.3) | | 0.675 (0.096) | | -2.3 [-5.7, +0.4] | 0.9036 | 7 of 18 |
| 4. `All@32 / HGB-reg / pick` | | 28.7 (11.3) | | 20.8 (8.2) | | 0.654 (0.095) | | -2.6 [-6.0, +0.3] | 0.9465 | 7 of 18 |
| 5. `G0 \| ef-rule` | | 28.3 (12.1) | | 20.5 (8.8) | | 0.659 (0.078) | | -2.9 [-6.3, -0.2] | 0.9635 | 7 of 18 |
| 6. `All@4 / HGB-mono / pick` | | 27.8 (8.6) | | 20.4 (6.9) | | 0.652 (0.088) | | -3.4 [-7.4, +1.5] | 0.9899 | 6 of 18 |
| 7. `All@4 / LR-L2 / pick` | | 27.7 (10.2) | | 20.1 (7.7) | | 0.646 (0.080) | | -3.5 [-6.8, +0.8] | 0.9723 | 5 of 18 |
| 8. `Emb@32 / HGB / pick` | | 27.7 (11.1) | | 20.3 (8.0) | | 0.637 (0.095) | | -3.5 [-5.3, -1.7] | 0.9990 | 5 of 18 |
| 9. `Emb@32 / LR-L2 / pick` | | 27.7 (14.2) | | 20.2 (10.0) | | 0.635 (0.114) | | -3.6 [-5.5, -1.5] | 0.9960 | 5 of 18 |
| 10. `All@16 / LR-L2 / pick` | | 27.6 (11.0) | | 20.2 (8.0) | | 0.659 (0.097) | | -3.6 [-5.5, -1.9] | 0.9999 | 5 of 18 |

The pick fit set's combinations (`G0 ⊕ ef`, `G0 | ef-rule`) are built from
its own `All@32 / HGB / pick`, not the stored `G0`. The pick winner against
the stored `G0`: `P@10` −0.2 [-2.6, +1.9] (p 0.3934); frame AUC against
`W0` +0.002 [-0.024, +0.021].

**Pick winner: `All@32 / HGB-mono / pick`; runner-up: `G0 ⊕ ef`.**

### Reading aids

Paired differences against the stored `G0`, point values; the intervals of
every cell are in `searchread.out.md`. Discard side on the discard fit set
(600 labels); pick side on the pick fit set (18 folders). `HGB-mono` is not
defined on `Emb@*`.

`DP@10` − `G0`, points (discard task):

| Form / target | `T+C` | `All@4` | `All@8` | `All@16` | `All@32` | `Emb@8` | `Emb@32` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `HGB` / `pick` | -8.9 | -7.0 | +0.3 | -0.9 | +0.0 | -20.0 | -22.0 |
| `HGB-reg` / `pick` | -10.0 | +1.8 | +3.9 | +2.7 | +4.0 | -5.3 | +1.6 |
| `HGB-mono` / `pick` | +2.7 | +2.6 | +6.4 | +1.7 | +5.3 | - | - |
| `LR-L2` / `pick` | -19.5 | -6.7 | +0.6 | -2.1 | -5.6 | -4.8 | +0.3 |
| `LR-L1` / `pick` | -23.4 | +0.2 | -1.3 | -2.1 | -7.5 | -4.7 | +0.4 |
| `HGB` / `discard` | +14.8 | -20.3 | +1.9 | -11.6 | -7.7 | -17.9 | -19.3 |
| `HGB-reg` / `discard` | +6.2 | +2.8 | +5.0 | +7.0 | -10.4 | -35.0 | -25.5 |
| `HGB-mono` / `discard` | +8.5 | -4.8 | +2.6 | -3.8 | -0.2 | - | - |
| `LR-L2` / `discard` | -1.0 | -3.9 | +4.5 | -5.1 | +9.5 | -10.6 | -1.7 |
| `LR-L1` / `discard` | +2.7 | +5.4 | +4.7 | -2.2 | +4.0 | -9.9 | -1.1 |

AUC − `G0` (discard task):

| Form / target | `T+C` | `All@4` | `All@8` | `All@16` | `All@32` | `Emb@8` | `Emb@32` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `HGB` / `pick` | -0.042 | -0.050 | -0.031 | -0.050 | -0.000 | -0.119 | -0.054 |
| `HGB-reg` / `pick` | -0.003 | -0.012 | -0.010 | -0.013 | +0.007 | -0.098 | -0.055 |
| `HGB-mono` / `pick` | -0.019 | -0.039 | -0.038 | -0.026 | +0.003 | - | - |
| `LR-L2` / `pick` | +0.020 | +0.029 | +0.018 | +0.015 | +0.022 | -0.074 | -0.026 |
| `LR-L1` / `pick` | +0.021 | +0.025 | +0.013 | +0.013 | +0.022 | -0.075 | -0.028 |
| `HGB` / `discard` | +0.006 | +0.020 | +0.004 | +0.015 | +0.046 | -0.135 | -0.009 |
| `HGB-reg` / `discard` | +0.050 | +0.051 | +0.046 | +0.047 | +0.062 | -0.098 | +0.001 |
| `HGB-mono` / `discard` | +0.035 | +0.046 | +0.019 | +0.042 | +0.066 | - | - |
| `LR-L2` / `discard` | +0.068 | +0.086 | +0.091 | +0.089 | +0.097 | -0.012 | +0.046 |
| `LR-L1` / `discard` | +0.070 | +0.098 | +0.098 | +0.099 | +0.077 | -0.021 | +0.037 |

`P@10` − `G0`, points (pick task; the candidates trained without the
lockbox, the stored `G0` with it):

| Form / target | `T+C` | `All@4` | `All@8` | `All@16` | `All@32` | `Emb@8` | `Emb@32` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `HGB` / `pick` | -5.8 | -4.4 | -5.1 | -3.5 | -1.6 | -7.4 | -2.9 |
| `HGB-reg` / `pick` | -6.2 | -6.3 | -4.7 | -4.2 | -1.9 | -8.2 | -6.0 |
| `HGB-mono` / `pick` | -7.2 | -2.7 | -5.5 | -3.2 | -0.2 | - | - |
| `LR-L2` / `pick` | -6.0 | -2.9 | -3.5 | -2.9 | -3.3 | -5.1 | -2.9 |
| `LR-L1` / `pick` | -6.3 | -5.5 | -3.7 | -3.3 | -3.7 | -6.3 | -3.2 |
| `HGB` / `discard` | -9.6 | -9.5 | -9.0 | -9.3 | -7.0 | -13.2 | -9.2 |
| `HGB-reg` / `discard` | -11.2 | -10.6 | -10.0 | -9.3 | -10.4 | -12.0 | -9.9 |
| `HGB-mono` / `discard` | -10.2 | -7.7 | -7.5 | -7.0 | -6.6 | - | - |
| `LR-L2` / `discard` | -8.4 | -6.2 | -7.0 | -8.0 | -9.2 | -8.5 | -7.8 |
| `LR-L1` / `discard` | -9.0 | -7.4 | -8.0 | -8.3 | -10.4 | -10.3 | -7.9 |

The form comparison at `All@32`, with intervals (stratified, cluster in
parentheses; pick side folder bootstrap and p):

| Candidate | `DP@10` − `G0`, points | `DP@5` − `G0`, points | AUC − `G0` | `P@10` − `G0`, points |
| --- | ---: | ---: | ---: | ---: |
| `All@32 / HGB / pick` | +0.0 | +0.0 | -0.000 | -1.6 [-5.5, +2.3] (p 0.9091) |
| `All@32 / HGB-reg / pick` | +4.0 [-1.7, +9.5] ([-1.9, +9.2]) | -4.1 [-12.0, +3.2] ([-12.9, +1.6]) | +0.007 [-0.021, +0.035] ([-0.025, +0.047]) | -1.9 [-5.4, +1.8] (p 0.9409) |
| `All@32 / HGB-mono / pick` | +5.3 [+1.3, +9.1] ([+2.3, +8.7]) | -3.8 [-8.2, +0.5] ([-9.5, +1.0]) | +0.003 [-0.016, +0.023] ([-0.014, +0.026]) | -0.2 [-2.6, +1.9] (p 0.3934) |
| `All@32 / LR-L2 / pick` | -5.6 [-18.0, +6.4] ([-19.1, +5.8]) | +0.4 [-11.8, +11.5] ([-14.6, +10.6]) | +0.022 [-0.020, +0.067] ([-0.042, +0.068]) | -3.3 [-6.5, -1.0] (p 0.9946) |
| `All@32 / LR-L1 / pick` | -7.5 [-20.0, +4.1] ([-21.0, +4.4]) | -0.6 [-12.5, +10.8] ([-15.1, +9.9]) | +0.022 [-0.020, +0.067] ([-0.040, +0.069]) | -3.7 [-6.9, -1.4] (p 0.9983) |
| `All@32 / HGB / discard` | -7.7 [-26.1, +10.2] ([-24.8, +11.0]) | -12.8 [-35.4, +10.0] ([-38.0, +10.8]) | +0.046 [-0.017, +0.115] ([-0.038, +0.123]) | -7.0 [-11.6, -1.5] (p 0.9913) |
| `All@32 / HGB-reg / discard` | -10.4 [-28.2, +10.0] ([-21.9, +5.1]) | -13.5 [-38.3, +14.3] ([-35.4, +15.9]) | +0.062 [+0.002, +0.131] ([-0.023, +0.132]) | -10.4 [-14.0, -6.6] (p 0.9999) |
| `All@32 / HGB-mono / discard` | -0.2 [-17.3, +15.0] ([-13.5, +14.3]) | -19.0 [-42.0, +7.5] ([-39.4, +6.1]) | +0.066 [+0.005, +0.135] ([-0.036, +0.152]) | -6.6 [-12.1, -0.4] (p 0.9776) |
| `All@32 / LR-L2 / discard` | +9.5 [+1.1, +16.5] ([+0.6, +17.1]) | +10.9 [+4.0, +17.1] ([+3.3, +17.7]) | +0.097 [+0.039, +0.163] ([+0.017, +0.169]) | -9.2 [-13.3, -5.4] (p 0.9999) |
| `All@32 / LR-L1 / discard` | +4.0 [-8.4, +15.8] ([-11.7, +15.3]) | +9.8 [+2.1, +16.4] ([+1.7, +16.5]) | +0.077 [+0.012, +0.151] ([+0.005, +0.152]) | -10.4 [-15.4, -6.1] (p 0.9997) |

The target comparison, the discard target minus the pick target of the same
design (paired, point values; intervals in `searchread.out.md`):

| Design | `DP@10`, points | `DP@5`, points | AUC | `P@10`, points |
| --- | ---: | ---: | ---: | ---: |
| `T+C / HGB` | +23.8 | +30.3 | +0.047 | -3.8 |
| `T+C / HGB-reg` | +16.2 | -3.5 | +0.053 | -5.0 |
| `T+C / HGB-mono` | +5.8 | +9.6 | +0.054 | -3.0 |
| `T+C / LR-L2` | +18.5 | +34.4 | +0.048 | -2.4 |
| `T+C / LR-L1` | +26.0 | +49.0 | +0.049 | -2.7 |
| `All@4 / HGB` | -13.4 | +8.7 | +0.069 | -5.2 |
| `All@4 / HGB-reg` | +1.0 | +14.7 | +0.063 | -4.3 |
| `All@4 / HGB-mono` | -7.5 | +10.7 | +0.085 | -4.9 |
| `All@4 / LR-L2` | +2.8 | -4.0 | +0.057 | -3.4 |
| `All@4 / LR-L1` | +5.2 | +1.7 | +0.073 | -1.8 |
| `All@8 / HGB` | +1.6 | +23.7 | +0.035 | -3.9 |
| `All@8 / HGB-reg` | +1.1 | +5.4 | +0.056 | -5.3 |
| `All@8 / HGB-mono` | -3.8 | +16.5 | +0.057 | -2.0 |
| `All@8 / LR-L2` | +3.9 | -8.9 | +0.073 | -3.5 |
| `All@8 / LR-L1` | +5.9 | +0.4 | +0.084 | -4.3 |
| `All@16 / HGB` | -10.7 | +1.2 | +0.066 | -5.8 |
| `All@16 / HGB-reg` | +4.3 | +19.9 | +0.060 | -5.1 |
| `All@16 / HGB-mono` | -5.5 | -4.6 | +0.069 | -3.9 |
| `All@16 / LR-L2` | -3.0 | -1.7 | +0.075 | -5.0 |
| `All@16 / LR-L1` | -0.1 | +3.5 | +0.086 | -5.0 |
| `All@32 / HGB` | -7.7 | -12.8 | +0.046 | -5.4 |
| `All@32 / HGB-reg` | -14.4 | -9.4 | +0.054 | -8.5 |
| `All@32 / HGB-mono` | -5.6 | -15.3 | +0.063 | -6.4 |
| `All@32 / LR-L2` | +15.1 | +10.5 | +0.075 | -5.9 |
| `All@32 / LR-L1` | +11.5 | +10.4 | +0.055 | -6.7 |
| `Emb@8 / HGB` | +2.1 | +9.6 | -0.016 | -5.8 |
| `Emb@8 / HGB-reg` | -29.7 | -43.1 | +0.000 | -3.8 |
| `Emb@8 / LR-L2` | -5.8 | -1.0 | +0.063 | -3.4 |
| `Emb@8 / LR-L1` | -5.2 | +12.3 | +0.054 | -4.0 |
| `Emb@32 / HGB` | +2.7 | -36.1 | +0.044 | -6.3 |
| `Emb@32 / HGB-reg` | -27.0 | -45.1 | +0.056 | -4.0 |
| `Emb@32 / LR-L2` | -2.0 | +5.5 | +0.072 | -4.9 |
| `Emb@32 / LR-L1` | -1.4 | +2.0 | +0.065 | -4.7 |

The combinations against `G0`, and the clip (`All@32 / HGB / discard`,
clipped, against the stored unclipped `D0`):

| Comparison | `DP@10`, points | `DP@5`, points | AUC | `P@10`, points |
| --- | ---: | ---: | ---: | ---: |
| `G0 ⊕ ef` − `G0` | +13.7 [+9.8, +17.7] ([+9.7, +19.4]) | +5.7 [+0.4, +11.4] ([+0.9, +10.7]) | +0.053 [+0.008, +0.099] ([+0.010, +0.091]) | -1.0 [-5.7, +4.4] (p 0.8082) |
| `G0 \| ef-rule` − `G0` | +3.4 [-3.6, +10.7] ([-4.0, +10.4]) | +1.1 [-6.2, +8.4] ([-6.4, +8.3]) | +0.043 [-0.007, +0.090] ([-0.000, +0.097]) | -2.2 [-6.1, +1.6] (p 0.9487) |
| `stack` − `G0` | +6.9 [-4.2, +16.4] ([-4.3, +17.9]) | +6.6 [+0.1, +12.9] ([+0.7, +14.1]) | +0.057 [+0.003, +0.114] ([+0.006, +0.115]) | -3.6 [-9.7, +2.7] (p 0.8570) |
| `All@32 / HGB / discard` − `D0` | +3.3 [-14.1, +20.8] ([-15.9, +16.5]) | +10.4 [-3.6, +23.9] ([-3.6, +25.5]) | -0.005 [-0.028, +0.019] ([-0.030, +0.021]) | |

The two label-free combinations applied to the pick winner, a reading only
(pick side on the pick fit set, discard side on the discard fit set; point
values):

| Score | `P@10` % | `R@10` % | Frame AUC | q5 `DP` / `RR` / `PL` % | q10 `DP` / `RR` / `PL` % | AUC |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `All@32 / HGB-mono / pick` | 30.4 (10.7) | 22.3 (8.4) | 0.671 (0.098) | 78.9 / 21.1 / 0.84 | 81.7 / 18.3 / 2.13 | 0.714 |
| ⊕ `eye_focus` | 28.9 (10.8) | 21.0 (8.0) | 0.660 (0.070) | 88.7 / 11.3 / 0.70 | 88.4 / 11.6 / 2.08 | 0.767 |
| \| `eye_focus` rule | 30.1 (10.8) | 22.0 (8.2) | 0.653 (0.081) | 83.7 / 16.3 / 1.13 | 82.0 / 18.0 / 2.87 | 0.754 |

### The coherence check on the discard winner's bands

| Score | Band | Sampled / discard | `discard` rate % | Strict frames / strict bad | Strict-bad rate % | Strict below discard |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `G0 ⊕ ef` | bottom 5% | 147 / 129 | 88.4 [82.9, 93.2] | 76 / 26 | 31.3 [18.4, 47.2] | yes |
| `G0 ⊕ ef` | 5–10% | 100 / 88 | 91.6 [85.8, 95.9] | 48 / 9 | 11.9 [4.8, 22.4] | yes |
| `G0 ⊕ ef` | 10–20% | 94 / 65 | 68.0 [50.2, 84.1] | 107 / 11 | 7.3 [2.4, 13.9] | yes |
| `G0 ⊕ ef` | above 20% | 259 / 107 | 41.2 [33.7, 48.8] | 969 / 34 | 3.7 [2.3, 5.2] | yes |

`G0`'s rows (in `searchread.out.md`) reproduce discard.md's per-band table.

### Reading

The stored `G0`, `eye_focus`, random and `D0` reproduce discard.md and
retarget.md on the discard task's code path, and the pick task's code path
reproduces folderwide.out.md's `W0` / `F0+ctx` and rejectfilter.out.md's
`G0` on all 27 folders; the pick task read 18 exploration folders and no
lockbox folder. **Discard**: 43 of 69 candidates pass the q10 label-free
conditions; by the rule the winner is `G0 ⊕ ef` (`DP@10` 90.0 [86.1, 93.3],
`RR@10` 10.0, `PL@10` 2.10%; q5 `DP` 88.4, `RR` 11.6, `PL` 0.77%; AUC 0.763
[0.703, 0.822]) and the runner-up `All@32 / LR-L2 / discard` (`DP@10` 85.9
[78.0, 92.6], `RR@10` 14.1, `PL@10` 2.83%; q5 `DP` 93.5, `RR` 6.5, `PL`
1.13%; AUC 0.807 [0.755, 0.857]). The winner's `DP` exceeds `G0`'s by +13.7
[+9.8, +17.7] points at q10 and +5.7 [+0.4, +11.4] at q5, and its AUC by
+0.053 [+0.008, +0.099]; its AUC is below `eye_focus`'s by 0.015 [-0.062,
+0.030]. The winner ranks 11th at q5; four of the five q5 leaders are
discard-target models that fail the q10 `PL` condition. The seven highest
candidate AUCs (0.796–0.809) are `LR-L1` and `LR-L2` discard-target models
with embedding dims, three of them (`All@4` / `All@8` / `All@16 / LR-L1 / discard`) with AUC −
`eye_focus` +0.031 and stratified 2.5% quantiles of +0.004 to +0.006; of
these three only `All@16` passes the q10 conditions (`DP@10` 74.1, rank
30). The 600 labels were drawn in `G0`'s bands, and the winner is a rank
mix of `G0` and `eye_focus`, the two scores this plan's design was chosen
after. The strict-bad rate is below the `discard` rate in every band of the
winner. **Pick**: no candidate's mean `P@10` reaches `W0`'s 31.2%; the
winner by the rule is `All@32 / HGB-mono / pick` (`P@10` 30.4, `R@10` 22.3,
frame AUC 0.671; −0.9 [-2.7, +0.5] points against `W0`, 12 of 18 folders at
or above, p 0.77; −0.2 [-2.6, +1.9] against the stored `G0`), the runner-up
`G0 ⊕ ef` built on the pick fit set (`P@10` 29.6). The pick fit set's own
`G0` recipe, trained without the lockbox, reads `P@10` 29.0 against the
stored `G0`'s 30.6. **Dimension curve**: on the pick side, `P@10` − `G0` of
the pick target is highest at `All@32` for every HGB form (−1.6, −1.9,
−0.2) and between −2.9 and −3.7 for the logistic forms from `All@8` on;
`T+C` is −5.8 to −7.2 for every pick-target form; `Emb@8` is the lowest or
near it in every row, and on the discard side its AUC − `G0` is negative in
every row (−0.012 to −0.135). **Forms** at `All@32`: `HGB-mono` is the only
pick-target form whose `DP@10` − `G0` interval is above zero (+5.3 [+1.3,
+9.1]); on the discard target the logistic forms raise AUC most (+0.097,
+0.077) and are the only discard-target forms with positive `DP@5` − `G0`
(+10.9 [+4.0, +17.1], +9.8 [+2.1, +16.4]), while the HGB forms on it lower
`DP@5` (−12.8 to −19.0). **Targets**: the discard target raises AUC over
the pick target in 31 of the 33 designs (`Emb@8 / HGB` −0.016, `Emb@8 /
HGB-reg` 0.000) and lowers `P@10` in all 33 (−1.8 to −8.5 points); its
`DP@10` effect ranges from −29.7 to +26.0. The clip changes `All@32 / HGB /
discard` against the unclipped `D0` by +3.3 [-14.1, +20.8] points of
`DP@10` and −0.005 [-0.028, +0.019] AUC. **Combinations**: `G0 ⊕ ef` moves
`DP@10` most (+13.7); `stack` +6.9 [-4.2, +16.4]; `G0 | ef-rule` +3.4
[-3.6, +10.7]; none moves `P@10` with an interval above zero. On the pick
winner, ⊕ `eye_focus` raises its q10 `DP` from 81.7 to 88.4 and its AUC from
0.714 to 0.767 and lowers its `P@10` from 30.4 to 28.9. These are a few of
several hundred intervals read without correction; Step 4 fixes the bars
and Step 5 reads the two winners once.

## Pre-registration of the confirmation

Written 2026-10-21, after Step 3's exploration read and **before any lockbox
pick or fresh label is read**: no lockbox folder's picks have been read
against a candidate (Step 3 read the 18 qualifying exploration folders only),
and the fresh sample below had not been drawn when its strata, allocation
and seed were written here; it has no labels yet. The merge commit of the
PR that adds this section is the record. Nothing in this section changes
after Step 5's read; a deviation found while implementing Step 5 is recorded
in `learnings.md` with its reason and does not touch the bars or the
follow-up rule.

### The frozen winners

Exactly as Step 3's rule chose them ("Results of the exploration"):

- **Discard winner: `G0 ⊕ ef`**, the mean of the per-folder normalized
  ranks of the stored `G0` and `eye_focus` (a frame without `eye_focus`
  takes `G0`'s rank alone). Step 5 reads its stored column of
  `scores_discard.tsv` as it is: it is built without a training step from
  the stored `G0` and the dump's `eye_focus`, so no label of either round
  enters it. Runner-up `All@32 / LR-L2 / discard`, named and **not read** in
  Step 5.
- **Pick winner: `All@32 / HGB-mono / pick`** (`rf.HGB` with the
  `monotonic_cst` of "The search space", the NaN kept, the pick target),
  **fit once on all 20 exploration folders** (PCA, mean fill and
  standardization on them; the pick target on their frames, their per-folder
  class weights rescaled to mean 1) and scored on every frame of the 7
  lockbox folders, as "Two fit sets" fixes. Runner-up `G0 ⊕ ef` (built on the
  pick fit set), named and **not read** in Step 5.

### Already known about the winners

These are readings of the exploration, on the labels and folders the winners
were selected on; they are not verdicts, and **no bar below was set or moved
because of them**:

- `G0 ⊕ ef` **did not beat `eye_focus` on AUC** in the exploration: AUC −
  `eye_focus` −0.015 [-0.062, +0.030] (stratified; cluster [-0.071,
  +0.040]). Its exploration `DP` − `G0` was +5.7 [+0.4, +11.4] at q5 and
  +13.7 [+9.8, +17.7] at q10; `RR` 11.6 / 10.0% at q5 / q10.
- Its label-free conditions are known before any fresh label, since they
  read the full dump: `PL` (full dump) **0.77%** at q5, **2.10%** at q10,
  **5.73%** at q20, so **q20 fails `PL` <= 5%** whatever the fresh labels
  say; the removed shares of the fresh population are printed by
  `confirmsample.py` (below).
- The pick winner's mean `P@10` on the 18 qualifying exploration folders,
  30.4%, is **below `W0`'s** 31.2% (−0.9 [-2.7, +0.5] points, 12 of 18
  folders at or above `W0`); no candidate reached `W0` there.

### The fresh discard sample

Drawn by `confirmsample.py` in this plan folder (the archived discard-labels
`sample.py`'s design and its helpers, imported by path after `badfilter`):

- **Population**: the dump minus the 1,800 frames already labeled, the 1,200
  of `D:\Photos\tests\2026-10-17-bad-labels\manifest.tsv` and the 600 of
  `D:\Photos\tests\2026-10-18-discard-labels\manifest.tsv`: **30,659
  frames**. Only those two manifests are opened in the earlier rounds' folders
  (no `labels.tsv`).
- **Strata**, every population frame in exactly one, with the cuts of
  `bf.graded_cut` on the full dump (per folder, the bottom `floor(q · n /
  100)` by score, ties by `numpy.random.default_rng(0).random(n)`, the same
  draw as every read):
  - `b05`: the winner's bottom 5%; `b10`: its bottom 10% minus `b05`;
    `b20`: its bottom 20% minus its bottom 10%;
  - `g0only`: `G0`'s bottom 20% minus the winner's bottom 20%, so that
    "beats `G0`" is read on a sample that holds both cuts' frames;
  - `control`: the rest, drawn **per folder** with `bs.allocate`'s
    proportional allocation and a **floor of 3** (or the whole cell).
- **Allocation**: **600 frames, 120 per stratum** (`b05`, `b10`, `b20`,
  `g0only`, `control`); each stratum (each `control` folder cell) a simple
  random sample without replacement; `π` the drawn count over the cell's
  population, written to `strata.tsv`.
- **Draw seed**: `numpy.random.default_rng(20261021)`, one generator for the
  draws in the order `b05`, `b10`, `b20`, `g0only`, then the `control`
  cells by folder name, then the shuffle of the 600.
- **Files**: `manifest.tsv` and `strata.tsv` in the data folder
  `D:\Photos\tests\2026-10-20-model-search\`, and the shuffled RAW copies
  named `<nnnn>_<folder>_<file>` in its `sample\`, with no sidecar; the script
  refuses to write under `D:\Photos\2026` and checks every copy against its
  source by size and SHA-256.
- **Labeling**: by [labeling.md](labeling.md), the discard-labels guide with
  the new path: the same "OK to discard?" question, rating 1 = keep,
  unmarked = discard, `aiEnabled` off, every frame viewed once.

**As drawn** (added after the draw; label-free): `python -I -B
confirmsample.py D:\Photos\tests\2026-10-12-rank-app
D:\Photos\tests\2026-10-15-folder-wide-eval
D:\Photos\tests\2026-10-16-reject-filter-eval
D:\Photos\tests\2026-10-18-discard-labels D:\Photos\tests\2026-10-17-bad-labels
D:\Photos\tests\2026-10-20-model-search`, its stdout saved as
`confirmsample.out.md` in the data folder. The 1,200 and 600 earlier frames
are disjoint; the population is 30,659 frames and the sum of `1/π` over the
600 drawn frames equals it.

| Stratum | Dump frames | Earlier-sample frames removed | Population | Drawn | `π` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `b05` | 1,609 | 223 | 1,386 | 120 | 0.0865801 |
| `b10` | 1,625 | 148 | 1,477 | 120 | 0.0812458 |
| `b20` | 3,248 | 201 | 3,047 | 120 | 0.039383 |
| `g0only` | 1,896 | 232 | 1,664 | 120 | 0.0721154 |
| `control` (27 folder cells) | 24,081 | 996 | 23,085 | 120 | 0.00519818 overall; 0.00355872–0.75 per cell |

The `control` cells draw 3 to 13 frames (`2026-07-23`'s 4 population frames
give `π` 0.75, `2026-07-21`'s 15 give 0.2; the large folders 0.0036–0.0047).
Of the winner's bands, `b05` holds 861 of `G0`'s bottom 5% frames of the
population, and `g0only` holds 161 / 334 / 1,169 frames of `G0`'s `b05` /
`b10` / `b20`. Label-free, on the fresh population: the winner removes
**4.52%** at q5 (condition 3 holds by 0.02 points), 9.34% at q10 and
19.28% at q20; `G0` 4.53%, 9.09%, 18.72%. `PL` (full dump) reproduces
Step 3: the winner 0.77 / 2.10 / 5.73%, `G0` 0.68 / 2.10 / 5.18%. The
sample holds 36 dump picks and 3 explicit rejects. The copies (27.4 GB): 600 files in `sample\`, 0 whose size or SHA-256
differs from the source, 0 `.xmp` / `.dop`, 0 files not in the manifest,
and the 58,227 library files of the 27 folders unchanged (size and mtime).

### The confirmation read (Step 5)

- **Labels**: `confirmlabels.py` (the archived discard-labels `labels.py`
  recipe: rating 1 = `keep`, reviewed and unmarked = `discard`, unexpected
  marks listed, not dropped). **`discard` is the positive class**. Every
  estimate Horvitz–Thompson weighted by `1/π` from the new `strata.tsv`, the
  unweighted value beside it. The exploration's 600 labels are not read.
- **Scores read on the fresh labels**: the discard winner, `G0` (stored),
  `eye_focus` (missing last; in the AUC as `bf.finite_by_folder`'s folder
  minimum minus 1) and random (`default_rng(0)`), each at q5 / q10 / q20 with
  `bf.graded_cut` on the full dump as above, and the weighted discard AUC.
  Metrics as discard.md's (`DP`, `RR`, `PL` full dump, `DR`, `MR`, `DO`,
  removed share of the fresh population).
- **Intervals**: a 1,000-draw **stratified bootstrap** within `b05`, `b10`,
  `b20`, `g0only` and each (`control`, folder) cell of the new `strata.tsv`,
  seed 0, **one count matrix for every score and quantity** (paired
  differences share draws); the 2.5% / 97.5% quantiles. A **folder-cluster
  bootstrap** (the 27 folders with replacement, seed 0, 1,000 draws) beside
  it; it does not decide, and a pass whose cluster interval crosses its bar
  is reported as **fragile**.
- **Base rate**: the weighted share of `discard` over the fresh labels (the
  fresh population's discardable share), its point estimate the bar's value,
  not resampled; its interval printed beside.
- **Pick**: the pick winner fit as above, `W0` from its stored scores, on the
  **7 lockbox folders** (all qualifying): per folder `P@5` / `P@10` / `P@20`,
  `R@10`, R-precision and the frame AUC, Step 3's `pick_folder`; the mean
  over the 7 with the per-folder SD and the pooled value; the paired
  per-folder difference against `W0` with `fw.boot_folders` (1,000 draws,
  seed 0) and the one-sided Wilcoxon signed-rank p. `W0` was fit leave one
  folder out on all 27, so it saw the other lockbox folders; that favors
  `W0`.
- Each winner is read **once**. The runner-ups and every other candidate are
  not read on the fresh labels or the lockbox picks.

### Bars

"Hide / sort last" semantics, as discard-labels: the cut's frames are hidden
from what the user picks from, or sorted last, and stay recoverable.

- **Discard**: a cut of the discard winner at **q5 or q10** is **usable**
  when all of these hold on the fresh labels:
  1. weighted `RR` <= **25%**, and its stratified 97.5% quantile <= **35%**;
  2. `PL` <= **5%** (full dump, label-free);
  3. it removes **>= 4.5%** of the fresh population's frames (label-free);
  4. **beats the base rate**: `DP`'s stratified 2.5% quantile is above the
     base rate measured on the fresh labels (point estimate);
  5. **beats `G0`**: weighted `DP` minus `G0`'s at the same `q`, paired,
     stratified **2.5% quantile above zero**;
  6. **beats `eye_focus`-alone**: weighted discard AUC minus `eye_focus`'s,
     paired, stratified **2.5% quantile above zero** (one value for every
     cut).

  **q20** is read with the same six conditions and reported, but a pass
  needs q5 or q10 (q20 already fails condition 2, 5.73%). **Best cut**: the
  deepest passing one of q5 and q10. The discard winner **passes** when at
  least one of q5 / q10 is usable.
- **Pick**: on the 7 lockbox folders, the pick winner **passes** when all of
  these hold:
  1. mean `P@10` >= `W0`'s mean `P@10` **+ 2.0 points**;
  2. the folder-bootstrap **2.5% quantile** of the mean paired `P@10`
     difference against `W0` is **above zero**;
  3. **at least 5 of the 7** folders have `P@10` at or above `W0`'s (16 of
     25 scaled, 64% of 7 rounded up).

  The frame AUC and `R@10` (and `P@5`, `P@20`, R-precision) are reported
  beside, not decided on.

### Follow-up rule

Applied per task, in this order; the Decision (Step 6) names the outcome as
a recommendation:

1. **The winner passes its bars** → an **in-app suggestion plan** for that
   task: for discard, discard candidates **sorted last** with a rescue loop
   at the best cut's depth; for pick, a **strip mark or sort for the
   folder's top 10%**.
2. **No pass, but close** → **more labels or more folders**, a later plan
   sized at about 3 s per frame (600 frames about 30 minutes):
   - discard: at q5 or q10 the winner's `RR` point is <= 25% and conditions
     2 and 3 hold, and every failing condition is a quantile condition (the
     `RR` 97.5% quantile, the `DP` 2.5% quantile against the base rate) or a
     "beats" condition (5 or 6); **or** its `DP` − `G0` point at q5 or q10
     is positive with the stratified interval spanning zero;
   - pick: the mean `P@10` gain over `W0` is positive with the
     folder-bootstrap interval spanning zero.
3. **Otherwise** → **nothing** for that task.

### Multiplicity and population

- **Multiplicity**: two frozen winners, each read once; the discard winner
  on two eligible cuts with six conditions each, the pick winner on one
  read with three. The selection's multiplicity (69 candidates × 2 tasks ×
  3 cuts) is spent in the exploration; the confirmation adds no selection.
  The Decision reads a lone marginal pass, or one fragile on the cluster
  bootstrap, as such.
- **Selection bias that the confirmation cannot remove**: the discard
  winner is a rank mix of `G0` and `eye_focus`, the two scores this plan's
  design was chosen after, and the fresh strata are cut by the winner and
  `G0` themselves; the fresh labels are new frames, not a new population.
- **Population caveat**: one photographer's 27 folders, mostly basketball;
  600 fresh labels on 30,659 frames; the pick read on 7 folders, so its
  folder bootstrap is wide and a fail is weakly informative.

## Results of the confirmation

Run once on 2026-10-21 by `confirmread.py` (`python -I -B confirmread.py
D:\Photos\tests\2026-10-12-rank-app D:\Photos\tests\2026-10-15-folder-wide-eval
D:\Photos\tests\2026-10-16-reject-filter-eval
D:\Photos\tests\2026-10-18-discard-labels D:\Photos\tests\2026-10-17-bad-labels
D:\Photos\tests\2026-10-20-model-search 8`), its stdout saved as
`confirmread.out.md` in the data folder; the script refuses to run when that
file exists. The labels came from `confirmlabels.py` (`python -I -B
confirmlabels.py D:\Photos\tests\2026-10-20-model-search`, stdout saved as
`confirmlabels.out.md` there). Discard intervals are the stratified bootstrap's
95% (1,000 draws, seed 0, within `b05`, `b10`, `b20`, `g0only` and each
`control` folder cell, one count matrix for every score), the folder-cluster
interval beside it where marked; values weighted by the fresh `1/π`. Pick
intervals are the 1,000-draw folder bootstrap (seed 0) of the mean paired
per-folder difference. The exploration's 600 labels were not opened (an
audit hook saw only `manifest.tsv` in each earlier round's folder).

### The fresh labels

600 labeled, 0 skipped, 0 copies missing, 0 other files in `sample\`; 248
copies carry a `.dop` + `.xmp` pair, the `.dop` deciding on all, 0 `.dop` /
XMP disagreements on rating 1; **0 unexpected marks** (no Pick, Reject,
color label or rating other than 1). Sampled dump picks 36 (30 keep, 6
discard), sampled explicit rejects 3 (0 keep, 3 discard).

| Stratum | Sampled | Keep | Discard | `π` |
| --- | ---: | ---: | ---: | ---: |
| `b05` | 120 | 28 | 92 | 0.0865801 |
| `b10` | 120 | 29 | 91 | 0.0812458 |
| `b20` | 120 | 36 | 84 | 0.039383 |
| `g0only` | 120 | 72 | 48 | 0.0721154 |
| `control` | 120 | 83 | 37 | per folder |
| Total | 600 | 248 | 352 | |

**Discardable base rate 42.31% [35.85, 48.91]** (cluster [33.44, 51.61];
unweighted 58.67%), against the exploration's 48.45% [42.03, 55.11] on its
own population.

### Checks

Every check passes: the 31 `strata.tsv` rows and every label's stratum and
`π` recompute from the dump; the sum of `1/π` is 30,659.0; no fresh label is
in an earlier sample; the winner's and `G0`'s `PL` (0.77 / 2.10 / 5.73%,
0.68 / 2.10 / 5.18%) and removed shares of the fresh population (4.52 /
9.34 / 19.28%, 4.53 / 9.09 / 18.72%) reproduce Steps 3 and 4; the pick
recipe refit on 19 exploration folders reproduces `scores_pick.tsv` on the
spot fold `2026-09-27-a` exactly (0.00e+00); the pick fit's training
folders hold no lockbox folder; all 7 lockbox folders qualify.

### Discard: the winner on the fresh labels

| Score | Cut | `DP` % | `RR` % (cluster) | `PL` % | Removed % | `DR` % | `DO` % |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `G0 ⊕ ef` | q5 | 76.7 [69.2, 84.2] | 23.3 [15.8, 30.8] ([17.4, 32.3]) | 0.77 | 4.52 | 8.2 | 40.7 |
| `G0 ⊕ ef` | q10 | 76.2 [70.8, 81.7] | 23.8 [18.3, 29.2] ([17.3, 32.9]) | 2.10 | 9.34 | 16.8 | 38.8 |
| `G0 ⊕ ef` | q20 | 73.0 [68.1, 77.8] | 27.0 [22.2, 31.9] ([20.5, 36.0]) | 5.73 | 19.28 | 33.3 | 35.0 |
| `G0` | q5 | 76.5 [69.1, 84.3] | 23.5 [15.7, 30.9] ([16.0, 34.9]) | 0.68 | 4.53 | 8.2 | 40.7 |
| `G0` | q10 | 69.9 [64.2, 76.3] | 30.1 [23.7, 35.8] ([23.5, 39.2]) | 2.10 | 9.09 | 16.2 | 39.3 |
| `G0` | q20 | 63.5 [59.0, 68.0] | 36.5 [32.0, 41.0] ([30.4, 45.2]) | 5.18 | 18.72 | 28.5 | 37.4 |
| `eye_focus` (missing last) | q5 | 94.5 [85.5, 98.9] | 5.5 [1.1, 14.5] ([1.3, 17.7]) | 4.44 | 4.80 | 9.5 | 40.0 |
| `eye_focus` (missing last) | q10 | 87.1 [78.2, 92.8] | 12.9 [7.2, 21.8] ([6.1, 26.5]) | 9.14 | 9.70 | 16.4 | 38.4 |
| `eye_focus` (missing last) | q20 | 87.2 [82.1, 91.1] | 12.8 [8.9, 17.9] ([7.7, 23.3]) | 15.59 | 19.64 | 32.6 | 33.9 |
| Random (seed 0) | q5 | 44.2 [18.7, 75.2] | 55.8 [24.8, 81.3] ([17.6, 82.4]) | 4.71 | 4.96 | 6.8 | 42.2 |
| Random (seed 0) | q10 | 46.1 [25.9, 67.4] | 53.9 [32.6, 74.1] ([28.5, 76.4]) | 9.48 | 10.01 | 12.2 | 41.8 |
| Random (seed 0) | q20 | 44.1 [28.9, 59.9] | 55.9 [40.1, 71.1] ([43.9, 71.3]) | 19.26 | 19.98 | 21.3 | 41.8 |

Weighted discard AUC: `G0 ⊕ ef` **0.684** [0.610, 0.756] (cluster [0.605,
0.762]); `G0` 0.630 [0.561, 0.703]; `eye_focus` **0.785** [0.723, 0.844]
(cluster [0.693, 0.881]); random 0.536 [0.458, 0.622].

Paired differences of the winner (stratified, cluster in parentheses):

| Difference | q5 | q10 | q20 |
| --- | ---: | ---: | ---: |
| `G0 ⊕ ef` − `G0`, `DP` points | +0.1 [-7.0, +6.9] ([-6.3, +7.0]) | +6.3 [+0.9, +11.4] ([+0.9, +11.4]) | +9.6 [+5.4, +13.7] ([+4.1, +14.9]) |
| `G0 ⊕ ef` − `eye_focus`, `DP` points | -17.8 [-25.8, -7.4] ([-27.1, -4.3]) | -10.9 [-17.6, -1.7] ([-20.8, +0.7]) | -14.2 [-19.0, -8.8] ([-19.8, -7.2]) |
| `G0 ⊕ ef` − random, `DP` points | +32.5 [-0.2, +59.0] ([-9.6, +58.4]) | +30.2 [+8.0, +51.9] ([+1.8, +51.9]) | +28.9 [+13.6, +44.4] ([+17.0, +41.6]) |
| `G0 ⊕ ef` − `G0`, AUC | +0.055 [-0.008, +0.116] ([-0.013, +0.127]) | | |
| `G0 ⊕ ef` − `eye_focus`, AUC | -0.101 [-0.159, -0.044] ([-0.201, -0.016]) | | |
| `G0 ⊕ ef` − random, AUC | +0.148 [+0.046, +0.259] ([+0.076, +0.220]) | | |

### Discard: the bars applied

| Cut | (1) `RR` <= 25%, 97.5% <= 35% | (2) `PL` <= 5% | (3) Removed >= 4.5% | (4) `DP` 2.5% > 42.31% | (5) `DP` − `G0` 2.5% > 0 | (6) AUC − `eye_focus` 2.5% > 0 | Verdict |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| q5 | 23.3, 30.8: yes | 0.77: yes | 4.52: yes | 69.2: yes | -7.0: **no** | -0.159: **no** | not usable |
| q10 | 23.8, 29.2: yes | 2.10: yes | 9.34: yes | 70.8: yes | +0.9: yes | -0.159: **no** | not usable |
| q20 (not eligible) | 27.0, 31.9: **no** | 5.73: **no** | 19.28: yes | 68.1: yes | +5.4: yes | -0.159: **no** | not usable |

**Discard verdict: `G0 ⊕ ef` does not pass** (no usable cut at q5 or q10).
**Follow-up rule: (2), more labels or more folders**: at q5 and at q10 the
`RR` point is <= 25% with conditions 2 and 3 holding, and every failing
condition is a "beats" condition (q5: 5 and 6; q10: 6 only); the second
branch also holds at q5 (`DP` − `G0` +0.1 with the interval spanning zero).

### Pick: the winner on the lockbox

`All@32 / HGB-mono / pick` fit once on the 20 exploration folders, scored on
the 7 lockbox folders (9,591 frames, 1,301 picks); `W0` stored. Mean
(per-folder SD); pooled is the frame-weighted mean.

| Score | `P@5` % | `P@10` % | `P@20` % | `R@10` % | R-precision % | Frame AUC | Pooled `P@10` | Pooled frame AUC |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `All@32 / HGB-mono / pick` | 36.3 (14.7) | 28.0 (13.1) | 24.6 (8.9) | 20.2 (9.5) | 25.7 (11.6) | 0.657 (0.103) | 37.4 | 0.732 |
| `W0` | 30.8 (16.8) | 33.2 (10.2) | 25.2 (9.7) | 24.0 (7.6) | 28.8 (9.5) | 0.658 (0.089) | 37.2 | 0.696 |

| Folder | Frames | Picks | Winner `P@10` % | `W0` `P@10` % | Winner frame AUC | `W0` frame AUC |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-07-26` | 64 | 7 | 16.7 | 33.3 | 0.569 | 0.649 |
| `2026-07-30` | 936 | 105 | 12.9 | 15.1 | 0.577 | 0.508 |
| `2026-07-31` | 349 | 52 | 23.5 | 38.2 | 0.607 | 0.651 |
| `2026-08-01` | 3,401 | 463 | 35.3 | 33.2 | 0.725 | 0.670 |
| `2026-08-02` | 150 | 28 | 20.0 | 26.7 | 0.555 | 0.605 |
| `2026-08-29` | 3,451 | 475 | 47.5 | 46.7 | 0.788 | 0.756 |
| `2026-09-27-b` | 1,240 | 171 | 40.3 | 39.5 | 0.779 | 0.768 |

Paired against `W0` (folder bootstrap 95%, one-sided Wilcoxon p, folders at
or above): `P@10` **−5.2 [-10.8, -0.7]** (p 0.92, 3 of 7); `R@10` −3.8
[-8.1, -0.3] (3 of 7); `P@5` +5.5 [-3.7, +16.4] (5 of 7); `P@20` −0.6
[-3.0, +1.8]; R-precision −3.1 [-7.9, +1.0]; frame AUC −0.001 [-0.039,
+0.036] (4 of 7).

| Pick bar | Value | Holds |
| --- | ---: | ---: |
| (1) mean `P@10` >= `W0`'s + 2.0 points | −5.2 points | no |
| (2) folder-bootstrap 2.5% quantile above zero | −10.8 | no |
| (3) at least 5 of 7 folders at or above `W0` | 3 of 7 | no |

**Pick verdict: `All@32 / HGB-mono / pick` does not pass. Follow-up rule:
(3), nothing** (the `P@10` gain point is negative, so the "close" branch
does not apply).

### Exploration beside confirmation

The same frozen winners and references, the exploration's numbers (Step 3,
"Results of the exploration") beside this read's:

| Reading | Exploration | Confirmation |
| --- | ---: | ---: |
| Discardable base rate % | 48.45 [42.03, 55.11] | 42.31 [35.85, 48.91] |
| `G0 ⊕ ef` `DP@5` / `RR@5` % | 88.4 [82.9, 93.2] / 11.6 | 76.7 [69.2, 84.2] / 23.3 |
| `G0 ⊕ ef` `DP@10` / `RR@10` % | 90.0 [86.1, 93.3] / 10.0 | 76.2 [70.8, 81.7] / 23.8 |
| `G0 ⊕ ef` `DP@20` / `RR@20` % | 80.1 / 19.9 | 73.0 [68.1, 77.8] / 27.0 |
| `G0 ⊕ ef` AUC | 0.763 [0.703, 0.822] | 0.684 [0.610, 0.756] |
| `G0 ⊕ ef` − `G0`, `DP@5` points | +5.7 [+0.4, +11.4] | +0.1 [-7.0, +6.9] |
| `G0 ⊕ ef` − `G0`, `DP@10` points | +13.7 [+9.8, +17.7] | +6.3 [+0.9, +11.4] |
| `G0 ⊕ ef` − `G0`, AUC | +0.053 [+0.008, +0.099] | +0.055 [-0.008, +0.116] |
| `G0 ⊕ ef` − `eye_focus`, AUC | -0.015 [-0.062, +0.030] | -0.101 [-0.159, -0.044] |
| `G0` `DP@5` / `DP@10` %; AUC | 82.7 / 76.3; 0.710 | 76.5 / 69.9; 0.630 |
| `eye_focus` `RR@5` / `RR@10` %; AUC | 16.8 / 14.3; 0.777 | 5.5 / 12.9; 0.785 |
| Random `DP@5` / `DP@10` %; AUC | 52.0 / 61.5; 0.528 | 44.2 / 46.1; 0.536 |
| Pick winner mean `P@10` % (18 exploration / 7 lockbox folders) | 30.4 (10.7) | 28.0 (13.1) |
| `W0` mean `P@10` % | 31.2 (11.6) | 33.2 (10.2) |
| Pick winner `P@10` − `W0`, points | -0.9 [-2.7, +0.5] | -5.2 [-10.8, -0.7] |
| Pick winner folders at or above `W0` | 12 of 18 | 3 of 7 |
| Pick winner mean `R@10` % / frame AUC | 22.3 / 0.671 | 20.2 / 0.657 |
| `W0` mean `R@10` % / frame AUC | 22.9 / 0.669 | 24.0 / 0.658 |

### Reading

Every check passes, and the read ran once. On 600 fresh labels (248 keep,
352 discard, no unexpected mark) the discardable base rate is 42.31%
[35.85, 48.91]. **Discard**: `G0 ⊕ ef` removes 76.7% discardable frames at
q5 (`RR` 23.3 [15.8, 30.8]) and 76.2% at q10 (`RR` 23.8 [18.3, 29.2]),
both within the `RR` bar with `PL` 0.77 / 2.10% and the removed share 4.52 /
9.34%; its `DP` exceeds `G0`'s by +6.3 [+0.9, +11.4] points at q10 and by
+0.1 [-7.0, +6.9] at q5; its AUC, 0.684, is below `eye_focus`'s 0.785 by
0.101 [-0.159, -0.044], so condition 6 fails at every cut and neither
eligible cut is usable; the follow-up rule gives (2). Against the
exploration, its `DP` fell by 11.7 points at q5 and 13.8 at q10, its `RR`
roughly doubled (11.6 → 23.3, 10.0 → 23.8), its AUC fell by 0.079 and its
lead over `G0` at q10 halved (+13.7 → +6.3); its AUC lead over `G0`
stayed (+0.053 → +0.055) while both fell; `eye_focus`'s AUC held (0.777 →
0.785) and its gap to the winner widened from 0.015 to 0.101. The fresh
strata were cut by the winner and `G0`, and `g0only` is 40% discard by
count against 70–77% in the winner's bands. **Pick**: on the 7 lockbox
folders the winner's mean `P@10` is 28.0% against `W0`'s 33.2%, −5.2
[-10.8, -0.7] points with 3 of 7 folders at or above; every pick bar
fails and the follow-up rule gives (3). Its `P@5` reads +5.5 [-3.7,
+16.4] and its frame AUC −0.001, and it is at or above `W0` on the three
largest folders (`2026-08-01`, `2026-08-29`, `2026-09-27-b`) and below on
the four smaller ones; the pooled `P@10` is 37.4 against 37.2. Its
exploration gap to `W0` (−0.9) widened to −5.2. `W0` was trained with the
other lockbox folders and the winner without them. Step 6 decides.
