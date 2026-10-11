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
