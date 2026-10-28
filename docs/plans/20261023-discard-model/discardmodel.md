# Discard model: improve on `eye_focus` with the 1,200 blind labels

## Pre-registration of the exploration

Written 2026-10-24, **before `discardfit.py` exists and before any candidate
has been fit or read**: when this section was written no model of this plan
had been fit, no score of any candidate existed anywhere, no script of this
plan existed, and `D:\Photos\tests\2026-10-23-discard-model\` held nothing of
this plan. It is merged on its own, before `discardfit.py` or
`discardread.py` exists; the merge commit of the PR that adds this section,
without results, is the record, as for the archived
[model-search exploration](../_archived/20261020-model-search/modelsearch.md)
and [face-choice evaluation](../_archived/20261023-face-choice/facechoice.md)
before it. Nothing in this section changes after a candidate's score
exists; a deviation found while implementing Steps 2–3 is recorded in
`learnings.md` with its reason and does not touch the selection rule or the
stop rule.

The question: can `eye_focus` plus a few technical columns, a fill for the
frames where `eye_focus` is missing, and a monotone or simple form order the
blind "OK to discard?" labels better than `eye_focus` alone? Riffle's AI
goal is to suggest frames to discard (sorted last, not hidden) so the human
picks from what remains. This section fixes the exploration (Steps 2–3) on
the 1,200 labels of both rounds and the stop rule that decides whether a
fresh blind round is drawn; the confirmation's bars are fixed in Step 5's
"Pre-registration of the confirmation", before any fresh label exists, and
their planned values are listed under "What happens after".

### What was known before this section

This pre-registration is **not blind**. The design (the candidates, the
columns, the fill, the stop rule) was chosen **after reading `eye_focus`'s
and `af-whole`'s numbers on both rounds**, so any winner is selected on
labels that shaped the search as well as the fit; the Decision weighs this
selection bias. On the 27-folder dump (32,459 frames, 4,419 picks):

| Round | Reading | Value | Source |
| --- | --- | --- | --- |
| 2 | `eye_focus` (missing last) weighted discard AUC | 0.785 [0.723, 0.844] (cluster [0.693, 0.881]) | [modelsearch.md, "Results of the confirmation"](../_archived/20261020-model-search/modelsearch.md); [facechoice.md, "Reproduction checks"](../_archived/20261023-face-choice/facechoice.md) |
| 2 | `eye_focus` `RR` q5 / q10 / q20 | 5.5 / 12.9 / 12.8% | same |
| 2 | `eye_focus` `PL` q5 / q10 / q20 (full dump) | 4.44 / 9.14 / 15.59% | same |
| 2 | `eye_focus` removed q5 / q10 / q20 (round 2 population) | 4.80 / 9.70 / 19.64% | same |
| 2 | Discardable base rate | 42.31% [35.85, 48.91] | same |
| 2 | `G0 ⊕ ef` (the model-search discard winner) / `G0` AUC | 0.684 / 0.630 | [model-search Decision](../_archived/20261020-model-search/plan.md) |
| 1 | `eye_focus` (missing last) weighted discard AUC | 0.777 [0.716, 0.841] | [discard.md, "Main table"](../_archived/20261018-discard-labels/discard.md); facechoice.md, "Reproduction checks" |
| 1 | `eye_focus` `RR` q5 / q10 / q20 | 16.8 / 14.3 / 28.3% | facechoice.md, "Reproduction checks" |
| 1 | Discardable base rate | 48.45% [42.03, 55.11] | same |
| 2 | `af-whole` AUC − `af` (paired) | +0.014 [−0.018, +0.052] | [facechoice.md, "Results"](../_archived/20261023-face-choice/facechoice.md) |
| 1 | `af-whole` AUC − `af` (paired) | −0.028 [−0.065, +0.009] (cluster [−0.063, +0.005]) | facechoice.md, "2. The rules, round 1 (replication)", all labeled frames |
| - | Missing share on the dump, `af` (today's) / `af-whole` | 12.4% / 4.5% | facechoice.md, "Results" (face counts) |

- `eye_focus` beat every learned model so far; the face-choice evaluation
  found no better face rule, `af-whole` the only near-tie on round 2 (and
  below `af` on round 1), filling more frames.
- Today a frame without `eye_focus` sorts last, so the 12.4% of the dump it
  leaves missing sit at the bottom of every folder's cut; that is where
  `eye_focus`'s `PL` comes from (4.44% at q5, 9.14% at q10). A fill that
  moves some of them up may lower `PL` and change `RR` at q5 at once.
- Round 2 was a confirmation sample (its strata cut by `G0 ⊕ ef` and `G0`)
  and has been read for that confirmation and by the face-choice plan;
  round 1 has been read by four plans. Selection on them is a selection,
  not a confirmation; only the fresh round of Steps 4–6 decides.

### Data, labels and weights

- **Labels.** Round 1: `D:\Photos\tests\2026-10-18-discard-labels\`
  (`labels.tsv` 600 rows, 211 keep / 389 discard; `manifest.tsv` with
  `band` and `inclusion_prob`; `strata.tsv`; population the dump minus the
  1,200 strict frames, 31,259). Round 2:
  `D:\Photos\tests\2026-10-20-model-search\` (600 rows, 248 keep / 352
  discard; `manifest.tsv`, `strata.tsv` with 31 cells; population the dump
  minus the 1,800 earlier-labeled frames, 30,659). Both are rows of the
  27-folder dump `D:\Photos\tests\2026-10-12-rank-app\`, loaded by the
  archived `gate.load` / `gate.build` and `folder_gate.extend`, as
  `facechoiceread.py` does; `badfilter.load` is **not** called (it opens
  the strict labels).
- **Per-face dump**: `D:\Photos\tests\2026-10-23-face-choice\faces\<folder>.tsv`
  (the crop and whole sets per frame; per face `eye_focus`, `state`,
  `scored`, `ear`, `yaw` / `pitch` / `roll`, `score`, box, and `nearest`,
  the set's `candidate::nearest_face` to the AF point), read by
  `facechoiceread.load_faces` with its 0.8 score floor. Every dump frame has
  a trusted AF point.
- **Training weights**: each labeled frame's `1/π` from its own round's
  `manifest.tsv`, **clipped at 5 × that round's median sampled `1/π` (the cap
  per round, over its 600 sampled `1/π`) and rescaled to mean 1 per round
  over each fold's training rows of that round, with no further rescale of
  the pooled training set**, so the two rounds carry equal total weight in
  every fold. The clip factor is `searchfit`'s (`CAP_FACTOR` = 5); the
  per-round cap and rescale are this plan's (model-search had one round).
  The Kish ESS (`retargetfit.kish`) is printed per fold with and without
  the clip.
- **Harness, imported by path, none modified**: `badfilter` (`read_tsv`,
  `strat_counts`, `cluster_counts`, `graded_cut`, `finite_by_folder`,
  `aucs`, `auc_pair`, `quant`, `iv`, `civ`, `Tee`), which brings
  `rejectfilter` as `bf.rf` (`HGB`, `load_ear`, `bottom`), `gate` (`load`,
  `build`, `tech_row`, `TECH`, `CS`, `logit`, `table`), `folder_gate`
  (`extend`, `design`, `choose_c`), `folderwide` as `bf.fw` and the
  archived bad-photo `sample` as `bf.sample` (aliased `bs` by the later
  samplers: `allocate`, `under`, `listing`, `sha256`, `cell`); then
  `discardfilter` (`metrics`, `unweighted`, `counts`, `QS`, `PL_BAR`,
  `REMOVED_BAR`); `retargetfit.kish`; `searchfit`'s weight clip and its
  inner-`GroupKFold` `C` choice (`choose_c`) with the `C = 1.0` fallback;
  `facechoiceread.load_faces` and `choose`; `confirmsample.py` (strata,
  allocation, the copy with SHA-256 checks, the audit hook on the earlier
  rounds' folders) and `confirmlabels.py` (rating 1 = keep, unmarked =
  discard, unexpected marks listed). No script of this plan is named
  `sample.py`.

### The columns

Every value is from the dump or the per-face dump; nothing is new to the
scan.

- **`S` (3)**:
  1. the logit (`gate.logit`) of `eye_focus` **filled**: today's value, or,
     where today's is missing, the `af-whole` face's (the whole set's
     `nearest` face, `facechoiceread.choose`'s rule);
  2. the indicator "no `eye_focus` today";
  3. ln relative sharpness (`gate.TECH`'s first column, as
     `gate.tech_row`).
- **`T` (10)**: `S` plus
  4. the indicator "no `eye_focus` after the fill";
  5. the EAR of the frame (`rf.load_ear`), today's frame-level value (the
     face behind today's `eye_focus`), NaN on the frames the fill touches;
  6. |yaw| (clipped at 90), 7. |pitch| (clipped at 60), 8. |roll| (clipped
     at 60), as `gate.tech_row`;
  9. the indicator "no pose", today's frame-level one (set on the frames
     the fill touches, where today's pose is NaN too);
  10. the number of whole-set faces at or above the 0.8 floor, capped at 3.
- **Missing values**: NaN is mean-filled on the training folders for the
  logistic forms (the mean taken on each fold's training folders) and kept
  for HGB.

### The candidates (5) and the references

Every score is "higher keeps"; a missing score sorts last (`-inf` in the
cut, the folder's lowest finite value minus 1 in the AUC), as every earlier
read.

- **References, not candidates**: `ef` (today's `eye_focus`, missing last)
  and random (seed 0).
- **`ef-fill`**: the label-free rule; today's `eye_focus`, the missing
  filled by `af-whole`'s value, the rest last. It differs from the
  face-choice `af-whole` rule, which replaces today's value on every frame
  where the whole set's nearest face differs; `ef-fill` changes only the
  frames today leaves missing.
- **`LR/S`** and **`LR/T`**: a standardized L2 logistic on `S` / `T`, `C` by
  the inner `GroupKFold` over `gate.CS` (0.01, 0.1, 1.0, 10.0) on the
  weighted discard AUC (`searchfit.choose_c`'s discard branch); when no `C`
  wins more than half the folds, the candidate is refit with `C = 1.0` in
  every fold (`searchfit`'s fallback).
- **`HGB-mono/T`**: `rf.HGB` with `monotonic_cst` over `T`: keep rises with
  the `eye_focus` logit, the sharpness and the EAR (+1), falls with |yaw|,
  |pitch|, |roll| (−1), 0 on the three indicators and the face count.
- **`HGB/T`**: `rf.HGB` as is on `T`, the reading of what the constraints
  cost.

Nothing else is searched: no embedding columns, no burst context, no
hyperparameter grid.

### `monotonic_cst` with missing values on sklearn 1.9.1

Taken from the documentation only, as
[modelsearch.md, "`monotonic_cst` with missing values on sklearn 1.9.1"](../_archived/20261020-model-search/modelsearch.md)
read it: the `HistGradientBoostingClassifier` API page and the User Guide
state no restriction on combining `monotonic_cst` with NaN (the
constraints hold for binary classification over the positive class's
probability; missing values are supported natively). So `HGB-mono/T` keeps
the NaN, as `HGB/T` does, and its constraint is read as holding among
non-missing values only. Model search's `HGB-mono` fits raised nothing on
that combination and passed its spot check
([model-search learnings](../_archived/20261020-model-search/learnings.md)).
**Fallback**: if a Step 2 fit raises, or its scores break a constraint among
non-missing values on a spot fold, `HGB-mono/T` uses the mean-filled
columns in every fold, recorded in `learnings.md`.

### Training

The labeled frames of the 26 other folders (both rounds pooled), target
`discard`, the clipped per-round weights above. One leave-one-folder-out fit
per candidate and fold over the 27 dump folders; every dump frame scored by
a model that did not see its folder. Step 2 prints label-free readings only
(the removed share and `PL` per candidate and q on the full dump, the
training counts, caps and Kish ESS per fold, the chosen `C`, the missing
shares); the labels enter training only.

### The read (Step 3)

Per round with its own population, `π` and strata (round 2 primary, round 1
replication, as face-choice), discard.md's vocabulary. Inputs beyond the
labels folders, as `facechoiceread.py` reads them: the strict sample's
`manifest.tsv` only, in `D:\Photos\tests\2026-10-17-bad-labels\` (to
exclude its 1,200 frames from round 1's population and 1,800 from round 2's;
no `labels.tsv` is opened there); `scores_reject.tsv` (`G0`: round 1's
bands and round 2's `g0only`, `G0`'s bottom 20% minus the winner's) in
`D:\Photos\tests\2026-10-16-reject-filter-eval\`; and round 2's
`scores_discard.tsv` (`G0 ⊕ ef`: round 2's `b*`, with `G0` for `g0only`) in
its model-search folder. Steps 2 and 3 take the reject-filter-eval and
strict-labels folders in their argument lists.

- the weighted discard AUC with the stratified (1,000 draws, seed 0) and
  folder-cluster bootstraps, and the unweighted value;
- `DP` (discard precision), `RR` (rescue rate, 1 − `DP`), `PL` (the share
  of the dump's picks inside the cut, on the full dump, label-free) and the
  removed share at q5 / q10 / q20, with the per-folder bottom-q cut on the
  full dump (`bf.graded_cut`);
- paired differences against `ef` (one count matrix per round, so every
  candidate shares the draws);
- the **missing-face subset** (frames without today's `eye_focus`) and its
  complement read beside, so the text can say whether the gain sits in the
  frames `eye_focus` leaves missing.

### Selection rule and stop rule

- **Selection rule, mechanical**: among the candidates whose **q10** cut
  passes the label-free conditions on the full dump (**`PL` <= 5%**,
  **removed >= 4.5%**), the one with the **highest weighted AUC on round
  2**; ties by round 1's AUC. The references are not eligible.
- **Stop rule**: the winner goes to the fresh round only if its **AUC − `ef`
  point is above zero on both rounds**. Otherwise Steps 4–6 are skipped and
  the Decision (Step 7) is "nothing, keep `eye_focus`", the model-search
  Decision's own condition for that branch. If no candidate passes the
  label-free conditions at q10, there is no winner and the stop rule ends
  the plan the same way.

### Amendment (2026-10-24): the selection rule's cut

Decided by the user on 2026-10-24, after Step 2's `discardfit.py` had run,
on its **label-free readings only**: no candidate score had been read
against any `labels.tsv`, and no AUC, `DP` or `RR` of any candidate existed.
The text above is left as merged; this note replaces its eligibility
condition, against the section's own "nothing changes after a candidate's
score exists", and the Decision weighs it with the selection bias.

- **The amended selection rule**: a candidate is eligible when its **q5 or
  its q10** cut passes the label-free conditions on the full dump (`PL` <=
  5%, removed >= 4.5%), matching the confirmation bars' "q5 or q10". Among
  the eligible, the one with the highest weighted AUC on round 2, ties by
  round 1's AUC, as before; the stop rule is unchanged, and "no candidate
  passes" now reads "no candidate passes at q5 or q10".
- **The readings that prompted it** (`discardfit.out.md`, full dump): at
  q10 every candidate fails `PL` <= 5% (`ef-fill` 6.99%, `LR/S` 6.72%,
  `LR/T` 6.54%, `HGB-mono/T` 6.00%, `HGB/T` 6.79%; `ef`'s own q10 `PL` is
  9.14%, random's 9.48%), while at q5 every candidate passes both
  conditions (`PL` 2.47–3.91%, removed 4.96%). At q5 both references pass
  as well (`ef` `PL` 4.44%, random 4.71%). Only the removed-share
  condition is fixed at q5: the removed share is 4.96% for every score,
  because `floor(q * n / 100)` fixes it. `PL` <= 5% can still fail at q5,
  for a score that cuts picks faster than random does (random's 4.71% is
  close to the bar). On Step 2's readings every candidate passes it
  (2.47–3.91%), so in this run the eligibility filter excludes no candidate
  and the selection reduces to the highest round-2 AUC. (Corrected in Step
  3: the amendment as first merged said the q5 condition "cannot fail",
  which holds for the removed share only.)
- **Rejected**: keeping the q10-only rule, which no candidate passes and
  which would have ended the plan with no winner before any label was read
  for a candidate.

### Multiplicity and population

- **Multiplicity**: 5 candidates × 2 rounds × 3 cuts (plus the
  missing-face subset and its complement), read **without correction** on
  labels the design was chosen after; hence the mechanical rule, the stop
  rule and the single fresh read.
- **Population caveat**: one photographer, mostly basketball; 27 dump
  folders and 1,200 labels; the fresh round adds six folders of the same
  photographer.

### The user's decisions (at planning time)

- **Fresh population: six folders, 14,525 frames**: the four ARW folders
  shot after the dump plus `2026-09-13-a` and `2026-09-27-c`, whose culling
  the user confirmed is now finished. Rejected: the four post-dump folders
  only (9,665 frames, 928 picks). Not proposed: the Leica DNG folders (no AF
  point).
- **Training weights**: pooled over both rounds, each frame's own round's
  `1/π`, clipped and rescaled to mean 1 within the round. Rejected: round 2
  only.
- **Selection statistic**: round 2 primary with round 1 as replication, the
  stop rule needing a positive AUC − `ef` point on both. Rejected: one
  pooled weighted AUC over both rounds.
- **Bars of the confirmation**: the five conditions below, with `PL` read on
  the fresh folders' picks rather than the dump's. Rejected: adding a
  cut-level condition "`RR` at the cut not above `ef`'s at q5 by more than
  +5 points (97.5% quantile)".
- **`G0` on the fresh folders**: none. Rejected: adding it as a reading.

### The fresh-folder inventory (label-free)

From `D:\Photos\2026` (read-only), frame, pick and reject counts from the
files and their sidecars (investigated 2026-10-23; the ARW counts rechecked
2026-10-24), no label:

| Folder | ARW frames | Picks | Rejects | Note |
| --- | ---: | ---: | ---: | --- |
| `2026-10-11` | 3,190 | 134 | 23 | shot after the dump, culled (`.dop` picks, `Output\`) |
| `2026-10-12a` | 3,185 | 304 | 20 | same |
| `2026-10-12b` | 1,615 | 285 | 22 | same |
| `2026-10-17` | 1,675 | 205 | 0 | same |
| `2026-09-13-a` | 3,045 | 620 | not counted | 1,424 unflagged `.dop`; left out of the dump as unfinished on 2026-10-08, now with `Output\` and `Output-sequenced\`, culling confirmed finished |
| `2026-09-27-c` | 1,815 | 247 | 3 | 1,565 unflagged `.dop`; same |
| **Fresh population** | **14,525** | | | the six folders, as the user chose |

Excluded: `2026-08-29-l` (211) and `2026-09-05` (411), Leica DNG without an
AF point (the no-AF cue path, never exercised in the dump); `2026-06-02` /
`06-06` / `06-09` / `06-21` / `06-27`, every frame a pick (no negatives,
`PL` undefined); the January–May folders and `100LEICA`, no sidecars at all
(no picks). The pick counts above are the sidecars'; Step 4 prints the ones
`gate.load` derives (a pick is `dop == Pick`, or exported in `Output\`, or
no `.dop` and `xmp == Pick`).

### What happens after

No step of this plan changes the app. Step 2 fits the four learned
candidates and builds `ef-fill`, printing label-free readings only; Step 3
reads them on both rounds and applies the rules above. If the stop rule
sends the winner on, Step 4 dumps the fresh folders (label-free), Step 5
pre-registers the confirmation and draws about 600 fresh blind frames
(strata `b05` / `b10` / `b20` of the winner, `efonly`, per-folder
`control`), and Step 6 reads them once. The planned bars, which Step 5
fixes before any fresh label exists: a cut of the winner at q5 or q10 is
usable when weighted `RR` <= 25% (stratified 97.5% quantile <= 35%), `PL`
<= 5% on the fresh folders' picks, removed >= 4.5% of the fresh population,
`DP`'s stratified 2.5% quantile above the fresh base rate, and the winner's
AUC − `ef`'s stratified 2.5% quantile above zero. Pass → an in-app
suggestion plan; close → more labels; otherwise → nothing, `eye_focus`
stays. Step 7 decides.

## Results of the exploration

Run once on 2026-10-24 by `discardread.py` (`python -I -B discardread.py
D:\Photos\tests\2026-10-12-rank-app D:\Photos\tests\2026-10-23-face-choice
D:\Photos\tests\2026-10-16-reject-filter-eval
D:\Photos\tests\2026-10-17-bad-labels
D:\Photos\tests\2026-10-18-discard-labels
D:\Photos\tests\2026-10-20-model-search
D:\Photos\tests\2026-10-23-discard-model`), its stdout saved as
`discardread.out.md` in the data folder; wall time 91 s. Every interval is
the round's stratified bootstrap 95% (1,000 draws, seed 0) unless marked
cluster (folder-cluster, 1,000 draws, seed 0); values are weighted by the
round's `1/π` unless marked unweighted. Differences are paired against `ef`
on the same draws (stratified, cluster in parentheses). A missing score
sorts last.

### Checks

All 29 label-free checks passed before any `labels.tsv` was opened, and all
17 label checks after:

- `scores_model.tsv` has a row for each of the 32,459 dump frames, its
  learned columns are finite everywhere, and its `ef-fill` equals the fill
  recomputed from the per-face dump on every frame; the per-face dump's
  `af` reproduces the stored `eye_focus` on every frame.
- Every score's `PL` and removed share at q5 / q10 / q20 on the full dump
  equal Step 2's (`discardfit.out.md`).
- Both rounds' populations (30,659 and 31,259), `strata.tsv` (31 and 30
  cells), bands, `π` and the sums of `1/π` recompute from the dump.
- The known numbers reproduce exactly: round 2's base rate 42.31% [35.85,
  48.91] (cluster [33.44, 51.61]), `ef` AUC 0.785 [0.723, 0.844] (cluster
  [0.693, 0.881], unweighted 0.764), `RR` 5.5 / 12.9 / 12.8%, removed 4.80 /
  9.70 / 19.64%; round 1's base rate 48.45% [42.03, 55.11] (cluster [40.74,
  53.87]), AUC 0.777 [0.716, 0.841] (unweighted 0.803), `RR` 16.8 / 14.3 /
  28.3%, removed 4.83 / 9.78 / 19.87%.
- The files opened stayed within the allow-list: `scores_reject.tsv` under
  the reject-filter-eval folder, `manifest.tsv` under the strict sample's,
  `labels.tsv` / `manifest.tsv` / `strata.tsv` (and round 2's
  `scores_discard.tsv`) under the rounds', `scores_model.tsv` in the data
  folder.

### Data

Labeled frames without `eye_focus` today (the missing-face subset): 95 in
round 2 (18 keep, 77 discard; 8.3% of its population, weighted) and 99 in
round 1 (14 keep, 85 discard; 11.6%). The discardable base rate on all
labeled frames, the subset and its complement:

| Round | Frames | Discardable base rate, weighted % | Stratified 95% | Folder-cluster 95% | Unweighted % |
| --- | ---: | ---: | ---: | ---: | ---: |
| 2 | all | 42.31 | [35.85, 48.91] | [33.44, 51.61] | 58.67 |
| 1 | all | 48.45 | [42.03, 55.11] | [40.74, 53.87] | 64.83 |
| 2 | missing-face | 89.5 | [82.8, 94.4] | [77.4, 95.7] | 81.1 |
| 2 | complement | 38.1 | [31.6, 45.2] | [29.7, 46.3] | 54.5 |
| 1 | missing-face | 77.7 | [61.6, 93.5] | [58.1, 92.1] | 85.9 |
| 1 | complement | 44.6 | [38.1, 51.6] | [36.1, 51.4] | 60.7 |

### Where the cuts sit (label-free)

Per score and cut on the full dump: the removed frames that have no
`eye_focus` today, and the dump's picks among the removed.

| Score | Cut | Removed | Removed without `eye_focus` today | Picks removed | Picks removed without `eye_focus` today |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ef` (reference) | q5 | 1609 | 1602 (99.6%) | 196 | 196 (100.0%) |
| `ef` (reference) | q10 | 3234 | 3001 (92.8%) | 404 | 379 (93.8%) |
| `ef` (reference) | q20 | 6482 | 3818 (58.9%) | 689 | 469 (68.1%) |
| `random` (reference) | q5 | 1609 | 220 (13.7%) | 208 | 23 (11.1%) |
| `random` (reference) | q10 | 3234 | 415 (12.8%) | 419 | 49 (11.7%) |
| `random` (reference) | q20 | 6482 | 810 (12.5%) | 851 | 97 (11.4%) |
| `ef-fill` | q5 | 1609 | 963 (59.9%) | 173 | 104 (60.1%) |
| `ef-fill` | q10 | 3234 | 1539 (47.6%) | 309 | 158 (51.1%) |
| `ef-fill` | q20 | 6482 | 2350 (36.3%) | 567 | 224 (39.5%) |
| `LR/S` | q5 | 1609 | 1261 (78.4%) | 132 | 107 (81.1%) |
| `LR/S` | q10 | 3234 | 1991 (61.6%) | 297 | 193 (65.0%) |
| `LR/S` | q20 | 6482 | 3096 (47.8%) | 620 | 342 (55.2%) |
| `LR/T` | q5 | 1609 | 1074 (66.7%) | 130 | 92 (70.8%) |
| `LR/T` | q10 | 3234 | 1803 (55.8%) | 289 | 174 (60.2%) |
| `LR/T` | q20 | 6482 | 2833 (43.7%) | 609 | 299 (49.1%) |
| `HGB-mono/T` | q5 | 1609 | 707 (43.9%) | 109 | 50 (45.9%) |
| `HGB-mono/T` | q10 | 3234 | 1482 (45.8%) | 265 | 130 (49.1%) |
| `HGB-mono/T` | q20 | 6482 | 2461 (38.0%) | 593 | 250 (42.2%) |
| `HGB/T` | q5 | 1609 | 648 (40.3%) | 140 | 68 (48.6%) |
| `HGB/T` | q10 | 3234 | 1313 (40.6%) | 300 | 146 (48.7%) |
| `HGB/T` | q20 | 6482 | 2261 (34.9%) | 627 | 242 (38.6%) |

### Round 2 (primary), all labeled frames

| Score | AUC [stratified 95%] | Cluster | Unweighted | AUC - `ef` |
| --- | ---: | ---: | ---: | ---: |
| `ef` (reference) | 0.785 [0.723, 0.844] | [0.693, 0.881] | 0.764 |  |
| `random` (reference) | 0.536 [0.458, 0.622] | [0.469, 0.608] | 0.513 | -0.249 [-0.349, -0.152] ([-0.335, -0.172]) |
| `ef-fill` | 0.775 [0.714, 0.835] | [0.682, 0.875] | 0.748 | -0.010 [-0.022, -0.001] ([-0.018, -0.002]) |
| `LR/S` | 0.773 [0.709, 0.832] | [0.682, 0.871] | 0.761 | -0.012 [-0.026, +0.000] ([-0.028, +0.002]) |
| `LR/T` | 0.812 [0.757, 0.863] | [0.724, 0.898] | 0.778 | +0.027 [-0.020, +0.078] ([-0.031, +0.069]) |
| `HGB-mono/T` | 0.786 [0.727, 0.840] | [0.685, 0.888] | 0.749 | +0.001 [-0.049, +0.051] ([-0.042, +0.056]) |
| `HGB/T` | 0.772 [0.705, 0.839] | [0.679, 0.861] | 0.740 | -0.013 [-0.081, +0.051] ([-0.059, +0.041]) |

| Score | Cut | Sampled removed / discard | `DP` % | `RR` % | `RR` cluster | `PL` % | Removed % | `RR` - `ef`, points |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ef` (reference) | q5 | 43 / 38 | 94.5 [85.5, 98.9] (88.4) | 5.5 [1.1, 14.5] (11.6) | [1.3, 17.7] | 4.44 | 4.80 |  |
| `ef` (reference) | q10 | 87 / 66 | 87.1 [78.2, 92.8] (75.9) | 12.9 [7.2, 21.8] (24.1) | [6.1, 26.5] | 9.14 | 9.70 |  |
| `ef` (reference) | q20 | 204 / 160 | 87.2 [82.1, 91.1] (78.4) | 12.8 [8.9, 17.9] (21.6) | [7.7, 23.3] | 15.59 | 19.64 |  |
| `random` (reference) | q5 | 37 / 23 | 44.2 [18.7, 75.2] (62.2) | 55.8 [24.8, 81.3] (37.8) | [17.6, 82.4] | 4.71 | 4.96 | +50.3 [+19.0, +75.7] ([+12.6, +73.9]) |
| `random` (reference) | q10 | 66 / 39 | 46.1 [25.9, 67.4] (59.1) | 53.9 [32.6, 74.1] (40.9) | [28.5, 76.4] | 9.48 | 10.01 | +41.0 [+18.2, +61.8] ([+18.4, +59.0]) |
| `random` (reference) | q20 | 128 / 77 | 44.1 [28.9, 59.9] (60.2) | 55.9 [40.1, 71.1] (39.8) | [43.9, 71.3] | 19.26 | 19.98 | +43.1 [+27.3, +58.4] ([+31.6, +55.7]) |
| `ef-fill` | q5 | 44 / 31 | 83.6 [63.0, 92.9] (70.5) | 16.4 [7.1, 37.0] (29.5) | [7.7, 37.4] | 3.91 | 4.84 | +10.9 [-0.3, +30.8] ([+0.8, +29.0]) |
| `ef-fill` | q10 | 95 / 76 | 90.2 [82.9, 94.8] (80.0) | 9.8 [5.2, 17.1] (20.0) | [5.5, 20.8] | 6.99 | 9.77 | -3.1 [-9.6, +1.8] ([-8.9, +1.5]) |
| `ef-fill` | q20 | 229 / 179 | 83.4 [73.6, 90.3] (78.2) | 16.6 [9.7, 26.4] (21.8) | [9.1, 28.5] | 12.83 | 19.72 | +3.8 [-3.0, +12.7] ([-2.6, +12.5]) |
| `LR/S` | q5 | 45 / 31 | 85.1 [69.4, 93.0] (68.9) | 14.9 [7.0, 30.6] (31.1) | [5.3, 43.9] | 2.99 | 4.81 | +9.3 [+0.2, +23.6] ([+1.2, +33.7]) |
| `LR/S` | q10 | 94 / 69 | 86.5 [78.2, 92.1] (73.4) | 13.5 [7.9, 21.8] (26.6) | [6.2, 29.3] | 6.72 | 9.73 | +0.6 [-5.2, +6.1] ([-4.1, +8.6]) |
| `LR/S` | q20 | 221 / 173 | 87.4 [82.7, 91.1] (78.3) | 12.6 [8.9, 17.3] (21.7) | [8.0, 20.7] | 14.03 | 19.70 | -0.2 [-2.7, +1.9] ([-3.5, +1.8]) |
| `LR/T` | q5 | 58 / 43 | 87.2 [75.3, 93.7] (74.1) | 12.8 [6.3, 24.7] (25.9) | [4.8, 32.1] | 2.94 | 4.81 | +7.2 [-1.8, +17.8] ([-0.1, +21.3]) |
| `LR/T` | q10 | 106 / 78 | 84.7 [75.4, 90.9] (73.6) | 15.3 [9.1, 24.6] (26.4) | [7.8, 28.1] | 6.54 | 9.76 | +2.4 [-3.1, +8.2] ([-2.8, +7.5]) |
| `LR/T` | q20 | 206 / 166 | 84.4 [74.3, 91.5] (80.6) | 15.6 [8.5, 25.7] (19.4) | [7.1, 28.8] | 13.78 | 19.69 | +2.8 [-4.3, +12.8] ([-4.3, +13.8]) |
| `HGB-mono/T` | q5 | 66 / 50 | 80.4 [61.1, 93.6] (75.8) | 19.6 [6.4, 38.9] (24.2) | [4.9, 47.3] | 2.47 | 4.83 | +14.0 [-0.5, +33.1] ([+0.6, +34.8]) |
| `HGB-mono/T` | q10 | 115 / 86 | 83.1 [69.7, 91.8] (74.8) | 16.9 [8.2, 30.3] (25.2) | [6.5, 36.4] | 6.00 | 9.78 | +4.0 [-6.1, +16.4] ([-5.5, +14.4]) |
| `HGB-mono/T` | q20 | 205 / 152 | 72.3 [59.5, 83.5] (74.1) | 27.7 [16.5, 40.5] (25.9) | [9.2, 48.8] | 13.42 | 19.73 | +14.9 [+3.9, +27.7] ([-1.7, +33.3]) |
| `HGB/T` | q5 | 58 / 45 | 87.3 [75.1, 93.5] (77.6) | 12.7 [6.5, 24.9] (22.4) | [5.7, 26.1] | 3.17 | 4.87 | +7.2 [-3.2, +20.3] ([-7.0, +20.5]) |
| `HGB/T` | q10 | 111 / 85 | 86.4 [78.1, 91.6] (76.6) | 13.6 [8.4, 21.9] (23.4) | [7.9, 22.4] | 6.79 | 9.83 | +0.7 [-8.4, +9.4] ([-9.7, +8.5]) |
| `HGB/T` | q20 | 201 / 155 | 83.0 [72.9, 90.0] (77.1) | 17.0 [10.0, 27.1] (22.9) | [8.9, 30.3] | 14.19 | 19.82 | +4.2 [-2.9, +14.0] ([-5.2, +15.6]) |

### Round 2, the missing-face subset and its complement

The cut is the full-dump cut; only the labeled frames read are restricted.
On the missing-face subset `ef` is the folder's lowest finite value minus 1
on every frame, so its AUC there orders folders only and its cut is the
seed-0 draw within each folder. The subsets' cut tables are in
`discardread.out.md`.

The missing-face subset (95 labels):

| Score | AUC [stratified 95%] | Cluster | Unweighted | AUC - `ef` |
| --- | ---: | ---: | ---: | ---: |
| `ef` (reference) | 0.531 [0.336, 0.726] | [0.347, 0.762] | 0.560 |  |
| `random` (reference) | 0.636 [0.432, 0.807] | [0.521, 0.775] | 0.600 | +0.105 [-0.107, +0.309] ([-0.145, +0.287]) |
| `ef-fill` | 0.621 [0.415, 0.805] | [0.409, 0.782] | 0.547 | +0.091 [-0.176, +0.359] ([-0.307, +0.307]) |
| `LR/S` | 0.613 [0.389, 0.804] | [0.413, 0.784] | 0.532 | +0.082 [-0.183, +0.345] ([-0.225, +0.301]) |
| `LR/T` | 0.576 [0.365, 0.771] | [0.389, 0.771] | 0.515 | +0.045 [-0.189, +0.286] ([-0.222, +0.242]) |
| `HGB-mono/T` | 0.616 [0.381, 0.791] | [0.435, 0.795] | 0.536 | +0.086 [-0.193, +0.342] ([-0.227, +0.322]) |
| `HGB/T` | 0.575 [0.384, 0.746] | [0.422, 0.736] | 0.500 | +0.045 [-0.167, +0.249] ([-0.232, +0.241]) |

The complement (505 labels):

| Score | AUC [stratified 95%] | Cluster | Unweighted | AUC - `ef` |
| --- | ---: | ---: | ---: | ---: |
| `ef` (reference) | 0.753 [0.681, 0.823] | [0.652, 0.860] | 0.762 |  |
| `random` (reference) | 0.520 [0.431, 0.611] | [0.458, 0.594] | 0.500 | -0.233 [-0.343, -0.124] ([-0.329, -0.146]) |
| `ef-fill` | 0.753 [0.681, 0.823] | [0.652, 0.860] | 0.762 | +0.000 [+0.000, +0.000] ([+0.000, +0.000]) |
| `LR/S` | 0.737 [0.662, 0.810] | [0.638, 0.847] | 0.755 | -0.015 [-0.031, +0.000] ([-0.033, +0.001]) |
| `LR/T` | 0.786 [0.726, 0.844] | [0.686, 0.878] | 0.779 | +0.034 [-0.022, +0.096] ([-0.036, +0.081]) |
| `HGB-mono/T` | 0.760 [0.689, 0.820] | [0.648, 0.872] | 0.751 | +0.007 [-0.053, +0.069] ([-0.044, +0.066]) |
| `HGB/T` | 0.745 [0.669, 0.823] | [0.644, 0.843] | 0.745 | -0.007 [-0.086, +0.072] ([-0.065, +0.051]) |

### Round 1 (replication), all labeled frames

| Score | AUC [stratified 95%] | Cluster | Unweighted | AUC - `ef` |
| --- | ---: | ---: | ---: | ---: |
| `ef` (reference) | 0.777 [0.716, 0.841] | [0.719, 0.832] | 0.803 |  |
| `random` (reference) | 0.528 [0.455, 0.604] | [0.445, 0.627] | 0.502 | -0.250 [-0.341, -0.156] ([-0.362, -0.114]) |
| `ef-fill` | 0.766 [0.699, 0.828] | [0.711, 0.821] | 0.807 | -0.011 [-0.028, +0.003] ([-0.025, +0.007]) |
| `LR/S` | 0.782 [0.723, 0.839] | [0.723, 0.833] | 0.808 | +0.004 [-0.010, +0.019] ([-0.012, +0.019]) |
| `LR/T` | 0.795 [0.737, 0.851] | [0.723, 0.857] | 0.796 | +0.018 [-0.018, +0.053] ([-0.014, +0.054]) |
| `HGB-mono/T` | 0.752 [0.686, 0.817] | [0.675, 0.813] | 0.781 | -0.025 [-0.069, +0.015] ([-0.061, +0.003]) |
| `HGB/T` | 0.725 [0.651, 0.796] | [0.673, 0.768] | 0.766 | -0.053 [-0.103, -0.007] ([-0.086, -0.006]) |

| Score | Cut | Sampled removed / discard | `DP` % | `RR` % | `RR` cluster | `PL` % | Removed % | `RR` - `ef`, points |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ef` (reference) | q5 | 38 / 33 | 83.2 [56.7, 98.8] (86.8) | 16.8 [1.2, 43.3] (13.2) | [1.1, 50.8] | 4.44 | 4.83 |  |
| `ef` (reference) | q10 | 82 / 72 | 85.7 [70.1, 97.1] (87.8) | 14.3 [2.9, 29.9] (12.2) | [3.1, 32.0] | 9.14 | 9.78 |  |
| `ef` (reference) | q20 | 189 / 160 | 71.7 [59.8, 84.3] (84.7) | 28.3 [15.7, 40.2] (15.3) | [18.1, 42.4] | 15.59 | 19.87 |  |
| `random` (reference) | q5 | 27 / 16 | 52.0 [19.8, 85.5] (59.3) | 48.0 [14.5, 80.2] (40.7) | [16.8, 85.1] | 4.71 | 4.95 | +31.2 [-11.3, +73.7] ([-15.7, +73.7]) |
| `random` (reference) | q10 | 52 / 36 | 61.5 [37.9, 84.8] (69.2) | 38.5 [15.2, 62.1] (30.8) | [10.9, 62.6] | 9.48 | 9.99 | +24.2 [-2.3, +50.5] ([-15.5, +49.3]) |
| `random` (reference) | q20 | 121 / 77 | 56.0 [40.5, 71.5] (63.6) | 44.0 [28.5, 59.5] (36.4) | [27.0, 61.9] | 19.26 | 19.98 | +15.7 [-2.9, +35.7] ([-9.4, +38.2]) |
| `ef-fill` | q5 | 52 / 49 | 86.5 [65.6, 100.0] (94.2) | 13.5 [0.0, 34.4] (5.8) | [0.0, 33.0] | 3.91 | 4.91 | -3.3 [-16.4, +6.9] ([-23.8, +4.3]) |
| `ef-fill` | q10 | 110 / 95 | 69.8 [52.1, 88.7] (86.4) | 30.2 [11.3, 47.9] (13.6) | [17.3, 45.8] | 6.99 | 9.93 | +16.0 [-2.5, +33.8] ([-3.9, +34.3]) |
| `ef-fill` | q20 | 219 / 189 | 70.7 [59.2, 83.5] (86.3) | 29.3 [16.5, 40.8] (13.7) | [19.3, 40.8] | 12.83 | 20.04 | +1.0 [-4.5, +7.3] ([-5.2, +5.9]) |
| `LR/S` | q5 | 55 / 49 | 95.7 [90.3, 99.0] (89.1) | 4.3 [1.0, 9.7] (10.9) | [1.3, 11.5] | 2.99 | 4.89 | -12.5 [-38.3, +4.0] ([-45.8, +4.9]) |
| `LR/S` | q10 | 111 / 97 | 88.8 [75.7, 97.1] (87.4) | 11.2 [2.9, 24.3] (12.6) | [2.9, 24.5] | 6.72 | 9.90 | -3.0 [-11.0, +2.8] ([-15.0, +1.8]) |
| `LR/S` | q20 | 204 / 176 | 75.7 [63.8, 86.7] (86.3) | 24.3 [13.3, 36.2] (13.7) | [13.5, 37.7] | 14.03 | 19.97 | -3.9 [-10.9, +2.0] ([-12.7, +2.9]) |
| `LR/T` | q5 | 53 / 46 | 90.5 [80.4, 97.0] (86.8) | 9.5 [3.0, 19.6] (13.2) | [3.0, 25.0] | 2.94 | 4.89 | -7.3 [-34.7, +12.5] ([-36.8, +15.3]) |
| `LR/T` | q10 | 107 / 94 | 78.3 [60.9, 94.7] (87.9) | 21.7 [5.3, 39.1] (12.1) | [7.4, 34.4] | 6.54 | 9.91 | +7.4 [-5.8, +22.4] ([-9.6, +21.2]) |
| `LR/T` | q20 | 201 / 171 | 81.0 [70.3, 91.3] (85.1) | 19.0 [8.7, 29.7] (14.9) | [10.0, 31.1] | 13.78 | 19.96 | -9.2 [-19.1, +0.2] ([-22.8, +0.3]) |
| `HGB-mono/T` | q5 | 54 / 45 | 84.0 [61.2, 97.6] (83.3) | 16.0 [2.4, 38.8] (16.7) | [2.2, 39.6] | 2.47 | 4.91 | -0.9 [-30.8, +27.8] ([-32.7, +29.7]) |
| `HGB-mono/T` | q10 | 104 / 88 | 80.7 [64.7, 94.4] (84.6) | 19.3 [5.6, 35.3] (15.4) | [5.3, 35.6] | 6.00 | 9.92 | +5.1 [-14.1, +24.0] ([-18.5, +25.4]) |
| `HGB-mono/T` | q20 | 200 / 168 | 71.6 [59.7, 83.6] (84.0) | 28.4 [16.4, 40.3] (16.0) | [17.8, 41.5] | 13.42 | 19.99 | +0.1 [-6.5, +5.7] ([-5.3, +5.4]) |
| `HGB/T` | q5 | 52 / 47 | 97.2 [93.4, 99.4] (90.4) | 2.8 [0.6, 6.6] (9.6) | [0.6, 7.6] | 3.17 | 4.94 | -14.0 [-40.3, +2.9] ([-45.6, +1.7]) |
| `HGB/T` | q10 | 100 / 82 | 76.2 [61.0, 89.7] (82.0) | 23.8 [10.3, 39.0] (18.0) | [7.7, 43.9] | 6.79 | 9.97 | +9.6 [-8.7, +28.0] ([-17.2, +36.9]) |
| `HGB/T` | q20 | 194 / 157 | 71.8 [59.2, 84.1] (80.9) | 28.2 [15.9, 40.8] (19.1) | [18.1, 38.8] | 14.19 | 20.06 | -0.1 [-10.6, +10.5] ([-12.7, +10.0]) |

### Round 1, the missing-face subset and its complement

The missing-face subset (99 labels):

| Score | AUC [stratified 95%] | Cluster | Unweighted | AUC - `ef` |
| --- | ---: | ---: | ---: | ---: |
| `ef` (reference) | 0.635 [0.367, 0.913] | [0.255, 0.915] | 0.576 |  |
| `random` (reference) | 0.541 [0.351, 0.734] | [0.379, 0.723] | 0.493 | -0.095 [-0.450, +0.239] ([-0.419, +0.284]) |
| `ef-fill` | 0.336 [0.155, 0.809] | [0.162, 0.704] | 0.627 | -0.300 [-0.549, +0.074] ([-0.574, +0.062]) |
| `LR/S` | 0.808 [0.616, 0.906] | [0.660, 0.888] | 0.608 | +0.173 [-0.168, +0.469] ([-0.201, +0.549]) |
| `LR/T` | 0.744 [0.549, 0.887] | [0.567, 0.856] | 0.618 | +0.108 [-0.251, +0.449] ([-0.202, +0.474]) |
| `HGB-mono/T` | 0.716 [0.481, 0.888] | [0.578, 0.855] | 0.629 | +0.081 [-0.285, +0.446] ([-0.228, +0.498]) |
| `HGB/T` | 0.655 [0.388, 0.878] | [0.495, 0.852] | 0.671 | +0.020 [-0.326, +0.397] ([-0.306, +0.438]) |

The complement (501 labels):

| Score | AUC [stratified 95%] | Cluster | Unweighted | AUC - `ef` |
| --- | ---: | ---: | ---: | ---: |
| `ef` (reference) | 0.769 [0.699, 0.833] | [0.719, 0.822] | 0.809 |  |
| `random` (reference) | 0.523 [0.449, 0.607] | [0.434, 0.631] | 0.500 | -0.246 [-0.349, -0.142] ([-0.355, -0.118]) |
| `ef-fill` | 0.769 [0.699, 0.833] | [0.719, 0.822] | 0.809 | +0.000 [+0.000, +0.000] ([+0.000, +0.000]) |
| `LR/S` | 0.768 [0.699, 0.829] | [0.707, 0.821] | 0.804 | -0.001 [-0.014, +0.014] ([-0.020, +0.013]) |
| `LR/T` | 0.787 [0.724, 0.846] | [0.718, 0.851] | 0.792 | +0.018 [-0.023, +0.064] ([-0.018, +0.057]) |
| `HGB-mono/T` | 0.738 [0.664, 0.811] | [0.661, 0.797] | 0.776 | -0.031 [-0.080, +0.015] ([-0.074, +0.002]) |
| `HGB/T` | 0.709 [0.631, 0.787] | [0.651, 0.760] | 0.758 | -0.060 [-0.120, -0.008] ([-0.101, -0.011]) |

### The selection rule and the stop rule applied

Selection rule as amended 2026-10-24 (q5 or q10 eligible), the stop rule
as pre-registered:

| Candidate | q5 passes | q10 passes | Eligible | Round 2 AUC | Round 1 AUC | Round 2 AUC - `ef` | Round 1 AUC - `ef` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ef-fill` | yes | no | yes | 0.775 | 0.766 | -0.010 [-0.022, -0.001] ([-0.018, -0.002]) | -0.011 [-0.028, +0.003] ([-0.025, +0.007]) |
| `LR/S` | yes | no | yes | 0.773 | 0.782 | -0.012 [-0.026, +0.000] ([-0.028, +0.002]) | +0.004 [-0.010, +0.019] ([-0.012, +0.019]) |
| `LR/T` | yes | no | yes | 0.812 | 0.795 | +0.027 [-0.020, +0.078] ([-0.031, +0.069]) | +0.018 [-0.018, +0.053] ([-0.014, +0.054]) |
| `HGB-mono/T` | yes | no | yes | 0.786 | 0.752 | +0.001 [-0.049, +0.051] ([-0.042, +0.056]) | -0.025 [-0.069, +0.015] ([-0.061, +0.003]) |
| `HGB/T` | yes | no | yes | 0.772 | 0.725 | -0.013 [-0.081, +0.051] ([-0.059, +0.041]) | -0.053 [-0.103, -0.007] ([-0.086, -0.006]) |

**Winner: `LR/T`**, round 2 AUC 0.812 (no tie). **Stop rule: fresh round.**
Its AUC − `ef` points are +0.027 on round 2 and +0.018 on round 1, both
above zero, so the winner goes on to Steps 4–6.

### Reading

The facts, without a decision (Step 7 decides):

- **Every candidate is eligible**: all five pass `PL` <= 5% and removed >=
  4.5% at q5 (`PL` 2.47–3.91%) and none at q10 (`PL` 6.00–6.99%), so the
  selection came down to round 2's AUC.
- **`LR/T` is the only candidate above `ef` on both rounds**: round 2 0.812
  against 0.785 (+0.027 [−0.020, +0.078], cluster [−0.031, +0.069]), round 1
  0.795 against 0.777 (+0.018 [−0.018, +0.053], cluster [−0.014, +0.054]).
  Both stratified intervals include zero. `HGB-mono/T` is level on round 2
  (+0.001) and below on round 1 (−0.025); `LR/S` is −0.012 and +0.004;
  `HGB/T` is below on both (−0.013, −0.053 [−0.103, −0.007]).
- **The fill alone reads below `ef`**: `ef-fill` −0.010 [−0.022, −0.001]
  on round 2 and −0.011 [−0.028, +0.003] on round 1. On the complement it
  equals `ef` (difference 0 by construction). The missing-face frames are
  mostly discardable (weighted 89.5% on round 2, 77.7% on round 1, against
  38.1% and 44.6% on the complement), and sorting them last, as `ef` does,
  ranks them below the frames that have an `eye_focus`.
- **Where `LR/T`'s gain sits**: on the complement (frames with `eye_focus`
  today) it is +0.034 [−0.022, +0.096] on round 2 and +0.018 [−0.023,
  +0.064] on round 1, about its gain on all frames. On the missing-face
  subset it is +0.045 [−0.189, +0.286] (round 2, 95 labels) and +0.108
  [−0.251, +0.449] (round 1, 99 labels), intervals spanning about half the
  AUC scale. There `ef`'s own subset AUC (0.531 and 0.635) orders folders
  only. The all-frames AUC also counts the pairs across the two subsets, so
  it is not an average of the two.
- **`LR/T`'s cuts**: at q5 `RR` 12.8% [6.3, 24.7] on round 2 against `ef`'s
  5.5% (+7.2 [−1.8, +17.8]) and 9.5% [3.0, 19.6] on round 1 against 16.8%
  (−7.3 [−34.7, +12.5]); at q10 15.3% [9.1, 24.6] against 12.9% (+2.4
  [−3.1, +8.2]) and 21.7% [5.3, 39.1] against 14.3% (+7.4 [−5.8, +22.4]).
  Its `PL` is 2.94% at q5 and 6.54% at q10 against `ef`'s 4.44% and 9.14%.
  Of its q5 cut, 66.7% of the frames have no `eye_focus` today (`ef`'s:
  99.6%), and it removes 130 of the dump's picks at q5 against `ef`'s 196.
- Both rounds have been read before (round 2 by the model-search
  confirmation and face choice, round 1 by four plans), and this design was
  chosen after reading `eye_focus`'s and `af-whole`'s numbers on them; the
  selection rule's cut was amended after Step 2's label-free readings. 5
  candidates × 2 rounds × 3 cuts plus the subsets were read without
  correction. One photographer, mostly basketball.

## Pre-registration of the confirmation

Written 2026-10-25, after Step 4's label-free dump of the six fresh folders
and **before any fresh label exists**: when this section was written no
frame of the six fresh folders had been drawn, no copy existed, no winner
score had been computed on a fresh frame, and `freshsample.py` did not
exist. The merge commit of the PR that adds this section is the record; its
"As drawn" paragraph is added in the same PR after the draw and is
label-free. Nothing in this section changes after Step 6's read; a
deviation found while implementing Steps 5–6 is recorded in `learnings.md`
with its reason and does not touch the bars or the follow-up rule.

### The frozen winner

**`LR/T`**, as Step 3's rule chose it ("The selection rule and the stop rule
applied"): a standardized L2 logistic (`searchfit.lr("LR-L2", C)`) on the ten
`T` columns, target `discard`, score `−decision_function` (higher keeps).
Nothing else is read on the fresh labels as a candidate.

- **Refit once on all 27 dump folders** with the Step 2 recipe: the 1,200
  labeled frames of both rounds, each frame's own round's `1/π` clipped at 5
  × that round's median sampled `1/π` and rescaled to mean 1 within the round
  (over its 600 rows); the NaN of `T` mean-filled and every column
  standardized on the 32,459 dump frames; `C` chosen by `discardfit.py`'s
  inner `GroupKFold` (5 folds over the 27 folders, `gate.CS`, the weighted
  discard AUC). The `C = 1.0` fallback ("no `C` wins more than half the
  folds") is a rule over the 27 leave-one-folder-out fits, so it does not
  apply to the one refit: the inner choice is used as it comes (the LOFO fits
  chose 0.1 in 15 of 27 folds, no fallback). The refit's mean fills,
  standardization and coefficients are applied unchanged to every fresh
  frame. `freshsample.py` prints the chosen `C` and the coefficients.
- **The `T` columns without `folder_gate.extend`.** `extend` opens the app's
  `prior_scores.tsv`, which the fresh dump does not have; it only appends
  columns after `gate.build`'s ten `tech` columns, so `T` is built from
  `gate.build`'s `tech`, `rf.load_ear` and the per-face dump alone, on the
  dump and on the fresh folders alike. Before scoring a fresh frame,
  `freshsample.py` checks on the 27-folder dump that this path gives the same
  `T` as Step 2's (`extend` called, the dump's stored `eye_focus`) on every
  frame, and that one leave-one-folder-out fold of the `LR/T` recipe on it
  (Step 2's spot folder, the largest) reproduces that folder's `LR/T` column
  of `scores_model.tsv`.
- **`eye_focus` on the fresh frames is the per-face dump's `af` value.**
  Today's `riffle-cli features` prints `eye_focus` to 4 decimals (the
  rank-app dump's TSVs carry full precision); the per-face dump's `af` face
  value is full precision and equals `scan::extract_analysis` (`--check`, 0
  mismatches). So on the fresh folders `ef` (the reference, missing last),
  the fill of `ef-fill` and `T`'s first column all take the `af` value, and
  `freshsample.py` checks that it is missing on exactly the frames the
  features TSV leaves missing and within 5.1e-05 of it elsewhere (Step 4
  read at most 5.0e-05). On the dump the same `af` value equals the stored
  `eye_focus` (Step 3's check), so the choice changes nothing there. The
  sharpness and EAR of `T` come from the features TSV as printed (4
  decimals).

### The fresh sample

Drawn by `freshsample.py` in this plan folder (`confirmsample.py`'s design
and helpers):

- **Population**: every frame of the six fresh folders, **14,525 frames**
  (`gate.load` / `gate.build` on
  `D:\Photos\tests\2026-10-23-discard-model\dump\` with the copied
  `genre_text.tsv`); no earlier sample to exclude. The picks (1,795) and
  rejects (68) are `gate.load`'s, as Step 4 printed.
- **Strata**, every frame in exactly one, with the cuts of `bf.graded_cut`
  on the fresh folders (per folder, the bottom `floor(q · n / 100)` frames
  by the score, ties by `numpy.random.default_rng(0).random(14525)` over the
  frames in `gate.build`'s order, a missing score last):
  - `b05`: the winner's bottom 5%; `b10`: its bottom 10% minus `b05`;
    `b20`: its bottom 20% minus its bottom 10%;
  - `efonly`: `ef`'s bottom 20% minus the winner's bottom 20%, so that
    "beats `ef`" is read on a sample holding both cuts' frames;
  - `control`: the rest, drawn **per folder** with `bs.allocate`'s
    proportional allocation and a **floor of 3** (or the whole cell).
- **Allocation**: **600 frames, 120 per stratum** (`b05`, `b10`, `b20`,
  `efonly`, `control`); each stratum (each `control` folder cell) a simple
  random sample without replacement; `π` the drawn count over the cell's
  population, written to `strata.tsv`.
- **Draw seed**: `numpy.random.default_rng(20261023)`, one generator for the
  draws in the order `b05`, `b10`, `b20`, `efonly`, then the `control`
  cells by folder name, then the shuffle of the 600.
- **Files** in `D:\Photos\tests\2026-10-23-discard-model\`:
  `scores_fresh.tsv` (`folder`, `file`, `LR/T`, `ef`, `ef-fill` on every
  fresh frame, full `repr` precision, `nan` where missing), `manifest.tsv`,
  `strata.tsv`, and the shuffled RAW copies named `<nnnn>_<folder>_<file>`
  in `sample\`, with no sidecar; the script refuses to write under
  `D:\Photos\2026`, checks every copy against its source by size and
  SHA-256, and compares the six library folders' size / mtime listing before
  and after the copy.
- **Labeling**: by [labeling.md](labeling.md), the model-search guide with
  the new path and folder count: the same "OK to discard?" question, rating
  1 = keep, unmarked = discard, `aiEnabled` off, never Pick / Reject, every
  frame viewed once. Nothing reads the sample's sidecars before the user
  says "done".

**As drawn** (added after the draw; label-free): `python -I -B
freshsample.py D:\Photos\tests\2026-10-12-rank-app
D:\Photos\tests\2026-10-23-face-choice
D:\Photos\tests\2026-10-18-discard-labels
D:\Photos\tests\2026-10-20-model-search
D:\Photos\tests\2026-10-23-discard-model\dump
D:\Photos\tests\2026-10-23-discard-model`, run first with `--no-copy` and
then with the copy (the second run's `manifest.tsv`, `strata.tsv` and
`scores_fresh.tsv` byte-identical to the first's), its stdout saved as
`freshsample.out.md` in the data folder. Every check passed: on the dump the
`T` columns built without `extend` equal Step 2's on all 32,459 frames, and
the spot fold (`2026-09-27-a`, `C` 0.1) reproduces its `LR/T` column of
`scores_model.tsv` exactly; the fresh `af` value is missing on exactly the
frames the features TSV leaves missing and within 5.0e-05 of it elsewhere.
The refit chose **`C` = 0.1**. The population is 14,525 frames and the sum
of `1/π` over the 600 drawn frames equals it.

| Stratum | Population | Drawn | `π` |
| --- | ---: | ---: | ---: |
| `b05` | 723 | 120 | 0.165975 |
| `b10` | 727 | 120 | 0.165062 |
| `b20` | 1,455 | 120 | 0.0824742 |
| `efonly` | 667 | 120 | 0.17991 |
| `control` (6 folder cells) | 10,953 | 120 | 0.0109559 overall; 0.0107794–0.0113544 per cell |

The `control` cells draw 14 to 26 frames. `efonly` holds 185 / 122 / 360
frames of `ef`'s `b05` / `b10` / `b20`; the winner's and `ef`'s bottom 20%
share 2,238 of their 2,905 frames each. Label-free, on the fresh folders
(1,795 picks):

| Score | q5 removed / `PL` | q10 removed / `PL` | q20 removed / `PL` |
| --- | ---: | ---: | ---: |
| `LR/T` (winner) | 4.98% / **2.56%** | 9.98% / **5.91%** | 20.00% / 14.48% |
| `ef` | 4.98% / 4.07% | 9.98% / 7.30% | 20.00% / 14.99% |
| `ef-fill` | 4.98% / 2.84% | 9.98% / 5.96% | 20.00% / 13.09% |
| random | 4.98% / 4.46% | 9.98% / 10.25% | 20.00% / 20.33% |

So bars 2 and 3 hold for the winner at **q5** and **bar 2 fails at q10**
(`PL` 5.91% > 5%) whatever the fresh labels say: only q5 can pass. The
sample holds 56 of the fresh folders' picks and 9 explicit rejects. The
copies (26.1 GB): 600 files in `sample\`, 0 whose size or SHA-256 differs
from the source, 0 `.xmp` / `.dop`, 0 files not in the manifest, no sidecar
opened under `sample\`, and the 26,410 library files of the six folders
unchanged (size and mtime).

### The confirmation read (Step 6)

- **Labels**: `freshlabels.py` (`confirmlabels.py`'s recipe with the five
  strata `b05`, `b10`, `b20`, `efonly`, `control`: rating 1 = `keep`,
  reviewed and unmarked = `discard`, unexpected marks listed, not dropped).
  **`discard` is the positive class.** Every estimate Horvitz–Thompson
  weighted by the fresh `1/π`, the unweighted value beside it. No label of
  rounds 1 and 2 is read.
- **Scores read once on the fresh labels**: the winner (`LR/T`, the refit
  above), `ef` (the `af` value, missing last; in the AUC as
  `bf.finite_by_folder`'s folder minimum minus 1), `ef-fill` and random
  (the seed-0 draw), each at q5 / q10 / q20 with `bf.graded_cut` on the
  fresh folders, and the weighted discard AUC. Metrics as discard.md's
  (`DP`, `RR`, `PL` on the fresh folders' picks, the removed share of the
  fresh population).
- **Intervals**: a 1,000-draw **stratified bootstrap** within `b05`, `b10`,
  `b20`, `efonly` and each (`control`, folder) cell of the fresh
  `strata.tsv`, seed 0, **one count matrix for every score and quantity**
  (paired differences share draws); the 2.5% / 97.5% quantiles. A
  **folder-cluster bootstrap** (the six folders with replacement, seed 0,
  1,000 draws) beside it; it does not decide, and a pass whose cluster
  interval crosses its bar is reported as **fragile**. Six clusters make it
  coarse.
- **Base rate**: the weighted share of `discard` over the fresh labels, its
  point estimate the bar's value, not resampled; its interval printed
  beside.

### Bars

"Sorted last" semantics: the cut's frames are suggested for discard and
stay recoverable. A cut of the winner at **q5 or q10** is **usable** when
all of these hold on the fresh labels:

1. weighted `RR` <= **25%**, and its stratified 97.5% quantile <= **35%**;
2. `PL` <= **5%** (the fresh folders' picks, label-free);
3. it removes **>= 4.5%** of the fresh population (label-free);
4. **beats the base rate**: `DP`'s stratified 2.5% quantile is above the
   fresh base rate (point estimate);
5. **beats `ef`**: the weighted discard AUC minus `ef`'s, paired,
   stratified **2.5% quantile above zero** (the key bar; one value for both
   cuts).

**q20** is read and reported, not eligible. **Best cut**: the deepest
passing one of q5 and q10. The winner **passes** when at least one of q5 /
q10 is usable. There is no "beats `G0`" condition: `G0` is not the
reference any more and is not computed on the fresh folders. Conditions 2
and 3 are label-free; `freshsample.py` prints them before any label exists.

### Follow-up rule

Applied in this order; the Decision (Step 7) names the outcome:

1. **Pass** → an **in-app suggestion plan**: discard candidates sorted last
   at the best cut's depth, with a rescue loop.
2. **Close** → **more labels** (another fresh round, about 1.5 s per
   frame): at q5 or q10 the `RR` point is <= 25% with conditions 2 and 3
   holding and only quantile or "beats" conditions failing; **or** the AUC
   − `ef` point is above zero with the stratified interval spanning zero.
3. **Otherwise** → **nothing, `eye_focus` stays** as the cue.

### Multiplicity and population

- **Multiplicity**: one frozen winner, read once, on two eligible cuts with
  five conditions each; `ef`, `ef-fill` and random are references. The
  selection's multiplicity (5 candidates × 2 rounds × 3 cuts, plus the
  subsets, and the amended selection rule) is spent in the exploration; the
  confirmation adds no selection. The Decision reads a lone marginal pass,
  or one fragile on the cluster bootstrap, as such.
- **Selection bias the confirmation cannot remove**: the design was chosen
  after reading `eye_focus`'s and `af-whole`'s numbers on both rounds, and
  the fresh strata are cut by the winner and `ef` themselves. The fresh
  folders are new frames and new folders (other dates), not a new
  photographer.
- **Population caveat**: one photographer, mostly basketball; 600 fresh
  labels on 14,525 frames in six folders.

### The user's decisions (at planning time)

As recorded in "The user's decisions" of the exploration's
pre-registration: the fresh population of six folders, 14,525 frames
(rejected: the four post-dump folders only, 9,665 frames); the five bars
with `PL` read on the fresh folders' picks (rejected: an extra cut-level
condition, "`RR` at the cut not above `ef`'s at q5 by more than +5 points
(97.5% quantile)"); no `G0` reading on the fresh folders (rejected: adding
it as a reading).

## Results of the confirmation

Read once on 2026-10-28, after the user said "done". The labels: `python -I
-B freshlabels.py D:\Photos\tests\2026-10-23-discard-model` (stdout saved as
`freshlabels.out.md` in the data folder). The read: `python -I -B
freshread.py D:\Photos\tests\2026-10-23-discard-model\dump
D:\Photos\tests\2026-10-23-discard-model`, its stdout saved as
`freshread.out.md` in the data folder; wall time 25 s. A dry run on
synthetic labels in the scratchpad came first (see `learnings.md`). Every
interval is the fresh stratified bootstrap 95% (1,000 draws, seed 0, within
`b05`, `b10`, `b20`, `efonly` and each `control` folder cell) unless marked
cluster (the six folders, 1,000 draws, seed 0); values are weighted by the
fresh `1/π` unless marked unweighted. Differences are paired against `ef` on
the same draws (stratified, cluster in parentheses).

### Labels

600 labels: **127 keep, 473 discard**. 128 copies have both a `.dop` and an
`.xmp` (the `.dop` decides; 0 disagreements on rating 1), 472 none. **One
anomaly**, kept and not dropped: `0250_2026-10-11__DSC5031.ARW` holds rating
0 in both sidecars (a mark taken back with `0`, which the guide allows), so
it is `discard`. No Pick, Reject or color label on any copy, no rating other
than 1 and 0, no missing copy, no other file in `sample\`. The sample's 56
picks from the user's earlier culling were labeled 22 keep / 34 discard, its
9 explicit rejects all discard. The user labeled the 600 frames in under one
hour, with `aiEnabled` off throughout.

| Stratum | Population | Sampled | Keep | Discard |
| --- | ---: | ---: | ---: | ---: |
| `b05` | 723 | 120 | 10 | 110 |
| `b10` | 727 | 120 | 10 | 110 |
| `b20` | 1,455 | 120 | 19 | 101 |
| `efonly` | 667 | 120 | 31 | 89 |
| `control` | 10,953 | 120 | 57 | 63 |

### Checks

All 19 label-free checks passed before `labels.tsv` was opened, and all 6
label checks after: the 14,525 frames, 1,795 picks and 68 rejects of the
fresh dump; `scores_fresh.tsv` covers every frame, the winner finite
everywhere, its `ef` missing on exactly the frames the features TSV leaves
missing and within 5.1e-05 elsewhere; every score's `PL` and removed share
equal Step 5's; `strata.tsv` (10 rows), each sampled frame's stratum and `π`
recompute from the cuts; the sum of `1/π` is 14,525; the labels match the
manifest; the files opened in the data folder are `scores_fresh.tsv`,
`manifest.tsv`, `strata.tsv` and `labels.tsv`, and nothing was opened under
`sample\`.

### Base rate

| Frames | Discardable, weighted % | Stratified 95% | Cluster 95% | Unweighted % |
| --- | ---: | ---: | ---: | ---: |
| all | **60.56** | [53.81, 67.29] | [53.52, 66.79] | 78.83 |
| missing-face (201 labels, 30 keep) | 86.5 | [81.7, 90.7] | [82.5, 89.7] | 85.1 |
| complement (399 labels) | 58.0 | [50.5, 65.2] | [50.4, 64.4] | 75.7 |

### Weighted discard AUC

| Score | AUC [stratified 95%] | Cluster | Unweighted | AUC - `ef` |
| --- | ---: | ---: | ---: | ---: |
| `LR/T` (winner) | 0.729 [0.669, 0.788] | [0.666, 0.782] | 0.764 | **-0.013 [-0.048, +0.023]** ([-0.044, +0.015]) |
| `ef` (reference) | 0.743 [0.679, 0.800] | [0.676, 0.796] | 0.676 |  |
| `ef-fill` (reference) | 0.744 [0.681, 0.803] | [0.672, 0.804] | 0.745 | +0.002 [-0.008, +0.012] ([-0.007, +0.012]) |
| random (reference) | 0.521 [0.442, 0.606] | [0.463, 0.581] | 0.475 | -0.222 [-0.322, -0.115] ([-0.317, -0.125]) |

`LR/T` − `ef-fill`: −0.015 [−0.053, +0.022] ([−0.048, +0.019]).

The missing-face subset and its complement, a reading beside the
pre-registered read (as Step 3's; it decides nothing):

| Score | Missing-face AUC | AUC - `ef` | Complement AUC | AUC - `ef` |
| --- | ---: | ---: | ---: | ---: |
| `LR/T` (winner) | 0.706 [0.597, 0.812] | +0.190 [+0.035, +0.327] ([+0.133, +0.300]) | 0.708 [0.642, 0.773] | -0.021 [-0.062, +0.021] ([-0.056, +0.017]) |
| `ef` (reference) | 0.517 [0.404, 0.643] |  | 0.729 [0.660, 0.795] |  |
| `ef-fill` (reference) | 0.793 [0.698, 0.880] | +0.277 [+0.131, +0.407] ([+0.210, +0.410]) | 0.729 [0.660, 0.795] | +0.000 (by construction) |
| random (reference) | 0.459 [0.348, 0.572] | -0.058 [-0.235, +0.120] | 0.528 [0.445, 0.618] | -0.201 [-0.311, -0.077] |

### The cuts

`PL` is on the fresh folders' 1,795 picks, removed the share of the 14,525
fresh frames (both label-free); `RR` − `ef` in points.

| Score | Cut | Sampled removed / discard | `DP` % | `RR` % | `RR` cluster | `PL` % | Removed % | `RR` - `ef` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `LR/T` (winner) | q5 | 120 / 110 | 91.7 [86.7, 96.7] (91.7) | 8.3 [3.3, 13.3] (8.3) | [4.3, 12.8] | 2.56 | 4.98 | -3.0 [-9.2, +3.1] ([-8.3, +1.2]) |
| `LR/T` (winner) | q10 | 240 / 220 | 91.7 [88.3, 94.6] (91.7) | 8.3 [5.4, 11.7] (8.3) | [6.5, 10.3] | 5.91 | 9.98 | -6.2 [-10.7, -2.2] ([-10.7, -2.8]) |
| `LR/T` (winner) | q20 | 360 / 321 | 87.9 [84.2, 91.7] (89.2) | 12.1 [8.3, 15.8] (10.8) | [9.6, 16.4] | 14.48 | 20.00 | -2.8 [-5.6, +0.2] ([-6.2, +0.9]) |
| `ef` (reference) | q5 | 116 / 101 | 88.7 [82.6, 93.7] (87.1) | 11.3 [6.3, 17.4] (12.9) | [7.1, 15.9] | 4.07 | 4.98 |  |
| `ef` (reference) | q10 | 220 / 186 | 85.4 [80.6, 89.8] (84.5) | 14.6 [10.2, 19.4] (15.5) | [12.2, 17.9] | 7.30 | 9.98 |  |
| `ef` (reference) | q20 | 412 / 351 | 85.2 [81.5, 88.5] (85.2) | 14.8 [11.5, 18.5] (14.8) | [12.2, 17.3] | 14.99 | 20.00 |  |
| `ef-fill` (reference) | q5 | 111 / 101 | 90.0 [83.7, 95.7] (91.0) | 10.0 [4.3, 16.3] (9.0) | [3.6, 17.6] | 2.84 | 4.98 | -1.3 [-9.1, +6.3] ([-7.8, +4.2]) |
| `ef-fill` (reference) | q10 | 208 / 184 | 86.9 [81.5, 91.8] (88.5) | 13.1 [8.2, 18.5] (11.5) | [8.7, 16.9] | 5.96 | 9.98 | -1.5 [-7.0, +4.2] ([-6.3, +2.4]) |
| `ef-fill` (reference) | q20 | 354 / 316 | 89.4 [85.3, 92.8] (89.3) | 10.6 [7.2, 14.7] (10.7) | [9.3, 12.6] | 13.09 | 20.00 | -4.3 [-6.5, -2.0] ([-6.7, -1.9]) |
| random (reference) | q5 | 30 / 21 | 45.9 [23.0, 71.5] (70.0) | 54.1 [28.5, 77.0] (30.0) | [21.9, 82.0] | 4.46 | 4.98 | +42.8 [+17.5, +66.3] ([+10.3, +71.0]) |
| random (reference) | q10 | 58 / 45 | 59.2 [40.6, 77.2] (77.6) | 40.8 [22.8, 59.4] (22.4) | [20.6, 58.4] | 10.25 | 9.98 | +26.2 [+8.1, +45.4] ([+6.6, +44.8]) |
| random (reference) | q20 | 124 / 95 | 63.8 [49.8, 78.3] (76.6) | 36.2 [21.7, 50.2] (23.4) | [27.8, 47.4] | 20.33 | 20.00 | +21.4 [+7.1, +36.2] ([+13.2, +32.7]) |

Label-free, `LR/T`'s q5 cut holds 468 of its 723 frames without
`eye_focus` (64.7%) and 46 picks (`ef`'s: 723 of 723, 73 picks).

### The bars applied

| Cut | (1) `RR` <= 25%, 97.5% <= 35% | (2) `PL` <= 5% | (3) Removed >= 4.5% | (4) `DP` 2.5% > base 60.56% | (5) AUC - `ef` 2.5% > 0 | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| q5 | 8.3, 97.5% 13.3 (cluster 12.8): yes | 2.56: yes | 4.98: yes | 86.7 (cluster 87.2): yes | -0.048 (cluster -0.044): **no** | not usable |
| q10 | 8.3, 97.5% 11.7 (cluster 10.3): yes | 5.91: **no** | 9.98: yes | 88.3 (cluster 89.7): yes | -0.048 (cluster -0.044): **no** | not usable |
| q20 | 12.1, 97.5% 15.8: yes | 14.48: no | 20.00: yes | 84.2: yes | -0.048: no | not eligible |

**Verdict: `LR/T` does not pass** (no usable cut at q5 or q10). Bar 5, the
key bar, fails on both cuts (the AUC − `ef` point is negative), and bar 2
fails at q10 as Step 5 printed before any label.

**Follow-up rule, applied mechanically: (2) close → more labels.** The first
"close" clause holds at q5: the `RR` point 8.3% is <= 25% with conditions 2
(`PL` 2.56%) and 3 (removed 4.98%) holding, and the only failing condition
is "beats `ef`". (At q10 condition 2 fails, so that cut does not count; the
second clause, the AUC − `ef` point above zero with the interval spanning
zero, does not hold: the point is −0.013.) Step 7 names the outcome.

### Exploration and confirmation

| Read | `LR/T` AUC | `ef` AUC | AUC - `ef` | `LR/T` `RR` % q5 | `LR/T` `RR` % q10 | `ef` `RR` % q5 / q10 | Base rate % |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Round 2 (exploration, primary) | 0.812 [0.757, 0.863] | 0.785 [0.723, 0.844] | +0.027 [-0.020, +0.078] | 12.8 [6.3, 24.7] | 15.3 [9.1, 24.6] | 5.5 / 12.9 | 42.31 |
| Round 1 (exploration, replication) | 0.795 [0.737, 0.851] | 0.777 [0.716, 0.841] | +0.018 [-0.018, +0.053] | 9.5 [3.0, 19.6] | 21.7 [5.3, 39.1] | 16.8 / 14.3 | 48.45 |
| Fresh (confirmation) | 0.729 [0.669, 0.788] | 0.743 [0.679, 0.800] | -0.013 [-0.048, +0.023] | 8.3 [3.3, 13.3] | 8.3 [5.4, 11.7] | 11.3 / 14.6 | 60.56 |

The gain over `ef` shrank from +0.027 (round 2) and +0.018 (round 1) to
−0.013 on the fresh folders, a change of −0.040 and −0.031.

### Reading

The facts, without a decision (Step 7 decides):

- **The winner does not beat `ef` on the fresh folders**: AUC 0.729 against
  `ef`'s 0.743, −0.013 [−0.048, +0.023] (cluster [−0.044, +0.015]). The
  exploration gains (+0.027, +0.018, both intervals spanning zero) did not
  replicate; the point changed sign. `ef-fill` is level with `ef` (+0.002
  [−0.008, +0.012]), and `LR/T` is below `ef-fill` (−0.015 [−0.053,
  +0.022]).
- **Where it sits**: on the missing-face frames (201 labels, 86.5%
  discardable) the winner orders better than `ef`, whose subset AUC (0.517)
  orders folders only: +0.190 [+0.035, +0.327]; but the label-free fill
  does more there (`ef-fill` +0.277 [+0.131, +0.407]). On the complement,
  where `eye_focus` exists, the winner is below `ef`: −0.021 [−0.062,
  +0.021]. So the learned columns lost on the frames that have an
  `eye_focus`, where the exploration put its gain (+0.034 and +0.018 there).
  The all-frames AUC also counts the pairs across the two subsets.
- **The cuts themselves are clean**: at q5 the winner's `RR` is 8.3% [3.3,
  13.3] against `ef`'s 11.3% (−3.0 [−9.2, +3.1]), at q10 8.3% [5.4, 11.7]
  against 14.6% (−6.2 [−10.7, −2.2]); its `DP` lower quantiles (86.7%,
  88.3%) sit far above the base rate, and its q5 `PL` is 2.56% against
  `ef`'s 4.07%. These bars (1, 3, 4, and 2 at q5) hold; `ef`'s own q5 cut
  would meet the same `RR`, `PL` and removed conditions (11.3% [6.3, 17.4],
  4.07%, 4.98%). The "close" verdict comes from that first clause, which
  does not compare with `ef`.
- **The fresh base rate is higher**: 60.56% [53.81, 67.29] against 42.31%
  (round 2) and 48.45% (round 1). Part of it may be the new folders, part
  the reworded question (Step 5 deviation, `learnings.md`): the fresh labels
  were taken under "sorted last", not the earlier rounds' "hid", which is the
  milder outcome and may call more frames OK to discard. This cannot be
  separated here. It bears on the comparison of `RR`, `DP` and the base rate
  with rounds 1 and 2; the AUC − `ef` comparison is within the fresh labels,
  so the wording affects both scores alike, though it may change which
  frames count as discardable.
- Both exploration rounds had been read before the design was chosen, the
  selection rule's cut was amended after Step 2's label-free readings, and
  the fresh strata are cut by the winner and `ef` themselves; the fresh
  read adds six folders (six cluster units, a coarse cluster bootstrap) of
  the same photographer, mostly basketball. The labeling took under one
  hour for 600 frames with `aiEnabled` off; one frame carried a rating taken
  back to 0, read as discard.
