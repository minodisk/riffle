# Face choice: does the face `eye_focus` judges hurt its discard prediction?

## Pre-registration

Written 2026-10-23, **before `riffle-cli facechoice` exists and before any
per-face `eye_focus` has been computed or read**: when this section was
written no per-face value of any dump frame existed anywhere (the index and
the dump store the judged face's values only), no script of this plan
existed, and `D:\Photos\tests\2026-10-23-face-choice\` held nothing of this
plan. It is merged on its own, before Step 2's subcommand or Step 3's
`facechoiceread.py` exists; the merge commit of the PR that adds this
section, without results, is the record, as for the archived
[model-search confirmation](../_archived/20261020-model-search/modelsearch.md)
and [discard filter evaluation](../_archived/20261018-discard-labels/discard.md)
before it. Nothing in this section changes after a per-face value exists; a
deviation found while implementing Steps 2–3 is recorded in `learnings.md`
with its reason and does not touch the reading rule.

The question, the user's hypothesis: today `eye_focus` judges the face
nearest the AF point; would the discard prediction change if it judged the
largest face, or the sharpest, because the face it judges is not always the
one a human looks at? On the two rounds of blind "OK to discard?" labels,
does any of a small fixed set of face-choice rules give a better discard
signal than today's?

### What was known before this section

This pre-registration is **not blind** to the stored `eye_focus` readings.
On the 27-folder dump (32,459 frames):

- On round 2's fresh labels (the model-search confirmation), the plain
  `eye_focus` (missing last) was the strongest discard signal read there:
  weighted AUC **0.785** [0.723, 0.844], against 0.684 for the learned
  winner `G0 ⊕ ef` and 0.630 for `G0`
  ([modelsearch.md, "Results of the confirmation"](../_archived/20261020-model-search/modelsearch.md)).
  That number motivated this plan.
- On round 1's labels (discard-labels), `eye_focus` read AUC 0.777 [0.716,
  0.841] ([discard.md, "Main table"](../_archived/20261018-discard-labels/discard.md)).
- Round 2 was a **confirmation sample**: its strata were cut by `G0 ⊕ ef`
  and `G0`, and its labels have been read once for that confirmation. This
  plan reuses both rounds for selection, so any winner here is selected on
  reused labels (see "What happens after").
- Nothing is known about any per-face value: how many dump frames hold two
  or more faces, how often the AF face is not the largest, or how the other
  faces' `eye_focus` relate to the labels.

### Today's rule, exactly

From the code on `main` at `bf519beb` (the face-detection-values change,
which added `nose`, `mouth_left` and `mouth_right` to `faces::Face`, is
merged; this plan edits neither `faces.rs` nor `candidate.rs`). The cue is
`scan::extract_analysis` in `crates/core/src/scan.rs`, which branches on
`sharpness::trusted_focus(&shot)` (the AF point, `None` when there is none
or the frame was shot in manual focus):

- **AF point present**: `scan::cue` -> `candidate::focus_cue_unless`
  decodes the full-size preview (`decode::decode_rgb`), then
  `faces::detect_around_rgb` turns it upright, cuts a **`faces::CATCH_CROP`
  (480 px) square** around the AF point (`partial::focus_point`, mapped
  upright), runs YuNet at the `faces::INPUT` (320 px) square input on that
  crop, drops faces below `faces::CATCH_CONFIDENCE` (=
  `faces::SCORE_THRESHOLD`, 0.6) and maps them to stored coordinates. The
  judged face is `candidate::nearest_face(faces, point)`: the face whose
  nearer eye is closest to the AF point, the distance divided by the box's
  long side (ties to the earlier face, `min_by`). **A face outside the
  480 px square is never seen**, and an empty crop gives an `Unknown` cue
  (no `eye_focus`); there is no fallback to the whole image.
- **AF point absent**: `scan::whole_image_detection` runs
  `faces::detect_around(preview, orientation, None)`, that is
  `faces::decode_whole` (the upright decode at the smallest `n/8` scale whose
  long edge is not below 640) and `faces::detect_whole_upright` (YuNet at
  `faces::WHOLE_INPUT` 640x448, or 448x640 for a portrait image, faces below
  0.6 dropped, mapped to stored coordinates). `scan::whole_image_cue` judges
  `eyes::judged_face(faces, None)`: the **largest box area** among the faces
  scoring at least `sharpness::FACE_CONFIDENCE` (0.8) (ties to the later
  face, `max_by`), then decodes the full-size preview once for it. No face at
  or above 0.8 gives an `Unknown` cue.
- **Scoring the judged face**, both cases: `candidate::face_cue_unless`
  takes `candidate::luma` of the full-size decode; when
  `candidate::meshes_face` accepts the face (box long side at least
  `eyes::EYES_MIN_FACE`, 60 px) `eyes::mesh_of` runs the face mesh, then
  `candidate::eye_focus(gray, w, h, face, mesh)` scores the sharper of the
  two eyelid regions that count, else the eye window between the eyes. The
  stored `eye_focus` is that `EyeFocus::probability`; the index stores the
  judged face's values only (`eye_focus`, `eyes_ear`, `pose_*`).
- **`sharpness::chosen_face`** (the highest-scoring face at or above 0.8) is
  the sharpness window's fallback without an AF point only; it is **not**
  the cue's face.

### Data and labels

- **Dump**: `D:\Photos\tests\2026-10-12-rank-app\` (`features\<folder>.tsv`
  from `riffle-cli features`, 27 folders, 32,459 frames), loaded by the
  archived harness (`badfilter.load`). The library `D:\Photos\2026` is
  read-only; Step 2 only reads it.
- **Round 1** (discard-labels): `D:\Photos\tests\2026-10-18-discard-labels\`,
  600 labels (211 keep, 389 discard), population the dump minus the 1,200
  strict-sample frames, **31,259 frames**; strata `b05`, `b10`, `b20` and
  `control` per folder (30 cells of `strata.tsv`); `π` from `manifest.tsv`.
- **Round 2** (model-search confirmation):
  `D:\Photos\tests\2026-10-20-model-search\`, 600 labels (248 keep, 352
  discard), population the dump minus the 1,800 earlier-labeled frames,
  **30,659 frames**; strata `b05`, `b10`, `b20`, `g0only` and `control` per
  folder (31 cells of `strata.tsv`), as `confirmread.py` builds them.
- **Label**: rating 1 = `keep`, reviewed and unmarked = `discard`, as each
  round's `labels.tsv` holds it. **`discard` is the positive class.**

### The two face sets

Recomputed per frame by Step 2's `riffle-cli facechoice` from the embedded
preview, through the same public functions the scan calls (`reader::read_preview`,
`trusted_focus`, one full-size `decode_rgb`), every face kept with its score
(the floors below are applied by Step 3):

- **crop**: `detect_around_rgb` around the AF point, exactly today's
  detection (floor 0.6 inside the detector); **empty without an AF point**.
- **whole**: `decode_whole` + `detect_whole_upright`, exactly today's no-AF
  detection (floor 0.6 inside the detector), run on **every** frame, AF
  point or not.

Every face of both sets is scored as today's judged face is (`meshes_face`
-> `mesh_of` -> `eye_focus` on the one full-size `luma`). The same person
found in both sets has two boxes and so possibly two `eye_focus` values (the
eye window and the mesh crop follow the box).

### The six rules

Each rule gives one `eye_focus` per frame, or **missing** when it picks no
face (or the preview could not be read or decoded). **The confidence floor
of the whole-set rules is 0.8** (`sharpness::FACE_CONFIDENCE`, today's no-AF
floor), applied to `largest`, `sharpest`, `score` and `mean` on every frame
and to the no-AF fallback; `af` and `af-whole` choose the AF face over their
set at the detector's 0.6, as `nearest_face` does today.

1. **`af`** (today, the reference): with an AF point, `nearest_face` over
   the crop set (missing when the crop set is empty); without one, the
   largest whole face at or above 0.8. **It must reproduce the stored
   `eye_focus`** (below).
2. **`af-whole`** (control): with an AF point, `nearest_face` over the whole
   set (0.6, the same floor as the crop set, so only the detection changes;
   the distance is measured from the crop detection's stored `focus_point`
   (`partial::focus_point(width, height, focus)`, what `nearest_face` uses
   today and the `af_x` / `af_y` Step 2 records), since the whole set has
   no point of its own; missing when the whole set is empty); without one, `af`'s fallback.
   It separates "which detection" from "which face".
3. **`largest`**: the largest box area over the whole set's faces at or
   above 0.8. On a frame without an AF point it equals `af`.
4. **`sharpest`**: the maximum `eye_focus` over the whole set's faces at or
   above 0.8.
5. **`score`**: the face with the highest YuNet score over the whole set,
   when that score is at least 0.8 (the face `chosen_face` would take).
6. **`mean`**: the mean `eye_focus` over the whole set's faces at or above
   0.8, the one aggregate.

Ties: a maximum goes to the later face in detection order (`max_by`, as
`judged_face` and `chosen_face`), `nearest_face`'s minimum to the earlier.
**Nothing else is searched**: no other floor, set, rule or combination is
read. Since `largest`, `sharpest` and `score` run on the whole-image
640x448 detection, which can miss small faces the 480 px crop finds at its
320 px input and vice versa, their comparison of interest is against
`af-whole` as well as against `af`; the reading rule still compares with
`af`.

**Missing last**, as the stored `eye_focus` was read in both rounds
(`fw.EYE`, `confirmread.py`'s `ef_last`): a missing value is the lowest
score, `-inf` in the per-folder cut (removed before any finite value, ties
by the seed-0 draw), and in the AUC the folder's lowest finite score minus 1
(`bf.finite_by_folder`).

### The two user decisions (at planning time)

- **Cut source: option A.** The per-face dump is recomputed on all 27 dump
  folders (32,459 frames), so each rule's per-folder bottom-q cut and `PL`
  are computed on the full dump exactly as discard.md's. Rejected: computing
  only the 1,200 labeled frames, with sample-weighted quantiles and a `PL`
  resting on about 86 sampled picks.
- **Label read: per round, round 2 primary.** Each round keeps its own
  population, `π` and strata as its harness defined them; round 2 is the
  primary read and round 1 the replication. Rejected: pooling the 1,200
  frames with round x stratum cells and approximate weights.

### What is measured

Vocabulary and formulas as discard.md's "Primary metrics": `DP` (discard
precision), `RR` = 100 − `DP` (rescue rate), `PL` (pick loss, the share of
the dump's 4,419 picks inside the cut, on the full dump, label-free),
`DR` / `MR`, `DO`, the **removed share** of the round's population
(label-free, exact), and the **weighted discard AUC**
`roc_auc_score(discard, -score, sample_weight=1/π)`. Every labeled estimate
is Horvitz–Thompson weighted by the round's `1/π`, the unweighted value
beside it.

- **Cuts**: per folder on the full dump, the bottom `floor(q · n / 100)` by
  the rule's `eye_focus` (`bf.graded_cut`, missing last, ties by the seed-0
  draw), **q = 5 / 10** (q20 as a reading).
- **Intervals**: a 1,000-draw **stratified bootstrap** per round within its
  own cells (round 1: `b05`, `b10`, `b20`, each `control` folder cell;
  round 2: `b05`, `b10`, `b20`, `g0only`, each `control` folder cell), seed
  0, **one count matrix per round shared by every rule and quantity**, so the
  differences against `af` are paired; the 2.5% / 97.5% quantiles. A
  **folder-cluster bootstrap** (the 27 folders with replacement, seed 0,
  1,000 draws) beside it; it does not decide.
- **Base rate**: each round's weighted discardable share, as its harness
  printed it.

In this order:

1. **Face counts first**: the share of frames with 0 / 1 / 2+ faces in the
   whole set at the 0.8 floor, and in the crop set (its 0.6 floor) among AF
   frames; weighted by `1/π` and unweighted per round, and on the whole
   dump (label-free); with the share of frames each rule leaves missing. If
   2+ is rare, the rules can move little, and the text says so before any
   score.
2. **Per rule, per round**: weighted AUC with stratified and cluster
   intervals; `DP` / `RR` / `PL` / removed share at q5 / q10 (q20 as a
   reading); the **paired** differences against `af` of the AUC and of
   `RR` at each cut, stratified and cluster. Overall, and restricted to the
   **multi-face frames** (2+ whole faces at or above 0.8; the cut stays the
   full-dump cut, only the labeled frames read are restricted).
3. **The disagreement test**: on the labeled frames where both `af` and
   `largest` pick a face and their boxes (stored coordinates) have **IoU
   < 0.5**, the subset size, how often the `af` face's box is smaller than
   the `largest` face's, each rule's weighted AUC on that subset with the
   paired difference against `af`, and the cross-tab of the label by "which
   rule's face has the lower `eye_focus`" (`af`'s, `largest`'s, equal).
   Frames where exactly one of the two picks a face are counted and left out
   of the subset. The same for `af` vs `sharpest`.

### Reproduction checks

Printed first by Step 3, and the ones that need no label before any
`labels.tsv` is opened; a failure stops the read there and is recorded in
`learnings.md`:

- `af` reproduces the stored `eye_focus` of all 1,200 labeled frames (the
  two `manifest.tsv`'s `eye_focus` columns, printed to 6 significant
  digits): |difference| <= 1e-6 where both are present, and missing exactly
  where the stored value is `nan`. Step 2 also checks the **`af` rule's**
  value against `scan::extract_analysis(path).cue.eye_focus` on every file
  (exact or within 1e-9): the crop set's `nearest` face with an AF point,
  and the whole set's largest face at or above 0.8 without one (the crop
  set is empty there, while `extract_analysis` still scores the whole-image
  fallback).
- Both rounds' stored `eye_focus` reference rows and base rates below
  recompute from the stored column (and from `af`).

**Known numbers to reproduce**:

| Round | Reading | Value | Source |
| --- | --- | ---: | --- |
| 2 | Discardable base rate | 42.31% [35.85, 48.91] (cluster [33.44, 51.61]) | [modelsearch.md, "The fresh labels"](../_archived/20261020-model-search/modelsearch.md); `confirmread.out.md` |
| 2 | `eye_focus` (missing last) AUC | 0.785 [0.723, 0.844] (cluster [0.693, 0.881]) | modelsearch.md, "Discard: the winner on the fresh labels" |
| 2 | `eye_focus` `RR` q5 / q10 / q20 | 5.5 [1.1, 14.5] / 12.9 [7.2, 21.8] / 12.8 [8.9, 17.9] % | same table |
| 2 | `eye_focus` `PL` q5 / q10 / q20 (full dump) | 4.44 / 9.14 / 15.59 % | same table |
| 2 | `eye_focus` removed q5 / q10 / q20 (round 2 population) | 4.80 / 9.70 / 19.64 % | same table |
| 1 | Discardable base rate | 48.45% [42.03, 55.11] (cluster [40.74, 53.87]) | [discard.md, "Discardable base rate"](../_archived/20261018-discard-labels/discard.md); `discardfilter.out.md` |
| 1 | `eye_focus` (missing last) AUC | 0.777 [0.716, 0.841] (unweighted 0.803) | discard.md, "Main table" |
| 1 | `eye_focus` `RR` q5 / q10 / q20 | 16.8 [1.2, 43.3] / 14.3 [2.9, 29.9] / 28.3 [15.7, 40.2] % | same table |
| 1 | `eye_focus` `PL` q5 / q10 / q20 (full dump) | 4.44 / 9.14 / 15.59 % | same table |
| 1 | `eye_focus` removed q5 / q10 / q20 (round 1 population) | 4.83 / 9.78 / 19.87 % | same table |

The `PL` agree between the rounds because both are the same full-dump cut.

### Reading rule

One mechanical rule, read on **round 2, all labeled frames**:

- A rule **beats today** when both hold:
  1. the paired AUC difference against `af` has its stratified **2.5%
     quantile above zero**;
  2. its `RR` at **q10** is not worse: the paired stratified **97.5%
     quantile** of `RR` minus `af`'s `RR` at q10 is **<= +5 points**.
- The **winner** is the beating rule with the highest round-2 AUC (a tie to
  the earlier rule in the list above); none beating is **"no winner"**.
  `af-whole` may win; that would say the detection, not the face, matters.
- A beat whose folder-cluster interval of the AUC difference reaches zero is
  reported as **fragile**; it still beats.
- Round 1, the q5 and q20 tables, the multi-face subset and the
  disagreement test are **readings that do not change the winner**.

### Multiplicity and population

- **Multiplicity**: 5 rules against `af` x 2 rounds x 3 cuts (plus the AUC,
  the multi-face subset and the disagreement test), read **without
  correction**; hence one mechanical rule on one round's all-frames AUC and
  q10 `RR`, and everything else a reading.
- **Selection on reused labels**: round 2 was a confirmation sample, already
  read once, and its 0.785 is what motivated this plan; round 1 has been
  read by three plans. A winner here is a selection, not a confirmation.
- **Population caveat**: one photographer's 27 folders, mostly basketball;
  600 labels per round. Multi-face frames may be rare there, and the AF
  crop already limits the crop set.

### What happens after

The Decision (Step 4) names the follow-up from the reading rule:

- **A winner** -> a plan for a **fresh blind confirmation round**, combined
  with more labels from folders not yet sampled (as the user agreed), before
  any app change; the shipping change is only sketched (which detection and
  which rule the scan would run, and the extra per-face mesh cost Step 2
  measures).
- **No winner** -> nothing changes in the app; the face-count and
  disagreement readings say why (too few multi-face frames, or the faces
  chosen agree, or they differ without helping).

No app behavior changes in this plan.

## Results

Run once on 2026-10-23 by `facechoiceread.py` (`python -I -B
facechoiceread.py D:\Photos\tests\2026-10-12-rank-app
D:\Photos\tests\2026-10-16-reject-filter-eval
D:\Photos\tests\2026-10-17-bad-labels
D:\Photos\tests\2026-10-18-discard-labels
D:\Photos\tests\2026-10-20-model-search
D:\Photos\tests\2026-10-23-face-choice`), its stdout saved as
`facechoiceread.out.md` in the data folder; wall time 102 s. The argument
list differs from the plan's (the strict sample's folder in place of the
unused folder-wide one; see `learnings.md`). Every interval is the round's
stratified bootstrap 95% (1,000 draws, seed 0) unless marked cluster
(folder-cluster, 1,000 draws, seed 0); values are weighted by the round's
`1/π` unless marked unweighted. Differences are paired against `af`.

### Checks

All 25 label-free checks passed before any `labels.tsv` was opened, and all
17 label checks after:

- `af` reproduces the stored `eye_focus` of all 1,200 labeled frames (0 of
  600 per round beyond 1e-6 or missing on one side only) and of every one of
  the 32,459 dump frames (0 beyond 1e-9).
- Both rounds' populations (31,259 and 30,659), `strata.tsv` (30 and 31
  cells), bands, `π` and the sums of `1/π` recompute from the dump.
- The known numbers reproduce exactly, from the stored column and from `af`
  alike: round 2's base rate 42.31% [35.85, 48.91] (cluster [33.44, 51.61]),
  `eye_focus` AUC 0.785 [0.723, 0.844] (cluster [0.693, 0.881], unweighted
  0.764), `RR` 5.5 / 12.9 / 12.8%, `PL` 4.44 / 9.14 / 15.59%, removed 4.80 /
  9.70 / 19.64%; round 1's base rate 48.45% [42.03, 55.11] (cluster [40.74,
  53.87]), AUC 0.777 [0.716, 0.841] (unweighted 0.803), `RR` 16.8 / 14.3 /
  28.3%, removed 4.83 / 9.78 / 19.87%.
- Only `manifest.tsv` was opened under the strict sample's folder.

Every one of the 32,459 dump frames has a trusted AF point, so the no-AF
fallback (the largest whole face at or above 0.8) never applies: `af` is the
crop set's `nearest_face` on every frame, and `af-whole` the whole set's.

### 1. Face counts

Frames with 0, 1 or 2+ faces in the whole set at or above 0.8, and in the
crop set (0.6) among the AF frames:

| Frames | n | Whole 0 % | Whole 1 % | Whole 2+ % | AF frames % | Crop 0 % (of AF) | Crop 1 % (of AF) | Crop 2+ % (of AF) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Round 2, weighted | 600 | 13.6 | 29.0 | 57.4 | 100.0 | 8.3 | 40.5 | 51.2 |
| Round 2, unweighted | 600 | 18.8 | 28.0 | 53.2 | 100.0 | 15.8 | 36.8 | 47.3 |
| Round 1, weighted | 600 | 18.8 | 31.8 | 49.5 | 100.0 | 11.6 | 36.9 | 51.5 |
| Round 1, unweighted | 600 | 21.8 | 27.5 | 50.7 | 100.0 | 16.5 | 36.2 | 47.3 |
| Whole dump | 32459 | 16.2 | 28.7 | 55.0 | 100.0 | 12.4 | 36.7 | 50.9 |

The share of frames each rule leaves missing (it picks no face):

| Rule | Round 2 weighted % | Round 2 unweighted % | Round 1 weighted % | Round 1 unweighted % | Whole dump % |
| --- | ---: | ---: | ---: | ---: | ---: |
| `af` | 8.3 | 15.8 | 11.6 | 16.5 | 12.4 |
| `af-whole` | 2.7 | 4.0 | 4.7 | 6.2 | 4.5 |
| `largest` | 13.6 | 18.8 | 18.8 | 21.8 | 16.2 |
| `sharpest` | 13.6 | 18.8 | 18.8 | 21.8 | 16.2 |
| `score` | 13.6 | 18.8 | 18.8 | 21.8 | 16.2 |
| `mean` | 13.6 | 18.8 | 18.8 | 21.8 | 16.2 |

Labeled multi-face frames (2+ whole faces at or above 0.8): 319 in round 2
(159 keep, 160 discard), 304 in round 1 (124 keep, 180 discard).

### 2. The rules, round 2 (primary)

All labeled frames:

| Rule | AUC [stratified 95%] | Cluster | Unweighted | AUC - `af` |
| --- | ---: | ---: | ---: | ---: |
| `af` | 0.785 [0.723, 0.844] | [0.693, 0.881] | 0.764 |  |
| `af-whole` | 0.800 [0.740, 0.855] | [0.714, 0.878] | 0.741 | +0.014 [-0.018, +0.052] ([-0.023, +0.047]) |
| `largest` | 0.716 [0.642, 0.794] | [0.622, 0.814] | 0.725 | -0.069 [-0.140, -0.001] ([-0.149, +0.010]) |
| `sharpest` | 0.694 [0.624, 0.769] | [0.604, 0.794] | 0.732 | -0.091 [-0.163, -0.025] ([-0.176, -0.009]) |
| `score` | 0.656 [0.582, 0.733] | [0.562, 0.759] | 0.693 | -0.130 [-0.203, -0.056] ([-0.229, -0.039]) |
| `mean` | 0.627 [0.555, 0.702] | [0.524, 0.736] | 0.679 | -0.159 [-0.235, -0.078] ([-0.255, -0.058]) |

| Rule | Cut | Sampled removed / discard | `DP` % | `RR` % | `RR` cluster | `PL` % | Removed % | `RR` - `af`, points |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `af` | q5 | 43 / 38 | 94.5 [85.5, 98.9] (88.4) | 5.5 [1.1, 14.5] (11.6) | [1.3, 17.7] | 4.44 | 4.80 |  |
| `af` | q10 | 87 / 66 | 87.1 [78.2, 92.8] (75.9) | 12.9 [7.2, 21.8] (24.1) | [6.1, 26.5] | 9.14 | 9.70 |  |
| `af` | q20 | 204 / 160 | 87.2 [82.1, 91.1] (78.4) | 12.8 [8.9, 17.9] (21.6) | [7.7, 23.3] | 15.59 | 19.64 |  |
| `af-whole` | q5 | 37 / 29 | 90.3 [73.8, 96.6] (78.4) | 9.7 [3.4, 26.2] (21.6) | [3.4, 23.8] | 4.23 | 4.86 | +4.1 [-5.4, +21.0] ([-6.8, +18.0]) |
| `af-whole` | q10 | 90 / 69 | 88.4 [79.8, 93.7] (76.7) | 11.6 [6.3, 20.2] (23.3) | [6.4, 21.5] | 7.78 | 9.81 | -1.3 [-8.3, +5.5] ([-10.5, +5.2]) |
| `af-whole` | q20 | 182 / 137 | 78.4 [66.9, 88.5] (75.3) | 21.6 [11.5, 33.1] (24.7) | [11.1, 37.5] | 14.37 | 19.68 | +8.8 [-1.1, +19.7] ([-1.5, +19.3]) |
| `largest` | q5 | 34 / 28 | 93.7 [82.9, 98.4] (82.4) | 6.3 [1.6, 17.1] (17.6) | [2.5, 15.1] | 4.23 | 4.92 | +0.8 [-7.4, +10.3] ([-6.3, +3.6]) |
| `largest` | q10 | 72 / 55 | 73.4 [51.0, 93.1] (76.4) | 26.6 [6.9, 49.0] (23.6) | [9.0, 43.4] | 8.24 | 9.85 | +13.7 [-6.3, +34.4] ([-9.4, +31.1]) |
| `largest` | q20 | 165 / 124 | 64.7 [50.1, 80.1] (75.2) | 35.3 [19.9, 49.9] (24.8) | [15.7, 53.8] | 15.61 | 19.74 | +22.5 [+7.1, +35.7] ([+2.8, +38.6]) |
| `sharpest` | q5 | 34 / 28 | 93.7 [82.9, 98.4] (82.4) | 6.3 [1.6, 17.1] (17.6) | [2.5, 15.1] | 4.21 | 4.92 | +0.8 [-7.4, +10.3] ([-6.3, +3.6]) |
| `sharpest` | q10 | 74 / 54 | 69.6 [46.3, 90.2] (73.0) | 30.4 [9.8, 53.7] (27.0) | [10.0, 53.4] | 8.12 | 9.86 | +17.5 [-3.1, +38.9] ([-7.3, +39.6]) |
| `sharpest` | q20 | 176 / 131 | 61.7 [47.0, 77.5] (74.4) | 38.3 [22.5, 53.0] (25.6) | [17.4, 56.0] | 15.91 | 19.76 | +25.5 [+10.2, +38.9] ([+3.2, +42.3]) |
| `score` | q5 | 34 / 28 | 93.7 [82.9, 98.4] (82.4) | 6.3 [1.6, 17.1] (17.6) | [2.5, 15.1] | 4.21 | 4.92 | +0.8 [-7.4, +10.3] ([-6.3, +3.6]) |
| `score` | q10 | 71 / 53 | 70.2 [46.6, 91.1] (74.6) | 29.8 [8.9, 53.4] (25.4) | [9.3, 53.3] | 8.28 | 9.85 | +16.9 [-4.2, +38.6] ([-8.6, +39.6]) |
| `score` | q20 | 167 / 117 | 59.8 [44.9, 75.3] (70.1) | 40.2 [24.7, 55.1] (29.9) | [20.1, 56.5] | 16.61 | 19.74 | +27.4 [+12.7, +41.0] ([+6.2, +42.6]) |
| `mean` | q5 | 34 / 28 | 93.7 [82.9, 98.4] (82.4) | 6.3 [1.6, 17.1] (17.6) | [2.5, 15.1] | 4.21 | 4.92 | +0.8 [-7.4, +10.3] ([-6.3, +3.6]) |
| `mean` | q10 | 71 / 54 | 70.6 [46.9, 91.4] (76.1) | 29.4 [8.6, 53.1] (23.9) | [9.0, 52.4] | 8.17 | 9.84 | +16.5 [-4.9, +38.0] ([-9.4, +38.9]) |
| `mean` | q20 | 164 / 122 | 61.1 [45.6, 77.3] (74.4) | 38.9 [22.7, 54.4] (25.6) | [17.4, 57.7] | 16.86 | 19.77 | +26.1 [+10.7, +40.0] ([+3.3, +43.2]) |

Multi-face frames (the AUC; the cut tables are in `facechoiceread.out.md`,
where the whole-set rules remove 0 to 4 sampled multi-face frames at q5 and
q10, so their `RR` there rests on almost nothing):

| Rule | AUC [stratified 95%] | Cluster | Unweighted | AUC - `af` |
| --- | ---: | ---: | ---: | ---: |
| `af` | 0.769 [0.680, 0.854] | [0.655, 0.898] | 0.772 |  |
| `af-whole` | 0.775 [0.686, 0.855] | [0.689, 0.873] | 0.745 | +0.006 [-0.036, +0.053] ([-0.039, +0.061]) |
| `largest` | 0.742 [0.639, 0.837] | [0.666, 0.841] | 0.736 | -0.027 [-0.114, +0.062] ([-0.101, +0.048]) |
| `sharpest` | 0.683 [0.573, 0.786] | [0.617, 0.783] | 0.756 | -0.086 [-0.190, +0.021] ([-0.179, +0.011]) |
| `score` | 0.635 [0.534, 0.736] | [0.532, 0.739] | 0.684 | -0.134 [-0.244, -0.018] ([-0.281, -0.017]) |
| `mean` | 0.603 [0.509, 0.707] | [0.504, 0.733] | 0.672 | -0.166 [-0.289, -0.027] ([-0.284, -0.027]) |

### 2. The rules, round 1 (replication)

All labeled frames:

| Rule | AUC [stratified 95%] | Cluster | Unweighted | AUC - `af` |
| --- | ---: | ---: | ---: | ---: |
| `af` | 0.777 [0.716, 0.841] | [0.719, 0.832] | 0.803 |  |
| `af-whole` | 0.749 [0.683, 0.811] | [0.687, 0.813] | 0.783 | -0.028 [-0.065, +0.009] ([-0.063, +0.005]) |
| `largest` | 0.741 [0.672, 0.805] | [0.682, 0.803] | 0.757 | -0.037 [-0.100, +0.024] ([-0.085, +0.032]) |
| `sharpest` | 0.747 [0.682, 0.809] | [0.672, 0.823] | 0.760 | -0.031 [-0.091, +0.028] ([-0.089, +0.043]) |
| `score` | 0.731 [0.665, 0.797] | [0.666, 0.802] | 0.741 | -0.047 [-0.115, +0.024] ([-0.100, +0.029]) |
| `mean` | 0.709 [0.645, 0.781] | [0.629, 0.787] | 0.727 | -0.068 [-0.141, +0.006] ([-0.135, +0.028]) |

| Rule | Cut | Sampled removed / discard | `DP` % | `RR` % | `RR` cluster | `PL` % | Removed % | `RR` - `af`, points |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `af` | q5 | 38 / 33 | 83.2 [56.7, 98.8] (86.8) | 16.8 [1.2, 43.3] (13.2) | [1.1, 50.8] | 4.44 | 4.83 |  |
| `af` | q10 | 82 / 72 | 85.7 [70.1, 97.1] (87.8) | 14.3 [2.9, 29.9] (12.2) | [3.1, 32.0] | 9.14 | 9.78 |  |
| `af` | q20 | 189 / 160 | 71.7 [59.8, 84.3] (84.7) | 28.3 [15.7, 40.2] (15.3) | [18.1, 42.4] | 15.59 | 19.87 |  |
| `af-whole` | q5 | 44 / 37 | 65.7 [36.7, 96.7] (84.1) | 34.3 [3.3, 63.3] (15.9) | [11.1, 64.3] | 4.23 | 4.91 | +17.5 [-3.6, +43.0] ([+2.3, +28.7]) |
| `af-whole` | q10 | 92 / 75 | 66.6 [48.6, 84.4] (81.5) | 33.4 [15.6, 51.4] (18.5) | [21.0, 51.3] | 7.78 | 9.91 | +19.1 [+1.6, +39.3] ([+0.3, +38.6]) |
| `af-whole` | q20 | 203 / 173 | 66.1 [54.1, 78.2] (85.2) | 33.9 [21.8, 45.9] (14.8) | [21.0, 51.4] | 14.37 | 19.95 | +5.7 [-4.1, +15.8] ([-3.3, +14.4]) |
| `largest` | q5 | 40 / 32 | 66.6 [41.1, 94.4] (80.0) | 33.4 [5.6, 58.9] (20.0) | [2.8, 60.9] | 4.23 | 4.96 | +16.6 [-8.0, +43.8] ([-13.2, +44.1]) |
| `largest` | q10 | 85 / 70 | 72.2 [56.1, 89.3] (82.4) | 27.8 [10.7, 43.9] (17.6) | [9.9, 49.7] | 8.24 | 9.93 | +13.6 [-3.2, +31.5] ([-6.0, +31.4]) |
| `largest` | q20 | 159 / 127 | 65.4 [53.2, 79.0] (79.9) | 34.6 [21.0, 46.8] (20.1) | [21.1, 49.5] | 15.61 | 19.87 | +6.3 [-9.4, +21.0] ([-8.1, +20.2]) |
| `sharpest` | q5 | 40 / 32 | 66.6 [41.1, 94.4] (80.0) | 33.4 [5.6, 58.9] (20.0) | [2.8, 60.9] | 4.21 | 4.96 | +16.6 [-8.0, +43.8] ([-13.2, +44.1]) |
| `sharpest` | q10 | 84 / 71 | 72.6 [56.3, 89.9] (84.5) | 27.4 [10.1, 43.7] (15.5) | [9.7, 48.6] | 8.12 | 9.94 | +13.1 [-3.7, +31.0] ([-6.4, +30.8]) |
| `sharpest` | q20 | 163 / 135 | 69.0 [57.3, 82.3] (82.8) | 31.0 [17.7, 42.7] (17.2) | [17.0, 46.6] | 15.91 | 19.90 | +2.7 [-13.1, +17.2] ([-10.1, +15.4]) |
| `score` | q5 | 40 / 32 | 66.6 [41.1, 94.4] (80.0) | 33.4 [5.6, 58.9] (20.0) | [2.8, 60.9] | 4.21 | 4.96 | +16.6 [-8.0, +43.8] ([-13.2, +44.1]) |
| `score` | q10 | 85 / 70 | 72.2 [56.1, 89.2] (82.4) | 27.8 [10.8, 43.9] (17.6) | [10.0, 49.6] | 8.28 | 9.94 | +13.6 [-3.1, +31.5] ([-5.8, +31.5]) |
| `score` | q20 | 157 / 130 | 63.2 [50.7, 77.2] (82.8) | 36.8 [22.8, 49.3] (17.2) | [22.8, 51.5] | 16.61 | 19.87 | +8.5 [-7.7, +23.3] ([-6.0, +20.9]) |
| `mean` | q5 | 40 / 32 | 66.6 [41.1, 94.4] (80.0) | 33.4 [5.6, 58.9] (20.0) | [2.8, 60.9] | 4.21 | 4.95 | +16.6 [-8.0, +43.8] ([-13.2, +44.1]) |
| `mean` | q10 | 86 / 71 | 72.3 [56.2, 89.2] (82.6) | 27.7 [10.8, 43.8] (17.4) | [10.0, 49.4] | 8.17 | 9.93 | +13.5 [-3.1, +31.5] ([-5.9, +31.4]) |
| `mean` | q20 | 160 / 131 | 67.7 [55.6, 81.0] (81.9) | 32.3 [19.0, 44.4] (18.1) | [19.4, 45.6] | 16.86 | 19.90 | +4.1 [-12.2, +18.7] ([-9.1, +16.0]) |

Multi-face frames:

| Rule | AUC [stratified 95%] | Cluster | Unweighted | AUC - `af` |
| --- | ---: | ---: | ---: | ---: |
| `af` | 0.787 [0.700, 0.866] | [0.692, 0.852] | 0.786 |  |
| `af-whole` | 0.759 [0.660, 0.846] | [0.628, 0.839] | 0.771 | -0.029 [-0.071, +0.014] ([-0.094, +0.015]) |
| `largest` | 0.749 [0.653, 0.838] | [0.665, 0.807] | 0.756 | -0.038 [-0.119, +0.051] ([-0.098, +0.054]) |
| `sharpest` | 0.763 [0.672, 0.843] | [0.645, 0.854] | 0.771 | -0.024 [-0.090, +0.052] ([-0.124, +0.073]) |
| `score` | 0.730 [0.634, 0.821] | [0.655, 0.792] | 0.727 | -0.058 [-0.172, +0.054] ([-0.117, +0.028]) |
| `mean` | 0.710 [0.620, 0.802] | [0.603, 0.816] | 0.702 | -0.077 [-0.188, +0.050] ([-0.186, +0.084]) |

### 3. The disagreement test

#### `af` vs `largest`

| Frames | Both pick a face | Exactly one picks | IoU < 0.5 (the subset) | Subset discard / keep | `af` smaller |
| --- | ---: | ---: | ---: | ---: | ---: |
| Round 2 | 435 | 122 | 253 (38.5% weighted) | 152 / 101 | 194 (76.7%) |
| Round 1 | 423 | 124 | 226 (36.7% weighted) | 148 / 78 | 179 (79.2%) |
| Whole dump | 25125 | 5381 | 12486 (38.5%) | - | 9114 (73.0%) |

Round 2, the subset's AUC:

| Rule | AUC [stratified 95%] | Cluster | Unweighted | AUC - `af` |
| --- | ---: | ---: | ---: | ---: |
| `af` | 0.833 [0.748, 0.901] | [0.722, 0.921] | 0.758 |  |
| `af-whole` | 0.830 [0.745, 0.899] | [0.737, 0.908] | 0.727 | -0.003 [-0.045, +0.043] ([-0.032, +0.027]) |
| `largest` | 0.682 [0.554, 0.813] | [0.526, 0.847] | 0.707 | -0.151 [-0.289, -0.004] ([-0.257, -0.004]) |
| `sharpest` | 0.709 [0.582, 0.829] | [0.561, 0.878] | 0.749 | -0.124 [-0.266, +0.019] ([-0.209, +0.004]) |
| `score` | 0.638 [0.508, 0.771] | [0.533, 0.764] | 0.663 | -0.195 [-0.346, -0.042] ([-0.265, -0.100]) |
| `mean` | 0.596 [0.483, 0.723] | [0.459, 0.787] | 0.667 | -0.237 [-0.363, -0.093] ([-0.356, -0.081]) |

Round 2, the label by which face has the lower `eye_focus`:

| Lower `eye_focus` | Frames | Keep | Discard | Weighted discard % |
| --- | ---: | ---: | ---: | ---: |
| `af`'s | 128 | 43 | 85 | 60.7 [45.2, 75.7] |
| `largest`'s | 125 | 58 | 67 | 38.8 [25.4, 53.5] |
| equal | 0 | 0 | 0 | - |

Round 1, the subset's AUC:

| Rule | AUC [stratified 95%] | Cluster | Unweighted | AUC - `af` |
| --- | ---: | ---: | ---: | ---: |
| `af` | 0.747 [0.636, 0.848] | [0.653, 0.819] | 0.793 |  |
| `af-whole` | 0.754 [0.638, 0.849] | [0.619, 0.859] | 0.782 | +0.007 [-0.089, +0.104] ([-0.087, +0.088]) |
| `largest` | 0.711 [0.588, 0.825] | [0.643, 0.808] | 0.729 | -0.036 [-0.192, +0.126] ([-0.136, +0.113]) |
| `sharpest` | 0.752 [0.639, 0.854] | [0.630, 0.879] | 0.769 | +0.005 [-0.127, +0.137] ([-0.133, +0.156]) |
| `score` | 0.681 [0.542, 0.798] | [0.621, 0.756] | 0.714 | -0.066 [-0.227, +0.089] ([-0.146, +0.043]) |
| `mean` | 0.749 [0.642, 0.850] | [0.646, 0.864] | 0.724 | +0.002 [-0.131, +0.141] ([-0.112, +0.151]) |

Round 1, the label by which face has the lower `eye_focus`:

| Lower `eye_focus` | Frames | Keep | Discard | Weighted discard % |
| --- | ---: | ---: | ---: | ---: |
| `af`'s | 93 | 28 | 65 | 60.0 [43.4, 76.4] |
| `largest`'s | 133 | 50 | 83 | 43.8 [30.6, 59.2] |
| equal | 0 | 0 | 0 | - |

#### `af` vs `sharpest`

`sharpest` takes the maximum over the whole set's faces at or above 0.8,
while the `af` face comes from the crop set (its own box, its own
`eye_focus`, and possibly below 0.8), so the `sharpest` face can have the
lower `eye_focus` of the two.

| Frames | Both pick a face | Exactly one picks | IoU < 0.5 (the subset) | Subset discard / keep | `af` smaller |
| --- | ---: | ---: | ---: | ---: | ---: |
| Round 2 | 435 | 122 | 236 (33.6% weighted) | 148 / 88 | 143 (60.6%) |
| Round 1 | 423 | 124 | 212 (30.3% weighted) | 151 / 61 | 119 (56.1%) |
| Whole dump | 25125 | 5381 | 11077 (34.1%) | - | 6200 (56.0%) |

Round 2, the subset's AUC:

| Rule | AUC [stratified 95%] | Cluster | Unweighted | AUC - `af` |
| --- | ---: | ---: | ---: | ---: |
| `af` | 0.784 [0.676, 0.872] | [0.657, 0.881] | 0.706 |  |
| `af-whole` | 0.808 [0.711, 0.883] | [0.718, 0.881] | 0.694 | +0.024 [-0.034, +0.092] ([-0.027, +0.082]) |
| `largest` | 0.674 [0.537, 0.811] | [0.518, 0.865] | 0.705 | -0.110 [-0.268, +0.064] ([-0.212, +0.068]) |
| `sharpest` | 0.647 [0.514, 0.783] | [0.505, 0.831] | 0.717 | -0.137 [-0.301, +0.045] ([-0.232, +0.018]) |
| `score` | 0.658 [0.525, 0.801] | [0.568, 0.812] | 0.688 | -0.126 [-0.291, +0.050] ([-0.223, +0.055]) |
| `mean` | 0.601 [0.474, 0.740] | [0.437, 0.810] | 0.670 | -0.183 [-0.335, -0.012] ([-0.303, +0.019]) |

Round 2, the label by which face has the lower `eye_focus`:

| Lower `eye_focus` | Frames | Keep | Discard | Weighted discard % |
| --- | ---: | ---: | ---: | ---: |
| `af`'s | 173 | 67 | 106 | 53.0 [40.1, 66.7] |
| `sharpest`'s | 63 | 21 | 42 | 44.8 [24.9, 66.9] |
| equal | 0 | 0 | 0 | - |

Round 1, the subset's AUC:

| Rule | AUC [stratified 95%] | Cluster | Unweighted | AUC - `af` |
| --- | ---: | ---: | ---: | ---: |
| `af` | 0.718 [0.577, 0.842] | [0.599, 0.837] | 0.791 |  |
| `af-whole` | 0.711 [0.561, 0.835] | [0.579, 0.831] | 0.756 | -0.007 [-0.142, +0.128] ([-0.115, +0.077]) |
| `largest` | 0.761 [0.640, 0.873] | [0.617, 0.888] | 0.775 | +0.043 [-0.129, +0.223] ([-0.152, +0.209]) |
| `sharpest` | 0.743 [0.616, 0.860] | [0.594, 0.882] | 0.777 | +0.025 [-0.151, +0.198] ([-0.164, +0.184]) |
| `score` | 0.739 [0.590, 0.850] | [0.596, 0.878] | 0.749 | +0.022 [-0.164, +0.187] ([-0.158, +0.167]) |
| `mean` | 0.761 [0.629, 0.878] | [0.612, 0.883] | 0.767 | +0.043 [-0.118, +0.205] ([-0.136, +0.180]) |

Round 1, the label by which face has the lower `eye_focus`:

| Lower `eye_focus` | Frames | Keep | Discard | Weighted discard % |
| --- | ---: | ---: | ---: | ---: |
| `af`'s | 146 | 45 | 101 | 55.0 [42.7, 67.8] |
| `sharpest`'s | 66 | 16 | 50 | 64.9 [45.5, 83.1] |
| equal | 0 | 0 | 0 | - |

### 4. The reading rule applied

| Rule | AUC | (1) AUC - `af` | (2) `RR` q10 - `af`, points | Verdict |
| --- | ---: | ---: | ---: | ---: |
| `af-whole` | 0.800 | +0.014, 2.5% -0.018 (cluster -0.023): no | -1.3, 97.5% +5.5 (cluster +5.2): no | does not beat |
| `largest` | 0.716 | -0.069, 2.5% -0.140 (cluster -0.149): no | +13.7, 97.5% +34.4 (cluster +31.1): no | does not beat |
| `sharpest` | 0.694 | -0.091, 2.5% -0.163 (cluster -0.176): no | +17.5, 97.5% +38.9 (cluster +39.6): no | does not beat |
| `score` | 0.656 | -0.130, 2.5% -0.203 (cluster -0.229): no | +16.9, 97.5% +38.6 (cluster +39.6): no | does not beat |
| `mean` | 0.627 | -0.159, 2.5% -0.235 (cluster -0.255): no | +16.5, 97.5% +38.0 (cluster +38.9): no | does not beat |

**No winner.** No rule beats `af` on round 2.

### Reading

The facts, without a decision (Step 4 decides):

- **Multi-face frames are frequent here**, so the question can matter: 55.0%
  of the dump's frames hold 2+ whole faces at or above 0.8 (57.4% of round
  2's population, 49.5% of round 1's), and 50.9% of the AF frames hold 2+
  crop faces. Where `af` and `largest` both pick a face, their boxes differ
  (IoU < 0.5) on 12,486 of the dump's 25,125 frames (38.5% of all frames),
  and there the AF face is the smaller one 73.0% of the time; `af` and
  `sharpest` differ on 34.1% of the dump's frames.
- **Today's face is the better discard signal.** On round 2's all frames,
  every whole-set face choice reads below `af` (0.785): `largest` 0.716
  (−0.069 [−0.140, −0.001]), `sharpest` 0.694 (−0.091 [−0.163, −0.025]),
  `score` 0.656 (−0.130 [−0.203, −0.056]), `mean` 0.627 (−0.159 [−0.235,
  −0.078]); the stratified intervals of all four exclude zero on the
  negative side. Round 1 points the same way (−0.031 to −0.068) with
  intervals that reach zero. At q10 their `RR` is 13.7 to 17.5 points above
  `af`'s on round 2 (13.1 to 13.6 on round 1). At q5 the whole-set rules'
  cuts are mostly the frames without a whole face at or above 0.8 (16.2% of
  the dump), which come first under missing last, so their sampled q5
  readings coincide.
- **The control `af-whole`** (the AF-nearest face of the whole-image
  detection) is the only rule above `af` on round 2, 0.800 (+0.014 [−0.018,
  +0.052]), with `RR` at q10 −1.3 points (97.5% quantile +5.5, just past the
  +5 bar); on round 1 it reads below, 0.749 (−0.028 [−0.065, +0.009]). It
  leaves 4.5% of the dump missing against `af`'s 12.4%; its `PL` at q10 is
  7.78% against `af`'s 9.14%.
- **On the disagreement frames the AF face carries the signal.** Where `af`
  and `largest` disagree, round 2's subset AUC is 0.833 for `af` against
  0.682 for `largest` (−0.151 [−0.289, −0.004]); a frame whose AF face has
  the lower `eye_focus` is discardable 60.7% [45.2, 75.7] of the time, one
  whose largest face has the lower value 38.8% [25.4, 53.5] (round 1: 60.0%
  and 43.8%). Where `af` and `sharpest` disagree, `af` reads 0.784 against
  0.647 (−0.137 [−0.301, +0.045]); round 1's subsets are within noise in
  both comparisons (and its `af` vs `sharpest` cross-tab runs the other way,
  55.0% against 64.9%).
- **On multi-face frames** `af` reads 0.769 (round 2) and 0.787 (round 1),
  and no rule's AUC difference against it is positive beyond +0.006.
- The user's hypothesis was that the face `eye_focus` judges is not the one
  a human looks at. On round 2, where the AF-nearest face differs from the
  largest or the sharpest face, its `eye_focus` agrees with the discard label
  better than theirs does; round 1's subsets are within noise, and its `af`
  vs `sharpest` cross-tab runs the other way.
- Both rounds have been read before. One photographer, mostly basketball.
