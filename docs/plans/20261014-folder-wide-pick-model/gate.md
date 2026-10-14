# Step 2: offline folder-wide pick model gate

## Pre-registration

Written 2026-10-14T13:00+09:00, before `folder_gate.py` exists and before any
fit has run, and merged on its own before the fit (the merge commit of the
PR that adds this section, without the results below, is the record, as
`ce0ea14` was for the in-burst
[gate](../_archived/20261012-image-embeddings/gate.md)). Nothing in this
section changes after the results are in; a deviation found while
implementing Step 2 is recorded in `learnings.md` with its reason and does
not touch the criterion.

The question: can one per-frame model score the probability that the user
picks a photo on a scale that compares frames of different bursts and singles
across a whole folder, better than what the folder's non-embedding signals
already give?

### Data

- The 27 sidecar-labeled ARW folders of
  [burst-keep-score data.md](../_archived/20261008-burst-keep-score/data.md),
  read from the app dump in `D:\Photos\tests\2026-10-12-rank-app\` as is:
  `features/<folder>.tsv`, `features/<folder>.output.txt` (the stems of
  `Output/`), `embed.tsv`, `genre_text.tsv`, `prior_scores.tsv`, through the
  archived [`gate.py`](../_archived/20261012-image-embeddings/gate.py)'s
  `load` and `build`, imported (not copied) the way
  [`rank_prior.py`](../_archived/20261012-image-embeddings/rank_prior.py)
  does.
- Labels as the in-burst gate's, through `gate.load`: a frame is a pick when
  its `.dop` flag (first item) is Pick, or it has a virtual copy in `Output/`
  (`<stem>_<n>`), or it has no `.dop` and its XMP flag is Pick. Every other
  frame is "not chosen" (these folders count as culled). An explicit reject
  is a frame whose deciding flag (the `.dop`'s, else the XMP's) is Reject; it
  stays among the "not chosen" frames and is also reported separately. The
  dump's own app `flag` column (from `index.sqlite`) is **not** the label: it
  misses the picks that exist only as a virtual copy in `Output/` (427 of
  them, 3992 app picks against 4419). gate.md's rule is used so that the
  numbers are comparable with the in-burst gate.
- Bursts: `gate.bursts_of`, which is `burst.ts` `groupBursts` and
  `rank::bursts` at gap 1000 ms (capture time, subsec right-padded, file
  name; a gap equal to 1000 ms stays in the burst). A single (a burst of one
  frame) is a burst like any other; no burst needs to be scorable.

### Unit and folds

- Every frame with a vector in `embed.tsv` is one sample; a frame without
  one is dropped and counted.
- Leave-one-folder-out over the 27 folders (shooting day = folder): each
  folder is held out once, every model and variant fit on the other 26.
  Every number below is on the held-out folders.
- `C` is chosen per outer fold from {0.01, 0.1, 1, 10} by an inner
  grouped-by-folder cross-validation on the training folders as `gate.py`
  does (`GroupKFold`, 5 splits or fewer when there are fewer folders),
  maximizing the mean per-folder frame AUC (metric 1 below) over the inner
  validation folders that hold at least one pick and one non-pick; on a tie
  the smaller `C`.
- **Qualifying folders**: the held-out folders with at least 50 frames and
  at least 5 picks. Means, per-folder SDs (sample SD, ddof 1) and the tests
  are over them only; a folder outside them is still trained on and its
  numbers are listed in the per-folder table.

### Model family

A per-frame (not pairwise) L2-regularized logistic regression with an
intercept (scikit-learn `LogisticRegression`, lbfgs, `max_iter` 2000), one
intercept per fit (not per folder).

- **Sample weights**: each training folder carries a total weight of 1;
  within a folder its picks carry half of it and its non-picks the other
  half (each pick `1 / (2 x picks)`, each non-pick `1 / (2 x non-picks)`),
  so a folder's pick rate does not become the model. A folder with no pick
  (or no non-pick) gives its whole weight of 1 to the class it has.
- **Technical columns**: `gate.py`'s `TECH`, as `gate.tech_row` computes
  them (ln(sharpness / burst max sharpness), `eye_focus` as a logit, eyes
  open, |yaw|, |pitch|, |roll|, `photoTier` good, and the indicators for no
  `eye_focus`, no eyes judgment and no pose), plus **absolute
  `ln sharpness`** (the natural log of the frame's sharpness score) beside
  the burst-relative one. A missing value is set to the training folders'
  mean, as in `gate.py`.
- **Context columns**: `ln burst size`; position (index / (burst size - 1),
  0 for a single); first frame (1 for the first frame of a burst, so 1 for a
  single).
- **Embedding columns**: the L2-normalized vector projected to 32 PCA
  dimensions, whitened, the PCA fit on every frame of the training folders
  only.
- Every column standardized (mean and SD, unweighted) on the training
  folders' frames.
- **Variants**, each with its own `C` per fold:
  - **Full model**: technical + context + embedding.
  - **Without embedding**: technical + context.
  - **Embedding only**: the 32 PCA columns.
  - **Context only**: `ln burst size`, position and first frame.
  - **Folder-centered embedding**: the full model with each frame's vector
    minus the mean vector of its folder's frames (held-out folder included;
    no label is used) before the PCA, which asks whether "stands out in this
    folder" helps.
  - **Full + genre**: the full model plus one 0 / 1 column per label of the
    shipped `crates/core/genres/siglip2.json` (9 labels), on when the label
    is on as `genre.rs` decides it (the cosine of the normalized vector with
    the label's vector at or above the label's threshold, a sub-genre turning
    its parent on); `Unknown` is every column 0.

### Baselines

On the same folds and folders, each a per-frame score (higher = more likely
picked); ties count half in every AUC.

- Random: a uniform draw, seed 0.
- Capture order, earlier first (score = minus capture time) and later first
  (score = capture time), in the `gate.order_key` order.
- Burst size, larger first.
- Absolute sharpness.
- `eye_focus`, a missing value ranking last.
- `photoTier` good (0 / 1).
- The in-burst prior's score from `prior_scores.tsv`, applied folder-wide
  (a frame missing from the file ranks last). It is centered per burst, so
  it is expected to read near chance across bursts; it is a reading aid,
  not a candidate for check 1, since it uses the embedding.

### Metrics

Each computed per held-out folder, then the mean and the per-folder SD over
the qualifying folders.

1. **Frame AUC**: pick vs not chosen over all the folder's frames.
2. **Burst AUC**, the cross-burst question: one unit per burst, singles
   included; the unit's score is its highest frame score and its label is
   whether the burst holds a pick. A folder whose units are all of one label
   has no burst AUC and is left out of this metric's mean and test.
3. **Singles AUC**: pick vs not chosen over the folder's singles only, where
   the in-burst ranking is silent; computed on the qualifying folders with at
   least 5 single picks and 5 single non-picks, the count of such folders
   reported beside it.
4. **Lift at the top tenth**: the pick share among the folder's top
   `ceil(frames / 10)` frames by score (ties at the cut broken by a seeded
   random draw, seed 0) divided by the folder's pick rate.
5. Also reported, not part of the criterion:
   - the pick vs explicit-reject AUC per folder with at least one of each,
     and its mean over those folders with their count;
   - a learning curve of the full model: each held-out folder with the model
     trained on 5, 10 and 20 of the other folders (three seeded draws each,
     the seed `1000 x k + 100 x draw + folder index` as `gate.py`'s curve)
     and on all 26, reading the mean frame AUC and burst AUC;
   - the weights of the full model fit on all 27 folders (`C` by the same
     inner cross-validation on all 27), each column and, per group, the
     L2 norm, grouped as `rank.rs`'s `REASONS` group them, with two
     extensions: `Burst position` (there the two `POSITION` columns only) is
     renamed `Burst context` and also takes `ln burst size`, and `Sharpness`
     (there column 0 only) also takes the new absolute `ln sharpness` column.
     The groups are `Look` (the PCA columns), `Burst context` (position,
     first frame, `ln burst size`), `Sharpness` (both sharpness columns),
     `AF eye focus`, `Eyes open`, `Head pose`, `Good photo`, each
     missing-value indicator joining the group of the value it stands for;
   - the data counts (frames, picks, explicit rejects, bursts, singles,
     per-folder pick rates), the `C` chosen per fold for every variant, and
     the per-folder table of the full model, the without-embedding variant
     and the burst-size baseline (frame AUC and burst AUC).

### Criterion

- **Significance rule**: a paired one-sided Wilcoxon signed-rank test over
  the per-folder values (model minus comparison, alternative "greater"),
  zero differences dropped (`scipy.stats.wilcoxon`, `zero_method="wilcox"`,
  `method="auto"`; without scipy, the exact signed-rank distribution with
  mid-ranks for ties), at p < 0.05. Not corrected for multiplicity: the
  checks are an intersection, all must hold.
- **The gate passes** when the chosen model (the full model, or the full +
  genre variant if it wins its test, see below), held out, over the qualifying
  folders, meets all three:
  1. its per-folder frame AUC is greater than that of the **best
     non-embedding comparison** by the test. That comparison is whichever of
     every baseline and variant that uses no embedding (random, capture
     order earlier first, capture order later first, burst size, absolute
     sharpness, `eye_focus`, `photoTier` good, without embedding, context
     only) has the highest mean frame AUC over the qualifying folders. The
     in-burst prior's score, embedding only, the folder-centered embedding
     and the genre variant are not candidates (they use the embedding);
  2. its per-folder burst AUC is greater than the burst-size baseline's by
     the test;
  3. its mean lift at the top tenth is at least **2.0**.
- For checks 1 and 2, the mean gain is also reported against the
  comparison's per-folder SD (the rule of the earlier gates: met when the
  gain exceeds that SD), for reading only; it does not decide.

### Genre variant

The full + genre variant replaces the full model as the chosen model only if
its per-folder frame AUC beats the full model's by the same test at
p < 0.05 over the qualifying folders; otherwise the full model is the chosen
model. The 27 folders are almost all sports, so this is not expected to
show; if it does, the shipped model depends on `genre.rs` at run time.

### What happens on fail

- **Pass**: Steps 3–5 proceed with the chosen model.
- **Fail**: Steps 3–5 are dropped, Decision 1 in the plan records the tables,
  and the plan ends with the wrap-up only.
