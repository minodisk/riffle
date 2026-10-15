# Folder-wide ranking evaluation

## Pre-registration

Written 2026-10-15, before `folderwide.py` or `ablate.py` exists and before
any fit of this plan has run, and merged on its own before the fit (the merge
commit of the PR that adds this section, without results, is the record, as
for the archived
[folder-wide gate](../_archived/20261014-folder-wide-pick-model/gate.md) and
the in-burst [gate](../_archived/20261012-image-embeddings/gate.md)). Nothing
in this section changes after the results are in; a deviation found while
implementing Step 2 or Step 3 is recorded in `learnings.md` with its reason
and does not touch the bars.

The question: ordering a whole folder (bursts and singles alike) by a score,
how well does the score put the folder's picks above everything else, and
does training on that folder-wide criterion, or adding the genre, Candidate 1
or Candidate 2 cues, do better than the shipped within-burst model? And what
does each column group contribute, folder-wide and within a burst?

### What was known before this section

This pre-registration is **not blind**. The archived folder-wide gate
([gate.md](../_archived/20261014-folder-wide-pick-model/gate.md),
`folder_gate.py`) ran on the same dump and reported, held out over the same 25
qualifying folders:

- per-folder frame AUC 0.666 (full model: 44 shipped columns + `ln burst size`
  + absolute `ln sharpness`), 0.624 (without embedding), 0.629 (embedding
  only), 0.598 (context only);
- the in-burst prior's **in-sample** score applied folder-wide: 0.671;
- full + genre columns: 0.667 (p = 0.245 against full);
- lift at the top tenth 2.08;
- burst AUC 0.710 against the burst-size baseline's 0.751 (the check it
  failed).

The in-burst gate gave the shipped model's within-burst numbers (weighted
pairwise AUC 0.710, Top-1 54.9%), and
[offsets.md](../_archived/20261014-genre-ranking-offsets/offsets.md) gave
0.709 with the shipped per-genre offsets. The
[ball-game-ranking-features](../_archived/20261014-ball-game-ranking-features/plan.md)
plan measured Candidates 1 and 2 only within bursts.

Not known before this section, with no prior number: R-precision of any
score; the shipped model scored folder-wide **held out**; the Candidate 1 / 2
columns and the per-genre offsets folder-wide; the folder-wide pairwise
formulation; every ablation; the bootstrap intervals.

The bars below were **set by the user on 2026-10-15 before any score of this
plan** (option C of the plan's trade-offs), not tuned to the numbers above.

### Data and labels

- The 27 folders of the app dump in `D:\Photos\tests\2026-10-12-rank-app\`,
  read through the archived
  [`gate.py`](../_archived/20261012-image-embeddings/gate.py)'s `load` and
  `build`, imported by path (not copied).
- Picks by the gate label rule (`gate.load`): a frame is a pick when its
  `.dop` flag is Pick, or it has a virtual copy in `Output/`, or it has no
  `.dop` and its XMP flag is Pick. The dump's own app `flag` is not the
  label.
- **Every non-pick is a negative.** The dump has no `views` rows (the
  folders are culled), so "viewed but unflagged" is not distinguishable
  offline. Explicit rejects stay among the negatives and are also reported
  separately (pick vs explicit reject).
- Bursts as `rank::bursts` (`gate.bursts_of`, gap 1000 ms). A single is a
  burst of one frame.
- The cue files of `D:\Photos\tests\2026-10-14-ball-labels\`:
  `extras_c1.tsv` (`ball visible`, `contested carrier`,
  `ball visible minus no ball`, `contested carrier minus single player`) and
  `extras_c2.tsv` (`ball present`, `ball indicator`, `ln persons`,
  `contested`, `ball size`, `ln persons x ball present`), joined by
  `ballgate.load_extras` (fails above 0.1% missing).

### Unit and folds

- Every frame is a sample.
- Leave one folder out over the 27 folders; every number is held out.
- `C` is chosen per outer fold from `gate.CS` (0.01, 0.1, 1, 10) by the inner
  grouped-by-folder cross-validation on the training folders: for a
  folder-wide-trained variant, maximizing the inner validation folders' mean
  primary metric (frame AUC, as `folder_gate.choose_c`); for a
  within-burst-trained variant, `gate.choose_c`'s weighted pairwise AUC. On a
  tie the smaller `C`.
- **Qualifying folders**: at least 50 frames and 5 picks (25 of 27). Means,
  per-folder SDs, intervals and tests are over them; every folder is trained
  on and listed in the per-folder table.

### Primary metric (folder-wide)

Each per held-out folder, then the unweighted mean over the qualifying
folders (one folder, one vote, as the earlier gates), with the
frame-weighted pooled value reported beside it:

1. **Frame AUC**: picks vs every other frame of the folder.
2. **R-precision**: the pick share among the folder's top `k` frames by
   score, `k` = the folder's pick count (precision@k = recall@k there), ties
   at the cut broken by a seed-0 draw; reported as a percentage, chance = the
   folder's pick rate.

### Secondary metric (within-burst)

From the **same held-out scores**: `gate.burst_metrics` weighted pairwise AUC
and pooled Top-1 over the 1,801 scorable bursts. The shipped model (`W0`)
must reproduce the archived 0.710 / 54.9%.

### Reading aids (reported, not deciding)

- Singles AUC, over the qualifying folders with at least 5 single picks and 5
  single non-picks (count reported).
- Lift at the top tenth.
- Burst AUC by the maximum frame score per burst against the burst-size
  baseline (continuity with the archived gate; confounded by burst size, so
  not a criterion).
- Pick vs explicit reject AUC.
- The burst-centered shipped score (what the app displays today: `W0`'s
  score minus its burst's mean) scored folder-wide, expected near chance.

### Variants

Names fixed here; each is one held-out score vector.

- `W0`, **shipped as shipped**: `gate.lofo(d, sc, "shared")`, the
  within-burst pairwise fit on the 44 columns (`TECH` 10, `POSITION` 2,
  32 PCA), its uncentered score `x @ w` applied to every frame of the
  held-out folder (singles get a score too; they only did not train). **The
  reference.**
- `W0+off`: `W0` with the shipped per-genre offsets
  (`genre_offsets.hier_design`, `OFFSET_SHRINK` 30, all columns, the burst
  genre by `genre_offsets.burst_genres`); the model `rank.rs` ships today.
- `F0`, **folder-wide pointwise**: `folder_gate`'s weighted per-frame
  logistic (per-folder class-balanced weights) on exactly the 44 shipped
  columns, no new column, so `W0` vs `F0` is the training criterion alone.
- `F0+ctx`: `F0` plus `ln burst size` and absolute `ln sharpness` (the
  archived full model; reproduces 0.666 frame AUC as a check).
- `P0`, **folder-wide pairwise**: pick vs non-pick pairs over the whole
  folder (not only within bursts), each folder's pairs summing to weight 1,
  capped at **20,000 seeded pairs per folder** (seed = folder index), no
  intercept, 44 columns.
- **Cue additions**, each on `F0+ctx` and on `W0`:
  - `+genre cols`: the 9 shipped 0 / 1 genre labels, `Unknown` all zero;
  - `+genre off`: per-genre offsets at `OFFSET_SHRINK` 30; for the pointwise
    model the genre is the frame's burst's genre, so a single uses its own
    frame's labels (the rule `of_burst` applies to a one-frame burst);
  - `+c1`: the 4 columns of `extras_c1.tsv`;
  - `+c2`: the 6 columns of `extras_c2.tsv`;
  - `+c1+c2`: both.

  Extras are standardized on the training folders as `ballgate.design` does.
- **Baselines**: random (seed 0); capture order, earlier first and later
  first; burst size, larger first; absolute sharpness; `eye_focus` (missing
  last); `photoTier` good.

### Ablations (Step 3)

On `F0+ctx` and on `W0`, each a full held-out rerun without the named
columns (same `C` grid and inner CV), reported as the change in the primary
and the secondary metrics against the unablated model, with its bootstrap
interval:

- drop each reason group of `rank.rs`'s `REASONS`: `Look` (the 32 PCA
  columns), `Burst position`, `Sharpness`, `AF eye focus`, `Eyes open`,
  `Head pose`, `Good photo`, each missing-value indicator leaving with the
  group of the value it stands for (`MISSING`); drop `ln burst size` alone
  (`F0+ctx` only);
- drop `|yaw|`, `|pitch|`, `|roll|` one at a time;
- **each group alone** (a model of only that group's columns, with its
  missing-value indicator): head pose, sharpness, AF eye focus, eyes open,
  good photo, burst position, the embedding, and, for reading, c1, c2 and
  the genre labels.

### Intervals and tests

- A 1,000-draw bootstrap over the qualifying folders (seed 0,
  `np.random.default_rng(0)`, 25 folder indices drawn with replacement) of
  the mean paired per-folder difference (variant minus comparison), for
  frame AUC and R-precision, reported as the 2.5% / 97.5% quantiles.
- The paired one-sided Wilcoxon signed-rank test across the qualifying
  folders (`folder_gate.signed_rank`, alternative "greater", zero
  differences dropped).
- For the within-burst secondary, `genre_offsets`'s per-burst bootstrap.

### Bars

Option C, chosen by the user on 2026-10-15 before any score. Rejected:
option A, effect size only (a consistent gain below the size is "no gain");
option B, significance only (a gain of +0.003 can pass, which may not be
worth an app change).

A comparison **passes** when all of these hold over the 25 qualifying
folders:

- mean frame AUC gain >= **+0.010**;
- mean R-precision gain >= **+2.0 points**;
- the variant at or above the comparison on frame AUC in **>= 16 of the 25**
  qualifying folders;
- the paired one-sided Wilcoxon **p < 0.05** on frame AUC **and** on
  R-precision.

Not corrected for multiplicity: the checks of one comparison are an
intersection. The Decision notes the number of comparisons read when a lone
marginal pass appears.

Applied as:

- **Folder-wide training wins** when the best of `F0`, `F0+ctx`, `P0` (by
  mean frame AUC) passes against `W0`; the share of `F0+ctx` due to
  `ln burst size` is noted from its ablation.
- **A cue adds value folder-wide** when the cue variant passes against the
  base it was added to (`F0+ctx` or `W0`); the within-burst secondary is
  reported, not required.
- **Genre keep / drop**: **keep** when `+genre cols` or `+genre off` passes
  on either base, **drop** (for the ranking) otherwise. What a drop covers in
  the app (the ranking offsets only, or the whole genre feature) is a
  separate user decision recorded in the plan's Decision 1.
- **A column contributes** when its ablation lowers the primary metric with
  a bootstrap interval entirely below zero; reported per column, not acted
  on in this plan.

### What happens after

No step of this plan changes the app. Decision 1 in the plan records the
outcome against these bars and names the follow-up plan(s), if any, that
would change the app.
