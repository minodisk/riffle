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

## Results

Run 2026-10-14 with `python -I folder_gate.py D:\Photos\tests\2026-10-12-rank-app 20` (the `%TEMP%\embedvenv` interpreter of the in-burst gate: numpy 2.5.4, scikit-learn 1.9.1, scipy 1.18.1, so the Wilcoxon test is `scipy.stats.wilcoxon`); its output, saved as `folder_gate.out.md` next to the dump, is pasted below with its headings one level down. Wall time 120 s on 20 jobs.

### Data

| Count | Value |
| --- | ---: |
| Folders / qualifying (>= 50 frames, >= 5 picks) | 27 / 25 |
| Frames (with a vector) / picks / explicit rejects | 32459 / 4419 / 58 |
| Explicit rejects that are also picks (a virtual copy) | 0 |
| Frames without a vector (dropped) | 0 |
| Bursts (gap 1000 ms), singles included | 6969 |
| Singles (bursts of one) / of them picks | 1943 / 242 |
| Bursts holding a pick | 2053 |
| Folder pick rate min / median / max | 10.0% / 13.8% / 22.2% |
| Frames with an on genre label / `Unknown` | 30538 / 1921 |

### Held-out results (leave one folder out, qualifying folders)

Each cell: the mean over the qualifying folders (per-folder SD in brackets); `n` the folders it is over.

| Score | Frame AUC | Burst AUC | n | Singles AUC | n | Lift at top tenth |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Full model (embedding + technical + context) | 0.666 (0.102) | 0.710 (0.069) | 25 | 0.611 (0.104) | 13 | 2.082 (0.982) |
| Full + genre | 0.667 (0.099) | 0.712 (0.071) | 25 | 0.594 (0.104) | 13 | 2.156 (0.913) |
| Without embedding (technical + context) | 0.624 (0.071) | 0.726 (0.067) | 25 | 0.602 (0.095) | 13 | 1.743 (0.681) |
| Embedding only | 0.629 (0.111) | 0.689 (0.111) | 25 | 0.552 (0.103) | 13 | 2.008 (0.947) |
| Context only | 0.598 (0.087) | 0.751 (0.066) | 25 | 0.500 (0.000) | 13 | 1.439 (0.543) |
| Folder-centered embedding | 0.666 (0.104) | 0.702 (0.081) | 25 | 0.569 (0.080) | 13 | 2.053 (0.989) |
| Random (seed 0) | 0.511 (0.062) | 0.651 (0.090) | 25 | 0.475 (0.072) | 13 | 1.039 (0.539) |
| Capture order, earlier first | 0.500 (0.075) | 0.465 (0.110) | 25 | 0.458 (0.163) | 13 | 1.044 (0.555) |
| Capture order, later first | 0.500 (0.075) | 0.535 (0.110) | 25 | 0.542 (0.163) | 13 | 0.783 (0.308) |
| Burst size, larger first | 0.540 (0.081) | 0.751 (0.066) | 25 | 0.500 (0.000) | 13 | 0.998 (0.513) |
| Absolute sharpness | 0.515 (0.058) | 0.580 (0.111) | 25 | 0.458 (0.108) | 13 | 0.979 (0.556) |
| `eye_focus` (missing last) | 0.580 (0.047) | 0.670 (0.071) | 25 | 0.564 (0.152) | 13 | 1.528 (0.497) |
| `photoTier` good | 0.546 (0.050) | 0.623 (0.058) | 25 | 0.530 (0.046) | 13 | 1.489 (0.901) |
| In-burst prior's score | 0.671 (0.099) | 0.697 (0.076) | 25 | 0.523 (0.117) | 13 | 2.256 (0.970) |

`C` chosen per fold (count of folds):

| Model | 0.01 | 0.1 | 1.0 | 10.0 |
| --- | ---: | ---: | ---: | ---: |
| Full model (embedding + technical + context) | 0 | 0 | 20 | 7 |
| Full + genre | 0 | 1 | 9 | 17 |
| Without embedding (technical + context) | 0 | 0 | 9 | 18 |
| Embedding only | 0 | 10 | 9 | 8 |
| Context only | 0 | 0 | 23 | 4 |
| Folder-centered embedding | 0 | 2 | 16 | 9 |

### Criterion

Best non-embedding comparison by mean frame AUC: Without embedding (technical + context) (0.624).

| Check | Statistic | p (one-sided) | Bar | Result |
| --- | ---: | ---: | ---: | ---: |
| Genre variant: Full + genre minus full, frame AUC (n = 25) | W+ = 189, mean gain 0.001 | 0.2454 | p < 0.05 | not adopted |
| Chosen model | Full model (embedding + technical + context) | - | - | - |
| 1. Frame AUC, chosen minus Without embedding (technical + context) (n = 25) | W+ = 287, mean gain 0.042 | 0.0002 | p < 0.05 | met |
| 2. Burst AUC, chosen minus Burst size, larger first (n = 25) | W+ = 39, mean gain -0.041 | 0.9998 | p < 0.05 | not met |
| 3. Mean lift at the top tenth | 2.082 | - | >= 2.0 | met |
| Gate | fail | - | - | - |

SD-gap reading (the earlier gates' rule; for reading only, does not decide):

| Check | Mean gain | Comparison's per-folder SD | Gain > SD |
| --- | ---: | ---: | ---: |
| 1. Frame AUC vs Without embedding (technical + context) | 0.042 | 0.071 | no |
| 2. Burst AUC vs Burst size, larger first | -0.041 | 0.066 | no |

### Per folder (held out)

| Folder | Frames | Picks | Pick rate | Qualifying | Full frame AUC | Full burst AUC | No-embedding frame AUC | No-embedding burst AUC | Burst size frame AUC | Burst size burst AUC |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | 1337 | 199 | 14.9% | yes | 0.776 | 0.818 | 0.671 | 0.758 | 0.495 | 0.728 |
| `2026-06-13` | 933 | 118 | 12.6% | yes | 0.714 | 0.712 | 0.668 | 0.732 | 0.529 | 0.738 |
| `2026-06-14` | 1520 | 161 | 10.6% | yes | 0.721 | 0.745 | 0.654 | 0.782 | 0.616 | 0.808 |
| `2026-06-20` | 1050 | 133 | 12.7% | yes | 0.689 | 0.688 | 0.606 | 0.695 | 0.482 | 0.694 |
| `2026-07-02` | 106 | 15 | 14.2% | yes | 0.414 | 0.667 | 0.473 | 0.729 | 0.470 | 0.855 |
| `2026-07-05` | 1415 | 255 | 18.0% | yes | 0.752 | 0.795 | 0.630 | 0.796 | 0.620 | 0.840 |
| `2026-07-11` | 468 | 57 | 12.2% | yes | 0.584 | 0.602 | 0.553 | 0.622 | 0.555 | 0.691 |
| `2026-07-18` | 1545 | 292 | 18.9% | yes | 0.732 | 0.753 | 0.698 | 0.752 | 0.630 | 0.775 |
| `2026-07-21` | 32 | 7 | 21.9% | no | 0.543 | 0.543 | 0.589 | 0.671 | 0.594 | 0.693 |
| `2026-07-22` | 189 | 32 | 16.9% | yes | 0.531 | 0.645 | 0.555 | 0.690 | 0.424 | 0.655 |
| `2026-07-23` | 18 | 4 | 22.2% | no | 0.696 | 0.673 | 0.661 | 0.635 | 0.429 | 0.462 |
| `2026-07-24` | 218 | 34 | 15.6% | yes | 0.544 | 0.543 | 0.585 | 0.607 | 0.510 | 0.634 |
| `2026-07-26` | 64 | 7 | 10.9% | yes | 0.509 | 0.700 | 0.534 | 0.833 | 0.538 | 0.867 |
| `2026-07-30` | 936 | 105 | 11.2% | yes | 0.596 | 0.647 | 0.571 | 0.657 | 0.514 | 0.695 |
| `2026-07-31` | 349 | 52 | 14.9% | yes | 0.627 | 0.637 | 0.538 | 0.641 | 0.415 | 0.697 |
| `2026-08-01` | 3401 | 463 | 13.6% | yes | 0.710 | 0.712 | 0.672 | 0.752 | 0.619 | 0.721 |
| `2026-08-02` | 150 | 28 | 18.7% | yes | 0.625 | 0.683 | 0.573 | 0.641 | 0.441 | 0.709 |
| `2026-08-08` | 415 | 73 | 17.6% | yes | 0.567 | 0.648 | 0.581 | 0.688 | 0.420 | 0.702 |
| `2026-08-15` | 324 | 37 | 11.4% | yes | 0.563 | 0.662 | 0.544 | 0.628 | 0.434 | 0.661 |
| `2026-08-16` | 59 | 6 | 10.2% | yes | 0.717 | 0.771 | 0.698 | 0.829 | 0.480 | 0.829 |
| `2026-08-22` | 2677 | 267 | 10.0% | yes | 0.784 | 0.777 | 0.708 | 0.789 | 0.622 | 0.825 |
| `2026-08-29` | 3451 | 475 | 13.8% | yes | 0.769 | 0.801 | 0.732 | 0.824 | 0.670 | 0.806 |
| `2026-09-13-b` | 1795 | 232 | 12.9% | yes | 0.729 | 0.755 | 0.672 | 0.727 | 0.642 | 0.769 |
| `2026-09-19` | 2134 | 282 | 13.2% | yes | 0.743 | 0.814 | 0.606 | 0.752 | 0.552 | 0.764 |
| `2026-09-27-a` | 4830 | 729 | 15.1% | yes | 0.739 | 0.714 | 0.683 | 0.750 | 0.622 | 0.763 |
| `2026-09-27-b` | 1240 | 171 | 13.8% | yes | 0.778 | 0.752 | 0.724 | 0.766 | 0.628 | 0.797 |
| `2026-10-03` | 1803 | 185 | 10.3% | yes | 0.743 | 0.712 | 0.680 | 0.715 | 0.563 | 0.759 |

### Pick vs explicit reject

| Score | Folders | Mean AUC |
| --- | ---: | ---: |
| Full model (embedding + technical + context) | 3 | 0.912 |
| Full + genre | 3 | 0.917 |
| Without embedding (technical + context) | 3 | 0.813 |
| Embedding only | 3 | 0.888 |
| Context only | 3 | 0.667 |
| Folder-centered embedding | 3 | 0.921 |
| Random (seed 0) | 3 | 0.375 |
| Capture order, earlier first | 3 | 0.354 |
| Capture order, later first | 3 | 0.646 |
| Burst size, larger first | 3 | 0.711 |
| Absolute sharpness | 3 | 0.531 |
| `eye_focus` (missing last) | 3 | 0.632 |
| `photoTier` good | 3 | 0.587 |
| In-burst prior's score | 3 | 0.912 |

### Learning curve (full model)

| Training folders | Frame AUC | Burst AUC |
| --- | ---: | ---: |
| 5 | 0.642 | 0.697 |
| 10 | 0.654 | 0.703 |
| 20 | 0.666 | 0.710 |
| 26 | 0.666 | 0.710 |

### Weights of the full model fit on every folder

`C` = 1.0 (27 folders); standardized columns; intercept -0.199.

| Column | Weight |
| --- | ---: |
| ln rel sharpness | +0.107 |
| `eye_focus` logit | +0.144 |
| eyes open | +0.117 |
| \|yaw\| | +0.061 |
| \|pitch\| | +0.025 |
| \|roll\| | +0.017 |
| `photoTier` good | +0.088 |
| no `eye_focus` | -0.060 |
| no eyes judgment | -0.007 |
| no pose | -0.010 |
| ln sharpness | +0.029 |
| ln burst size | +0.154 |
| position | -0.225 |
| first frame | +0.023 |
| PCA dims (L2 norm of the 32) | 0.490 |

| Reason group | Columns | L2 norm |
| --- | ---: | ---: |
| Look | 32 | 0.490 |
| Burst context | 3 | 0.273 |
| Sharpness | 2 | 0.111 |
| AF eye focus | 2 | 0.155 |
| Eyes open | 2 | 0.117 |
| Head pose | 4 | 0.069 |
| Good photo | 1 | 0.088 |

### Reading

The gate **fails** on check 2. The full model ranks single frames across a
folder better than every signal without the embedding: frame AUC 0.666
against 0.624 for the best non-embedding comparison (the without-embedding
variant), higher in 20 of the 25 qualifying folders (one-sided Wilcoxon
p = 0.0002), and its mean lift at the top tenth, 2.08, just clears the 2.0
bar. But the cross-burst question, which bursts hold a pick, is answered
better by burst length alone: burst AUC 0.710 against 0.751 for "larger
first", above it in only 3 of the 25 folders (p = 0.9998). Long bursts are
where the picks are, and the model's best frame per burst does not order the
bursts better than their length does; the embedding adds frame-level signal
but not the "was this the right moment" signal across bursts that would let
one score compare a burst's best frame with another burst's or with a single.
The rest agrees: singles AUC is 0.611 over the 13 folders with enough singles
(the without-embedding variant 0.602, `eye_focus` 0.564); the in-burst prior
applied folder-wide reads 0.671 frame AUC rather than near chance, because
most of a folder's pick vs not-chosen comparisons are within bursts where it
was trained, and it too stays under burst size on burst AUC (0.697); the
learning curve is flat from 20 to 26 training folders (0.666 / 0.710), so more
folders of the same kind would not close the gap; the genre columns change
nothing (p = 0.245) and the folder-centered embedding is no better (0.666 /
0.702). Even the random score reads burst AUC 0.651 under the max-per-burst
rule, a reminder that this metric rewards size by construction, which is what
check 2 was there to control. By the pre-registered rule, Steps 3–5 are
dropped.
