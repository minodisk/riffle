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
