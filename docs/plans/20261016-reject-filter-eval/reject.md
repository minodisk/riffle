# Reject-filter evaluation

## Pre-registration

Written 2026-10-16, before `rejectfilter.py` or `rejectablate.py` exists and
before any bottom-oriented number of this plan has been computed, and merged
on its own before any script (the merge commit of the PR that adds this
section, without results, is the record, as for the archived
[folder-wide evaluation](../_archived/20261015-folder-wide-ranking-eval/folderwide.md),
the [folder-wide gate](../_archived/20261014-folder-wide-pick-model/gate.md)
and the in-burst [gate](../_archived/20261012-image-embeddings/gate.md)).
Nothing in this section changes after the results are in; a deviation found
while implementing Step 2 or Step 3 is recorded in `learnings.md` with its
reason and does not touch the bars.

The question: used as a **reject filter** (hide the bottom of a folder,
single shots and burst frames alike, and leave the human a candidate set to
pick from), how many non-picks can each score remove while nearly all picks
survive? Does a rule filter on the "obviously bad" technical cues, or a model
trained to find the bottom, do better than the shipped weights? Which column
groups contribute at the bottom, does Decision 1's "drop genre for the
ranking" still hold under this criterion, and is the best filter worth an app
change at all?

### What was known before this section

This pre-registration is **not blind**. On the same dump and the same 25
qualifying folders, the archived
[folder-wide evaluation](../_archived/20261015-folder-wide-ranking-eval/folderwide.md)
(Decision 1 in its
[plan](../_archived/20261015-folder-wide-ranking-eval/plan.md)) reported,
held out:

- the per-folder frame AUC of every variant, 0.663–0.678 (`W0` 0.666,
  `F0+ctx` 0.666); the full-ROC area bounds how steep the curve can be at any
  recall;
- the baselines' frame AUC: `eye_focus` 0.579, `photoTier` good 0.546,
  absolute sharpness 0.515;
- the pick vs explicit reject AUC over the 3 folders that hold one;
- that no cue variant (genre, Candidate 1, Candidate 2) passed its bar on the
  top-oriented metrics, and that the "obviously bad" technical cues (closed
  eyes, an out-of-focus eye, blur, a face turned away) were redundant at the
  top.

Not known before this section, with no prior number: any bottom-oriented
number of any score. Nothing in the archived plans reports the non-pick
removal at a fixed pick recall, a partial AUC at high recall, or the picks
lost in a bottom share. The held-out scores in
`D:\Photos\tests\2026-10-15-folder-wide-eval\scores.tsv` exist, but only
their header has been read for this section.

The bars below were **set by the user on 2026-10-16 before any score of this
plan** (option B of the plan's trade-offs), not tuned to the numbers above.
The reject-trained variants were chosen by the user the same day (option
(b): `R0` and `G0`).

### Data, labels and negatives

- The 27 folders of the app dump in `D:\Photos\tests\2026-10-12-rank-app\`,
  read through the archived
  [`gate.py`](../_archived/20261012-image-embeddings/gate.py)'s `load` and
  `build` and
  [`folder_gate.py`](../_archived/20261014-folder-wide-pick-model/folder_gate.py)'s
  `extend`, imported by path (not copied, not modified).
- Picks by the gate label rule (`gate.load`): a frame is a pick when its
  `.dop` flag is Pick, or it has a virtual copy in `Output/`, or it has no
  `.dop` and its XMP flag is Pick. 4,419 picks among 32,459 frames.
- **Every non-pick is a negative**, with this caveat: "non-pick" is not
  "bad". A culled folder's non-picks are the bad frames *and* the
  fine-but-redundant ones (the second-best frame of a burst, a duplicate). So
  the removed-non-pick share is an **upper bound on "bad removed"**, and the
  **pick recall constraint is the real safety number**: a filter that keeps
  99% of the picks costs the human one pick in a hundred whatever it removed.
- Explicit rejects (58, in 3 folders) are the only frames known to be bad.
  They stay among the negatives, are too few to train on or to bar on, and
  are reported as one reading aid (the share of the 58 removed at each
  operating point).
- Bursts as `rank::bursts` (`gate.bursts_of`, gap 1000 ms); a single is a
  burst of one frame. Bursts play no role in the metric; they enter only
  through the scores' columns (burst position, `ln burst size`) and the
  burst-centered `W0` reading.
- The cue files of `D:\Photos\tests\2026-10-14-ball-labels\`
  (`extras_c1.tsv`, `extras_c2.tsv`, full coverage), joined by
  [`ballgate.py`](../_archived/20261014-ball-game-ranking-features/ballgate.py)'s
  `load_extras`, for the ablations' each-alone readings and the new fits'
  inputs where a variant names them.

### Unit and folds

- Every frame is a sample.
- Every number is held out by folder. The scores of `scores.tsv` are leave
  one folder out already; every new fit (`R0`, `G0`, the ablations) follows
  the same 27 folds, with `C` (where a model has one) chosen per outer fold
  from `gate.CS` (0.01, 0.1, 1, 10) by the same inner grouped-by-folder
  cross-validation on the training folders, on a tie the smaller `C`.
- **Qualifying folders**: at least 50 frames and 5 picks (25 of 27). Means,
  intervals and tests are over them; every folder is trained on and listed in
  the per-folder table.
- A **sub-table over the qualifying folders with >= 20 picks** is reported,
  because at 5–19 picks a 95% target means "lose none" and `RN@99` always
  does.

### Primary metric (bottom-oriented, recall-constrained)

Each per held-out folder, then the unweighted mean over the qualifying
folders (one folder, one vote), with the frame-weighted pooled value beside
it:

1. **Non-pick removal at pick recall r, `RN@r`**, r = 95% and 99%. Sort the
   folder's frames by score ascending, ties broken by the seed-0 uniform draw
   the earlier plans use. The folder must keep `ceil(r * picks)` picks;
   remove the longest bottom prefix that holds at most
   `picks - ceil(r * picks)` picks. Report the removed share of the folder's
   **non-picks** (`RN@r`, the specificity at sensitivity >= r) and of all its
   **frames** (`RF@r`). This is the **oracle** threshold (it uses the
   held-out folder's own labels) and is an upper bound on what a label-free
   cut achieves. **`RN@95` is the primary number**; `RN@99` the second.
2. **Partial AUC at high pick recall, `pAUC95`**: the McClish-standardized
   partial AUC over pick recall in [0.95, 1], computed as
   `sklearn.metrics.roc_auc_score(~pick, -score, max_fpr=0.05)` (the
   non-picks as the positive class, so that the restricted false-positive
   rate is the pick loss). Chance 0.5; threshold-free, so it reads the whole
   high-recall region, not one cut.

### Label-free thresholds

What the app could actually apply, since the oracle cut needs the folder's
own picks. Per held-out folder, without its labels, each reported as the
pair (**realized pick recall**, `RN`) and as the gap between the realized and
the target recall:

- `quantile@r`: remove the bottom `q_r` share of the held-out folder's
  frames, `q_r` = the largest share such that the mean per-folder pick recall
  over the **other** qualifying folders' held-out scores (the same column,
  so no refit) is >= r. Scale-free, so it applies to the uncentered `W0`
  family as well as to the pointwise logits.
- `absolute@r`: remove the held-out folder's frames whose score is below
  `t_r`, `t_r` = the score at which the other qualifying folders' pooled
  held-out scores keep >= r of their picks. Sensitive to the score scale
  across folds (each fold is its own model; `W0` has no intercept), reported
  with that caveat.

Non-qualifying folders are excluded from the threshold estimate and from the
means. **`quantile@95` is the label-free primary.** A score's "filter at 95%"
means its `quantile@95` pair unless stated.

### Reading aids (reported, not deciding)

- **Picks lost in a bottom share**: the share of the folder's picks inside
  its bottom X% of frames by score, X = 10 / 20 / 30 / 50 (the "what if the
  user just hides the bottom fifth" reading).
- The **explicit-reject removal share** (of the 58) at `RN@95` and at
  `quantile@95`.
- The **frame AUC** from the archived
  [folderwide.md](../_archived/20261015-folder-wide-ranking-eval/folderwide.md)
  beside each row, so the top and the bottom are read side by side.

### Scores

Names fixed here; each is one held-out score vector.

- **From `scores.tsv`, no refit** (names as there): `W0` (**the reference**:
  the shipped weights' uncentered score), `F0`, `F0+ctx`, `P0`, and the
  ten cue variants `W0+off` (shipped; the genre offsets on `W0`),
  `F0+ctx +genre cols`,
  `F0+ctx +genre off`, `F0+ctx +c1`, `F0+ctx +c2`, `F0+ctx +c1+c2`,
  `W0 +genre cols`, `W0 +c1`, `W0 +c2`, `W0 +c1+c2`; plus the **`W0`
  burst-centered** score (what the app displays), rebuilt by subtracting
  each burst's mean `W0` score. `nan` scores (the `W0` family on folder
  `2026-07-23`, which is not qualifying) leave that folder out of that
  score's rows.
- **Baselines**, rebuilt as `folderwide.main` does: random (seed 0); capture
  order, earlier first and later first; burst size, larger first; absolute
  sharpness; `eye_focus` (missing last); `photoTier` good.

### Rule filters

Using the app's constants from `focus.ts` / `rank.rs`: `GOOD_EYE_FOCUS` 0.9,
`GOOD_EYE_EAR` 0.25, `EYES_CLOSED_EAR` 0.137, `GOOD_MAX_YAW` 35,
`GOOD_MAX_PITCH` 45. A frame without the value a rule needs is never removed
by that rule. A binary rule gives one operating point (its realized pick
recall and `RN`, no curve); a graded one gets the full treatment.

- `not good`: remove every frame whose `photoTier` is not good (expected to
  remove most of a folder, faceless frames included; listed because it is
  the one bad / good split the app already shows).
- `eyes closed`: remove the frames with `eo == 0` (the EAR at or below
  `EYES_CLOSED_EAR`).
- `eye focus low`: remove the frames with an AF face whose `ef` <
  `GOOD_EYE_FOCUS`.
- `face away`: remove the frames with `|yaw| > GOOD_MAX_YAW` or
  `|pitch| > GOOD_MAX_PITCH`.
- `sharpness bottom q`: remove the folder's bottom q by absolute sharpness,
  q = 10% and 20%. At a fixed q it is one cut, so it is a binary rule for
  `RN@r` (the curve is the absolute-sharpness baseline's row).
- `any of`: `eyes closed` or `eye focus low` or `face away` or
  `sharpness bottom 10%`.
- `rule count`: the graded score "number of the four rules of `any of` that
  fire" (0–4), more fired removed first, ties by the seed-0 draw.

For a binary rule, `RN@r` is the point itself when the rule's realized recall
is >= r, else "does not reach r" (a rule cannot be relaxed); the table shows
the pair. In every mean, bootstrap interval, count and Wilcoxon of the
comparison below, a folder where a binary rule does not reach r scores
`RN@r` = 0 (the rule cannot be applied safely there). `rule count` is graded
(any prefix length can be cut, ties by the seed-0 draw), so its `RN@r` is
always defined.

### Reject-trained variants

New held-out fits on the same 27 folds and on the same columns as `F0+ctx`
(the 44 shipped columns, `ln burst size` and absolute `ln sharpness`). Both
run (option (b), chosen by the user on 2026-10-16):

- `R0`, **asymmetric pointwise**: `F0+ctx`'s weighted logistic with the
  picks' per-folder weight multiplied by **4** (a fixed choice, not tuned):
  the fit pays four times more for a misplaced pick than for a misplaced
  non-pick, so it moves its errors toward the "keep" side. `C` by the inner
  CV maximizing the inner validation folders' mean `RN@95`.
- `G0`, **nonlinear pointwise**:
  `sklearn.ensemble.HistGradientBoostingClassifier` on the same columns with
  the same per-folder class-balanced weights, at one fixed small setting
  (`max_iter` 200, `learning_rate` 0.05, `max_leaf_nodes` 15,
  `l2_regularization` 1.0, missing values handled natively so the indicator
  columns stay), no tuning. It can express the disjunction "closed eyes
  **or** out of focus **or** blur" a linear model cannot.
- Explicit rejects are **not** upweighted in any variant: 58 frames in 3
  folders cannot move a 27-fold fit, and the app's `REJECT_WEIGHT` 2 belongs
  to the `rank.rs` session's scope, not this plan's.

### Ablations (Step 3)

- On `F0+ctx` and on `W0`, the archived plan's exact `ablate.specs`, each a
  full held-out rerun without the named columns: drop each `REASONS` group
  (`Look`, `Burst position`, `Sharpness`, `AF eye focus`, `Eyes open`,
  `Head pose`, `Good photo`, each missing-value indicator leaving with its
  value's group); drop `|yaw|`, `|pitch|`, `|roll|` one at a time; drop
  `ln burst size` (`F0+ctx` only); each group alone, plus c1, c2 and the
  genre labels alone.
- Reported as the change in `RN@95`, `RN@99` and `pAUC95` against the
  unablated `scores.tsv` score, with the folder bootstrap interval; each
  alone also with its label-free `quantile@95` pair.
- On `G0`, the drop-one groups only, **and only if** `G0` is the best filter
  in Step 2 (highest mean `RN@95`).

### Intervals and tests

- A 1,000-draw bootstrap over the qualifying folders (seed 0,
  `np.random.default_rng(0)` afresh per comparison and metric, 25 folder
  indices drawn with replacement) of the mean paired per-folder difference
  (score minus comparison), for `RN@95`, `RN@99` and `pAUC95`, reported as
  the 2.5% / 97.5% quantiles.
- The paired one-sided Wilcoxon signed-rank test across the qualifying
  folders (`folder_gate.signed_rank`, alternative "greater", zero
  differences dropped), on `RN@95`.
- The count of qualifying folders where the score is at or above the
  comparison on `RN@95`.

Comparisons read: each score, baseline, rule and new variant minus `W0`;
each cue variant minus its base (`F0+ctx` or `W0`, `W0+off` against `W0`
being the genre offsets on `W0`); `R0` and `G0` minus
`F0+ctx`.

### Bars

Option B, chosen by the user on 2026-10-16 before any score. Rejected:
option A (lenient, G = +2.0 points, X = 15%: an app change for a small
effect) and option C (strict, G = +5.0 points, X = 40%: a useful but modest
filter declared "none").

Two bars, over the 25 qualifying folders:

- **Comparison bar**: a score or rule **filters better** than its comparison
  when all of these hold:
  - mean `RN@95` gain >= **+3.0 points**;
  - the score at or above the comparison on `RN@95` in **>= 16 of the 25**
    qualifying folders;
  - the paired one-sided Wilcoxon **p < 0.05** on `RN@95`.

  `RN@99` and `pAUC95` are reported, not required.
- **Worth-a-filter bar**: the best filter is **worth an app change** at all
  when, on its **label-free** `quantile@95` pair:
  - the mean realized pick recall is >= **93%** (within 2 points of the
    target); and
  - the mean `RN` is >= **25%**.

Not corrected for multiplicity: about 33 score rows (15 scores, 8 baselines,
8 rule rows, `R0`, `G0`), about 45 comparisons (each row against `W0`, 10
cue-vs-base, 2 against `F0+ctx`) and 60 ablation cells are read at p < 0.05
or interval-below-zero, so a few false positives are expected; the Decision reads a lone marginal pass as
such.

Applied as:

- **Best filter at 95% / 99%**: the score with the highest mean `RN@95`
  (respectively `RN@99`), oracle, with its label-free pair beside it; it
  **wins over the shipped weights** when it passes the comparison bar against
  `W0`.
- **A cue contributes at the bottom** when its drop-one lowers `RN@95` with
  the bootstrap interval entirely below zero; reported per column, as
  Decision 1 did.
- **Genre under this criterion**: Decision 1's "drop for the ranking"
  **stands** unless `F0+ctx +genre cols`, `F0+ctx +genre off`, `W0 +genre cols` or
  `W0+off` passes the comparison bar against its base (`F0+ctx` or `W0`) on
  `RN@95`; otherwise it is **reopened** and recorded as
  such (the scope question, ranking only or the whole feature, stays where
  Decision 1 left it).
- **Follow-up**: an app change is recommended only when the best filter
  passes the worth-a-filter bar.

The Decision speaks of pick safety (realized recall) and removed volume,
never of "rejects found": with no `views` rows and 58 explicit rejects, a
filter's precision on "bad" is not measurable on this data.

### What happens after

No step of this plan changes the app. Decision 2 in the plan records the
outcome against these bars and names the follow-up plan, if any, as a
recommendation only.

## Results

Run 2026-10-16 by `python -I -B rejectfilter.py D:\Photos\tests\2026-10-12-rank-app D:\Photos\tests\2026-10-14-ball-labels D:\Photos\tests\2026-10-15-folder-wide-eval D:\Photos\tests\2026-10-16-reject-filter-eval 20`
(Windows 11, 20 jobs). Its stdout is `rejectfilter.out.md` and the held-out
scores of the new models per frame `scores_reject.tsv` (`R0`, `G0` and
`rule count`, the last negated so that, as in every other column, a higher
value keeps), both in `D:\Photos\tests\2026-10-16-reject-filter-eval\`. The
tables below are that output as printed. Deviations from the pre-registered
text and the choices it left open are in `learnings.md` (Step 2); the two
deviations that move numbers: the `eyes closed` rule reads the raw EAR (the
dump's `eyes_open` is never 0), and `G0`'s sample weights are rescaled to mean
1 per training fold (the plan's text says `sample_weight = d["weight"]`; the
unrescaled fit is printed below as a check).

### Data

| Count | Value |
| --- | ---: |
| Folders / qualifying (>= 50 frames, >= 5 picks) / qualifying with >= 20 picks | 27 / 25 / 22 |
| Frames (with a vector) / picks / explicit rejects | 32459 / 4419 / 58 |
| Explicit rejects that are not picks / in qualifying folders / folders holding one | 58 / 58 / 3 |
| Frames without a vector (dropped) | 0 |
| Bursts (gap 1000 ms), singles included / singles | 6969 / 1943 |
| Folder pick rate min / median / max | 10.0% / 13.8% / 22.2% |
| `scores.tsv` rows / matched to the dump's frames | 32459 / 32459 |
| Frames with a value: `ef` (AF face), EAR (eyes judgment), pose, sharpness | 28437 / 21563 / 21440 / 32459 |
| Frames removed by each rule (all folders): not good, eyes closed, eye focus low, face away, sharpness bottom 10%, sharpness bottom 20%, any of | 26923 / 2385 / 9072 / 8346 / 3234 / 6482 / 17428 |
| `c1` / `c2` columns (frames without a row) | 4 (0) / 6 (0) |

### Reference checks

| Check | Expected | This run |
| --- | ---: | ---: |
| `W0` mean frame AUC from `scores.tsv` (qualifying folders) | 0.666 | 0.666 |
| `F0+ctx` mean frame AUC from `scores.tsv` (qualifying folders) | 0.666 | 0.666 |
| Random (seed 0) mean `RN@95` % | near 5 | 5.8 |
| Random (seed 0) mean `pAUC95` | near 0.5 | 0.504 |
| `R0` at pick weight 1 (`C` by AUC): max abs score difference from `F0+ctx` | ~5e-6 (6 significant digits) | 5.00e-06 |
| `R0` at pick weight 1 (`C` by AUC): mean frame AUC / `RN@95` % | 0.666 / 18.4 | 0.666 / 18.4 |

### Main table: bottom metrics (held out, qualifying folders)

Over the 25 qualifying folders: the mean (per-folder SD in brackets) and the frame-weighted mean (pooled). `RN@r`: the share of the folder's non-picks removed by the oracle cut that keeps >= r% of its picks; `RF@95` the share of all its frames removed there. `pAUC95`: McClish-standardized partial AUC over pick recall [0.95, 1], chance 0.5. Label-free pairs: the mean realized pick recall % / mean `RN` % (realized minus target recall in brackets); `quantile@r` removes the bottom share the other qualifying folders' held-out scores allow, `absolute@r` the frames below the other folders' pooled pick-score cut (sensitive to the score scale across folds). For a binary rule (`Rule: ...`), `RN@r` and `RF@r` are its single point in the folders where it keeps >= r% of the picks and 0 elsewhere, `pAUC95` and `absolute@r` do not apply, the `quantile@95` cell is the rule's own point (it needs no threshold), and its frame AUC is that of the keep / remove split. Frame AUC is recomputed from the same scores.

| Score | `RN@95` % | Pooled | `RF@95` % | `RN@99` % | Pooled | `pAUC95` | Pooled | `quantile@95` recall / `RN` (gap) | `absolute@95` recall / `RN` (gap) | Frame AUC |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| W0 | 16.6 (9.2) | 17.6 | 14.8 | 7.1 (8.4) | 4.0 | 0.544 (0.041) | 0.536 | 95.0 / 11.5 (+0.0) | 91.4 / 14.5 (-3.6) | 0.666 |
| W0+off | 17.4 (9.8) | 19.6 | 15.5 | 7.1 (7.7) | 5.2 | 0.544 (0.041) | 0.542 | 94.9 / 11.4 (-0.1) | 91.3 / 19.3 (-3.7) | 0.669 |
| F0 | 19.3 (11.8) | 21.7 | 17.3 | 8.2 (10.9) | 6.3 | 0.554 (0.055) | 0.553 | 95.0 / 14.6 (-0.0) | 92.8 / 17.8 (-2.2) | 0.664 |
| F0+ctx | 18.4 (10.8) | 22.2 | 16.4 | 8.1 (8.9) | 7.0 | 0.550 (0.046) | 0.554 | 95.0 / 14.6 (+0.0) | 93.7 / 17.4 (-1.3) | 0.666 |
| P0 | 17.9 (10.7) | 19.9 | 16.0 | 7.4 (9.7) | 5.2 | 0.547 (0.049) | 0.543 | 95.0 / 13.5 (-0.0) | 89.7 / 18.1 (-5.3) | 0.664 |
| F0+ctx +genre cols | 17.8 (9.7) | 22.0 | 15.9 | 7.6 (7.4) | 6.6 | 0.549 (0.040) | 0.555 | 95.0 / 13.9 (-0.0) | 94.9 / 16.8 (-0.1) | 0.667 |
| F0+ctx +genre off | 19.2 (9.8) | 23.2 | 17.1 | 8.8 (7.5) | 7.9 | 0.553 (0.040) | 0.559 | 95.0 / 14.2 (+0.0) | 93.8 / 18.8 (-1.2) | 0.678 |
| F0+ctx +c1 | 17.7 (9.1) | 21.3 | 15.9 | 7.7 (7.4) | 6.4 | 0.548 (0.038) | 0.551 | 94.9 / 15.2 (-0.1) | 94.5 / 15.5 (-0.5) | 0.664 |
| F0+ctx +c2 | 18.5 (10.8) | 21.7 | 16.5 | 8.4 (9.0) | 7.1 | 0.550 (0.047) | 0.553 | 95.0 / 13.8 (-0.0) | 93.8 / 17.3 (-1.2) | 0.664 |
| F0+ctx +c1+c2 | 18.8 (9.9) | 21.6 | 16.8 | 8.1 (9.1) | 6.6 | 0.551 (0.045) | 0.552 | 94.8 / 15.9 (-0.2) | 93.9 / 15.5 (-1.1) | 0.666 |
| W0 +genre cols | 17.5 (9.8) | 18.5 | 15.7 | 8.0 (8.6) | 5.4 | 0.547 (0.043) | 0.540 | 94.9 / 13.4 (-0.1) | 89.3 / 17.8 (-5.7) | 0.671 |
| W0 +c1 | 17.2 (9.6) | 18.3 | 15.4 | 7.7 (8.6) | 4.9 | 0.546 (0.043) | 0.539 | 94.9 / 12.5 (-0.1) | 92.4 / 13.9 (-2.6) | 0.667 |
| W0 +c2 | 15.7 (9.0) | 16.6 | 14.0 | 6.8 (7.8) | 4.0 | 0.542 (0.038) | 0.536 | 95.0 / 11.2 (+0.0) | 89.2 / 18.0 (-5.8) | 0.663 |
| W0 +c1+c2 | 16.2 (8.7) | 17.8 | 14.5 | 7.4 (7.4) | 4.9 | 0.544 (0.036) | 0.540 | 94.9 / 13.0 (-0.1) | 88.8 / 19.3 (-6.2) | 0.665 |
| W0 burst-centered (what the app displays) | 16.8 (7.7) | 19.8 | 15.0 | 7.2 (4.2) | 6.6 | 0.545 (0.025) | 0.551 | 94.9 / 12.9 (-0.1) | 96.3 / 14.6 (+1.3) | 0.662 |
| Random (seed 0) | 5.8 (4.3) | 5.6 | 5.5 | 1.7 (3.5) | 1.3 | 0.504 (0.018) | 0.503 | 95.1 / 4.3 (+0.1) | 94.3 / 5.5 (-0.7) | 0.511 |
| Capture order, earlier first | 7.1 (5.8) | 5.5 | 6.7 | 2.6 (5.9) | 1.4 | 0.509 (0.029) | 0.502 | 94.9 / 5.2 (-0.1) | 98.5 / 1.7 (+3.5) | 0.500 |
| Capture order, later first | 6.7 (5.5) | 5.1 | 6.3 | 4.0 (6.1) | 2.2 | 0.513 (0.029) | 0.504 | 94.9 / 5.0 (-0.1) | 80.7 / 19.0 (-14.3) | 0.500 |
| Burst size, larger first | 8.3 (4.5) | 9.8 | 7.7 | 2.5 (3.8) | 1.5 | 0.511 (0.018) | 0.511 | 95.0 / 6.7 (+0.0) | 91.5 / 8.9 (-3.5) | 0.540 |
| Absolute sharpness | 9.2 (4.9) | 9.5 | 8.5 | 4.3 (4.9) | 3.1 | 0.520 (0.023) | 0.517 | 95.0 / 8.5 (-0.0) | 89.5 / 13.7 (-5.5) | 0.515 |
| `eye_focus` (missing last) | 7.2 (4.3) | 6.2 | 6.8 | 3.4 (4.7) | 1.7 | 0.511 (0.023) | 0.504 | 94.9 / 6.1 (-0.1) | 100.0 / 0.0 (+5.0) | 0.579 |
| `photoTier` good | 9.3 (10.7) | 6.2 | 8.6 | 4.9 (11.6) | 1.7 | 0.502 (0.003) | 0.501 | 95.0 / 7.3 (-0.0) | 100.0 / 0.0 (+5.0) | 0.546 |
| Rule: not good | 0.0 (0.0) | 0.0 | 0.0 | 0.0 (0.0) | 0.0 | - | - | 28.8 / 80.3 (-66.2) | - | 0.546 |
| Rule: eyes closed | 2.9 (3.6) | 3.1 | 2.7 | 0.2 (0.8) | 0.0 | - | - | 92.8 / 11.3 (-2.2) | - | 0.521 |
| Rule: eye focus low | 0.9 (2.2) | 0.2 | 0.8 | 0.3 (1.2) | 0.1 | - | - | 87.1 / 19.8 (-7.9) | - | 0.534 |
| Rule: face away | 0.2 (1.1) | 0.0 | 0.2 | 0.2 (1.1) | 0.0 | - | - | 75.1 / 23.5 (-19.9) | - | 0.493 |
| Rule: sharpness bottom 10% | 3.9 (5.3) | 4.4 | 3.5 | 1.2 (3.5) | 0.4 | - | - | 93.4 / 10.3 (-1.6) | - | 0.519 |
| Rule: sharpness bottom 20% | 0.8 (4.2) | 0.0 | 0.8 | 0.8 (4.2) | 0.0 | - | - | 84.2 / 20.5 (-10.8) | - | 0.523 |
| Rule: any of | 0.6 (3.0) | 0.0 | 0.5 | 0.6 (3.0) | 0.0 | - | - | 59.7 / 49.6 (-35.3) | - | 0.547 |
| rule count | 7.9 (3.6) | 8.0 | 7.4 | 3.6 (3.1) | 2.7 | 0.512 (0.016) | 0.511 | 95.0 / 6.7 (-0.0) | 98.8 / 2.1 (+3.8) | 0.548 |
| R0 | 18.3 (10.9) | 22.1 | 16.4 | 7.9 (8.3) | 7.3 | 0.550 (0.044) | 0.556 | 94.9 / 14.4 (-0.1) | 92.1 / 16.8 (-2.9) | 0.664 |
| G0 | 22.0 (11.5) | 24.1 | 19.6 | 11.5 (12.1) | 8.2 | 0.568 (0.059) | 0.562 | 94.7 / 16.8 (-0.3) | 90.8 / 21.1 (-4.2) | 0.670 |

### Main table, continued: reading aids and the 99% label-free pairs

Picks lost: the mean share of the folder's picks inside its bottom X% of frames by score. Rejects removed: of the 58 explicit rejects (not picks) in the qualifying folders, the number removed at the oracle `RN@95` cut and at the `quantile@95` cut (a binary rule: its own point). The 99% label-free pairs as above.

| Score | Picks lost, bottom 10% | 20% | 30% | 50% | Rejects removed at `RN@95` | at `quantile@95` | `quantile@99` recall / `RN` (gap) | `absolute@99` recall / `RN` (gap) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| W0 | 4.7 | 8.5 | 14.5 | 29.4 | 31 of 58 | 15 of 58 | 98.9 / 3.0 (-0.1) | 99.0 / 2.0 (+0.0) |
| W0+off | 4.3 | 8.2 | 13.7 | 29.8 | 35 of 58 | 17 of 58 | 98.9 / 3.4 (-0.1) | 97.7 / 4.8 (-1.3) |
| F0 | 3.7 | 8.3 | 13.7 | 29.1 | 36 of 58 | 21 of 58 | 98.8 / 3.2 (-0.2) | 98.6 / 5.2 (-0.4) |
| F0+ctx | 3.9 | 8.5 | 13.9 | 29.3 | 38 of 58 | 21 of 58 | 98.6 / 3.2 (-0.4) | 98.6 / 4.6 (-0.4) |
| P0 | 3.7 | 8.7 | 14.2 | 29.8 | 40 of 58 | 19 of 58 | 98.7 / 3.2 (-0.3) | 97.6 / 4.6 (-1.4) |
| F0+ctx +genre cols | 3.9 | 8.5 | 13.5 | 29.6 | 36 of 58 | 17 of 58 | 98.9 / 4.1 (-0.1) | 98.8 / 4.8 (-0.2) |
| F0+ctx +genre off | 3.3 | 8.3 | 13.4 | 27.5 | 36 of 58 | 18 of 58 | 99.0 / 5.8 (-0.0) | 98.7 / 4.9 (-0.3) |
| F0+ctx +c1 | 3.8 | 8.3 | 13.3 | 29.9 | 37 of 58 | 20 of 58 | 98.9 / 4.5 (-0.1) | 98.7 / 4.3 (-0.3) |
| F0+ctx +c2 | 4.1 | 8.3 | 14.3 | 30.1 | 37 of 58 | 18 of 58 | 98.9 / 4.1 (-0.1) | 98.7 / 4.6 (-0.3) |
| F0+ctx +c1+c2 | 3.6 | 8.0 | 14.5 | 30.2 | 35 of 58 | 19 of 58 | 99.0 / 4.0 (+0.0) | 98.8 / 4.6 (-0.2) |
| W0 +genre cols | 4.2 | 8.3 | 14.1 | 29.2 | 35 of 58 | 18 of 58 | 99.0 / 4.0 (+0.0) | 98.4 / 3.1 (-0.6) |
| W0 +c1 | 4.4 | 8.4 | 14.4 | 29.8 | 31 of 58 | 16 of 58 | 98.9 / 3.2 (-0.1) | 99.2 / 1.9 (+0.2) |
| W0 +c2 | 4.7 | 8.8 | 14.9 | 29.7 | 33 of 58 | 17 of 58 | 98.9 / 3.0 (-0.1) | 98.1 / 4.9 (-0.9) |
| W0 +c1+c2 | 4.2 | 8.5 | 14.6 | 30.1 | 30 of 58 | 18 of 58 | 98.9 / 3.2 (-0.1) | 97.0 / 4.6 (-2.0) |
| W0 burst-centered (what the app displays) | 4.0 | 9.2 | 16.7 | 31.5 | 12 of 58 | 10 of 58 | 99.0 / 5.2 (+0.0) | 99.4 / 4.8 (+0.4) |
| Random (seed 0) | 9.4 | 18.9 | 28.9 | 49.3 | 2 of 58 | 0 of 58 | 99.0 / 0.6 (-0.0) | 98.5 / 0.7 (-0.5) |
| Capture order, earlier first | 7.8 | 18.2 | 29.8 | 51.6 | 0 of 58 | 0 of 58 | 99.0 / 0.8 (-0.0) | 98.9 / 1.3 (-0.1) |
| Capture order, later first | 9.8 | 20.1 | 29.5 | 48.4 | 11 of 58 | 12 of 58 | 98.9 / 0.7 (-0.1) | 97.6 / 3.6 (-1.4) |
| Burst size, larger first | 7.3 | 15.2 | 24.5 | 43.4 | 4 of 58 | 4 of 58 | 99.0 / 0.7 (-0.0) | 100.0 / 0.0 (+1.0) |
| Absolute sharpness | 6.6 | 15.8 | 25.6 | 47.9 | 11 of 58 | 11 of 58 | 98.9 / 2.1 (-0.1) | 99.2 / 2.8 (+0.2) |
| `eye_focus` (missing last) | 8.4 | 14.6 | 21.7 | 38.7 | 4 of 58 | 4 of 58 | 99.0 / 1.6 (-0.0) | 100.0 / 0.0 (+1.0) |
| `photoTier` good | 8.0 | 16.5 | 25.3 | 43.3 | 3 of 58 | 3 of 58 | 99.0 / 0.9 (-0.0) | 100.0 / 0.0 (+1.0) |
| Rule: not good | - | - | - | - | 0 of 58 | 54 of 58 | 28.8 / 80.3 (-70.2) | - |
| Rule: eyes closed | - | - | - | - | 0 of 58 | 0 of 58 | 92.8 / 11.3 (-6.2) | - |
| Rule: eye focus low | - | - | - | - | 0 of 58 | 30 of 58 | 87.1 / 19.8 (-11.9) | - |
| Rule: face away | - | - | - | - | 0 of 58 | 7 of 58 | 75.1 / 23.5 (-23.9) | - |
| Rule: sharpness bottom 10% | - | - | - | - | 0 of 58 | 11 of 58 | 93.4 / 10.3 (-5.6) | - |
| Rule: sharpness bottom 20% | - | - | - | - | 0 of 58 | 17 of 58 | 84.2 / 20.5 (-14.8) | - |
| Rule: any of | - | - | - | - | 0 of 58 | 38 of 58 | 59.7 / 49.6 (-39.3) | - |
| rule count | 8.0 | 15.9 | 24.4 | 42.9 | 7 of 58 | 2 of 58 | 99.0 / 2.0 (+0.0) | 100.0 / 0.2 (+1.0) |
| R0 | 3.9 | 8.1 | 13.9 | 30.1 | 38 of 58 | 22 of 58 | 98.7 / 3.2 (-0.3) | 98.0 / 5.4 (-1.0) |
| G0 | 2.8 | 7.8 | 13.4 | 30.1 | 36 of 58 | 29 of 58 | 99.0 / 5.9 (+0.0) | 98.1 / 6.7 (-0.9) |

### Rule points

Each binary rule's single operating point, the mean over the 25 qualifying folders, and the folders where it keeps >= 95% / 99% of the picks.

| Rule | Realized pick recall % | `RN` % | `RF` % | Folders reaching 95% | Folders reaching 99% | Rejects removed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rule: not good | 28.8 | 80.3 | 79.1 | 0 of 25 | 0 of 25 | 54 of 58 |
| Rule: eyes closed | 92.8 | 11.3 | 10.7 | 11 of 25 | 1 of 25 | 0 of 58 |
| Rule: eye focus low | 87.1 | 19.8 | 18.8 | 5 of 25 | 3 of 25 | 30 of 58 |
| Rule: face away | 75.1 | 23.5 | 23.8 | 1 of 25 | 1 of 25 | 7 of 58 |
| Rule: sharpness bottom 10% | 93.4 | 10.3 | 9.8 | 9 of 25 | 3 of 25 | 11 of 58 |
| Rule: sharpness bottom 20% | 84.2 | 20.5 | 19.8 | 1 of 25 | 1 of 25 | 17 of 58 |
| Rule: any of | 59.7 | 49.6 | 48.3 | 1 of 25 | 1 of 25 | 38 of 58 |

### Sub-table: the 22 qualifying folders with >= 20 picks

The same per-folder values (the label-free cuts still from the other qualifying folders), averaged over these folders only.

| Score | `RN@95` % | `RN@99` % | `pAUC95` | `quantile@95` recall / `RN` (gap) | `quantile@99` recall / `RN` (gap) |
| --- | ---: | ---: | ---: | ---: | ---: |
| W0 | 16.9 (8.5) | 6.2 (6.9) | 0.542 (0.035) | 95.6 / 11.6 (+0.6) | 99.1 / 3.2 (+0.1) |
| W0+off | 17.5 (8.4) | 5.9 (3.9) | 0.541 (0.027) | 95.1 / 11.5 (+0.1) | 99.1 / 3.5 (+0.1) |
| F0 | 19.0 (8.0) | 6.4 (3.2) | 0.548 (0.024) | 95.8 / 14.7 (+0.8) | 99.3 / 3.4 (+0.3) |
| F0+ctx | 18.6 (8.3) | 6.9 (3.7) | 0.547 (0.026) | 95.6 / 14.7 (+0.6) | 99.1 / 3.3 (+0.1) |
| P0 | 17.7 (7.6) | 5.7 (3.6) | 0.542 (0.025) | 95.6 / 13.7 (+0.6) | 99.2 / 3.3 (+0.2) |
| F0+ctx +genre cols | 17.8 (8.4) | 6.3 (3.5) | 0.546 (0.027) | 95.5 / 14.1 (+0.5) | 98.7 / 4.2 (-0.3) |
| F0+ctx +genre off | 19.3 (8.9) | 7.5 (4.2) | 0.549 (0.028) | 95.6 / 14.2 (+0.6) | 98.8 / 5.9 (-0.2) |
| F0+ctx +c1 | 17.9 (7.8) | 6.5 (4.3) | 0.545 (0.026) | 95.5 / 15.3 (+0.5) | 98.7 / 4.6 (-0.3) |
| F0+ctx +c2 | 18.6 (8.5) | 7.1 (4.1) | 0.546 (0.028) | 95.5 / 13.9 (+0.5) | 99.1 / 4.2 (+0.1) |
| F0+ctx +c1+c2 | 19.0 (7.5) | 6.9 (5.0) | 0.548 (0.028) | 95.4 / 16.1 (+0.4) | 98.9 / 4.1 (-0.1) |
| W0 +genre cols | 17.8 (8.6) | 6.9 (6.3) | 0.544 (0.034) | 95.5 / 13.5 (+0.5) | 99.2 / 4.0 (+0.2) |
| W0 +c1 | 17.6 (8.7) | 6.8 (7.0) | 0.544 (0.037) | 95.4 / 12.6 (+0.4) | 99.1 / 3.4 (+0.1) |
| W0 +c2 | 15.7 (7.9) | 5.6 (5.2) | 0.538 (0.027) | 95.6 / 11.3 (+0.6) | 99.0 / 3.1 (+0.0) |
| W0 +c1+c2 | 16.6 (7.9) | 6.5 (5.6) | 0.542 (0.029) | 95.5 / 13.1 (+0.5) | 99.1 / 3.3 (+0.1) |
| W0 burst-centered (what the app displays) | 17.5 (7.6) | 6.6 (3.2) | 0.545 (0.024) | 95.3 / 13.0 (+0.3) | 98.9 / 5.3 (-0.1) |
| Random (seed 0) | 5.8 (3.4) | 1.1 (1.0) | 0.502 (0.009) | 95.8 / 4.3 (+0.8) | 98.9 / 0.7 (-0.1) |
| Capture order, earlier first | 6.4 (3.3) | 1.4 (1.4) | 0.503 (0.007) | 95.2 / 5.3 (+0.2) | 98.8 / 0.9 (-0.2) |
| Capture order, later first | 5.4 (3.9) | 2.3 (3.7) | 0.505 (0.019) | 94.2 / 5.0 (-0.8) | 98.8 / 0.8 (-0.2) |
| Burst size, larger first | 7.9 (4.2) | 1.4 (1.4) | 0.506 (0.010) | 94.7 / 6.7 (-0.3) | 98.9 / 0.8 (-0.1) |
| Absolute sharpness | 8.8 (4.2) | 3.2 (2.9) | 0.515 (0.016) | 94.6 / 8.5 (-0.4) | 98.8 / 2.2 (-0.2) |
| `eye_focus` (missing last) | 6.3 (2.9) | 2.0 (1.3) | 0.505 (0.008) | 94.2 / 6.1 (-0.8) | 98.8 / 1.7 (-0.2) |
| `photoTier` good | 6.8 (3.7) | 1.9 (3.0) | 0.502 (0.001) | 94.3 / 7.2 (-0.7) | 98.8 / 0.9 (-0.2) |
| Rule: not good | 0.0 (0.0) | 0.0 (0.0) | - | 26.0 / 82.6 (-69.0) | 26.0 / 82.6 (-73.0) |
| Rule: eyes closed | 3.1 (3.7) | 0.0 (0.0) | - | 93.5 / 9.8 (-1.5) | 93.5 / 9.8 (-5.5) |
| Rule: eye focus low | 0.9 (2.3) | 0.3 (1.2) | - | 86.0 / 21.6 (-9.0) | 86.0 / 21.6 (-13.0) |
| Rule: face away | 0.0 (0.0) | 0.0 (0.0) | - | 72.9 / 25.8 (-22.1) | 72.9 / 25.8 (-26.1) |
| Rule: sharpness bottom 10% | 3.5 (5.3) | 0.5 (2.4) | - | 92.9 / 10.4 (-2.1) | 92.9 / 10.4 (-6.1) |
| Rule: sharpness bottom 20% | 0.0 (0.0) | 0.0 (0.0) | - | 83.7 / 20.5 (-11.3) | 83.7 / 20.5 (-15.3) |
| Rule: any of | 0.0 (0.0) | 0.0 (0.0) | - | 56.4 / 51.6 (-38.6) | 56.4 / 51.6 (-42.6) |
| rule count | 7.8 (3.4) | 3.0 (2.0) | 0.509 (0.010) | 95.2 / 6.9 (+0.2) | 98.9 / 2.0 (-0.1) |
| R0 | 18.6 (9.1) | 6.8 (3.8) | 0.547 (0.028) | 95.4 / 14.5 (+0.4) | 99.2 / 3.4 (+0.2) |
| G0 | 20.4 (8.4) | 8.4 (5.3) | 0.555 (0.032) | 94.9 / 17.0 (-0.1) | 98.9 / 5.9 (-0.1) |

### Paired differences (score minus comparison)

Over the 25 qualifying folders: the mean paired per-folder difference with the 1,000-draw folder bootstrap 95% interval (seed 0, fresh per comparison and metric), in points for `RN`; the one-sided Wilcoxon signed-rank p on `RN@95` (zeros dropped); the folders where the score's `RN@95` is at or above the comparison's. Comparison bar: `RN@95` gain >= +3.0 points, >= 16 of 25 folders at or above, p < 0.05.

| Comparison | `RN@95`, points | p | Folders at or above | `RN@99`, points | `pAUC95` | Bars met | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `W0+off` - `W0` | +0.8 [-0.7, +2.5] | 0.3110 | 14 of 25 | -0.0 [-1.9, +1.6] | +0.000 [-0.008, +0.008] | 0 of 3 | fail |
| `F0` - `W0` | +2.8 [-0.3, +6.0] | 0.0401 | 16 of 25 | +1.1 [-1.9, +4.2] | +0.010 [-0.005, +0.026] | 2 of 3 | fail |
| `F0+ctx` - `W0` | +1.8 [-1.1, +4.8] | 0.1514 | 14 of 25 | +1.0 [-1.6, +3.3] | +0.006 [-0.006, +0.019] | 0 of 3 | fail |
| `P0` - `W0` | +1.3 [-1.1, +3.9] | 0.1436 | 13 of 25 | +0.2 [-2.3, +2.7] | +0.003 [-0.009, +0.016] | 0 of 3 | fail |
| `F0+ctx +genre cols` - `W0` | +1.2 [-2.1, +4.3] | 0.1562 | 15 of 25 | +0.5 [-2.3, +2.7] | +0.005 [-0.009, +0.018] | 0 of 3 | fail |
| `F0+ctx +genre off` - `W0` | +2.6 [-0.5, +6.0] | 0.0369 | 16 of 25 | +1.6 [-1.2, +3.9] | +0.009 [-0.005, +0.022] | 2 of 3 | fail |
| `F0+ctx +c1` - `W0` | +1.2 [-1.3, +3.8] | 0.1152 | 13 of 25 | +0.6 [-1.7, +2.6] | +0.004 [-0.007, +0.014] | 0 of 3 | fail |
| `F0+ctx +c2` - `W0` | +1.9 [-1.0, +4.8] | 0.1436 | 15 of 25 | +1.3 [-1.3, +3.7] | +0.006 [-0.006, +0.018] | 0 of 3 | fail |
| `F0+ctx +c1+c2` - `W0` | +2.3 [-0.3, +4.8] | 0.0505 | 16 of 25 | +1.0 [-1.2, +3.2] | +0.007 [-0.003, +0.017] | 1 of 3 | fail |
| `W0 +genre cols` - `W0` | +1.0 [-0.2, +2.0] | 0.0640 | 16 of 25 | +0.8 [-0.0, +1.7] | +0.003 [-0.001, +0.007] | 1 of 3 | fail |
| `W0 +c1` - `W0` | +0.6 [-0.1, +1.3] | 0.1514 | 14 of 25 | +0.5 [-0.3, +1.2] | +0.002 [-0.001, +0.004] | 0 of 3 | fail |
| `W0 +c2` - `W0` | -0.9 [-2.0, +0.1] | 0.9313 | 12 of 25 | -0.3 [-1.8, +0.7] | -0.002 [-0.008, +0.002] | 0 of 3 | fail |
| `W0 +c1+c2` - `W0` | -0.4 [-1.7, +1.2] | 0.8826 | 10 of 25 | +0.2 [-1.0, +1.2] | +0.000 [-0.006, +0.005] | 0 of 3 | fail |
| `W0 burst-centered (what the app displays)` - `W0` | +0.2 [-3.1, +3.4] | 0.3755 | 12 of 25 | +0.0 [-4.3, +3.4] | +0.001 [-0.018, +0.017] | 0 of 3 | fail |
| `Random (seed 0)` - `W0` | -10.8 [-15.1, -6.2] | 1.0000 | 4 of 25 | -5.5 [-9.4, -1.8] | -0.040 [-0.060, -0.020] | 0 of 3 | fail |
| `Capture order, earlier first` - `W0` | -9.5 [-12.9, -6.3] | 1.0000 | 3 of 25 | -4.5 [-7.3, -2.4] | -0.035 [-0.049, -0.025] | 0 of 3 | fail |
| `Capture order, later first` - `W0` | -9.9 [-13.9, -5.4] | 0.9998 | 3 of 25 | -3.2 [-7.0, +0.6] | -0.031 [-0.051, -0.011] | 0 of 3 | fail |
| `Burst size, larger first` - `W0` | -8.3 [-11.6, -5.3] | 1.0000 | 4 of 25 | -4.6 [-7.3, -2.2] | -0.034 [-0.049, -0.021] | 0 of 3 | fail |
| `Absolute sharpness` - `W0` | -7.4 [-10.8, -3.8] | 0.9997 | 5 of 25 | -2.8 [-6.1, +0.2] | -0.024 [-0.040, -0.009] | 0 of 3 | fail |
| `eye_focus` (missing last) - `W0` | -9.4 [-13.0, -5.5] | 0.9999 | 4 of 25 | -3.7 [-6.7, -0.9] | -0.033 [-0.049, -0.018] | 0 of 3 | fail |
| `photoTier` good - `W0` | -7.3 [-11.3, -3.0] | 0.9985 | 6 of 25 | -2.2 [-5.1, +1.1] | -0.042 [-0.059, -0.028] | 0 of 3 | fail |
| `Rule: not good` - `W0` | -16.6 [-20.1, -13.2] | 1.0000 | 0 of 25 | -7.1 [-11.0, -4.4] | - | 0 of 3 | fail |
| `Rule: eyes closed` - `W0` | -13.7 [-17.1, -10.5] | 1.0000 | 1 of 25 | -7.0 [-10.5, -4.4] | - | 0 of 3 | fail |
| `Rule: eye focus low` - `W0` | -15.7 [-19.5, -12.0] | 1.0000 | 0 of 25 | -6.8 [-10.4, -4.0] | - | 0 of 3 | fail |
| `Rule: face away` - `W0` | -16.4 [-19.7, -12.9] | 1.0000 | 0 of 25 | -6.9 [-10.3, -4.4] | - | 0 of 3 | fail |
| `Rule: sharpness bottom 10%` - `W0` | -12.7 [-16.6, -8.8] | 1.0000 | 2 of 25 | -5.9 [-9.2, -3.2] | - | 0 of 3 | fail |
| `Rule: sharpness bottom 20%` - `W0` | -15.7 [-19.8, -11.5] | 1.0000 | 1 of 25 | -6.3 [-10.2, -2.9] | - | 0 of 3 | fail |
| `Rule: any of` - `W0` | -16.0 [-19.4, -12.7] | 1.0000 | 0 of 25 | -6.5 [-9.7, -4.3] | - | 0 of 3 | fail |
| `rule count` - `W0` | -8.7 [-12.1, -5.5] | 1.0000 | 2 of 25 | -3.5 [-6.8, -0.8] | -0.032 [-0.047, -0.020] | 0 of 3 | fail |
| `R0` - `W0` | +1.8 [-1.4, +4.7] | 0.0865 | 16 of 25 | +0.7 [-2.0, +3.0] | +0.006 [-0.008, +0.019] | 1 of 3 | fail |
| `G0` - `W0` | +5.4 [+2.3, +9.0] | 0.0021 | 19 of 25 | +4.3 [+1.4, +7.8] | +0.024 [+0.009, +0.042] | 3 of 3 | pass |
| `F0+ctx +genre cols` - `F0+ctx` | -0.7 [-2.6, +1.2] | 0.6159 | 13 of 25 | -0.5 [-1.6, +0.5] | -0.001 [-0.008, +0.005] | 0 of 3 | fail |
| `F0+ctx +genre off` - `F0+ctx` | +0.8 [-0.7, +2.3] | 0.1498 | 14 of 25 | +0.6 [-0.6, +2.0] | +0.003 [-0.003, +0.009] | 0 of 3 | fail |
| `F0+ctx +c1` - `F0+ctx` | -0.7 [-1.9, +0.5] | 0.7774 | 11 of 25 | -0.4 [-1.5, +0.5] | -0.002 [-0.008, +0.002] | 0 of 3 | fail |
| `F0+ctx +c2` - `F0+ctx` | +0.1 [-0.5, +0.7] | 0.5297 | 10 of 25 | +0.3 [-0.2, +0.8] | -0.000 [-0.002, +0.002] | 0 of 3 | fail |
| `F0+ctx +c1+c2` - `F0+ctx` | +0.4 [-0.5, +1.5] | 0.3017 | 12 of 25 | -0.0 [-0.7, +0.9] | +0.001 [-0.002, +0.004] | 0 of 3 | fail |
| `R0` - `F0+ctx` | -0.1 [-1.0, +1.0] | 0.8561 | 12 of 25 | -0.3 [-0.9, +0.3] | -0.000 [-0.004, +0.003] | 0 of 3 | fail |
| `G0` - `F0+ctx` | +3.6 [+0.8, +7.0] | 0.0275 | 17 of 25 | +3.3 [+0.8, +6.8] | +0.017 [+0.004, +0.035] | 3 of 3 | pass |

### `G0` on the raw weights (check)

`G0` fit with `sample_weight` the dump's per-frame weight as is (it sums to 1 per folder), not rescaled to mean 1 per training fold; everything else as `G0`. The plan's literal text; shown so the reader can see whether the comparison bar still passes without the rescale.

| Score | `RN@95` % | `RN@99` % | `pAUC95` | Frame AUC |
| --- | ---: | ---: | ---: | ---: |
| `G0` | 22.0 (11.5) | 11.5 (12.1) | 0.568 (0.059) | 0.670 |
| `G0 on the raw weights (check)` | 17.6 (10.1) | 9.0 (8.0) | 0.551 (0.042) | 0.649 |

| Comparison | `RN@95`, points | p | Folders at or above | `RN@99`, points | `pAUC95` | Bars met | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `G0 on the raw weights (check)` - `W0` | +1.0 [-1.2, +3.4] | 0.2891 | 13 of 25 | +1.8 [+0.1, +3.7] | +0.007 [-0.002, +0.017] | 0 of 3 | fail |
| `G0 on the raw weights (check)` - `F0+ctx` | -0.9 [-3.5, +1.8] | 0.7546 | 11 of 25 | +0.8 [-1.2, +3.0] | +0.001 [-0.010, +0.012] | 0 of 3 | fail |

### Best filter and the worth-a-filter bar

| Reading | Score | Oracle `RN` % | Label-free `quantile` recall / `RN` (gap) | Worth-a-filter bar (`quantile@95` recall >= 93%, `RN` >= 25%) |
| --- | ---: | ---: | ---: | ---: |
| Best at 95% (highest mean `RN@95`) | `G0` | 22.0 | 94.7 / 16.8 (-0.3) | not met |
| Best at 99% (highest mean `RN@99`) | `G0` | 11.5 | 99.0 / 5.9 (+0.0) | - |

### Per folder (held out)

Best filter by mean `RN@95`: `G0`. Label-free pairs only for the qualifying folders.

| Folder | Frames | Picks | Pick rate | Qualifying | `W0` `RN@95` % | `W0` `quantile@95` | `G0` `RN@95` % | `G0` `quantile@95` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | 1337 | 199 | 14.9% | yes | 24.5 | 99.5 / 11.7 (+4.5) | 29.3 | 99.0 / 17.5 (+4.0) |
| `2026-06-13` | 933 | 118 | 12.6% | yes | 21.7 | 100.0 / 11.4 (+5.0) | 22.2 | 97.5 / 16.8 (+2.5) |
| `2026-06-14` | 1520 | 161 | 10.6% | yes | 27.0 | 98.1 / 11.2 (+3.1) | 28.9 | 98.8 / 16.7 (+3.8) |
| `2026-06-20` | 1050 | 133 | 12.7% | yes | 8.1 | 94.0 / 11.8 (-1.0) | 13.8 | 94.7 / 17.1 (-0.3) |
| `2026-07-02` | 106 | 15 | 14.2% | yes | 2.2 | 86.7 / 11.0 (-8.3) | 7.7 | 80.0 / 15.4 (-15.0) |
| `2026-07-05` | 1415 | 255 | 18.0% | yes | 15.5 | 95.3 / 12.2 (+0.3) | 26.0 | 97.3 / 17.8 (+2.3) |
| `2026-07-11` | 468 | 57 | 12.2% | yes | 7.5 | 94.7 / 11.7 (-0.3) | 10.9 | 91.2 / 16.8 (-3.8) |
| `2026-07-18` | 1545 | 292 | 18.9% | yes | 14.0 | 95.9 / 12.1 (+0.9) | 23.6 | 97.3 / 18.0 (+2.3) |
| `2026-07-21` | 32 | 7 | 21.9% | no | 12.0 | - | 16.0 | - |
| `2026-07-22` | 189 | 32 | 16.9% | yes | 10.8 | 93.8 / 11.5 (-1.2) | 15.9 | 93.8 / 17.2 (-1.2) |
| `2026-07-23` | 18 | 4 | 22.2% | no | - | - | 64.3 | - |
| `2026-07-24` | 218 | 34 | 15.6% | yes | 14.1 | 97.1 / 11.4 (+2.1) | 4.9 | 94.1 / 17.4 (-0.9) |
| `2026-07-26` | 64 | 7 | 10.9% | yes | 8.8 | 85.7 / 10.5 (-9.3) | 36.8 | 100.0 / 15.8 (+5.0) |
| `2026-07-30` | 936 | 105 | 11.2% | yes | 2.6 | 84.8 / 11.7 (-10.2) | 7.7 | 86.7 / 16.2 (-8.3) |
| `2026-07-31` | 349 | 52 | 14.9% | yes | 14.1 | 96.2 / 11.4 (+1.2) | 11.1 | 88.5 / 16.5 (-6.5) |
| `2026-08-01` | 3401 | 463 | 13.6% | yes | 6.5 | 91.4 / 11.7 (-3.6) | 20.1 | 95.7 / 17.3 (+0.7) |
| `2026-08-02` | 150 | 28 | 18.7% | yes | 34.4 | 100.0 / 12.3 (+5.0) | 27.9 | 100.0 / 18.0 (+5.0) |
| `2026-08-08` | 415 | 73 | 17.6% | yes | 12.3 | 95.9 / 12.0 (+0.9) | 8.8 | 89.0 / 17.0 (-6.0) |
| `2026-08-15` | 324 | 37 | 11.4% | yes | 7.7 | 86.5 / 11.5 (-8.5) | 9.4 | 83.8 / 15.7 (-11.2) |
| `2026-08-16` | 59 | 6 | 10.2% | yes | 32.1 | 100.0 / 9.4 (+5.0) | 56.6 | 100.0 / 15.1 (+5.0) |
| `2026-08-22` | 2677 | 267 | 10.0% | yes | 24.4 | 97.0 / 11.2 (+2.0) | 24.2 | 97.4 / 16.5 (+2.4) |
| `2026-08-29` | 3451 | 475 | 13.8% | yes | 13.9 | 95.6 / 11.6 (+0.6) | 30.1 | 98.1 / 17.2 (+3.1) |
| `2026-09-13-b` | 1795 | 232 | 12.9% | yes | 12.3 | 96.1 / 11.6 (+1.1) | 24.8 | 95.3 / 17.2 (+0.3) |
| `2026-09-19` | 2134 | 282 | 13.2% | yes | 28.5 | 98.2 / 11.3 (+3.2) | 24.7 | 96.8 / 16.9 (+1.8) |
| `2026-09-27-a` | 4830 | 729 | 15.1% | yes | 19.9 | 96.2 / 11.8 (+1.2) | 27.6 | 96.8 / 17.2 (+1.8) |
| `2026-09-27-b` | 1240 | 171 | 13.8% | yes | 27.3 | 98.2 / 11.3 (+3.2) | 31.0 | 98.2 / 17.2 (+3.2) |
| `2026-10-03` | 1803 | 185 | 10.3% | yes | 24.2 | 98.4 / 11.0 (+3.4) | 25.2 | 98.4 / 16.6 (+3.4) |

`C` chosen per fold (count of folds):

| Model | 0.01 | 0.1 | 1.0 | 10.0 |
| --- | ---: | ---: | ---: | ---: |
| R0 | 2 | 17 | 7 | 1 |
| R0 at pick weight 1, `C` by AUC (check) | 0 | 0 | 20 | 7 |

### Wall time

| Stage | Seconds |
| --- | ---: |
| Load (dump, cue files, `scores.tsv`, rules) | 4 |
| R0 at pick weight 1, `C` by AUC (check) | 11 |
| R0 | 10 |
| G0 | 17 |
| G0 on the raw weights (check) | 20 |
| Metrics | 1 |

Total 63 s on 20 jobs.

### Reading

Facts only; the Decision is Step 3's, after the ablations.

- The reference checks hold: `W0` and `F0+ctx` read frame AUC 0.666 from
  `scores.tsv`, random reads `RN@95` 5.8% and `pAUC95` 0.504, and `R0`'s
  code path at pick weight 1 with the AUC-chosen `C` reproduces `F0+ctx` to
  5e-6 (the file's rounding).
- The reference `W0` removes 16.6% of the non-picks at 95% pick recall
  (oracle) and 7.1% at 99%, `pAUC95` 0.544. Its label-free `quantile@95`
  keeps 95.0% of the picks on average and removes 11.5% of the non-picks;
  `absolute@95` realizes 91.4% recall.
- The highest mean `RN@95` and `RN@99` are `G0`'s, 22.0% and 11.5%
  (`pAUC95` 0.568). Against `W0`: +5.4 points [+2.3, +9.0], p = 0.0021, 19 of
  25 folders at or above, all 3 comparison bars met; against `F0+ctx`: +3.6
  [+0.8, +7.0], p = 0.0275, 17 of 25, all 3 met. Its label-free
  `quantile@95` keeps 94.7% of the picks and removes 16.8% of the non-picks;
  per folder its realized recall is below 95% in 9 of the 25 folders and
  below 90% in 5 (80.0% at the lowest, `2026-07-02`, 15 picks); `W0`'s is
  below 95% in 8 and below 90% in 4.
- `G0`'s pass depends on a deviation from the plan: its per-folder
  class-balanced weights are rescaled to mean 1 on each fold's training
  frames, where the plan says `sample_weight = d["weight"]` (each folder
  summing to 1). Fit on the raw weights, the same setting (the committed
  check row `G0 on the raw weights (check)`) reads `RN@95` 17.6%, `RN@99`
  9.0%, `pAUC95` 0.551, frame AUC 0.649; against `W0` +1.0 points [-1.2,
  +3.4], p = 0.2891, 13 of 25 folders at or above, 0 of 3 bars met; against
  `F0+ctx` -0.9 [-3.5, +1.8], p = 0.7546, 11 of 25, 0 of 3. So without the
  rescale `G0` does not pass the comparison bar.
- `R0` reads as `F0+ctx` (-0.1 points [-1.0, +1.0]); its inner CV chose
  `C` = 0.1 in 17 of 27 folds.
- No other score passes the comparison bar against `W0`. Closest: `F0`
  +2.8 points (p = 0.040, 16 of 25) and `F0+ctx +genre off` +2.6 (p = 0.037,
  16 of 25), each meeting 2 of 3 bars. The burst-centered `W0` reads +0.2.
- Genre against its base: `F0+ctx +genre cols` -0.7, `F0+ctx +genre off`
  +0.8, `W0 +genre cols` +1.0 (p = 0.064, 16 of 25), `W0+off` +0.8; none
  meets the bar. c1 / c2 read -0.9 to +0.6 points on both bases.
- The baselines remove 5.8–9.3% of the non-picks at 95% (random lowest,
  `photoTier` good highest), 7.3–10.8 points below `W0`.
- No binary rule keeps 95% of the picks in more than 11 of the 25 folders.
  Their points (mean realized recall / `RN`): `eyes closed` (EAR <= 0.137,
  2,385 frames) 92.8% / 11.3%, reaching 95% in 11 folders; `sharpness bottom
  10%` 93.4% / 10.3%, 9 folders; `eye focus low` 87.1% / 19.8%, 5; `sharpness
  bottom 20%` 84.2% / 20.5%, 1; `face away` 75.1% / 23.5%, 1; `any of`
  59.7% / 49.6%, 1; `not good` 28.8% / 80.3%, 0. Scored 0 where they do not
  reach 95%, their mean `RN@95` is 0.0–3.9%. The graded `rule count` reads
  `RN@95` 7.9%, `pAUC95` 0.512.
- Picks lost in the bottom 10% / 20% of a folder's frames: `W0` 4.7% / 8.5%,
  `F0+ctx` 3.9% / 8.5%, `G0` 2.8% / 7.8%, random 9.4% / 18.9%.
- Explicit rejects (58, 3 folders): at the oracle `RN@95` cut `W0` removes
  31, `F0+ctx` 38, `G0` 36; at `quantile@95` `W0` 15, `G0` 29. Among the
  rules, `eyes closed` removes none of them, `eye focus low` 30, `any of` 38,
  `not good` 54.
- Over the 22 folders with >= 20 picks the order holds: `G0` `RN@95` 20.4%,
  `F0+ctx` 18.6%, `W0` 16.9%.
