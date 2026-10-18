# Discard filter evaluation

## Pre-registration

Written 2026-10-18, after Step 1 drew the sample and copied it into
`D:\Photos\tests\2026-10-18-discard-labels\sample\`, **before the user has
marked any frame of it and before any label is read**: when this section was
written the user had not started labeling, `sample\` held the 600 copies and
no `.xmp` / `.dop` file (its file names were listed, none opened), so no
discard label existed anywhere. It is merged on its own, before `labels.py`
or `discardfilter.py` exists; the merge commit of the PR that adds this
section, without results, is the record, as for the archived
[bad-photo filter evaluation](../_archived/20261017-bad-photo-labels/badfilter.md)
and the
[reject-filter evaluation](../_archived/20261016-reject-filter-eval/reject.md)
before it. Nothing in this section changes after the labels are in; a
deviation found while implementing Steps 3–5 is recorded in `learnings.md`
with its reason and does not touch the bars.

The question: on blind labels of **"OK to discard?"** (the human-rescue
scheme of `labeling.md`: mark a frame when it should not be discarded, when
unsure mark it), not of "bad" and not of "not picked", does `G0`'s per-folder
bottom cut remove frames the user would not miss, at which depth, at what
cost in rescued and lost frames, and how does it compare with the other
scores and with random? Is "non-pick ≈ OK to discard" a usable reading? Which
follow-up, if any, deserves its own plan?

### What was known before this section

This pre-registration is **not blind** to the scores or to an earlier,
unblinded reading of the same question. On the same dump of 27 folders and
32,459 frames:

- the archived
  [bad-photo-labels](../_archived/20261017-bad-photo-labels/plan.md) plan,
  on its **strict** labels ("you would never keep this frame on its own; when
  unsure, do not reject", 1,200 frames, 80 strict-bad): strict base rate
  5.52% [4.15, 7.04]; `G0` q5 `BR` 18.9% / `NL` 2.9% / `P` 27.3%; the
  `g0_bottom` stratum (the folder's bottom 15% by `G0`) strict-bad rate
  12.7%; the strict-bad rate among the dump's non-picks 6.41%, so the
  non-pick criterion overstated **strict** bad about fifteenfold; no usable
  bad-photo filter (best `R0` / `G0` at q5, `P` 23–27% against a 50% bar);
- Decision 2 of the archived
  [reject-filter evaluation](../_archived/20261016-reject-filter-eval/plan.md):
  `G0`'s label-free `quantile@95` cut keeps 94.7% of the picks (mean over
  the qualifying folders), removes 16.8% of the non-picks and 15.2% of the
  dump's frames;
- **the unblinded 70/71 reading**: after that plan, the 49 frames of its
  1,200-frame sample that `G0` q5 (full dump) removes but the user had left
  unflagged were given a Red label and reviewed; the user said all but one
  (`0935_2026-07-31__DSC6292.ARW`) were fine to discard, so `G0` q5 read
  about 70 of 71 "OK to discard" in that sample. The user knew the frames
  were the model's candidates; the number is a hypothesis (Bars), not a
  measurement.

Also known, from Step 1 (label-free; `strata.tsv` and `sample.out.md` in the
data folder): the population is the dump minus the 1,200 frames of the
strict sample, **31,259 frames**; the bands are `G0`'s per-folder bottom
`floor(q * n / 100)` frames on the full dump, ties by the seed-0 draw:

| Band | Dump frames | Removed by the exclusion | Population | Drawn | `π` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `b05` (bottom 5%) | 1,609 | 71 | 1,538 | 150 | 0.0975 |
| `b10` (5–10%) | 1,625 | 77 | 1,548 | 150 | 0.0969 |
| `b20` (10–20%) | 3,248 | 146 | 3,102 | 150 | 0.0484 |
| `control` (above 20%, per folder, floor 3) | 25,977 | 906 | 25,071 | 150 | 0.0042–0.43 per folder, 0.00598 overall |
| Total | 32,459 | 1,200 | 31,259 | 600 | |

The pick share of each band and the pick loss of each `G0` cut, label-free:

| Band | Pick share of the band, dump / population | Share of all picks, dump / population |
| --- | ---: | ---: |
| `b05` | 1.86% / 1.89% | 0.68% / 0.68% |
| `b10` | 3.88% / 3.68% | 1.43% / 1.34% |
| `b20` | 4.19% / 4.13% | 3.08% / 3.01% |
| `control` | 16.13% / 16.11% | 94.82% / 94.97% |

| Cut | Removed share, dump / population | Pick loss, dump / population |
| --- | ---: | ---: |
| `G0` q5 | 4.96% / 4.92% | 0.68% / 0.68% |
| `G0` q10 | 9.96% / 9.87% | 2.10% / 2.02% |
| `G0` q20 | 19.97% / 19.80% | **5.18% / 5.03%** |

So two of the bars below are already decided for two `G0` cuts **before any
label**, and are stated here so no one reads them as a result:

- **q20's pick loss sits at the 5% bar**, 5.18% on the full dump (5.03% on
  the population), above it on either definition: `G0` q20 fails the `PL`
  bar whatever the labels say.
- **q5 removes 4.96% of the full dump's frames (4.92% of the
  population's)**, under the ">= 5% of frames" condition, because
  `floor(q * n / 100)` rounds each folder down: applied as written, `G0` q5
  fails the removed-share condition whatever the labels say. Its labels are
  still read for every other metric, the hypothesis and the follow-up rule.

So among `G0`'s three `q` cuts only q10 can pass; `G0`'s `quantile@95`
cut (15.2% of the dump's frames) is the fourth candidate, its pooled pick
loss computed in Step 4 (Decision 2's mean per-folder pick recall 94.7%
puts it near the bar).

The sample holds 39 picks, 5 explicit rejects and 34 singles, and every
folder (3 to 96 frames).

Not known before this section, with no prior number: any discard label, the
discardable base rate, and any score's rescue rate or recall on "OK to
discard" measured blind.

The bars below, the semantics behind them, the cuts, the sample size and the
labeling key were **chosen by the user on 2026-10-18 before any label**
(the plan's "Choices"), not tuned to anything above.

### Labels and population

- **`keep`**: the copy's sidecar carries rating 1 (XMP `xmp:Rating` 1 or
  `.dop` `Rating` 1, the `.dop` deciding when both exist, as `labels.py`
  reads them in Step 3). **`discard`**: reviewed and unmarked; the stopping
  rule of `labeling.md` (every frame viewed once, none skipped) makes
  "unmarked" a label, and "when unsure, keep" puts the doubt on the rescue
  side, so `RR` is the conservative reading.
- A Pick, a Reject, a color label or a rating other than 1 found in a
  sidecar is **listed as unexpected** and the frame counts by the rating-1
  mark alone.
- **Population**: the dump minus the strict sample, 31,259 frames. Every
  estimate is reported **weighted by the inverse inclusion probability**
  (Horvitz–Thompson, the manifest's `inclusion_prob`; a share is the ratio
  of the `1/π` sums over its numerator and denominator sets), with the
  **unweighted sample value** beside it.
- **The dump's `pick` / `reject` labels are consistency checks and the
  pick-loss metric's input, never a target**: the weighted share of sampled
  picks labeled `discard` (expected low), the count of the 5 sampled
  explicit rejects labeled `keep`.
- Frames skipped in Step 3 (`skip.txt`, with the reason) leave the sample;
  their `π` is not re-normalized, and the deviation is reported.

### Unit

Every sampled frame is a sample. No per-folder means (3 to 96 sampled frames
per folder); the folder enters only as the cluster of the folder-cluster
bootstrap (Intervals).

### Cuts

- Every graded score is cut **per folder at a quantile** on the **full
  dump**: remove the bottom `floor(q * n / 100)` of the folder's frames by
  the score, ties by the seed-0 draw (`badfilter.graded_cut`), **`q` = 5 /
  10 / 20**. For `G0` these are the bands' boundaries: q5 removes `b05`, q10
  `b05` + `b10`, q20 the three bands.
- Plus **`G0`'s label-free `quantile@95`** cut (`badfilter.g0_q95_cut`, as
  the archived plan read it).
- **Comparison scores**, on the same `q` grid: `W0`, `R0`, absolute
  sharpness, `eye_focus` (missing last) and random (seed 0). A frame outside
  `G0`'s deepest band (a `control`) can still be removed by a comparison
  score's cut, which is why the controls are labeled too. A score missing on
  a folder (`W0` on `2026-07-23`) leaves that folder's sampled frames out of
  that score's rows, and the count is printed.
- The cut is computed on the full dump and then read on the sampled frames;
  the label-free quantities (removed share, pick loss) are exact counts, not
  estimates.

### Primary metrics

Per cut, weighted, with the unweighted value and the sampled counts beside
them:

1. **Discard precision `DP@q`**: the share of removed frames labeled
   `discard`.
2. **Rescue rate `RR@q`** = 1 − `DP@q`: the share of removed frames the
   user would keep.
3. **Pick loss `PL@q`**: the share of the dump's picks inside the cut,
   label-free. **The bar reads it on the full dump** (the 32,459 frames and
   4,419 picks, the strict sample included), with the population value
   (31,259 frames, 4,252 picks) beside it. Reason: the cut is defined on the
   full dump, and a discard cut shipped in the app acts on every frame of a
   folder; the strict sample's exclusion is a property of this plan's
   labeling sample, not of the library the cut would hide picks from. The
   full-dump value is also the higher of the two at every `G0` cut (q20
   5.18% vs 5.03%), so it is the conservative side. Beside it, the weighted
   share of sampled picks labeled `discard` (a consistency reading: a pick
   should rarely be OK to discard).
4. **Discard recall `DR@q`** and **miss rate `MR@q`** = 1 − `DR@q`: the
   share of `discard` frames the cut removes / leaves.
5. **Discardable share outside the cut `DO@q`**: the `discard` rate among
   the kept frames (what the human still wades through).

Plus the **removed share** of each cut: the share of the **population's**
frames inside the cut, as the bar states it, exact and label-free, with the
full dump's beside it (the two agree on every `G0` verdict: q5 4.92% /
4.96%, q10 9.87% / 9.96%, q20 19.80% / 19.97%). And, threshold-free, the
**weighted AUC** of `discard` vs `keep` per score,
`roc_auc_score(discard, -score, sample_weight=1/π)`, missing values as the
archived `pAUC95` handles them (the folder's lowest finite score minus 1).
The **discardable base rate** of the population, with its interval, is the
first number reported.

### Comparisons

- **Paired differences** (the same bootstrap draws) of `DP` and `DR` between
  `G0` and each comparison score at the same `q`, and `G0` against random.
- `G0`'s `quantile@95` cut is read against random at the same removed share
  only through the `q` grid (no extra random cut).

### The non-pick framing re-read

- The weighted `discard` rate among the dump's non-picks and among its
  picks.
- The share of the non-picks removed by `G0`'s `quantile@95` cut (Decision
  2's `RN`) that are `discard`.
- Per band, the `discard` rate beside the band's pick share (the table
  above), so the answer to "is non-pick ≈ OK to discard?" is a number with
  an interval.

### The secondary label set

The strict labels of bad-photo-labels
(`D:\Photos\tests\2026-10-17-bad-labels\labels.tsv`), read as the
**strict-bad rate per `G0` band**, weighted by that sample's own `π`
(`manifest.tsv` there), with its stratified interval. Expected if the two
label sets are coherent: a band's strict-bad rate **below** its `discard`
rate (strict bad is the narrower question). This is a **check, not a bar**;
an inversion is reported and discussed, not acted on. The strict sample and
this sample share no frame, so the two rates come from disjoint frames.

### Intervals

- A 1,000-draw **stratified bootstrap** over the sampled frames: resample
  with replacement within each band (`b05`, `b10`, `b20`) and each
  (`control`, folder) cell, its drawn count kept, so the design's cell sizes
  and the total `1/π` are kept; seed 0; **one count matrix reused across
  quantities**, so paired differences share draws; the cuts and the scores
  held fixed. Reported as the 2.5% / 97.5% quantiles.
- A **folder-cluster bootstrap** (resample the 27 folders with replacement,
  seed 0, 1,000 draws) as a sensitivity reading beside it, for the base
  rate, the bars' quantities and the paired differences. It does not
  decide; a pass whose cluster interval crosses the bar is reported as
  fragile.

### Bars

**"Hide / sort last" semantics**, chosen by the user on 2026-10-18 before
any label: a discard cut hides its frames from what the user picks from, or
sorts them last, and they stay recoverable. Rejected: **"delete"** semantics
(the cut's frames go to the Trash), whose bars would have been `RR` <= **5%**
with the stratified 97.5% quantile <= **15%** and `PL` <= **1%**, the other
conditions the same.

- **Usable discard cut**: a cut of `G0` (q5, q10, q20 or `quantile@95`) is
  usable when all of these hold:
  - weighted `RR` <= **15%**, with the stratified bootstrap 97.5% quantile
    of `RR` <= **25%**;
  - `PL` <= **5%** (full dump, label-free, Primary metrics 3);
  - it removes **>= 5%** of the population's frames (label-free);
  - its weighted `DP` exceeds random's at the same `q`, with the paired
    stratified bootstrap **2.5% quantile above zero** (for `quantile@95`,
    against random at q20, the nearest grid cut at or above its removed
    share).
- **Best cut**: the deepest passing one (the largest removed share).
- **Follow-up rule**, applied in this order:
  1. a usable cut exists → the Decision recommends a later plan for an
     **in-app discard candidates view with a rescue loop** at the best cut's
     depth;
  2. no usable cut, but `RR` at the shallowest cut (`G0` q5) is within
     **2x the `RR` bar** (weighted `RR@5` <= 30%) and `DO@5` >= **20%** (the
     model misses much that is discardable: something to learn) → the
     Decision recommends a later plan that **trains on the rescue labels
     with the controls as negatives** (never on candidates alone);
  3. otherwise → **nothing**.
- **Hypothesis from the unblinded reading**: `RR@5` <= **5%** (`G0` q5,
  about 1 in 71 rescued). Stated as **confirmed** when the weighted `RR@5`
  is <= 5%, **not confirmed** otherwise, with its stratified interval
  beside it. It is not a bar and does not enter the follow-up rule; a blind
  `RR@5` far above 5% is itself a finding about automation bias in the
  unblinded review.

Applied as:

- **Known before labels** (above): `G0` q20 fails the `PL` bar (5.18%) and
  `G0` q5 fails the removed-share condition (4.96% / 4.92%); both are
  reported with their labeled metrics anyway. The bars are not moved for
  either.
- The comparison scores are not candidates for the usable-cut bar; they are
  read through the paired differences only.
- **A low discardable base rate** makes `DP` hard to raise at any cut; if it
  occurs, that is itself a finding ("the library has few frames the user
  would not miss") and is stated, not a reason to move a bar.
- **Multiplicity**: about 6 scores (`G0`, `W0`, `R0`, sharpness,
  `eye_focus`, random), 4 cuts and a few readings (the non-pick framing, the
  secondary labels, the hypothesis) are read without correction, so a chance
  pass is possible; the Decision reads a lone marginal pass as such.
- **Population caveat**: one photographer's 27 folders, mostly basketball,
  minus the 1,200 frames already labeled; the Decision speaks of this dump.

### What happens after

No step of this plan changes the app. The Decision in the plan records the
outcome against these bars and names the follow-up by the rule above (an
in-app discard candidates view with a rescue loop; a retrain on the rescue
labels with the controls as negatives; or nothing), as a recommendation
only.
