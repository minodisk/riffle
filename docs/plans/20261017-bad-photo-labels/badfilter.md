# Bad-photo filter evaluation

## Pre-registration

Written 2026-10-17, after Step 1 drew the sample and copied it into
`D:\Photos\tests\2026-10-17-bad-labels\sample\`, **before the user has
labeled any frame of it and before any label is read**: when this section was
written no copy carried a sidecar (Step 1's check: no `.xmp` / `.dop` in
`sample\`), so no bad label existed anywhere, and the sample folder's
sidecars were not opened for it. It is merged on its own, before
`labels.py`, `badfilter.py` or `badmodel.py` exists; the merge commit of the
PR that adds this section, without results, is the record, as for the
archived
[reject-filter evaluation](../_archived/20261016-reject-filter-eval/reject.md)
and the folder-wide evaluation before it. Nothing in this section changes
after the labels are in; a deviation found while implementing Steps 3–5 is
recorded in `learnings.md` with its reason and does not touch the bars.

The question: on labels of **bad** (the user would never keep the frame on
its own), not of "not picked", does any existing score, rule or cue remove
most of a folder's bad frames while losing few of its not-bad ones? Which
cues catch bad photos? Is a model trained on bad / not-bad labels worth a
next plan?

### What was known before this section

This pre-registration is **not blind** to the scores. On the same dump of 27
folders and 32,459 frames, the archived
[reject-filter evaluation](../_archived/20261016-reject-filter-eval/reject.md)
(Decision 2 in its
[plan](../_archived/20261016-reject-filter-eval/plan.md)) reported, held
out, on the **non-pick** criterion (every non-pick a negative, so an upper
bound on "bad removed"):

- the best filter `G0` (gradient boosting on `F0+ctx`'s columns, weights
  rescaled to mean 1 per fold): oracle `RN@95` 22.0%, `RN@99` 11.5%,
  `pAUC95` 0.568; label-free `quantile@95` realized pick recall 94.7%,
  non-pick removal 16.8%; `W0` 16.6% / 7.1% / 0.544 and 95.0% / 11.5%;
  `F0+ctx` 18.4% / 8.1% / 0.550 and 95.0% / 14.6%; random 5.8% `RN@95`;
- the baselines' `RN@95` 5.8–9.3% (absolute sharpness 9.2%, `eye_focus`
  7.2%, `photoTier` good 9.3%), and `rule count` 7.9%;
- the binary rules' points (mean realized pick recall / non-pick removal):
  `eyes closed` 92.8% / 11.3%, `eye focus low` 87.1% / 19.8%, `face away`
  75.1% / 23.5%, `sharpness bottom 10%` 93.4% / 10.3%, `any of` 59.7% /
  49.6%, `not good` 28.8% / 80.3%;
- of the 58 explicit rejects (3 folders), `G0`'s `quantile@95` cut removes
  29, `W0`'s 15, `eye focus low` 30, `any of` 38, `not good` 54;
- that the "obviously bad" cues (sharpness, AF eye focus, eyes open, head
  pose, good photo) contribute at the bottom on no linear base, and only
  weakly in `G0`; the embedding (Look) and burst position carry the bottom.

Also known, from Step 1 (label-free; `strata.tsv` and `sample.out.md` in the
data folder): the strata, their population, draw and inclusion probability
`π`, each frame in the first that matches:

| Stratum | Population | Drawn | `π` |
| --- | ---: | ---: | ---: |
| `closed` (raw EAR <= 0.137) | 2,385 | 120 | 0.0503 |
| `focus_low` (AF face `eye_focus` < 0.9) | 8,545 | 120 | 0.0140 |
| `away` (`\|yaw\|` > 35 or `\|pitch\|` > 45) | 5,076 | 120 | 0.0236 |
| `sharp10` (folder's bottom 10% by sharpness) | 1,422 | 120 | 0.0844 |
| `g0_bottom` (folder's bottom 15% by `G0`) | 1,441 | 150 | 0.1041 |
| `w0_bottom` (folder's bottom 15% by `W0`) | 945 | 70 | 0.0741 |
| `rest` (per folder, floor 8) | 12,645 | 500 | 0.0338–0.5714 per folder, 0.0395 overall |
| Total | 32,459 | 1,200 | |

The sample holds 167 picks, 6 explicit rejects (5 from `2026-09-19`), 82
singles, 256 `photoTier` good frames, and every folder (8 to 163 frames).

Not known before this section, with no prior number: any bad label, the bad
base rate of the library, and any score's recall or precision on "bad".

The bars below were **chosen by the user on 2026-10-17 before any label**
(option B of the plan's trade-offs), not tuned to anything above.

### Labels and population

- **`bad`**: the copy carries a Reject in its sidecar (XMP `xmpDM:good`
  `False` or `.dop` `ShouldProcess` 1, the `.dop` deciding when both exist,
  as `labels.py` reads them in Step 3). **`not bad`**: reviewed and
  unflagged; the stopping rule of `labeling.md` (every frame viewed once,
  none skipped, "when unsure, do not reject") makes "unflagged" a label. A
  copy whose sidecar holds a Pick or a rating counts by its reject state
  alone and is listed.
- **Population**: the dump's 32,459 frames. Every estimate is reported
  **weighted by the inverse inclusion probability** (Horvitz–Thompson, the
  manifest's `inclusion_prob`; a share is the ratio of the `1/π` sums over
  its numerator and denominator sets), with the **unweighted sample value**
  beside it.
- **The dump's `pick` / `reject` labels are consistency checks only**, never
  a target: the weighted share of the dump's picks labeled bad (expected near
  zero), the count of the 6 sampled explicit rejects labeled bad (a
  test-retest reading), and the weighted bad rate among the dump's non-picks
  (how much "bad" the non-pick criterion overstated).
- Frames skipped in Step 3 (a copy that would not open, `skip.txt`) leave
  the sample with their reason; their `π` is not re-normalized, and the
  deviation is reported.

### Unit

Every sampled frame is a sample. No per-folder means (about 45 sampled
frames per folder); the folder enters only as the cluster of the
folder-cluster bootstrap (Intervals).

### Cuts

- Every graded score is cut **per folder at a quantile**, as the archived
  label-free `quantile@r` is: remove the bottom `q`% of the folder's frames
  by the score (`floor(q * n / 100)` frames, ties by the seed-0 draw the
  archived metrics use), **`q` = 5 / 10 / 20 / 30 / 40** (keep rate 95 / 90 / 80
  / 70 / 60%). `q` = 5 is there because the smallest cut must be able to
  reach `NL` <= 5% (see Bars, feasibility).
- Plus the one cut Decision 2 named: **`G0` at its label-free `quantile@95`
  share**, the per-folder share `rejectfilter` computes for a qualifying
  folder (the largest share whose mean pick recall over the other qualifying
  folders is >= 95%); for the 2 non-qualifying folders, to which it gives no
  share, the share computed the same way from all 25 qualifying folders.
- A **binary rule is one cut** (its own point); a frame without the value a
  rule needs is never removed by it.
- The cut is computed on the **full dump** (all frames of the folder) and
  then read on the sampled frames, so the removed share is the library's,
  not the sample's. A score missing on a folder (`W0`'s family on
  `2026-07-23`) leaves that folder's sampled frames out of that score's rows,
  and the count is printed.

### Primary metrics

Per cut, weighted (`1/π`), with the unweighted value beside it:

1. **Bad recall `BR@q`**: the share of bad frames removed.
2. **Not-bad loss `NL@q`**: the share of not-bad frames removed (what the
   human loses).
3. **Precision `P@q`**: the share of removed frames that are bad.

And, threshold-free: the **weighted AUC** of bad vs not bad,
`roc_auc_score(bad, -score, sample_weight=1/π)`, and the **partial AUC at
not-bad loss <= 10%**, `roc_auc_score(bad, -score, max_fpr=0.10,
sample_weight=1/π)` (McClish-standardized, chance 0.5), both pooled over the
sampled frames (a score's per-folder scale enters as it does in the archived
pooled readings; the quantile cuts above are the scale-free reading).
Missing values in a graded score are handled as the archived `pAUC95` does
(the folder's lowest finite score minus 1).

**The primary reading is the pair (`BR`, `NL`) at the cut whose `NL` is
nearest the worth-a-filter bar's loss ceiling (5%)**, with `P` beside it.

### Scores and rules

Names as in the archived files; each is one held-out score vector, higher
keeps.

- **Stored, no refit**: `W0`, `W0+off`, `F0`, `F0+ctx`, `P0` (from
  `scores.tsv`), `R0`, `G0`, `rule count` (from `scores_reject.tsv`), and the
  **`W0` burst-centered** score (each burst's mean `W0` subtracted).
- **Baselines**, rebuilt as the archived scripts do: random (seed 0),
  absolute sharpness, `eye_focus` (missing last), `photoTier` good.
- **Binary rules**, the app's constants (`GOOD_EYE_FOCUS` 0.9,
  `EYES_CLOSED_EAR` 0.137, `GOOD_MAX_YAW` 35, `GOOD_MAX_PITCH` 45), as
  `rejectfilter.rules` builds them: `eyes closed` (raw EAR <= 0.137), `eye
  focus low`, `face away`, `sharpness bottom 10%`, `any of` (the four
  above), `not good` (`photoTier` not good).
- **Per-cue reading**: the weighted bad rate (with its interval and the
  sampled count) per stratum and per bin of each raw cue, so "which cues
  catch bad photos" is answered from the labels, not from a model. The bins:
  - raw EAR: <= 0.137 (closed), (0.137, 0.25], > 0.25, missing;
  - `eye_focus`: < 0.5, [0.5, 0.9), >= 0.9, missing (no AF face);
  - `|yaw|`: <= 35, > 35, missing; `|pitch|`: <= 45, > 45, missing;
  - absolute sharpness: the folder's deciles (1 the bottom), computed on the
    full dump;
  - `tier`: good, not good.

  A cue's **firing bin** is the one its rule removes: EAR <= 0.137,
  `eye_focus` < 0.9 (the two lower bins pooled), `|yaw|` > 35, `|pitch|` >
  45, sharpness decile 1, `tier` not good.

### Dedicated model (Step 5)

- **`B0`**: `HistGradientBoostingClassifier` at `rejectfilter.HGB`'s fixed
  setting (`max_iter` 200, `learning_rate` 0.05, `max_leaf_nodes` 15,
  `l2_regularization` 1.0, `early_stopping` off, `random_state` 0) on
  `folder_gate.design(d, mask, "full")`'s 46 columns, NaN restored as
  `rejectfilter.g0_fold` does; trained on the sampled frames' bad / not-bad
  labels with **`sample_weight = 1/π` rescaled to mean 1 on each fold's
  training frames** (the weight scale fixed here, in advance, as Decision 2
  asked); **leave one folder out** over the 27 folders, scored on the
  held-out folder's sampled frames.
- **`L0`**: the same folds, design and weights with
  `sklearn.linear_model.LogisticRegression` (`C` 1, the columns standardized
  on the training frames, NaN as `folder_gate.design` encodes it), each
  class's weights rescaled to equal total within the fold
  (class-balanced within the weights), as the linear comparison.
- **Cuts**: `B0` and `L0` score sampled frames only, so their per-folder
  quantile cuts are taken **within the folder's sampled frames**, weighted by
  `1/π` (remove the lowest-scored sampled frames until their `1/π` sum
  reaches `q`% of the folder's `1/π` sum: the **longest prefix whose `1/π`
  sum is <= `q`%** of it, so the crossing frame is not removed; ties by the
  seed-0 draw). A folder's `1/π` sum is its estimated frame count, not the
  sampled count; one heavy cue-stratum frame can exceed `q`% alone, and then
  nothing is removed in that folder. Caveat: on about 45 frames per
  folder this cut is coarse, and it is not the full-dump cut the stored
  scores get. So in the dedicated-model comparison the best existing score is
  read **on the same within-sample cut as well**, beside its full-dump row,
  and the paired difference uses the within-sample rows of both.
- No tuning, no inner CV, no other model.

### Intervals

- The weighted **bad base rate** with its interval is the first number
  reported.
- A 1,000-draw **stratified bootstrap** over the sampled frames (resample
  with replacement within each stratum, its drawn count kept; `rest` is
  resampled within each (`rest`, folder) cell, keeping each cell's drawn
  count, so the design's fixed cell sizes and the total `1/π` are kept,
  `np.random.default_rng(0)` afresh per quantity), the cuts and the scores
  held fixed (only which sampled frames enter is resampled; `B0` / `L0` are
  not refit per draw), for every weighted metric and for **paired
  differences** between scores at the same cut (the same draws for both
  scores), reported as the 2.5% / 97.5% quantiles.
- A **folder-cluster bootstrap** (resample the 27 folders with replacement,
  seed 0, 1,000 draws) as a sensitivity reading beside it, for the primary
  pairs and the bars' quantities.

### Bars

Option B, chosen by the user on 2026-10-17 before any label. Rejected:
option A (L 10%, R 40%, P 40%: a filter that drags a tenth of the keepers
along) and option C (L 2%, R 60%, P 60%: near-perfect, likely declared
"none" whatever the cues do).

- **Worth-a-filter bar**: a score or rule is a **usable bad-photo filter**
  when at some pre-listed cut (`q` = 5 / 10 / 20 / 30 / 40, `G0`'s
  `quantile@95`, a binary rule's own point) all of these hold, weighted:
  - `NL` <= **5%**;
  - `BR` >= **50%**, with the stratified bootstrap 2.5% quantile of `BR`
    above **40%** (R − 10 points);
  - `P` >= **50%**.
- **Dedicated-model bar**: a next plan trains and ships a dedicated model
  when either holds:
  - `B0` (or `L0`) exceeds the best existing score's `BR` by >= **+10
    points**, each at its pre-listed cut with the highest `BR` among those
    with `NL` <= 5% (on the within-sample cuts, Dedicated model), with the
    paired stratified bootstrap 2.5% quantile above zero; or
  - no existing score or rule passes the worth-a-filter bar but `B0` (or
    `L0`) does, on its within-sample cuts.
- **A cue catches bad photos** when the weighted bad rate of its firing bin
  is at least **twice** the weighted base rate and the bin's stratified
  bootstrap 2.5% quantile is above the base rate.

Applied as:

- **Best existing filter**: the score or rule with the highest `BR` among
  the pre-listed cuts with `NL` <= 5%; it is reported whether or not it
  passes.
- **A very low base rate** (under 5%) makes every `P` floor hard to reach;
  if it occurs, that is itself a finding ("the library has few bad frames to
  filter") and is stated, not a reason to move a bar.
- **Feasibility of `NL` <= 5%**: a graded cut at `q` removes `q`% of every
  folder, so `NL` <= 5% needs `BR · b` >= `q`/100 − 0.05·(1 − b) with `b` the
  base rate, i.e. at `q` = 5 it is feasible at any `b`, but at `q` >= 10 it
  needs `BR` >= 55% at `b` = 10%, and is impossible for `b` below about
  5.3% at `q` = 10. At a low or moderate base rate only the `q` = 5 cut, the
  binary rules and `G0`'s `quantile@95` cut can be live candidates; a "no
  usable filter" outcome there is read with this limit in mind, not as
  evidence about the scores alone.
- **Multiplicity**: about 25 score rows, 6 cuts and 20 cue bins are read
  without correction, so a few chance passes are expected; the Decision
  reads a lone marginal pass as such.
- The folder-cluster bootstrap does not decide; a pass whose cluster
  interval falls below the bar is reported as fragile.

### What happens after

No step of this plan changes the app. The Decision in the plan records the
outcome against these bars and names the follow-up plan, if any (none; a
larger labeling round with the same sampler; or a plan that trains and ships
a bad-photo model), as a recommendation only.

## Results

Run 2026-10-18 by `badfilter.py` (Step 4) as `python -I -B badfilter.py
D:\Photos\tests\2026-10-12-rank-app D:\Photos\tests\2026-10-15-folder-wide-eval
D:\Photos\tests\2026-10-16-reject-filter-eval D:\Photos\tests\2026-10-17-bad-labels`;
the tables below are its stdout, saved as `badfilter.out.md` in the data
folder. The labels are Step 3's `labels.tsv` (80 bad, 1,120 not bad, no copy
skipped). Nothing in the Pre-registration above was changed; the choices the
text left open are recorded in `learnings.md` (Step 4).

### Data

| Count | Value |
| --- | ---: |
| Folders / dump frames (with a vector) / frames without a vector (dropped) | 27 / 32459 / 0 |
| Sampled frames (`labels.tsv`) / bad / not bad | 1200 / 80 / 1120 |
| Sum of `1/π` over the sampled frames | 32459.0 |
| Sampled picks / explicit rejects / singles | 167 / 6 / 82 |
| Sampled frames without `W0` (folder `2026-07-23`) | 8 |

### Bad base rate

The weighted (Horvitz-Thompson, `1/π`) share of the dump's frames labeled bad, the stratified bootstrap 95% interval, the folder-cluster interval, and the unweighted sample share.

| Reading | Weighted % | Stratified 95% | Folder-cluster 95% | Unweighted % |
| --- | ---: | ---: | ---: | ---: |
| Bad base rate | 5.52 | [4.15, 7.04] | [3.71, 7.61] | 6.67 |

### Reference checks

| Check | Expected | This run |
| --- | ---: | ---: |
| Strata (`strata.tsv` rows) whose population, drawn count or `π` differ when recomputed from the dump | 0 | 0 of 33 |
| Sampled frames whose recomputed stratum / `π` differ from `labels.tsv` | 0 / 0 | 0 / 0 |
| Sampled frames whose `G0` / `W0` / `sharpness` in `manifest.tsv` differ from the dump's | 0 / 0 / 0 | 0 / 0 / 0 |
| Sum of `1/π` over the sampled frames | 32459 | 32459.0 |
| Random (seed 0) weighted `BR` % at q = 5 / 10 / 20 / 30 / 40 | near q | 4.7 / 13.1 / 24.0 / 32.0 / 45.6 |
| Random (seed 0) weighted AUC / partial AUC | near 0.5 | 0.542 / 0.512 |
| `G0` `quantile@95` over the qualifying folders: mean pick recall / non-pick removal % (`reject.md`: 94.7 / 16.8) | 94.7 / 16.8 (`rf.label_free` 94.7 / 16.8) | 94.7 / 16.8 |
| `G0` `quantile@95`: mean frame share removed, qualifying folders / dump frames removed, all folders | - | 15.2% / 4943 (15.2%) |

### Per stratum

| Stratum | Population | Sampled | Bad (sampled) | Weighted bad rate % [stratified 95%] | Unweighted % |
| --- | ---: | ---: | ---: | ---: | ---: |
| `closed` | 2385 | 120 | 11 | 9.2 [4.2, 15.0] | 9.2 |
| `focus_low` | 8545 | 120 | 9 | 7.5 [3.3, 12.5] | 7.5 |
| `away` | 5076 | 120 | 3 | 2.5 [0.0, 5.8] | 2.5 |
| `sharp10` | 1422 | 120 | 26 | 21.7 [15.0, 29.2] | 21.7 |
| `g0_bottom` | 1441 | 150 | 19 | 12.7 [8.0, 18.0] | 12.7 |
| `w0_bottom` | 945 | 70 | 2 | 2.9 [0.0, 7.1] | 2.9 |
| `rest` | 12645 | 500 | 10 | 2.3 [1.1, 3.7] | 2.0 |

### Dump-label consistency

| Reading | Value |
| --- | ---: |
| Dump picks labeled bad: weighted share % / sampled count | 0.00 / 0 of 167 |
| Sampled explicit dump rejects labeled bad | 4 of 6 |
| Weighted bad rate among the dump's non-picks % [stratified 95%] | 6.41 [4.79, 8.22] |

### Main table: bad recall, not-bad loss and precision per cut

Weighted (`1/π`), with the stratified bootstrap 95% interval (1,000 draws, seed 0, within each stratum and each `rest` folder cell) and the unweighted sample value in brackets. `qX`: the folder's bottom X% by the score, cut on the full dump; `q@95`: `G0`'s label-free `quantile@95` share; `rule`: the rule's own point. Removed: the share of the dump's frames the cut removes (on the folders the score covers). Out: sampled frames in folders the score does not cover. Bar: the worth-a-filter bar (`NL` <= 5%, `BR` >= 50% with its 2.5% quantile above 40%, `P` >= 50%). Primary: the score's cut whose `NL` is nearest 5%.

| Score | Cut | Removed % | Out | `BR` % | `NL` % | `P` % | Bar |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `W0` | q5 (primary) | 5.0 | 8 | 13.6 [6.3, 23.7] (17.5) | 4.1 [2.9, 5.4] (5.6) | 16.3 [7.7, 27.7] (18.4) | - |
| `W0` | q10 | 10.0 | 8 | 26.4 [15.4, 40.6] (30.0) | 8.5 [6.7, 10.3] (11.4) | 15.3 [8.9, 23.7] (15.9) | - |
| `W0` | q20 | 20.0 | 8 | 35.4 [23.8, 49.7] (43.8) | 16.8 [14.6, 19.3] (21.0) | 10.9 [6.7, 16.1] (13.1) | - |
| `W0` | q30 | 30.0 | 8 | 51.2 [36.8, 66.7] (63.8) | 26.7 [24.0, 29.7] (31.6) | 10.1 [6.9, 13.6] (12.7) | - |
| `W0` | q40 | 40.0 | 8 | 64.7 [51.5, 79.1] (73.8) | 35.4 [32.2, 38.5] (40.6) | 9.7 [6.7, 13.0] (11.6) | - |
| `W0+off` | q5 (primary) | 5.0 | 8 | 16.2 [8.0, 26.7] (22.5) | 3.5 [2.3, 4.7] (5.0) | 21.4 [11.1, 35.2] (24.3) | - |
| `W0+off` | q10 | 10.0 | 8 | 24.8 [14.6, 37.1] (32.5) | 7.5 [5.8, 9.3] (10.0) | 16.3 [9.6, 25.0] (19.0) | - |
| `W0+off` | q20 | 20.0 | 8 | 37.7 [25.5, 51.9] (48.8) | 17.2 [14.6, 19.9] (21.6) | 11.4 [7.2, 16.3] (14.0) | - |
| `W0+off` | q30 | 30.0 | 8 | 52.8 [38.7, 68.4] (65.0) | 27.4 [24.7, 30.4] (32.6) | 10.1 [6.9, 13.9] (12.6) | - |
| `W0+off` | q40 | 40.0 | 8 | 59.7 [45.0, 74.2] (71.2) | 35.3 [32.4, 38.4] (40.9) | 9.0 [6.2, 12.2] (11.1) | - |
| `F0` | q5 (primary) | 5.0 | 0 | 14.3 [6.9, 24.5] (18.8) | 3.6 [2.4, 4.9] (5.0) | 18.9 [9.3, 32.1] (21.1) | - |
| `F0` | q10 | 10.0 | 0 | 29.8 [19.2, 43.9] (41.2) | 8.4 [6.5, 10.4] (10.4) | 17.1 [10.7, 25.0] (22.0) | - |
| `F0` | q20 | 20.0 | 0 | 40.4 [28.2, 54.6] (53.8) | 17.1 [14.6, 19.6] (21.0) | 12.1 [8.2, 17.0] (15.5) | - |
| `F0` | q30 | 30.0 | 0 | 46.2 [33.3, 61.2] (63.8) | 26.2 [23.5, 29.2] (31.3) | 9.3 [6.4, 12.7] (12.7) | - |
| `F0` | q40 | 40.0 | 0 | 53.5 [40.3, 68.0] (71.2) | 35.7 [32.6, 38.9] (40.8) | 8.0 [5.6, 10.7] (11.1) | - |
| `F0+ctx` | q5 (primary) | 5.0 | 0 | 16.2 [8.6, 26.6] (22.5) | 4.0 [2.7, 5.4] (5.1) | 19.3 [10.2, 31.7] (24.0) | - |
| `F0+ctx` | q10 | 10.0 | 0 | 29.3 [18.8, 43.4] (38.8) | 8.2 [6.5, 10.3] (10.3) | 17.2 [10.7, 26.0] (21.2) | - |
| `F0+ctx` | q20 | 20.0 | 0 | 40.5 [28.2, 55.0] (55.0) | 17.2 [14.8, 19.8] (21.2) | 12.1 [8.1, 16.8] (15.7) | - |
| `F0+ctx` | q30 | 30.0 | 0 | 47.3 [34.3, 61.9] (63.8) | 27.0 [24.2, 30.0] (31.3) | 9.3 [6.2, 12.7] (12.7) | - |
| `F0+ctx` | q40 | 40.0 | 0 | 55.1 [41.1, 70.1] (71.2) | 36.6 [33.5, 39.8] (41.7) | 8.1 [5.6, 10.8] (10.9) | - |
| `P0` | q5 (primary) | 5.0 | 0 | 13.7 [6.1, 24.4] (16.2) | 3.5 [2.4, 4.7] (5.1) | 18.6 [9.0, 31.3] (18.6) | - |
| `P0` | q10 | 10.0 | 0 | 25.0 [14.5, 37.6] (31.2) | 8.2 [6.4, 10.0] (11.1) | 15.1 [8.7, 23.5] (16.8) | - |
| `P0` | q20 | 20.0 | 0 | 40.4 [27.7, 55.1] (53.8) | 17.3 [15.1, 19.8] (21.0) | 12.0 [8.0, 16.5] (15.5) | - |
| `P0` | q30 | 30.0 | 0 | 44.9 [31.9, 59.7] (61.2) | 27.0 [24.1, 30.0] (31.6) | 8.9 [6.1, 12.1] (12.2) | - |
| `P0` | q40 | 40.0 | 0 | 53.5 [40.3, 68.0] (71.2) | 36.6 [33.3, 39.8] (41.4) | 7.9 [5.6, 10.5] (10.9) | - |
| `R0` | q5 (primary) | 5.0 | 0 | 19.3 [10.9, 30.0] (27.5) | 3.7 [2.5, 5.1] (5.1) | 23.1 [13.3, 35.6] (27.8) | - |
| `R0` | q10 | 10.0 | 0 | 28.1 [17.9, 41.5] (37.5) | 7.7 [6.0, 9.6] (9.7) | 17.6 [11.0, 26.3] (21.6) | - |
| `R0` | q20 | 20.0 | 0 | 42.2 [29.1, 56.6] (57.5) | 16.5 [14.1, 18.9] (20.0) | 13.0 [8.9, 18.3] (17.0) | - |
| `R0` | q30 | 30.0 | 0 | 47.8 [34.8, 62.9] (65.0) | 26.2 [23.4, 29.2] (30.6) | 9.6 [6.6, 13.1] (13.2) | - |
| `R0` | q40 | 40.0 | 0 | 60.8 [47.2, 75.8] (73.8) | 37.3 [34.1, 40.4] (41.4) | 8.7 [6.0, 11.8] (11.3) | - |
| `G0` | q5 | 5.0 | 0 | 18.9 [10.6, 29.6] (27.5) | 2.9 [1.9, 4.2] (4.4) | 27.3 [16.1, 41.4] (31.0) | - |
| `G0` | q10 (primary) | 10.0 | 0 | 28.4 [17.4, 42.3] (40.0) | 6.8 [5.3, 8.6] (10.4) | 19.6 [11.8, 28.5] (21.6) | - |
| `G0` | q20 | 20.0 | 0 | 45.0 [32.5, 60.1] (61.2) | 15.5 [13.3, 17.8] (21.9) | 14.5 [10.1, 20.0] (16.7) | - |
| `G0` | q30 | 30.0 | 0 | 58.1 [44.8, 73.9] (70.0) | 26.3 [23.5, 29.2] (31.9) | 11.4 [8.0, 15.4] (13.6) | - |
| `G0` | q40 | 40.0 | 0 | 70.7 [57.1, 83.1] (76.2) | 35.7 [32.8, 38.9] (40.6) | 10.4 [7.2, 13.9] (11.8) | - |
| `rule count` | q5 (primary) | 5.0 | 0 | 5.5 [1.1, 11.0] (6.2) | 4.0 [2.5, 5.5] (2.6) | 7.6 [1.6, 15.3] (14.7) | - |
| `rule count` | q10 | 10.0 | 0 | 17.5 [5.2, 29.8] (10.0) | 9.2 [7.1, 11.4] (6.1) | 10.0 [3.1, 18.2] (10.5) | - |
| `rule count` | q20 | 20.0 | 0 | 40.0 [25.6, 54.0] (25.0) | 19.5 [16.6, 22.1] (13.5) | 10.7 [5.8, 16.3] (11.7) | - |
| `rule count` | q30 | 30.0 | 0 | 57.8 [42.8, 69.9] (41.2) | 28.3 [25.2, 31.1] (21.4) | 10.7 [6.7, 14.9] (12.1) | - |
| `rule count` | q40 | 40.0 | 0 | 65.1 [52.7, 76.5] (51.2) | 36.2 [33.5, 38.9] (27.9) | 9.5 [6.3, 13.0] (11.6) | - |
| `W0 burst-centered (what the app displays)` | q5 (primary) | 5.0 | 8 | 14.9 [6.8, 25.0] (18.8) | 3.7 [2.6, 5.0] (5.5) | 19.1 [8.7, 31.7] (19.7) | - |
| `W0 burst-centered (what the app displays)` | q10 | 10.0 | 8 | 22.3 [13.2, 34.0] (28.8) | 9.4 [7.5, 11.5] (11.7) | 12.2 [6.8, 19.4] (15.0) | - |
| `W0 burst-centered (what the app displays)` | q20 | 20.0 | 8 | 33.6 [21.5, 47.2] (40.0) | 20.0 [17.5, 22.9] (23.7) | 8.9 [5.6, 13.2] (10.8) | - |
| `W0 burst-centered (what the app displays)` | q30 | 30.0 | 8 | 42.3 [29.8, 56.1] (53.8) | 27.7 [24.8, 30.7] (31.7) | 8.2 [5.5, 11.4] (10.9) | - |
| `W0 burst-centered (what the app displays)` | q40 | 40.0 | 8 | 57.8 [44.6, 72.5] (66.2) | 37.3 [34.2, 40.6] (41.5) | 8.3 [5.8, 11.3] (10.3) | - |
| `Random (seed 0)` | q5 (primary) | 5.0 | 0 | 4.7 [1.3, 9.7] (7.5) | 4.8 [3.4, 6.4] (5.0) | 5.4 [1.5, 11.1] (9.7) | - |
| `Random (seed 0)` | q10 | 10.0 | 0 | 13.1 [5.0, 23.5] (15.0) | 8.2 [6.4, 10.1] (8.9) | 8.5 [3.3, 15.2] (10.7) | - |
| `Random (seed 0)` | q20 | 20.0 | 0 | 24.0 [13.3, 36.8] (26.2) | 17.8 [15.3, 20.3] (19.1) | 7.3 [3.8, 11.7] (8.9) | - |
| `Random (seed 0)` | q30 | 30.0 | 0 | 32.0 [19.6, 45.8] (32.5) | 27.6 [24.5, 30.6] (29.0) | 6.3 [3.6, 9.8] (7.4) | - |
| `Random (seed 0)` | q40 | 40.0 | 0 | 45.6 [31.9, 60.4] (42.5) | 39.3 [36.0, 42.5] (40.6) | 6.4 [4.0, 9.2] (7.0) | - |
| `Absolute sharpness` | q5 (primary) | 5.0 | 0 | 16.8 [8.9, 27.4] (23.8) | 2.6 [1.7, 3.6] (4.3) | 27.1 [15.3, 42.4] (28.4) | - |
| `Absolute sharpness` | q10 | 10.0 | 0 | 30.7 [19.4, 43.8] (41.2) | 8.3 [6.7, 10.2] (11.0) | 17.7 [11.1, 25.5] (21.2) | - |
| `Absolute sharpness` | q20 | 20.0 | 0 | 43.6 [29.9, 57.7] (51.2) | 17.9 [15.5, 20.5] (20.3) | 12.5 [8.3, 17.4] (15.3) | - |
| `Absolute sharpness` | q30 | 30.0 | 0 | 60.3 [45.6, 73.3] (60.0) | 29.4 [26.4, 32.7] (30.5) | 10.7 [7.2, 14.5] (12.3) | - |
| `Absolute sharpness` | q40 | 40.0 | 0 | 71.0 [58.5, 82.0] (67.5) | 39.8 [36.7, 43.2] (40.4) | 9.4 [6.6, 12.7] (10.7) | - |
| `eye_focus` (missing last) | q5 (primary) | 5.0 | 0 | 16.2 [10.0, 25.5] (28.8) | 4.2 [3.3, 5.3] (6.8) | 18.3 [11.8, 26.8] (23.2) | - |
| `eye_focus` (missing last) | q10 | 10.0 | 0 | 26.9 [18.9, 38.9] (46.2) | 8.6 [7.1, 10.2] (12.5) | 15.5 [11.0, 21.1] (20.9) | - |
| `eye_focus` (missing last) | q20 | 20.0 | 0 | 41.6 [29.3, 55.9] (60.0) | 18.1 [15.5, 20.8] (20.0) | 11.8 [8.2, 16.1] (17.6) | - |
| `eye_focus` (missing last) | q30 | 30.0 | 0 | 68.9 [56.2, 81.4] (73.8) | 27.7 [24.9, 30.5] (26.0) | 12.7 [9.0, 17.1] (16.9) | - |
| `eye_focus` (missing last) | q40 | 40.0 | 0 | 74.0 [62.5, 84.7] (76.2) | 37.7 [35.1, 40.4] (32.9) | 10.3 [7.4, 13.8] (14.2) | - |
| `photoTier` good | q5 (primary) | 5.0 | 0 | 5.9 [1.9, 11.1] (10.0) | 4.6 [3.0, 6.2] (4.3) | 7.0 [2.3, 13.7] (14.3) | - |
| `photoTier` good | q10 | 10.0 | 0 | 13.7 [5.4, 24.2] (16.2) | 8.0 [6.1, 9.9] (7.9) | 9.1 [3.7, 16.4] (12.9) | - |
| `photoTier` good | q20 | 20.0 | 0 | 27.3 [15.9, 40.5] (30.0) | 17.3 [14.7, 19.8] (16.9) | 8.5 [4.8, 13.1] (11.3) | - |
| `photoTier` good | q30 | 30.0 | 0 | 41.1 [27.4, 55.9] (40.0) | 28.1 [25.1, 31.3] (27.1) | 7.9 [4.9, 11.7] (9.5) | - |
| `photoTier` good | q40 | 40.0 | 0 | 48.8 [34.9, 62.9] (48.8) | 38.0 [34.7, 41.1] (36.9) | 7.0 [4.5, 10.0] (8.6) | - |
| G0 `quantile@95` | q@95 (primary) | 15.2 | 0 | 35.6 [24.5, 50.0] (55.0) | 10.6 [8.9, 12.5] (17.5) | 16.4 [11.0, 23.0] (18.3) | - |
| `eyes closed` | rule (primary) | 7.3 | 0 | 12.2 [5.9, 19.4] (13.8) | 7.1 [6.7, 7.4] (9.7) | 9.2 [4.2, 15.0] (9.2) | - |
| `eye focus low` | rule (primary) | 27.9 | 0 | 43.5 [27.0, 56.7] (20.0) | 27.1 [26.0, 28.1] (11.8) | 8.6 [4.4, 13.3] (10.8) | - |
| `face away` | rule (primary) | 25.7 | 0 | 26.3 [12.2, 39.7] (12.5) | 26.3 [23.9, 28.5] (16.2) | 5.5 [2.2, 9.1] (5.2) | - |
| `sharpness bottom 10%` | rule (primary) | 10.0 | 0 | 30.7 [19.4, 43.8] (41.2) | 8.3 [6.7, 10.2] (11.0) | 17.7 [11.1, 25.5] (21.2) | - |
| `any of` | rule (primary) | 53.7 | 0 | 72.2 [61.0, 81.9] (61.2) | 52.6 [51.8, 53.3] (38.5) | 7.4 [5.0, 10.2] (10.2) | - |
| `not good` | rule (primary) | 82.9 | 0 | 99.5 [98.2, 100.0] (98.8) | 82.8 [80.9, 84.6] (77.2) | 6.6 [4.9, 8.4] (8.4) | - |

### Threshold-free: weighted AUC and partial AUC at not-bad loss <= 10%

Bad vs not bad on the sampled frames, `1/π` weights, pooled over folders (missing values at the folder's lowest finite score minus 1); partial AUC McClish-standardized, chance 0.5. A rule's AUC is that of its keep / remove split. Stratified 95% interval; unweighted in brackets.

| Score | AUC | Partial AUC (`NL` <= 10%) |
| --- | ---: | ---: |
| `W0` | 0.661 [0.581, 0.747] (0.671) | 0.564 [0.524, 0.618] (0.541) |
| `W0+off` | 0.667 [0.585, 0.755] (0.693) | 0.568 [0.529, 0.621] (0.567) |
| `F0` | 0.662 [0.578, 0.752] (0.724) | 0.592 [0.545, 0.654] (0.599) |
| `F0+ctx` | 0.671 [0.591, 0.754] (0.730) | 0.590 [0.545, 0.651] (0.608) |
| `P0` | 0.640 [0.557, 0.728] (0.685) | 0.564 [0.522, 0.618] (0.548) |
| `R0` | 0.683 [0.603, 0.763] (0.732) | 0.570 [0.526, 0.627] (0.557) |
| `G0` | 0.731 [0.657, 0.801] (0.758) | 0.602 [0.555, 0.662] (0.620) |
| `rule count` | 0.634 [0.555, 0.702] (0.622) | 0.538 [0.507, 0.572] (0.535) |
| `W0 burst-centered (what the app displays)` | 0.608 [0.518, 0.698] (0.643) | 0.568 [0.530, 0.620] (0.581) |
| `Random (seed 0)` | 0.542 [0.459, 0.623] (0.530) | 0.512 [0.489, 0.545] (0.522) |
| `Absolute sharpness` | 0.687 [0.610, 0.764] (0.674) | 0.591 [0.548, 0.646] (0.597) |
| `eye_focus` (missing last) | 0.746 [0.696, 0.798] (0.798) | 0.560 [0.532, 0.602] (0.585) |
| `photoTier` good | 0.583 [0.572, 0.593] (0.608) | 0.505 [0.505, 0.506] (0.507) |
| G0 `quantile@95` | 0.625 [0.568, 0.697] (0.688) | 0.562 [0.533, 0.602] (0.556) |
| `eyes closed` | 0.526 [0.492, 0.564] (0.520) | 0.517 [0.495, 0.544] (0.511) |
| `eye focus low` | 0.582 [0.494, 0.654] (0.541) | 0.516 [0.499, 0.531] (0.518) |
| `face away` | 0.500 [0.430, 0.568] (0.482) | 0.500 [0.486, 0.514] (0.494) |
| `sharpness bottom 10%` | 0.612 [0.554, 0.678] (0.651) | 0.568 [0.531, 0.615] (0.573) |
| `any of` | 0.598 [0.538, 0.650] (0.614) | 0.510 [0.504, 0.515] (0.516) |
| `not good` | 0.583 [0.572, 0.593] (0.608) | 0.505 [0.505, 0.506] (0.507) |

### Per cue: weighted bad rate per bin

The weighted bad rate of each bin (stratified 95% interval; sampled frames and bad among them), against the base rate 5.52%. Firing: the bin the cue's rule removes. Catches: the firing bin's rate is >= 2x the base rate and its 2.5% quantile above the base rate.

| Cue | Bin | Sampled | Bad | Weighted bad rate % | Unweighted % | x base | Firing | Catches |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| raw EAR | <= 0.137 (closed) | 120 | 11 | 9.2 [4.2, 15.0] | 9.2 | 1.66 | yes | no |
| raw EAR | (0.137, 0.25] | 189 | 9 | 6.4 [2.5, 11.0] | 4.8 | 1.16 |  |  |
| raw EAR | > 0.25 | 442 | 9 | 3.4 [1.3, 5.9] | 2.0 | 0.61 |  |  |
| raw EAR | missing | 449 | 51 | 6.8 [4.9, 9.2] | 11.4 | 1.24 |  |  |
| `eye_focus` | < 0.5 | 27 | 5 | 9.3 [1.8, 20.3] | 18.5 | 1.69 |  |  |
| `eye_focus` | [0.5, 0.9) | 121 | 11 | 8.4 [3.7, 13.6] | 9.1 | 1.52 |  |  |
| `eye_focus` | < 0.9 (the two lower bins) | 148 | 16 | 8.6 [4.4, 13.3] | 10.8 | 1.55 | yes | no |
| `eye_focus` | >= 0.9 | 818 | 19 | 2.3 [1.3, 3.4] | 2.3 | 0.41 |  |  |
| `eye_focus` | missing (no AF face) | 234 | 45 | 14.6 [10.8, 19.1] | 19.2 | 2.65 |  |  |
| `\|yaw\|` | <= 35 | 561 | 18 | 3.9 [1.9, 6.1] | 3.2 | 0.70 |  |  |
| `\|yaw\|` | > 35 | 186 | 10 | 5.6 [2.2, 9.3] | 5.4 | 1.02 | yes | no |
| `\|yaw\|` | missing | 453 | 52 | 7.4 [5.3, 10.0] | 11.5 | 1.33 |  |  |
| `\|pitch\|` | <= 45 | 728 | 25 | 4.1 [2.4, 6.0] | 3.4 | 0.74 |  |  |
| `\|pitch\|` | > 45 | 19 | 3 | 17.6 [0.0, 40.5] | 15.8 | 3.20 | yes | no |
| `\|pitch\|` | missing | 453 | 52 | 7.4 [5.3, 10.0] | 11.5 | 1.33 |  |  |
| Sharpness decile | 1 | 156 | 33 | 17.7 [11.1, 25.5] | 21.2 | 3.20 | yes | yes |
| Sharpness decile | 2 | 115 | 8 | 7.3 [2.4, 13.3] | 7.0 | 1.32 |  |  |
| Sharpness decile | 3 | 124 | 7 | 7.7 [2.1, 14.5] | 5.6 | 1.40 |  |  |
| Sharpness decile | 4 | 113 | 6 | 5.7 [0.9, 12.3] | 5.3 | 1.03 |  |  |
| Sharpness decile | 5 | 99 | 3 | 1.0 [0.0, 2.4] | 3.0 | 0.19 |  |  |
| Sharpness decile | 6 | 103 | 8 | 7.2 [1.7, 13.6] | 7.8 | 1.31 |  |  |
| Sharpness decile | 7 | 112 | 1 | 1.0 [0.0, 3.0] | 0.9 | 0.17 |  |  |
| Sharpness decile | 8 | 120 | 5 | 3.9 [0.9, 7.9] | 4.2 | 0.71 |  |  |
| Sharpness decile | 9 | 129 | 2 | 0.6 [0.0, 1.7] | 1.6 | 0.11 |  |  |
| Sharpness decile | 10 | 129 | 7 | 3.1 [1.0, 6.9] | 5.4 | 0.57 |  |  |
| Sharpness decile | missing | 0 | 0 | - | - | - |  | - |
| `tier` | good | 256 | 1 | 0.2 [0.0, 0.6] | 0.4 | 0.03 |  |  |
| `tier` | not good | 944 | 79 | 6.6 [4.9, 8.4] | 8.4 | 1.19 | yes | no |

### Worth-a-filter bar

Per score: its primary cut, the best `BR` among its cuts with `NL` <= 5%, and whether any of its cuts passes the bar.

| Score | Primary cut | Primary `BR` / `NL` / `P` % | Best `BR` % with `NL` <= 5% | Passes at |
| --- | ---: | ---: | ---: | ---: |
| `W0` | q5 | 13.6 / 4.1 / 16.3 | q5: 13.6 [6.3, 23.7] | no |
| `W0+off` | q5 | 16.2 / 3.5 / 21.4 | q5: 16.2 [8.0, 26.7] | no |
| `F0` | q5 | 14.3 / 3.6 / 18.9 | q5: 14.3 [6.9, 24.5] | no |
| `F0+ctx` | q5 | 16.2 / 4.0 / 19.3 | q5: 16.2 [8.6, 26.6] | no |
| `P0` | q5 | 13.7 / 3.5 / 18.6 | q5: 13.7 [6.1, 24.4] | no |
| `R0` | q5 | 19.3 / 3.7 / 23.1 | q5: 19.3 [10.9, 30.0] | no |
| `G0` | q10 | 28.4 / 6.8 / 19.6 | q5: 18.9 [10.6, 29.6] | no |
| `rule count` | q5 | 5.5 / 4.0 / 7.6 | q5: 5.5 [1.1, 11.0] | no |
| `W0 burst-centered (what the app displays)` | q5 | 14.9 / 3.7 / 19.1 | q5: 14.9 [6.8, 25.0] | no |
| `Random (seed 0)` | q5 | 4.7 / 4.8 / 5.4 | q5: 4.7 [1.3, 9.7] | no |
| `Absolute sharpness` | q5 | 16.8 / 2.6 / 27.1 | q5: 16.8 [8.9, 27.4] | no |
| `eye_focus` (missing last) | q5 | 16.2 / 4.2 / 18.3 | q5: 16.2 [10.0, 25.5] | no |
| `photoTier` good | q5 | 5.9 / 4.6 / 7.0 | q5: 5.9 [1.9, 11.1] | no |
| G0 `quantile@95` | q@95 | 35.6 / 10.6 / 16.4 | - | no |
| `eyes closed` | rule | 12.2 / 7.1 / 9.2 | - | no |
| `eye focus low` | rule | 43.5 / 27.1 / 8.6 | - | no |
| `face away` | rule | 26.3 / 26.3 / 5.5 | - | no |
| `sharpness bottom 10%` | rule | 30.7 / 8.3 / 17.7 | - | no |
| `any of` | rule | 72.2 / 52.6 / 7.4 | - | no |
| `not good` | rule | 99.5 / 82.8 / 6.6 | - | no |

Best existing filter (highest weighted `BR` among the cuts with `NL` <= 5%): `R0` at `q5`, `BR` 19.3 [10.9, 30.0]%, `NL` 3.7 [2.5, 5.1]%, `P` 23.1 [13.3, 35.6]%: does not pass the worth-a-filter bar.

### Paired differences at the primary cut

Score minus comparison, in points, with the stratified bootstrap 95% interval (the same draws for both). A graded score and a graded comparison are read at the score's primary cut; a rule or `G0` `quantile@95` against the comparison at the comparison's primary cut. Frames in folders either score does not cover are left out of both.

| Comparison | `BR`, points | `NL`, points | `P`, points |
| --- | ---: | ---: | ---: |
| `W0` (q5) - `G0` (q5) | -5.3 [-17.0, +6.9] | +1.1 [-0.3, +2.5] | -11.0 [-25.9, +3.6] |
| `W0+off` (q5) - `G0` (q5) | -2.8 [-14.9, +8.9] | +0.5 [-0.7, +1.7] | -5.9 [-20.5, +8.7] |
| `F0` (q5) - `G0` (q5) | -4.6 [-10.1, -0.1] | +0.6 [-0.6, +1.8] | -8.4 [-17.5, -0.2] |
| `F0+ctx` (q5) - `G0` (q5) | -2.7 [-7.2, +1.7] | +1.0 [-0.5, +2.5] | -8.0 [-18.0, +0.9] |
| `P0` (q5) - `G0` (q5) | -5.2 [-11.9, +0.2] | +0.6 [-0.8, +1.9] | -8.7 [-19.7, +1.5] |
| `R0` (q5) - `G0` (q5) | +0.4 [-3.8, +5.0] | +0.8 [-0.6, +2.1] | -4.2 [-14.2, +4.8] |
| `rule count` (q5) - `G0` (q5) | -13.4 [-24.5, -4.5] | +1.0 [-0.9, +2.8] | -19.8 [-34.8, -6.6] |
| `W0 burst-centered (what the app displays)` (q5) - `G0` (q5) | -4.0 [-16.0, +8.6] | +0.7 [-0.6, +2.1] | -8.2 [-24.3, +6.4] |
| `Random (seed 0)` (q5) - `G0` (q5) | -14.2 [-25.8, -4.5] | +1.9 [-0.0, +3.8] | -21.9 [-37.1, -8.8] |
| `Absolute sharpness` (q5) - `G0` (q5) | -2.1 [-9.2, +5.5] | -0.3 [-1.8, +1.2] | -0.2 [-13.3, +13.5] |
| `eye_focus` (missing last) (q5) - `G0` (q5) | -2.7 [-15.2, +7.9] | +1.3 [-0.1, +2.7] | -9.0 [-24.0, +3.6] |
| `photoTier` good (q5) - `G0` (q5) | -13.0 [-24.7, -3.2] | +1.6 [-0.4, +3.6] | -20.4 [-36.1, -6.8] |
| G0 `quantile@95` (q@95) - `G0` (q10) | +7.2 [+3.7, +11.9] | +3.8 [+2.8, +4.9] | -3.1 [-7.2, +0.1] |
| `eyes closed` (rule) - `G0` (q10) | -16.2 [-30.0, -5.3] | +0.2 [-1.5, +1.8] | -10.4 [-20.1, -2.7] |
| `eye focus low` (rule) - `G0` (q10) | +15.1 [-5.1, +31.3] | +20.3 [+18.3, +22.1] | -11.0 [-19.0, -3.8] |
| `face away` (rule) - `G0` (q10) | -2.1 [-26.6, +17.7] | +19.5 [+16.4, +22.2] | -14.0 [-23.6, -5.6] |
| `sharpness bottom 10%` (rule) - `G0` (q10) | +2.3 [-11.2, +15.6] | +1.5 [-0.9, +3.9] | -1.9 [-11.0, +6.8] |
| `any of` (rule) - `G0` (q10) | +43.8 [+26.7, +57.6] | +45.8 [+43.9, +47.4] | -12.1 [-20.2, -4.6] |
| `not good` (rule) - `G0` (q10) | +71.1 [+57.1, +82.1] | +76.0 [+73.6, +78.3] | -13.0 [-21.3, -5.5] |
| `W0+off` (q5) - `W0` (q5) | +2.5 [-8.3, +14.3] | -0.6 [-1.5, +0.3] | +5.1 [-6.7, +18.2] |
| `F0` (q5) - `W0` (q5) | +0.7 [-10.2, +11.8] | -0.5 [-1.8, +0.8] | +2.6 [-10.2, +16.2] |
| `F0+ctx` (q5) - `W0` (q5) | +2.6 [-9.0, +14.4] | -0.1 [-1.6, +1.4] | +2.9 [-10.6, +16.1] |
| `P0` (q5) - `W0` (q5) | +0.0 [-11.5, +11.8] | -0.6 [-1.8, +0.6] | +2.3 [-11.4, +15.1] |
| `R0` (q5) - `W0` (q5) | +5.6 [-6.3, +18.1] | -0.3 [-1.8, +1.1] | +6.8 [-7.1, +20.2] |
| `G0` (q10) - `W0` (q10) | +2.0 [-7.4, +10.6] | -1.7 [-3.4, +0.0] | +4.3 [-2.1, +10.2] |
| `rule count` (q5) - `W0` (q5) | -8.1 [-18.8, +1.4] | -0.1 [-2.1, +2.0] | -8.8 [-21.5, +2.5] |
| `W0 burst-centered (what the app displays)` (q5) - `W0` (q5) | +1.2 [-10.4, +13.8] | -0.4 [-1.7, +1.0] | +2.8 [-10.8, +15.4] |
| `Random (seed 0)` (q5) - `W0` (q5) | -9.0 [-19.6, -0.5] | +0.7 [-1.3, +2.7] | -11.0 [-24.0, -1.5] |
| `Absolute sharpness` (q5) - `W0` (q5) | +3.1 [-9.1, +15.5] | -1.4 [-3.0, +0.2] | +10.7 [-4.9, +28.1] |
| `eye_focus` (missing last) (q5) - `W0` (q5) | +2.6 [-8.4, +12.7] | +0.2 [-1.5, +1.7] | +1.9 [-10.8, +12.4] |
| `photoTier` good (q5) - `W0` (q5) | -7.8 [-18.1, +0.9] | +0.5 [-1.6, +2.6] | -9.4 [-22.7, +1.2] |
| G0 `quantile@95` (q@95) - `W0` (q5) | +22.0 [+7.8, +37.8] | +6.5 [+4.9, +8.3] | +0.1 [-11.9, +9.7] |
| `eyes closed` (rule) - `W0` (q5) | -1.4 [-12.8, +7.4] | +3.0 [+1.6, +4.3] | -7.2 [-19.4, +1.0] |
| `eye focus low` (rule) - `W0` (q5) | +29.9 [+10.1, +45.2] | +23.1 [+21.4, +24.7] | -7.8 [-19.1, +0.9] |
| `face away` (rule) - `W0` (q5) | +12.7 [-2.5, +27.0] | +22.2 [+19.7, +24.8] | -10.8 [-22.1, -2.6] |
| `sharpness bottom 10%` (rule) - `W0` (q5) | +17.1 [+1.8, +32.6] | +4.3 [+2.1, +6.6] | +1.3 [-12.2, +12.8] |
| `any of` (rule) - `W0` (q5) | +58.6 [+44.0, +70.8] | +48.6 [+47.0, +50.0] | -8.9 [-20.1, -0.8] |
| `not good` (rule) - `W0` (q5) | +85.8 [+75.5, +93.1] | +78.7 [+76.6, +81.0] | -9.8 [-21.1, -1.7] |

### Folder-cluster sensitivity

The 1,000-draw folder-cluster bootstrap (resample the 27 folders, seed 0) beside the stratified interval, for the base rate, each score at its primary cut and each cue's firing bin.

| Reading | Cut | `BR` % (stratified) | Cluster | `NL` % (stratified) | Cluster | `P` % (stratified) | Cluster | Bar |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Bad base rate | - | 5.5 [4.2, 7.0] | [3.7, 7.6] |  |  |  |  |  |
| `W0` | q5 | 13.6 [6.3, 23.7] | [4.3, 27.4] | 4.1 [2.9, 5.4] | [3.0, 5.0] | 16.3 [7.7, 27.7] | [6.4, 26.9] | - |
| `W0+off` | q5 | 16.2 [8.0, 26.7] | [5.9, 31.8] | 3.5 [2.3, 4.7] | [2.5, 4.3] | 21.4 [11.1, 35.2] | [11.0, 33.5] | - |
| `F0` | q5 | 14.3 [6.9, 24.5] | [6.9, 24.5] | 3.6 [2.4, 4.9] | [2.7, 4.4] | 18.9 [9.3, 32.1] | [11.2, 27.2] | - |
| `F0+ctx` | q5 | 16.2 [8.6, 26.6] | [8.6, 27.6] | 4.0 [2.7, 5.4] | [2.9, 5.1] | 19.3 [10.2, 31.7] | [12.1, 26.6] | - |
| `P0` | q5 | 13.7 [6.1, 24.4] | [7.4, 22.6] | 3.5 [2.4, 4.7] | [2.4, 4.6] | 18.6 [9.0, 31.3] | [9.8, 30.8] | - |
| `R0` | q5 | 19.3 [10.9, 30.0] | [11.5, 30.7] | 3.7 [2.5, 5.1] | [2.6, 5.1] | 23.1 [13.3, 35.6] | [14.0, 31.4] | - |
| `G0` | q10 | 28.4 [17.4, 42.3] | [16.1, 45.9] | 6.8 [5.3, 8.6] | [4.9, 9.1] | 19.6 [11.8, 28.5] | [11.4, 29.4] | - |
| `rule count` | q5 | 5.5 [1.1, 11.0] | [0.0, 9.5] | 4.0 [2.5, 5.5] | [2.5, 5.9] | 7.6 [1.6, 15.3] | [0.0, 19.0] | - |
| `W0 burst-centered (what the app displays)` | q5 | 14.9 [6.8, 25.0] | [6.2, 28.7] | 3.7 [2.6, 5.0] | [2.7, 4.6] | 19.1 [8.7, 31.7] | [9.7, 30.2] | - |
| `Random (seed 0)` | q5 | 4.7 [1.3, 9.7] | [0.8, 10.9] | 4.8 [3.4, 6.4] | [3.9, 6.0] | 5.4 [1.5, 11.1] | [1.1, 10.9] | - |
| `Absolute sharpness` | q5 | 16.8 [8.9, 27.4] | [9.8, 26.5] | 2.6 [1.7, 3.6] | [1.8, 3.5] | 27.1 [15.3, 42.4] | [19.4, 33.8] | - |
| `eye_focus` (missing last) | q5 | 16.2 [10.0, 25.5] | [10.3, 23.1] | 4.2 [3.3, 5.3] | [2.7, 5.9] | 18.3 [11.8, 26.8] | [11.3, 26.0] | - |
| `photoTier` good | q5 | 5.9 [1.9, 11.1] | [1.4, 13.0] | 4.6 [3.0, 6.2] | [3.6, 5.7] | 7.0 [2.3, 13.7] | [1.9, 14.5] | - |
| G0 `quantile@95` | q@95 | 35.6 [24.5, 50.0] | [21.7, 55.5] | 10.6 [8.9, 12.5] | [8.9, 12.4] | 16.4 [11.0, 23.0] | [10.9, 22.4] | - |
| `eyes closed` | rule | 12.2 [5.9, 19.4] | [6.3, 18.6] | 7.1 [6.7, 7.4] | [5.6, 9.1] | 9.2 [4.2, 15.0] | [4.1, 14.9] | - |
| `eye focus low` | rule | 43.5 [27.0, 56.7] | [27.6, 56.3] | 27.1 [26.0, 28.1] | [19.0, 32.6] | 8.6 [4.4, 13.3] | [4.1, 14.8] | - |
| `face away` | rule | 26.3 [12.2, 39.7] | [7.7, 40.6] | 26.3 [23.9, 28.5] | [22.8, 29.6] | 5.5 [2.2, 9.1] | [1.4, 10.4] | - |
| `sharpness bottom 10%` | rule | 30.7 [19.4, 43.8] | [19.3, 42.9] | 8.3 [6.7, 10.2] | [7.0, 9.4] | 17.7 [11.1, 25.5] | [10.5, 25.2] | - |
| `any of` | rule | 72.2 [61.0, 81.9] | [63.2, 79.4] | 52.6 [51.8, 53.3] | [46.6, 56.5] | 7.4 [5.0, 10.2] | [4.6, 10.6] | - |
| `not good` | rule | 99.5 [98.2, 100.0] | [98.1, 100.0] | 82.8 [80.9, 84.6] | [79.1, 85.8] | 6.6 [4.9, 8.4] | [4.4, 9.0] | - |

| Cue firing bin | Bin | Weighted bad rate % (stratified) | Cluster | Catches |
| --- | ---: | ---: | ---: | ---: |
| raw EAR | <= 0.137 (closed) | 9.2 [4.2, 15.0] | [4.1, 14.9] | no |
| `eye_focus` | < 0.9 (the two lower bins) | 8.6 [4.4, 13.3] | [4.1, 14.8] | no |
| `\|yaw\|` | > 35 | 5.6 [2.2, 9.3] | [1.4, 10.7] | no |
| `\|pitch\|` | > 45 | 17.6 [0.0, 40.5] | [0.0, 38.4] | no |
| Sharpness decile | 1 | 17.7 [11.1, 25.5] | [10.5, 25.2] | yes |
| `tier` | not good | 6.6 [4.9, 8.4] | [4.4, 9.0] | no |

### Wall time

| Stage | Seconds |
| --- | ---: |
| Load (dump, stored scores, rules, labels) | 4 |
| Cuts, metrics and bootstraps | 51 |

Total 55 s.

### Reading

The facts, without the Decision (Step 5 writes it against the bars):

- The weighted bad base rate of the dump is **5.52%** [4.15, 7.04]
  (folder-cluster [3.71, 7.61]; unweighted 6.67%, 80 of 1,200), just above
  the 5% the Pre-registration names as "very low"; its lower bound is below
  it. Among the dump's non-picks the weighted bad rate is 6.41% [4.79,
  8.22]: about 94% of the non-picks the earlier criterion counted as
  negatives are not bad. No sampled pick is bad (0 of 167); 4 of the 6
  sampled explicit rejects are.
- **No score, baseline or rule passes the worth-a-filter bar at any cut.**
  The highest weighted `P` at any cut is 27.3% (`G0` at q5), against the
  50% floor; `BR` reaches 50% only at q30 / q40 or with `any of` / `not
  good`, all at `NL` above 25%.
- The best existing filter (highest `BR` with `NL` <= 5%) is **`R0` at
  q5: `BR` 19.3% [10.9, 30.0], `NL` 3.7% [2.5, 5.1], `P` 23.1% [13.3,
  35.6]**; next `G0` at q5 (18.9% / 2.9% / 27.3%), absolute sharpness at q5
  (16.8% / 2.6% / 27.1%), `eye_focus` and `F0+ctx` at q5 (16.2%); random at
  q5 reads 4.7% / 4.8% / 5.4%. `G0`'s `quantile@95` cut removes 15.2% of the
  dump: `BR` 35.6%, `NL` 10.6%, `P` 16.4%.
- Threshold-free, the highest weighted AUCs are `eye_focus` (missing last)
  0.746 [0.696, 0.798] and `G0` 0.731 [0.657, 0.801]; the highest partial
  AUC at `NL` <= 10% is `G0`'s 0.602 [0.555, 0.662]. Random reads 0.542 /
  0.512.
- At q5 the paired differences against `G0` have intervals spanning zero
  for every learned score except `F0` (−4.6 points `BR` [−10.1, −0.1]);
  `rule count`, `photoTier` good and random are below `G0`. Against `W0` at
  q5 no graded score's `BR` difference excludes zero except random's
  (below); `G0`'s `quantile@95` cut and the rules `sharpness bottom 10%`
  and `eye focus low` remove more bad frames than `W0` at q5, at a higher
  `NL`.
- **Per cue, only the sharpness decile 1 catches bad photos** by the
  pre-registered rule: 17.7% [11.1, 25.5], 3.20x the base rate
  (folder-cluster lower bound 10.5%, also above it). The other firing bins
  do not: raw EAR <= 0.137 9.2% (1.66x), `eye_focus` < 0.9 8.6% (1.55x),
  `|yaw|` > 35 5.6% (1.02x), `|pitch|` > 45 17.6% (3.20x, but 3 of 19
  sampled frames, interval [0.0, 40.5]), `tier` not good 6.6% (1.19x).
  Outside the firing bins: frames with no AF face (`eye_focus` missing) read
  14.6% [10.8, 19.1] (2.65x); `tier` good reads 0.2% (1 of 256 sampled);
  the sharpness deciles above 1 read 0.6–7.7%.
