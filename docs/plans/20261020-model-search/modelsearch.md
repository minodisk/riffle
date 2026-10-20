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
