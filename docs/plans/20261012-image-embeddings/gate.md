# Step 2: offline ranking feasibility gate

## Pre-registration

Written 2026-10-12T08:15+09:00, before `gate.py` exists and before any fit
has run, and committed on its own before the fit (the commit that adds this
section, without the results below, is the record). Nothing in this section
changes after the results are in; a deviation found while implementing is
recorded in `learnings.md` with its reason and does not touch the criterion.

### Data

- The 27 sidecar-labeled ARW folders of
  [burst-keep-score data.md](../_archived/20261008-burst-keep-score/data.md),
  dumped fresh by [`gate_dump.sh`](gate_dump.sh) into
  `D:\Photos\tests\2026-10-12-embed-gate\`: `features/<folder>.tsv`
  (`riffle-cli features`, 24 threads), `features/<folder>.output.txt` (the
  stems of `Output/`) and `embed.tsv` (`riffle-cli embed` with the SigLIP 2
  B/32 @256 FP32 image tower of Decision 1, `--near 404`), joined by path.
- Labels as data.md: a frame is a pick when its `.dop` flag (first item) is
  Pick, or it has a virtual copy in `Output/` (`<stem>_<n>`), or it has no
  `.dop` and its XMP flag is Pick. Every other frame is "not chosen" (these
  folders count as culling-finished). An explicit reject is a frame whose
  deciding flag (the `.dop`'s, else the XMP's) is Reject; it stays among the
  "not chosen" frames for the gate and is also reported separately.
- Bursts: `burst.ts` `groupBursts` at gap 1000 ms (capture time, subsec
  right-padded, file name; a gap equal to 1000 ms stays in the burst), as
  burst-keep-score's `metrics.py`. A scorable burst has 2+ frames, at least
  one pick and at least one non-pick. Pairs are every pick x non-pick of one
  scorable burst; each burst's pairs carry a total weight of 1 (each pair
  `1 / (picks x non-picks)`).

### Provisional genres

- Prompts: [`genre_prompts.json`](genre_prompts.json) (7 top-level genres,
  5 sub-genres), text vectors by [`genre_text.py`](genre_text.py) with the
  SigLIP 2 text tower (`onnx/text_model.onnx` of the same pinned revision
  `7efccb86b5b6601bfe9e14326e1c11b40b68d44c`) into
  `genre_text.tsv`: each label the normalized mean of its normalized prompt
  vectors.
- Per file: the SigLIP sigmoid score of every label,
  `sigmoid(111.8326 x cos - 16.7669)` (`exp(logit_scale)` and `logit_bias`
  of `google/siglip2-base-patch32-256` at
  `94dffa8cb1179de3e03f091dbc3917e5d5a9ae84`), written to
  `genres.tsv`; and the softmax over the 7 top-level genres of
  `111.8326 x cos`, which is the genre feature of the gate.
- Per burst: the mean of its frames' top-level softmax; the burst's genre for
  the counts and the per-genre AUCs is that mean's argmax.

### Model

Pairwise logistic regression on pick-minus-non-pick differences, no
intercept, each pair entered as `(d, 1)` and `(-d, 0)` with the pair's
weight, L2-regularized (scikit-learn `LogisticRegression`, lbfgs).

- Technical features per frame: ln(sharpness / burst max sharpness);
  `eye_focus` as a logit (clipped to [0.001, 0.999]); eyes open
  (`eyes_open`); |yaw| (clipped at 90), |pitch| and |roll| (clipped at 60);
  `photoTier` good (0 / 1, the `focus.ts` rule on `state`, `eye_focus`,
  `ear`, the pose, `eye_offset` and `edge_gap`); indicators for no
  `eye_focus`, no eyes judgment and no pose, the missing value itself set
  to the training mean.
- Position: index / (burst size - 1) and a first-frame indicator.
- Embedding: PCA to 32 dimensions, whitened, fit on every frame of the
  training folders only; the pair feature is the difference of the
  projections.
- Every column standardized on the training folders' frames (pair features
  are differences, so only the scale matters).
- `C` chosen per outer fold from {0.01, 0.1, 1, 10} by an inner 5-fold
  grouped-by-folder cross-validation on the training folders, maximizing
  the weighted pairwise AUC.
- **Shared model**: technical + position + embedding.
- **Per-genre variant**: the shared model plus, for each top-level genre g,
  the columns `p_g x (technical feature difference)`, `p_g` the burst's mean
  genre probability; not for the position or embedding columns. The offset
  columns are multiplied by `1 / sqrt(10)`, which puts a 10x stronger L2
  penalty on the offsets than on the shared weights under one `C`.
- **Without embedding**: the shared model minus the PCA columns.

### Evaluation

Leave-one-folder-out over the 27 folders (shooting day = folder); every
number below is on the held-out folders, pooled over all held-out bursts
unless it says per folder.

- Weighted pairwise AUC: each scorable burst contributes its pairs'
  win share (ties 0.5) with weight 1; the mean over bursts. Unweighted
  (every pair equal, as burst-keep-score's 0.653) is reported beside it.
- Top-1: the burst's top-ranked frame is a pick, ties split (as
  `metrics.py`); the mean over bursts. Top-3: a pick among the three
  top-ranked frames, ties broken by a seeded random draw (seed 0).
- Baselines on the same bursts: first frame (rank by position), `eye_focus`
  (missing ranks last), absolute sharpness, `photoTier` good (ties
  everywhere else), random (seed 0), and the model without the embedding.
- Per-folder spread: the sample standard deviation (ddof 1) across the
  held-out folders with at least 20 scorable bursts.

### Criterion

- The **chosen variant** is the per-genre variant if its held-out weighted
  pairwise AUC exceeds the shared model's by more than the per-folder spread
  of the shared model's weighted AUC; otherwise the shared model.
- The **gate passes** when the chosen variant has both:
  1. held-out weighted pairwise AUC >= **0.703** (burst-keep-score's
     first-frame 0.653 + 0.05, an absolute bar whatever the first-frame
     baseline reads here); and
  2. a held-out Top-1 gain over the first-frame baseline (chosen variant's
     pooled Top-1 minus first frame's pooled Top-1) larger than the
     per-folder spread of the first-frame baseline's Top-1.
- Pass: Step 7 proceeds, with per-genre weights if the per-genre variant was
  chosen. Fail: Step 7 is dropped and Decision 2 says so with the numbers.

### Also reported (not part of the criterion)

Top-1 / Top-3 of every model and baseline; a learning curve (each held-out
folder with the model trained on 5, 10 and 20 of the other folders, three
seeded draws each, and on all 26); bursts per provisional genre; the per-genre
weighted AUC of both variants and of the first-frame baseline; the pick vs
explicit-reject pairs separately; the embedding-only model (PCA columns
alone) as a reading aid.

## Results

Run 2026-10-12 after the pre-registration commit `ce0ea14` (committed
2026-10-12T08:12:18+09:00; the "08:15" in its first line is the planned
time, the commit is the record). `gate_dump.sh` with the release `riffle-cli`
of this branch (copied to the gate directory) dumped the 27 folders: 32,459
files, no feature or embedding error; `embed` took 611.6 s of wall time on 24
threads (18.8 ms per image, peak working set 1148 MB). The folder
`2026-07-23` (18 frames, 0 picks) has no scorable burst, so the run held out
26 folders, not 27, and each fold fits on 25 folders that contribute pairs
(`2026-07-23` adds only frames to the imputation, standardization and PCA).
Then:

```sh
python -I genre_text.py <models>/siglip2-b32-256-text genre_prompts.json <gate>/genre_text.tsv
python -I gate.py <gate> 12
```

`<models>` is `D:\Photos\tests\2026-10-12-embed-models` (the text tower
`onnx/text_model.onnx`, 1,129,469,657 B, and `tokenizer.json` of the pinned
revision, in their own directory `siglip2-b32-256-text`), `<gate>` is
`D:\Photos\tests\2026-10-12-embed-gate`. `gate.py` wrote `genres.tsv` there
and printed the tables below (`gate.out.md`), about three minutes on 12
workers.

### Reading

- **The gate passes, narrowly on the AUC bar.** The shared model reads a
  held-out weighted pairwise AUC of 0.708 against the bar of 0.703 (margin
  0.005, under one per-folder SD of 0.060), and a Top-1 of 54.4% against the
  first frame's 39.4% (+15.0 points against the first frame's per-folder SD
  of 8.7 points). The second check is met by a wide margin; the first by a
  small one, which the criterion counts as met.
- **The embedding carries the gain.** Without it the same model reads 0.652
  (Top-1 44.5%), level with the first frame alone (0.650); the embedding
  alone (no technical feature, no position) reads 0.668 and a Top-1 of
  51.5%. The technical features add little beyond the position, as
  burst-keep-score found.
- **Per-genre weights do not pay.** The per-genre variant reads 0.707 (Top-1
  55.2%), level with the shared model (-0.000 rounded), against a required
  gain of 0.060.
  The provisional genres are almost all sports (1478 of 1801 scorable
  bursts) and snap (287); every other genre has 20 scorable bursts or fewer,
  so the data cannot show a per-genre effect either way.
- **The unweighted AUC is higher (0.763)**: the bursts with many pairs,
  which the weighting scales down, are easier for the model than the short
  ones.
- **The learning curve still rises slightly at 26 folders** (0.687 at 5,
  0.699 at 10, 0.704 at 20, 0.708 at 26), which suggests more finished
  folders would still help a little. With 5 folders the model already clears
  the first frame (0.650) by a wide margin but not the bar.
- **Explicit rejects are too few to read**: 3 scorable bursts hold one.
- **The first frame reads 0.650 weighted / 0.653 unweighted here**, the same
  as burst-keep-score's 0.653, so the fresh dump and the labels agree with
  that work.
- Per folder, the shared model's weighted AUC is below the first frame's in
  four folders (`2026-07-24`, `2026-07-31`, `2026-08-16`, `2026-10-03`),
  three of them with 32 scorable bursts or fewer, and equal in
  `2026-07-21` (3 bursts).

### Data

| Count | Value |
| --- | ---: |
| Folders | 27 |
| Frames (with a vector) / picks / explicit rejects | 32459 / 4419 / 58 |
| Frames without a vector (dropped) | 0 |
| Bursts (gap 1000 ms) | 6969 |
| Scorable bursts | 1801 |
| Pick x not-chosen pairs in them | 41942 |
| Scorable bursts with an explicit reject | 3 |

### Held-out results (leave one folder out)

| Ranking | Bursts | Weighted pairwise AUC | Per-folder SD | Unweighted pairwise AUC | Top-1 | Per-folder SD | Top-3 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| First frame | 1801 | 0.650 | 0.060 | 0.653 | 39.4% | 8.7% | 79.2% |
| `eye_focus` (missing last) | 1801 | 0.564 | 0.046 | 0.567 | 37.1% | 7.4% | 76.1% |
| Sharpness | 1801 | 0.583 | 0.054 | 0.577 | 40.5% | 7.2% | 78.7% |
| `photoTier` good | 1801 | 0.533 | 0.026 | 0.534 | 33.9% | 6.1% | 74.9% |
| Random (seed 1) | 1801 | 0.508 | 0.056 | 0.500 | 29.6% | 9.8% | 71.2% |
| Shared model (embedding + technical + position) | 1801 | 0.708 | 0.060 | 0.763 | 54.4% | 8.4% | 87.7% |
| Per-genre variant | 1801 | 0.707 | 0.062 | 0.762 | 55.2% | 8.9% | 87.7% |
| Without embedding (technical + position) | 1801 | 0.652 | 0.045 | 0.671 | 44.5% | 8.0% | 83.7% |
| Embedding only | 1801 | 0.668 | 0.081 | 0.730 | 51.5% | 10.4% | 85.1% |

`C` chosen per fold (count of folds):

| Model | 0.01 | 0.1 | 1.0 | 10.0 |
| --- | ---: | ---: | ---: | ---: |
| Shared model (embedding + technical + position) | 5 | 9 | 4 | 8 |
| Per-genre variant | 2 | 4 | 9 | 11 |
| Without embedding (technical + position) | 25 | 0 | 1 | 0 |
| Embedding only | 1 | 9 | 7 | 9 |

### Criterion

| Check | Value | Bar | Met |
| --- | ---: | ---: | ---: |
| Per-genre minus shared, weighted AUC | -0.000 | > 0.060 (shared per-folder SD) | no |
| Chosen variant | Shared model (embedding + technical + position) | - | - |
| Chosen weighted AUC | 0.708 | >= 0.703 | yes |
| Chosen Top-1 minus first frame Top-1 | 15.0% | > 8.7% (first-frame per-folder SD) | yes |
| Gate | pass | - | - |

### Per folder (held out)

| Folder | Bursts | First frame AUC | Top-1 | Shared AUC | Top-1 | Per-genre AUC | Top-1 | No-embedding AUC | Top-1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | 121 | 0.693 | 43.8% | 0.726 | 49.6% | 0.718 | 50.4% | 0.676 | 44.6% |
| `2026-06-13` | 92 | 0.623 | 31.5% | 0.726 | 56.5% | 0.729 | 56.5% | 0.647 | 34.8% |
| `2026-06-14` | 65 | 0.653 | 30.8% | 0.773 | 44.6% | 0.763 | 44.6% | 0.663 | 36.9% |
| `2026-06-20` | 98 | 0.607 | 35.7% | 0.667 | 43.9% | 0.666 | 45.9% | 0.612 | 39.8% |
| `2026-07-02` | 7 | 0.438 | 28.6% | 0.486 | 28.6% | 0.485 | 28.6% | 0.543 | 14.3% |
| `2026-07-05` | 75 | 0.593 | 30.7% | 0.682 | 56.0% | 0.685 | 56.0% | 0.581 | 33.3% |
| `2026-07-11` | 29 | 0.695 | 62.1% | 0.761 | 69.0% | 0.756 | 69.0% | 0.688 | 62.1% |
| `2026-07-18` | 109 | 0.700 | 48.6% | 0.801 | 75.2% | 0.799 | 74.3% | 0.736 | 61.5% |
| `2026-07-21` | 3 | 0.667 | 66.7% | 0.667 | 66.7% | 0.333 | 33.3% | 0.667 | 66.7% |
| `2026-07-22` | 17 | 0.500 | 41.2% | 0.561 | 41.2% | 0.526 | 41.2% | 0.510 | 41.2% |
| `2026-07-24` | 15 | 0.578 | 46.7% | 0.517 | 46.7% | 0.439 | 33.3% | 0.550 | 33.3% |
| `2026-07-26` | 6 | 0.490 | 50.0% | 0.719 | 33.3% | 0.646 | 50.0% | 0.458 | 16.7% |
| `2026-07-30` | 57 | 0.573 | 42.1% | 0.628 | 50.9% | 0.646 | 54.4% | 0.609 | 47.4% |
| `2026-07-31` | 32 | 0.649 | 50.0% | 0.642 | 50.0% | 0.646 | 53.1% | 0.639 | 50.0% |
| `2026-08-01` | 176 | 0.612 | 36.9% | 0.662 | 50.0% | 0.664 | 50.6% | 0.619 | 42.0% |
| `2026-08-02` | 20 | 0.615 | 45.0% | 0.725 | 60.0% | 0.740 | 70.0% | 0.679 | 60.0% |
| `2026-08-08` | 52 | 0.611 | 46.2% | 0.648 | 51.9% | 0.654 | 53.8% | 0.632 | 46.2% |
| `2026-08-15` | 21 | 0.534 | 28.6% | 0.557 | 42.9% | 0.528 | 42.9% | 0.609 | 47.6% |
| `2026-08-16` | 5 | 0.672 | 40.0% | 0.608 | 40.0% | 0.608 | 40.0% | 0.725 | 80.0% |
| `2026-08-22` | 116 | 0.693 | 44.0% | 0.748 | 52.6% | 0.751 | 53.4% | 0.688 | 45.7% |
| `2026-08-29` | 133 | 0.669 | 36.1% | 0.745 | 65.4% | 0.760 | 69.9% | 0.671 | 44.4% |
| `2026-09-13-b` | 79 | 0.631 | 38.0% | 0.652 | 49.4% | 0.658 | 48.1% | 0.618 | 46.8% |
| `2026-09-19` | 107 | 0.565 | 26.2% | 0.728 | 56.1% | 0.727 | 56.1% | 0.590 | 38.3% |
| `2026-09-27-a` | 199 | 0.684 | 39.7% | 0.706 | 53.8% | 0.706 | 53.8% | 0.661 | 44.7% |
| `2026-09-27-b` | 56 | 0.718 | 39.3% | 0.739 | 64.3% | 0.733 | 64.3% | 0.720 | 48.2% |
| `2026-10-03` | 111 | 0.779 | 48.6% | 0.757 | 53.2% | 0.760 | 55.0% | 0.724 | 47.7% |

### Pick vs explicit reject

| Ranking | Bursts | Weighted AUC | Unweighted AUC |
| --- | ---: | ---: | ---: |
| First frame | 3 | 0.400 | 0.037 |
| `eye_focus` (missing last) | 3 | 0.467 | 0.926 |
| Sharpness | 3 | 0.632 | 0.889 |
| `photoTier` good | 3 | 0.694 | 0.583 |
| Random (seed 1) | 3 | 0.588 | 0.537 |
| Shared model (embedding + technical + position) | 3 | 0.926 | 0.963 |
| Per-genre variant | 3 | 0.926 | 0.963 |
| Without embedding (technical + position) | 3 | 0.799 | 0.463 |
| Embedding only | 3 | 0.733 | 0.926 |

### Provisional genres

| Genre (burst argmax) | Bursts | Scorable bursts | First frame AUC | Top-1 | Shared AUC | Top-1 | Per-genre AUC | Top-1 | No-embedding AUC | Top-1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| sports | 4762 | 1478 | 0.668 | 38.8% | 0.728 | 56.0% | 0.729 | 56.8% | 0.665 | 44.7% |
| portrait | 77 | 20 | 0.448 | 25.0% | 0.629 | 45.0% | 0.554 | 50.0% | 0.506 | 30.0% |
| landscape | 16 | 3 | 0.667 | 33.3% | 0.667 | 33.3% | 0.667 | 33.3% | 0.667 | 33.3% |
| birds / animals | 5 | 0 | - | - | - | - | - | - | - | - |
| snap | 2030 | 287 | 0.575 | 42.9% | 0.621 | 47.4% | 0.623 | 48.8% | 0.594 | 43.6% |
| macro | 8 | 3 | 0.444 | 33.3% | 0.444 | 33.3% | 0.444 | 33.3% | 0.444 | 33.3% |
| other | 71 | 10 | 0.667 | 60.0% | 0.500 | 50.0% | 0.400 | 40.0% | 0.700 | 70.0% |

Frames per top-level genre (frame argmax): sports 28444, portrait 178, landscape 23, birds / animals 5, snap 3681, macro 15, other 113.

### Learning curve (shared model)

| Training folders | Weighted AUC | Top-1 |
| --- | ---: | ---: |
| 5 | 0.687 | 50.8% |
| 10 | 0.699 | 53.6% |
| 20 | 0.704 | 54.2% |
| 26 | 0.708 | 54.4% |

The "26" row trains on 26 folders, of which 25 contribute pairs. The draws
for 5, 10 and 20 sample from the other folders including `2026-07-23`, so a
draw can hold one folder fewer with pairs than its label says.

### Weights of the shared model fit on every folder

`C` = 0.1 (27 folders); standardized columns.

| Column | Weight |
| --- | ---: |
| ln rel sharpness | +0.231 |
| `eye_focus` logit | +0.171 |
| eyes open | +0.212 |
| \|yaw\| | -0.024 |
| \|pitch\| | +0.028 |
| \|roll\| | +0.026 |
| `photoTier` good | +0.123 |
| no `eye_focus` | -0.047 |
| no eyes judgment | +0.203 |
| no pose | -0.171 |
| position | -0.373 |
| first frame | -0.156 |
| PCA dims (L2 norm of the 32) | 1.225 |
