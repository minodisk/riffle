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
  q = 10% and 20% (graded: the absolute-sharpness baseline already gives the
  curve).
- `any of`: `eyes closed` or `eye focus low` or `face away` or
  `sharpness bottom 10%`.
- `rule count`: the graded score "number of the four rules of `any of` that
  fire" (0–4), more fired removed first, ties by the seed-0 draw.

For a binary rule and for `rule count`, `RN@r` is the point itself when the
rule's realized recall is >= r, else "does not reach r" (a rule cannot be
relaxed); the table shows the pair.

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

Not corrected for multiplicity: about 25 score rows, 15 comparisons and 60
ablation cells are read at p < 0.05 or interval-below-zero, so two or three
false positives are expected; the Decision reads a lone marginal pass as
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
  **stands** unless `+genre cols` or `+genre off` passes the comparison bar
  against its base on `RN@95`; otherwise it is **reopened** and recorded as
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
