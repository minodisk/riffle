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
  as well (`ef` `PL` 4.44%, random 4.71%), and the removed share is 4.96%
  for every score, because `floor(q * n / 100)` fixes it: at q5 the
  condition cannot fail for any candidate, nor for random. The amendment
  therefore drops the label-free eligibility filter in effect, and the
  selection reduces to the highest round-2 AUC.
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
