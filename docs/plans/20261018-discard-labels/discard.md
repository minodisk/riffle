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

So the label-free conditions of the bars below are already decided for the
three `G0` `q` cuts **before any label**, and are stated here so no one
reads them as a result:

- **q20's pick loss sits at the 5% bar**, 5.18% on the full dump (5.03% on
  the population), above it on either definition: `G0` q20 fails the `PL`
  bar whatever the labels say.
- **q5 removes 4.92% of the population's frames (4.96% of the full
  dump's)**, under 5% because `floor(q * n / 100)` rounds each folder down.
  The removed-share condition is therefore set at **>= 4.5%** (Bars), which
  allows for that rounding: q5 passes it, and its `PL` (0.68%) passes too.
- **q10** removes 9.87% of the population's frames with `PL` 2.10%: it
  passes both label-free conditions.

So q5 and q10 can pass, q20 cannot; `G0`'s `quantile@95` cut (15.2% of the
dump's frames, so it clears the removed-share condition) is the fourth
candidate, its pooled pick loss computed in Step 4 (Decision 2's mean
per-folder pick recall 94.7% puts it near the bar).

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
conditions the same (removed share >= 4.5% of the population's frames, `DP`
above random's).

- **Usable discard cut**: a cut of `G0` (q5, q10, q20 or `quantile@95`) is
  usable when all of these hold:
  - weighted `RR` <= **15%**, with the stratified bootstrap 97.5% quantile
    of `RR` <= **25%**;
  - `PL` <= **5%** (full dump, label-free, Primary metrics 3);
  - it removes **>= 4.5%** of the population's frames (label-free; set by
    the user on 2026-10-18 before any label, down from the plan's first
    5%, to allow for the per-folder `floor(q * n / 100)` rounding that puts
    q5 at 4.92%);
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
  is reported with its labeled metrics anyway; `G0` q5 (4.92%) and q10
  (9.87%) pass the removed-share condition and the `PL` bar (0.68%,
  2.10%), so their verdicts rest on `RR` and the random comparison alone.
  The bars are not moved after this section.
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

## Results

Run 2026-10-19 by `discardfilter.py` (`python -I -B discardfilter.py
D:\Photos\tests\2026-10-12-rank-app D:\Photos\tests\2026-10-15-folder-wide-eval
D:\Photos\tests\2026-10-16-reject-filter-eval
D:\Photos\tests\2026-10-17-bad-labels
D:\Photos\tests\2026-10-18-discard-labels`), its stdout saved as
`discardfilter.out.md` in the data folder. On the 600 labels of Step 3 (211
`keep`, 389 `discard`, none skipped). Every interval is the stratified
bootstrap's 95% (1,000 draws, seed 0); values are weighted by `1/π` unless
marked unweighted. Wall time 18 s (load 5 s; cuts, metrics and bootstraps
12 s).

### Reference checks

| Check | Expected | This run |
| --- | ---: | ---: |
| `strata.tsv` rows whose population, drawn count or `π` differ when recomputed from the dump | 0 | 0 of 30 |
| Sampled frames whose recomputed `G0` band / `π` differ from `labels.tsv` | 0 / 0 | 0 / 0 |
| Sampled frames in the strict sample | 0 | 0 |
| Sum of `1/π` over the sampled frames | 31,259 | 31,259.0 |
| Random (seed 0) `DR` % at q5 / q10 / q20 | near q | 2.6 / 10.6 / 21.0 |
| Random (seed 0) AUC | near 0.5 | 0.528 |
| `G0` `quantile@95`: mean pick recall / non-pick removal % over the qualifying folders | 94.7 / 16.8 | 94.7 / 16.8 |
| Strict base rate from the strict sample, % | 5.52 [4.15, 7.04] | 5.52 [4.15, 7.04] |

### Discardable base rate

**48.45% [42.03, 55.11]** of the population's frames are labeled `discard`
(folder-cluster [40.74, 53.87]; unweighted 64.83%, the sample being
weighted toward the bands).

### Per band

| Band | Population | Sampled / discard | `discard` rate % | Pick share % (label-free) | Strict sample frames / strict bad | Strict-bad rate % | Strict below discard |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `b05` | 1,538 | 150 / 124 | 82.7 [76.0, 88.7] | 1.89 | 71 / 22 | 27.3 [16.1, 41.4] | yes |
| `b10` | 1,548 | 150 / 105 | 70.0 [62.7, 78.0] | 3.68 | 77 / 10 | 12.5 [3.9, 25.3] | yes |
| `b20` | 3,102 | 150 / 102 | 68.0 [60.7, 75.3] | 4.13 | 146 / 17 | 10.1 [5.2, 16.9] | yes |
| `control` | 25,071 | 150 / 58 | 42.6 [34.8, 50.7] | 16.11 | 906 / 31 | 3.7 [2.2, 5.2] | yes |

The strict-bad rate is read on the strict sample (disjoint frames) with its
own `π` and strata.

### Main table

`DP` / `RR` / `DR` / `MR` / `DO` weighted with the stratified interval and
the unweighted value in brackets; `PL` on the full dump (population after
the slash), label-free; Removed: the population's share (the full dump's
after the slash), label-free; Sampled: the sampled frames the cut removes /
of them `discard`; Picks removed / discard: the sampled dump picks the cut removes / of them `discard`; Out: sampled frames in folders the score does not cover. `W0` leaves out the 4 sampled frames of `2026-07-23`.

| Score | Cut | Out | Sampled / discard | Picks removed / discard | `DP` % | `RR` % | `PL` % | `DR` % | `MR` % | `DO` % | Removed % |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `G0` | q5 | 0 | 150 / 124 | 7 / 5 | 82.7 [76.0, 88.7] (82.7) | 17.3 [11.3, 24.0] (17.3) | 0.68 / 0.68 | 8.4 [7.2, 9.8] (31.9) | 91.6 [90.2, 92.8] (68.1) | 46.7 [39.9, 53.8] (58.9) | 4.92 / 4.96 |
| `G0` | q10 | 0 | 300 / 229 | 11 / 6 | 76.3 [71.3, 81.3] (76.3) | 23.7 [18.7, 28.7] (23.7) | 2.10 / 2.02 | 15.5 [13.6, 18.1] (58.9) | 84.5 [81.9, 86.4] (41.1) | 45.4 [38.3, 52.9] (53.3) | 9.87 / 9.96 |
| `G0` | q20 | 0 | 450 / 331 | 16 / 8 | 72.1 [67.5, 76.7] (73.6) | 27.9 [23.3, 32.5] (26.4) | 5.18 / 5.03 | 29.5 [25.9, 33.9] (85.1) | 70.5 [66.1, 74.1] (14.9) | 42.6 [34.8, 50.7] (38.7) | 19.80 / 19.97 |
| `G0` `quantile@95` | q@95 | 0 | 375 / 281 | 12 / 6 | 74.0 [69.0, 78.7] (74.9) | 26.0 [21.3, 31.0] (25.1) | 3.64 / 3.55 | 22.7 [19.5, 26.3] (72.2) | 77.3 [73.7, 80.5] (27.8) | 44.0 [36.6, 51.9] (48.0) | 15.05 / 15.23 |
| `W0` | q5 | 4 | 91 / 74 | 2 / 1 | 76.9 [63.6, 88.6] (81.3) | 23.1 [11.4, 36.4] (18.7) | 1.68 / 1.69 | 7.1 [4.7, 10.4] (19.0) | 92.9 [89.6, 95.3] (81.0) | 47.2 [40.8, 54.0] (62.4) | 4.91 / 4.96 |
| `W0` | q10 | 4 | 180 / 139 | 9 / 5 | 66.6 [52.5, 80.2] (77.2) | 33.4 [19.8, 47.5] (22.8) | 4.05 / 4.05 | 13.7 [9.9, 18.4] (35.7) | 86.3 [81.6, 90.1] (64.3) | 46.5 [39.6, 53.5] (60.1) | 9.86 / 9.97 |
| `W0` | q20 | 4 | 303 / 233 | 13 / 7 | 64.1 [54.2, 73.8] (76.9) | 35.9 [26.2, 45.8] (23.1) | 7.34 / 7.32 | 33.1 [25.9, 40.7] (59.9) | 66.9 [59.3, 74.1] (40.1) | 43.3 [35.6, 51.3] (53.2) | 19.88 / 19.97 |
| `R0` | q5 | 0 | 123 / 104 | 7 / 6 | 82.1 [71.1, 92.3] (84.6) | 17.9 [7.7, 28.9] (15.4) | 1.02 / 1.01 | 9.1 [6.5, 12.6] (26.7) | 90.9 [87.4, 93.5] (73.3) | 46.5 [39.9, 53.5] (59.7) | 4.89 / 4.96 |
| `R0` | q10 | 0 | 212 / 173 | 8 / 6 | 77.0 [66.3, 87.3] (81.6) | 23.0 [12.7, 33.7] (18.4) | 2.24 / 2.23 | 17.6 [13.0, 22.6] (44.5) | 82.4 [77.4, 87.0] (55.5) | 44.9 [38.1, 52.2] (55.7) | 9.90 / 9.96 |
| `R0` | q20 | 0 | 347 / 271 | 11 / 6 | 73.6 [65.0, 81.9] (78.1) | 26.4 [18.1, 35.0] (21.9) | 5.43 / 5.41 | 32.4 [25.8, 39.2] (69.7) | 67.6 [60.8, 74.2] (30.3) | 41.6 [34.2, 49.8] (46.6) | 19.87 / 19.97 |
| Absolute sharpness | q5 | 0 | 73 / 55 | 6 / 2 | 78.0 [57.6, 92.4] (75.3) | 22.0 [7.6, 42.4] (24.7) | 2.47 / 2.42 | 9.7 [5.0, 15.5] (14.1) | 90.3 [84.5, 95.0] (85.9) | 46.6 [39.9, 53.5] (63.4) | 4.93 / 4.96 |
| Absolute sharpness | q10 | 0 | 108 / 79 | 7 / 3 | 72.2 [53.5, 89.0] (73.1) | 27.8 [11.0, 46.5] (26.9) | 6.04 / 6.00 | 15.5 [9.5, 22.6] (20.3) | 84.5 [77.4, 90.5] (79.7) | 45.7 [38.9, 52.8] (63.0) | 9.85 / 9.96 |
| Absolute sharpness | q20 | 0 | 164 / 119 | 11 / 5 | 60.3 [45.7, 75.2] (72.6) | 39.7 [24.8, 54.3] (27.4) | 15.09 / 15.05 | 24.6 [17.2, 32.5] (30.6) | 75.4 [67.5, 82.8] (69.4) | 45.5 [38.5, 52.5] (61.9) | 19.88 / 19.97 |
| `eye_focus` (missing last) | q5 | 0 | 38 / 33 | 2 / 1 | 83.2 [56.7, 98.8] (86.8) | 16.8 [1.2, 43.3] (13.2) | 4.44 / 4.44 | 8.1 [3.7, 13.2] (8.5) | 91.9 [86.8, 96.3] (91.5) | 46.7 [40.4, 53.4] (63.3) | 4.83 / 4.96 |
| `eye_focus` (missing last) | q10 | 0 | 82 / 72 | 3 / 2 | 85.7 [70.1, 97.1] (87.8) | 14.3 [2.9, 29.9] (12.2) | 9.14 / 8.98 | 15.2 [9.4, 22.0] (18.5) | 84.8 [78.0, 90.6] (81.5) | 45.0 [38.6, 51.9] (61.2) | 9.78 / 9.96 |
| `eye_focus` (missing last) | q20 | 0 | 189 / 160 | 8 / 5 | 71.7 [59.8, 84.3] (84.7) | 28.3 [15.7, 40.2] (15.3) | 15.59 / 15.50 | 31.7 [24.1, 40.0] (41.1) | 68.3 [60.0, 75.9] (58.9) | 42.1 [35.1, 49.9] (55.7) | 19.87 / 19.97 |
| Random (seed 0) | q5 | 0 | 27 / 16 | 0 / 0 | 52.0 [19.8, 85.5] (59.3) | 48.0 [14.5, 80.2] (40.7) | 4.71 / 4.70 | 2.6 [0.8, 5.4] (4.1) | 97.4 [94.6, 99.2] (95.9) | 48.4 [41.9, 55.2] (65.1) | 4.95 / 4.96 |
| Random (seed 0) | q10 | 0 | 52 / 36 | 1 / 1 | 61.5 [37.9, 84.8] (69.2) | 38.5 [15.2, 62.1] (30.8) | 9.48 / 9.48 | 10.6 [5.4, 16.7] (9.3) | 89.4 [83.3, 94.6] (90.7) | 47.3 [40.7, 54.0] (64.4) | 9.99 / 9.96 |
| Random (seed 0) | q20 | 0 | 121 / 77 | 6 / 2 | 56.0 [40.5, 71.5] (63.6) | 44.0 [28.5, 59.5] (36.4) | 19.26 / 19.36 | 21.0 [13.8, 28.2] (19.8) | 79.0 [71.8, 86.2] (80.2) | 46.8 [39.9, 54.4] (65.1) | 19.98 / 19.97 |

Weighted AUC of `discard` vs `keep` (unweighted in brackets): `G0` 0.710
[0.646, 0.771] (0.699), `W0` 0.671 [0.601, 0.737] (0.651), `R0` 0.740
[0.673, 0.800] (0.716), absolute sharpness 0.540 [0.463, 0.615] (0.502),
`eye_focus` (missing last) 0.777 [0.716, 0.841] (0.803), random 0.528
[0.455, 0.604] (0.502).

### Paired differences

`G0` minus the comparison at the same `q`, in points, with the stratified
interval and the folder-cluster interval.

| Comparison | `DP`, points | Cluster | `DR`, points | Cluster |
| --- | ---: | ---: | ---: | ---: |
| `G0` q5 - `W0` q5 | +5.8 [-7.0, +19.8] | [-6.8, +19.8] | +1.3 [-2.0, +3.8] | [-1.6, +4.2] |
| `G0` q5 - `R0` q5 | +0.5 [-9.5, +11.7] | [-9.9, +14.5] | -0.8 [-4.0, +1.6] | [-3.7, +1.8] |
| `G0` q5 - sharpness q5 | +4.7 [-10.8, +25.6] | [-10.3, +33.6] | -1.3 [-7.1, +3.5] | [-6.2, +4.4] |
| `G0` q5 - `eye_focus` q5 | -0.5 [-18.3, +26.6] | [-18.7, +31.6] | +0.3 [-5.1, +5.1] | [-4.4, +5.1] |
| `G0` q5 - random q5 | +30.7 [-3.8, +63.9] | [-1.2, +67.1] | +5.8 [+2.8, +8.2] | [+2.2, +9.5] |
| `G0` q10 - `W0` q10 | +9.7 [-4.3, +24.4] | [-4.5, +26.8] | +1.9 [-3.0, +5.4] | [-2.5, +5.9] |
| `G0` q10 - `R0` q10 | -0.7 [-10.6, +10.4] | [-11.5, +12.0] | -2.0 [-7.0, +2.3] | [-6.3, +2.5] |
| `G0` q10 - sharpness q10 | +4.1 [-13.0, +23.1] | [-12.8, +24.9] | +0.0 [-7.1, +6.5] | [-5.4, +6.8] |
| `G0` q10 - `eye_focus` q10 | -9.4 [-21.2, +7.6] | [-20.8, +4.2] | +0.4 [-6.3, +6.6] | [-5.5, +7.2] |
| `G0` q10 - random q10 | +14.8 [-8.7, +38.6] | [-17.8, +39.4] | +4.9 [-1.2, +11.3] | [-0.6, +10.9] |
| `G0` q20 - `W0` q20 | +8.3 [-1.2, +18.9] | [-6.1, +23.4] | -3.7 [-11.9, +4.2] | [-13.2, +4.6] |
| `G0` q20 - `R0` q20 | -1.5 [-9.0, +6.5] | [-10.2, +8.8] | -2.9 [-9.9, +3.3] | [-10.1, +4.3] |
| `G0` q20 - sharpness q20 | +11.9 [-3.0, +26.7] | [-7.2, +25.1] | +4.8 [-3.8, +13.4] | [-4.0, +14.8] |
| `G0` q20 - `eye_focus` q20 | +0.4 [-12.1, +12.6] | [-9.1, +9.6] | -2.2 [-11.3, +6.3] | [-10.1, +7.4] |
| `G0` q20 - random q20 | +16.1 [+0.3, +31.7] | [-5.2, +34.6] | +8.4 [+0.6, +17.1] | [-1.0, +17.8] |
| `G0` `quantile@95` - random q20 | +18.0 [+2.0, +34.7] | [-5.4, +37.9] | +1.6 [-5.9, +10.1] | [-8.2, +11.0] |

### The non-pick framing

| Reading | Sampled / discard | Weighted % | Folder-cluster 95% | Unweighted % |
| --- | ---: | ---: | ---: | ---: |
| `discard` rate among the dump's non-picks | 561 / 377 | 51.7 [44.9, 59.0] | [43.8, 57.2] | 67.2 |
| `discard` rate among the dump's picks | 39 / 12 | 24.4 [8.6, 42.8] | [2.8, 47.7] | 30.8 |
| `discard` share of the non-picks `G0` `quantile@95` removes (`RN`) | 363 / 275 | 74.8 [69.9, 79.4] | [65.2, 82.3] | 75.8 |

Dump-label consistency: 12 of the 39 sampled dump picks are labeled
`discard` (weighted 24.4% [8.6, 42.8]); 0 of the 5 sampled explicit rejects
are labeled `keep`. The per-band `discard` rate beside the band's pick
share is in the per-band table.

### The secondary label set

The strict-bad rate of every `G0` band is below its `discard` rate (the
per-band table): `b05` 27.3% vs 82.7%, `b10` 12.5% vs 70.0%, `b20` 10.1% vs
68.0%, `control` 3.7% vs 42.6%; overall 5.52% strict bad vs 48.45%
discardable. The expected ordering holds in every band; there is no
inversion.

### Folder-cluster sensitivity

| Reading | Cut | `RR` % (stratified) | Cluster | `DR` % (stratified) | Cluster | `DO` % (stratified) | Cluster |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Discardable base rate | - | 48.5 [42.0, 55.1] | [40.7, 53.9] | | | | |
| `G0` | q5 | 17.3 [11.3, 24.0] | [11.7, 25.7] | 8.4 [7.2, 9.8] | [6.7, 10.8] | 46.7 [39.9, 53.8] | [38.7, 52.4] |
| `G0` | q10 | 23.7 [18.7, 28.7] | [17.9, 32.8] | 15.5 [13.6, 18.1] | [13.0, 18.5] | 45.4 [38.3, 52.9] | [37.2, 51.3] |
| `G0` | q20 | 27.9 [23.3, 32.5] | [21.4, 36.3] | 29.5 [25.9, 33.9] | [25.1, 33.9] | 42.6 [34.8, 50.7] | [33.9, 49.3] |
| `G0` `quantile@95` | q@95 | 26.0 [21.3, 31.0] | [18.4, 35.8] | 22.7 [19.5, 26.3] | [18.2, 26.9] | 44.0 [36.6, 51.9] | [35.2, 50.4] |

The paired `DP` differences against random above are given with their
cluster interval too: the two stratified passes of the random condition
(q20 and `quantile@95`) have cluster 2.5% quantiles below zero (-5.2,
-5.4).

### The bars applied

| Cut | `RR` <= 15% | `RR` 97.5% quantile <= 25% | `PL` <= 5% | Removed >= 4.5% | `DP` > random, 2.5% quantile > 0 | Verdict |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `G0` q5 | no (17.3) | yes (24.0) | yes (0.68) | yes (4.92) | no (+30.7, 2.5% quantile -3.8) | not usable |
| `G0` q10 | no (23.7) | no (28.7) | yes (2.10) | yes (9.87) | no (+14.8, -8.7) | not usable |
| `G0` q20 | no (27.9) | no (32.5) | no (5.18) | yes (19.80) | yes (+16.1, +0.3) | not usable |
| `G0` `quantile@95` | no (26.0) | no (31.0) | yes (3.64) | yes (15.05) | yes (+18.0 vs random q20, +2.0) | not usable |

**No usable discard cut; no best cut.**

The follow-up rule's inputs: no usable cut; weighted `RR@5` 17.3% (within
2x the `RR` bar, <= 30%); `DO@5` 46.7% (>= 20%).

**Hypothesis `RR@5` <= 5%: not confirmed.** Weighted `RR@5` 17.3% [11.3,
24.0] (unweighted 17.3%, 26 of the 150 sampled `b05` frames marked keep),
against about 1 in 71 in the unblinded review.

### Reading

On blind labels, about half of the population's frames (48.5% [42.0, 55.1])
are ones the user would not miss, almost nine times the 5.5% strict-bad
rate. `G0`'s bands order the `discard` rate as expected (82.7% in `b05`,
70.0% in `b10`, 68.0% in `b20`, 42.6% in the controls) and its AUC is
0.710, but no `G0` cut meets the bars. The shallowest, q5, rescues 17.3%
[11.3, 24.0] of what it removes, above the 15% bar with its upper quantile
just under 25%, and its `DP` lead over random (+30.7 points) has a
stratified 2.5% quantile of -3.8, the random q5 cut removing only 27
sampled frames; the deeper cuts rescue 24–28%. The unblinded 70/71 (`RR@5`
about 1.4%) did not hold blind: the blind `RR@5` is about twelve times
higher. Every cut leaves the discardable share among the kept frames near
the base rate (`DO` 42.6–46.7%) and removes 8–30% of the discardable
frames. Against the other scores at the same `q`, every paired `DP` and
`DR` difference of `G0` has an interval spanning zero; `eye_focus` (missing
last) has the highest AUC (0.777), then `R0` (0.740), and sharpness is
near chance (0.540). On the non-pick framing, half of the non-picks are
`discard` (51.7%) and so are a quarter of the sampled picks (24.4%, 12 of
39); of the non-picks `quantile@95` removes, 74.8% are `discard`. The
strict-bad rate is below the `discard` rate in every band. On these
numbers the conditions of the follow-up rule's second branch (no usable
cut, `RR@5` <= 30%, `DO@5` >= 20%) hold; the Decision (Step 5) reads them.
