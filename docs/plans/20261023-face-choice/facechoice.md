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
   missing when the whole set is empty); without one, `af`'s fallback.
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
  where the stored value is `nan`. Step 2 also checks the crop set's
  `nearest` face against `scan::extract_analysis(path).cue.eye_focus` on
  every file (exact or within 1e-9).
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
