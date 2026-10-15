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

## Results

Run 2026-10-16 by `python -I folderwide.py D:\Photos\tests\2026-10-12-rank-app D:\Photos\tests\2026-10-14-ball-labels D:\Photos\tests\2026-10-15-folder-wide-eval 20`
(Windows 11, 20 jobs). Its stdout is `folderwide.out.md` and the held-out
score of every variant per frame `scores.tsv`, both in
`D:\Photos\tests\2026-10-15-folder-wide-eval\`. The tables below are that
output as printed. Deviations from the pre-registered text and the choices it
left open are in `learnings.md` (Step 2).

### Data

| Count | Value |
| --- | ---: |
| Folders / qualifying (>= 50 frames, >= 5 picks) | 27 / 25 |
| Frames (with a vector) / picks / explicit rejects | 32459 / 4419 / 58 |
| Frames without a vector (dropped) | 0 |
| Bursts (gap 1000 ms), singles included / scorable | 6969 / 1801 |
| Singles / of them picks | 1943 / 242 |
| Folders with a scorable burst (`W0`-family folds) | 26 |
| Folder pick rate min / median / max | 10.0% / 13.8% / 22.2% |
| Bursts per genre (shipped rule) | sports 4462, portrait 299, landscape 26, birds / animals 7, snap 1371, Unknown 804 |
| `c1` columns (frames without a row) | ball visible, contested carrier, ball visible minus no ball, contested carrier minus single player (0) |
| `c2` columns (frames without a row) | ball present, ball indicator, ln persons, contested, ball size, ln persons x ball present (0) |
| `P0` pairs / folders capped at 20,000 | 383072 / 17 |
| `P0` all-folder pair design (pairs x 44 x 8 bytes) | 129 MiB |

### Reference checks

| Check | Archived | This run |
| --- | ---: | ---: |
| `W0` within-burst weighted AUC / Top-1 | 0.710 / 54.9% | 0.710 / 54.9% |
| `F0+ctx` mean frame AUC (qualifying folders) | 0.666 | 0.666 |
| `W0+off` within-burst weighted AUC | 0.709 | 0.709 |

### Held-out results (leave one folder out)

Frame AUC and R-precision: the mean over the 25 qualifying folders (per-folder SD in brackets) and the frame-weighted mean beside it (pooled). R-precision chance (mean pick rate) 13.8%. Within-burst: weighted pairwise AUC and Top-1 over the scorable bursts. Singles AUC over the folders with >= 5 single picks and 5 single non-picks (`n`).

| Score | Frame AUC | Pooled | R-precision % | Pooled | Within-burst wAUC | Top-1 | Singles AUC | n | Lift at top tenth | Burst AUC |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| W0 | 0.666 (0.095) | 0.718 | 28.6 (10.8) | 34.4 | 0.710 | 54.9% | 0.508 (0.135) | 13 | 2.32 (0.82) | 0.694 (0.075) |
| W0+off | 0.669 (0.094) | 0.722 | 27.9 (11.9) | 34.1 | 0.709 | 54.9% | 0.537 (0.102) | 13 | 2.15 (0.95) | 0.702 (0.073) |
| F0 | 0.664 (0.101) | 0.721 | 27.5 (10.4) | 32.6 | 0.699 | 53.3% | 0.600 (0.091) | 13 | 2.16 (0.84) | 0.686 (0.077) |
| F0+ctx | 0.666 (0.102) | 0.727 | 27.0 (11.5) | 33.5 | 0.698 | 52.7% | 0.611 (0.104) | 13 | 2.08 (0.98) | 0.710 (0.069) |
| P0 | 0.664 (0.103) | 0.719 | 26.9 (11.6) | 32.6 | 0.695 | 52.9% | 0.572 (0.079) | 13 | 2.08 (0.98) | 0.684 (0.080) |
| F0+ctx +genre cols | 0.667 (0.099) | 0.728 | 27.3 (11.0) | 33.7 | 0.699 | 53.0% | 0.594 (0.104) | 13 | 2.16 (0.91) | 0.712 (0.071) |
| F0+ctx +genre off | 0.678 (0.099) | 0.737 | 27.7 (11.9) | 35.0 | 0.698 | 52.9% | 0.605 (0.097) | 13 | 2.10 (1.08) | 0.722 (0.089) |
| F0+ctx +c1 | 0.664 (0.099) | 0.724 | 27.0 (10.3) | 33.0 | 0.695 | 52.5% | 0.594 (0.113) | 13 | 2.12 (0.88) | 0.705 (0.072) |
| F0+ctx +c2 | 0.664 (0.101) | 0.723 | 26.6 (11.5) | 33.4 | 0.697 | 53.3% | 0.604 (0.107) | 13 | 2.03 (1.01) | 0.709 (0.069) |
| F0+ctx +c1+c2 | 0.666 (0.099) | 0.723 | 27.2 (10.4) | 33.4 | 0.697 | 53.1% | 0.604 (0.104) | 13 | 2.19 (0.80) | 0.706 (0.068) |
| W0 +genre cols | 0.671 (0.096) | 0.719 | 29.0 (10.8) | 34.3 | 0.708 | 54.7% | 0.517 (0.116) | 13 | 2.29 (0.92) | 0.698 (0.078) |
| W0 +c1 | 0.667 (0.095) | 0.718 | 28.6 (11.1) | 34.4 | 0.708 | 54.4% | 0.517 (0.110) | 13 | 2.28 (0.84) | 0.694 (0.074) |
| W0 +c2 | 0.663 (0.097) | 0.717 | 28.1 (11.5) | 34.2 | 0.709 | 55.0% | 0.511 (0.132) | 13 | 2.20 (0.95) | 0.692 (0.074) |
| W0 +c1+c2 | 0.665 (0.097) | 0.718 | 28.8 (10.8) | 34.1 | 0.705 | 54.1% | 0.523 (0.112) | 13 | 2.23 (0.88) | 0.695 (0.074) |
| W0 burst-centered (what the app displays) | 0.662 (0.084) | 0.708 | 28.1 (9.4) | 32.8 | 0.710 | 54.9% | 0.500 (0.000) | 13 | 2.20 (0.83) | 0.719 (0.075) |
| Random (seed 0) | 0.511 (0.062) | 0.503 | 14.2 (5.5) | 13.3 | 0.502 | 28.8% | 0.475 (0.072) | 13 | 1.04 (0.54) | 0.651 (0.090) |
| Capture order, earlier first | 0.500 (0.075) | 0.503 | 14.5 (6.9) | 15.4 | 0.650 | 39.4% | 0.458 (0.163) | 13 | 1.04 (0.55) | 0.465 (0.110) |
| Capture order, later first | 0.500 (0.075) | 0.497 | 11.9 (4.9) | 12.0 | 0.350 | 15.4% | 0.542 (0.163) | 13 | 0.78 (0.31) | 0.535 (0.110) |
| Burst size, larger first | 0.540 (0.081) | 0.593 | 13.9 (7.2) | 17.5 | 0.500 | 29.8% | 0.500 (0.000) | 13 | 1.00 (0.51) | 0.751 (0.066) |
| Absolute sharpness | 0.515 (0.058) | 0.535 | 13.3 (7.6) | 14.4 | 0.583 | 40.5% | 0.458 (0.108) | 13 | 0.98 (0.56) | 0.580 (0.111) |
| `eye_focus` (missing last) | 0.579 (0.048) | 0.585 | 20.0 (7.0) | 19.9 | 0.567 | 37.3% | 0.562 (0.152) | 13 | 1.42 (0.52) | 0.673 (0.074) |
| `photoTier` good | 0.546 (0.050) | 0.540 | 19.5 (9.4) | 18.8 | 0.533 | 33.9% | 0.530 (0.046) | 13 | 1.49 (0.90) | 0.623 (0.058) |

### Paired differences (variant minus comparison)

Over the 25 qualifying folders: mean paired per-folder difference with the 1,000-draw folder bootstrap 95% interval (seed 0); the one-sided Wilcoxon signed-rank p (zeros dropped); folders where the variant's frame AUC is at or above the comparison's. Within-burst: the per-burst weighted AUC difference with `genre_offsets.boot`'s per-burst bootstrap, and the Top-1 difference. Bars: frame AUC >= +0.010, R-precision >= +2.0 points, >= 16 folders at or above, p < 0.05 on both.

| Comparison | Frame AUC | p | R-precision, points | p | Folders at or above | Within-burst wAUC | Top-1, points | Bars met | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `F0` - `W0` | -0.002 [-0.020, +0.013] | 0.4895 | -1.0 [-2.9, +1.0] | 0.8949 | 12 of 25 | -0.011 [-0.020, -0.002] | -1.6 | 0 of 5 | fail |
| `F0+ctx` - `W0` | -0.000 [-0.017, +0.014] | 0.2982 | -1.6 [-3.8, +0.6] | 0.9626 | 15 of 25 | -0.012 [-0.021, -0.002] | -2.1 | 0 of 5 | fail |
| `P0` - `W0` | -0.002 [-0.022, +0.015] | 0.5940 | -1.7 [-4.3, +0.8] | 0.9242 | 11 of 25 | -0.014 [-0.023, -0.006] | -1.9 | 0 of 5 | fail |
| `F0+ctx +genre cols` - `F0+ctx` | +0.001 [-0.009, +0.010] | 0.2454 | +0.2 [-1.3, +2.1] | 0.5938 | 18 of 25 | +0.001 [-0.004, +0.005] | +0.2 | 1 of 5 | fail |
| `F0+ctx +genre off` - `F0+ctx` | +0.011 [+0.005, +0.019] | 0.0010 | +0.7 [-0.8, +2.0] | 0.1061 | 21 of 25 | +0.001 [-0.004, +0.005] | +0.1 | 3 of 5 | fail |
| `F0+ctx +c1` - `F0+ctx` | -0.002 [-0.006, +0.002] | 0.9218 | -0.0 [-1.1, +1.7] | 0.9561 | 7 of 25 | -0.003 [-0.006, +0.000] | -0.2 | 0 of 5 | fail |
| `F0+ctx +c2` - `F0+ctx` | -0.003 [-0.005, +0.000] | 0.9666 | -0.4 [-1.2, +0.2] | 0.8486 | 9 of 25 | -0.001 [-0.004, +0.002] | +0.6 | 0 of 5 | fail |
| `F0+ctx +c1+c2` - `F0+ctx` | -0.001 [-0.004, +0.003] | 0.8795 | +0.2 [-1.4, +2.1] | 0.6603 | 8 of 25 | -0.000 [-0.004, +0.004] | +0.3 | 0 of 5 | fail |
| `W0 +genre cols` - `W0` | +0.004 [+0.001, +0.008] | 0.0211 | +0.4 [-0.8, +2.2] | 0.6090 | 17 of 25 | -0.002 [-0.007, +0.003] | -0.2 | 2 of 5 | fail |
| `W0+off` - `W0` | +0.003 [-0.001, +0.007] | 0.0334 | -0.7 [-2.6, +0.8] | 0.6887 | 18 of 25 | -0.000 [-0.006, +0.005] | +0.1 | 2 of 5 | fail |
| `W0 +c1` - `W0` | +0.000 [-0.002, +0.002] | 0.3735 | +0.0 [-0.7, +0.6] | 0.3559 | 12 of 25 | -0.002 [-0.006, +0.001] | -0.4 | 0 of 5 | fail |
| `W0 +c2` - `W0` | -0.003 [-0.006, -0.001] | 0.9803 | -0.5 [-2.3, +0.8] | 0.5675 | 8 of 25 | -0.001 [-0.005, +0.005] | +0.1 | 0 of 5 | fail |
| `W0 +c1+c2` - `W0` | -0.002 [-0.005, +0.001] | 0.9178 | +0.2 [-0.9, +1.1] | 0.3720 | 8 of 25 | -0.005 [-0.010, +0.001] | -0.8 | 0 of 5 | fail |

### Per folder (held out)

Best folder-wide by mean frame AUC: `F0+ctx`; best cue variant: `F0+ctx +genre off`.

| Folder | Frames | Picks | Pick rate | Qualifying | `W0` frame AUC | R-prec % | `F0+ctx` frame AUC | R-prec % | `F0+ctx +genre off` frame AUC | R-prec % |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | 1337 | 199 | 14.9% | yes | 0.779 | 44.2 | 0.776 | 41.7 | 0.779 | 41.7 |
| `2026-06-13` | 933 | 118 | 12.6% | yes | 0.715 | 36.4 | 0.714 | 34.7 | 0.729 | 31.4 |
| `2026-06-14` | 1520 | 161 | 10.6% | yes | 0.713 | 24.2 | 0.721 | 24.8 | 0.731 | 26.7 |
| `2026-06-20` | 1050 | 133 | 12.7% | yes | 0.680 | 29.3 | 0.689 | 28.6 | 0.691 | 28.6 |
| `2026-07-02` | 106 | 15 | 14.2% | yes | 0.431 | 6.7 | 0.414 | 6.7 | 0.451 | 13.3 |
| `2026-07-05` | 1415 | 255 | 18.0% | yes | 0.747 | 42.4 | 0.752 | 42.7 | 0.760 | 44.7 |
| `2026-07-11` | 468 | 57 | 12.2% | yes | 0.593 | 21.1 | 0.584 | 15.8 | 0.602 | 17.5 |
| `2026-07-18` | 1545 | 292 | 18.9% | yes | 0.708 | 40.8 | 0.732 | 38.7 | 0.747 | 40.4 |
| `2026-07-21` | 32 | 7 | 21.9% | no | 0.526 | 14.3 | 0.543 | 28.6 | 0.611 | 14.3 |
| `2026-07-22` | 189 | 32 | 16.9% | yes | 0.552 | 15.6 | 0.531 | 21.9 | 0.559 | 28.1 |
| `2026-07-23` | 18 | 4 | 22.2% | no | - | - | 0.696 | 50.0 | 0.732 | 50.0 |
| `2026-07-24` | 218 | 34 | 15.6% | yes | 0.569 | 14.7 | 0.544 | 17.6 | 0.534 | 17.6 |
| `2026-07-26` | 64 | 7 | 10.9% | yes | 0.649 | 28.6 | 0.509 | 14.3 | 0.571 | 14.3 |
| `2026-07-30` | 936 | 105 | 11.2% | yes | 0.508 | 15.2 | 0.596 | 15.2 | 0.619 | 18.1 |
| `2026-07-31` | 349 | 52 | 14.9% | yes | 0.651 | 30.8 | 0.627 | 28.8 | 0.623 | 23.1 |
| `2026-08-01` | 3401 | 463 | 13.6% | yes | 0.670 | 30.2 | 0.710 | 29.6 | 0.729 | 31.7 |
| `2026-08-02` | 150 | 28 | 18.7% | yes | 0.605 | 17.9 | 0.625 | 28.6 | 0.594 | 17.9 |
| `2026-08-08` | 415 | 73 | 17.6% | yes | 0.586 | 21.9 | 0.567 | 20.5 | 0.550 | 20.5 |
| `2026-08-15` | 324 | 37 | 11.4% | yes | 0.536 | 16.2 | 0.563 | 13.5 | 0.567 | 10.8 |
| `2026-08-16` | 59 | 6 | 10.2% | yes | 0.695 | 16.7 | 0.717 | 0.0 | 0.748 | 0.0 |
| `2026-08-22` | 2677 | 267 | 10.0% | yes | 0.778 | 37.1 | 0.784 | 36.0 | 0.790 | 36.0 |
| `2026-08-29` | 3451 | 475 | 13.8% | yes | 0.756 | 40.0 | 0.769 | 41.1 | 0.780 | 42.7 |
| `2026-09-13-b` | 1795 | 232 | 12.9% | yes | 0.722 | 36.6 | 0.729 | 34.1 | 0.739 | 36.2 |
| `2026-09-19` | 2134 | 282 | 13.2% | yes | 0.770 | 41.8 | 0.743 | 33.3 | 0.750 | 37.6 |
| `2026-09-27-a` | 4830 | 729 | 15.1% | yes | 0.736 | 34.8 | 0.739 | 35.5 | 0.750 | 38.1 |
| `2026-09-27-b` | 1240 | 171 | 13.8% | yes | 0.768 | 39.2 | 0.778 | 38.0 | 0.789 | 40.9 |
| `2026-10-03` | 1803 | 185 | 10.3% | yes | 0.740 | 32.4 | 0.743 | 34.1 | 0.760 | 35.1 |

`C` chosen per fold (count of folds):

| Model | 0.01 | 0.1 | 1.0 | 10.0 |
| --- | ---: | ---: | ---: | ---: |
| W0 | 6 | 7 | 5 | 8 |
| W0+off | 9 | 16 | 0 | 1 |
| F0 | 0 | 0 | 24 | 3 |
| F0+ctx | 0 | 0 | 20 | 7 |
| P0 | 0 | 6 | 12 | 9 |
| F0+ctx +genre cols | 0 | 1 | 9 | 17 |
| F0+ctx +genre off | 0 | 0 | 0 | 27 |
| F0+ctx +c1 | 0 | 8 | 18 | 1 |
| F0+ctx +c2 | 0 | 1 | 25 | 1 |
| F0+ctx +c1+c2 | 0 | 7 | 20 | 0 |
| W0 +genre cols | 16 | 6 | 2 | 2 |
| W0 +c1 | 10 | 7 | 2 | 7 |
| W0 +c2 | 3 | 8 | 7 | 8 |
| W0 +c1+c2 | 6 | 8 | 3 | 9 |

### Pick vs explicit reject

| Score | Folders | Mean AUC |
| --- | ---: | ---: |
| W0 | 3 | 0.907 |
| W0+off | 3 | 0.915 |
| F0 | 3 | 0.912 |
| F0+ctx | 3 | 0.912 |
| P0 | 3 | 0.897 |
| F0+ctx +genre cols | 3 | 0.917 |
| F0+ctx +genre off | 3 | 0.933 |
| F0+ctx +c1 | 3 | 0.910 |
| F0+ctx +c2 | 3 | 0.914 |
| F0+ctx +c1+c2 | 3 | 0.912 |
| W0 +genre cols | 3 | 0.919 |
| W0 +c1 | 3 | 0.906 |
| W0 +c2 | 3 | 0.911 |
| W0 +c1+c2 | 3 | 0.905 |
| W0 burst-centered (what the app displays) | 3 | 0.829 |
| Random (seed 0) | 3 | 0.375 |
| Capture order, earlier first | 3 | 0.354 |
| Capture order, later first | 3 | 0.646 |
| Burst size, larger first | 3 | 0.711 |
| Absolute sharpness | 3 | 0.531 |
| `eye_focus` (missing last) | 3 | 0.633 |
| `photoTier` good | 3 | 0.587 |

### Wall time

| Model | Seconds |
| --- | ---: |
| W0 | 18 |
| W0+off | 152 |
| F0 | 9 |
| F0+ctx | 10 |
| P0 | 86 |
| F0+ctx +genre cols | 14 |
| F0+ctx +genre off | 46 |
| F0+ctx +c1 | 12 |
| F0+ctx +c2 | 10 |
| F0+ctx +c1+c2 | 12 |
| W0 +genre cols | 19 |
| W0 +c1 | 19 |
| W0 +c2 | 19 |
| W0 +c1+c2 | 25 |

Total 456 s on 20 jobs.

### Reading

Facts only; the Decision is Step 3's, after the ablations.

- The three reference checks reproduce the archived numbers exactly: `W0`
  0.710 / 54.9% within bursts, `F0+ctx` 0.666 frame AUC, `W0+off` 0.709.
- Held out and scored folder-wide, the shipped model `W0` reads frame AUC
  0.666 and R-precision 28.6% (chance 13.8%), the same frame AUC as the
  archived pointwise model.
- The folder-wide formulations against `W0`: `F0` -0.002, `F0+ctx` -0.000,
  `P0` -0.002 frame AUC, every interval spanning zero; R-precision -1.0 to
  -1.7 points; within-burst weighted AUC -0.011 to -0.014 with intervals
  below zero. None meets any of the five bars.
- Cues: the only comparison with a mean frame AUC gain of +0.010 or more is
  `F0+ctx +genre off` (+0.011 [+0.005, +0.019], p = 0.0010, 21 of 25
  folders); its R-precision gain is +0.7 points [-0.8, +2.0], p = 0.106, so
  it meets 3 of the 5 bars, and its inner CV chose `C` = 10, the top of the
  grid, in all 27 folds. On `W0`, `+genre cols` (+0.004, p = 0.021) and the
  offsets `W0+off` (+0.003, p = 0.033) gain on frame AUC below the +0.010
  size and not on R-precision. `+c1`, `+c2` and `+c1+c2` read -0.003 to
  +0.000 frame AUC and -0.5 to +0.2 points R-precision on both bases.
- Reading aids: singles AUC is 0.508 for `W0` against 0.600 for `F0` and
  0.611 for `F0+ctx` (13 folders). The burst-centered `W0` score (what the
  app displays) reads frame AUC 0.662, not near chance as the
  pre-registration expected. Pick vs explicit reject covers only 3 folders.
