# Step 1: per-genre offsets on the app dump

The offline evaluation of the hierarchical (shared + per-genre offset)
ranking model on the app dump `D:\Photos\tests\2026-10-12-rank-app`, by
[genre_offsets.py](genre_offsets.py) on top of the archived
[gate.py](../_archived/20261012-image-embeddings/gate.py).

## The bar (written before the run)

At an `OFFSET_SHRINK` in {3, 10, 30, 100}, a variant **passes** when both
hold:

- **Pooled**: its held-out (leave one folder out) weighted pairwise AUC over
  every scorable burst is not below the shared model's by more than 0.002
  (hierarchical minus shared >= -0.002).
- **Fallback**: with the second-largest genre's training bursts cut to 20
  in every fold (three seeded draws, the same cut for both models), the
  mean over the three draws of that genre's held-out weighted AUC under the
  variant differs from the shared model's mean on the same cuts by at most
  0.005 in absolute value.

**Keep on** when an all-columns variant passes; `OFFSET_SHRINK` is then
the passing value with the highest pooled weighted AUC. If no all-columns
variant passes, the technical-only variant is tried at every value and
kept on if one passes (the column scope is then technical only). **Drop**
otherwise. A significant gain on a genre (bootstrap interval above zero) is
reported but not required.

## Method

- Data, bursts, scorable bursts, columns, pairs, the inner grouped choice
  of `C` and the burst metrics are `gate.py`'s, unchanged; the tie-break
  draws are `gate.py`'s (seed 0).
- **Burst genre**: each frame's labels by `genre::genres_of`'s rule on the
  shipped `crates/core/genres/siglip2.json` (the cosine of the L2-normalized
  embedding with each label's vector at its threshold, a sub-genre turning
  its parent on); the burst goes to the top-level label on for the most of
  its frames, ties to the label earlier in the file, `Unknown` when no frame
  turns one on.
- **Hierarchical design**: `gate.py`'s shared columns (10 technical, 2
  position, 32 whitened PCA, standardized on the training folders) plus,
  per top-level genre `g`, `[burst is g] * x / sqrt(OFFSET_SHRINK)` over the
  offset's columns (all 44, or the 10 technical). One L2 penalty on every
  coefficient at `C` then is the shared weights at `1 / (2 C)` and each
  offset at `OFFSET_SHRINK` times that. `Unknown` bursts have no offset.
- **Bootstrap**: the per-burst weighted AUC difference (variant minus
  shared), mean over bursts, 1000 draws of the bursts with replacement
  (seed 0), 2.5% and 97.5% quantiles.

## Results

`python -I genre_offsets.py D:\Photos\tests\2026-10-12-rank-app crates/core/genres/siglip2.json 20`
(numpy / scikit-learn 1.9.1 / joblib in the image-embeddings plan's venv),
49 minutes on 20 workers (Windows 11, i7-13700). Its full output is also in
`D:\Photos\tests\2026-10-12-rank-app\offsets.out.md`. The shared model
reads 0.710 / 54.9% as in `ranking.md`, so the baseline has not moved
(`gate.py` and the dump are untouched; `rank.rs` still has the 44 columns).
The pass / fail column is computed on unrounded values, so the -0.002 and
+0.005 that round onto a margin are inside it.

27 folders, 32459 frames (0 without a vector), 6969 bursts, 1801 scorable.

### Bursts per genre (shipped rule)

| Genre | Bursts | Scorable bursts |
| --- | ---: | ---: |
| sports | 4462 | 1439 |
| portrait | 299 | 57 |
| landscape | 26 | 3 |
| birds / animals | 7 | 0 |
| snap | 1371 | 214 |
| Unknown | 804 | 88 |

### Pooled (held out, leave one folder out)

| Variant | Bursts | Weighted AUC | Unweighted AUC | Top-1 | Top-3 | Minus shared |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| shared | 1801 | 0.710 | 0.763 | 54.9% | 87.7% | - |
| all, OFFSET_SHRINK 3 | 1801 | 0.705 | 0.763 | 54.8% | 88.5% | -0.005 |
| all, OFFSET_SHRINK 10 | 1801 | 0.708 | 0.763 | 55.1% | 88.1% | -0.002 |
| all, OFFSET_SHRINK 30 | 1801 | 0.709 | 0.763 | 54.9% | 88.3% | -0.000 |
| all, OFFSET_SHRINK 100 | 1801 | 0.708 | 0.763 | 54.6% | 88.2% | -0.002 |
| tech, OFFSET_SHRINK 30 | 1801 | 0.707 | 0.762 | 54.7% | 87.7% | -0.002 |

`C` chosen per fold (count of folds):

| Variant | 0.01 | 0.1 | 1.0 | 10.0 |
| --- | ---: | ---: | ---: | ---: |
| shared | 6 | 7 | 5 | 8 |
| all, OFFSET_SHRINK 3 | 23 | 3 | 0 | 0 |
| all, OFFSET_SHRINK 10 | 20 | 3 | 3 | 0 |
| all, OFFSET_SHRINK 30 | 9 | 16 | 0 | 1 |
| all, OFFSET_SHRINK 100 | 10 | 15 | 0 | 1 |
| tech, OFFSET_SHRINK 30 | 4 | 10 | 7 | 5 |

### Per genre (genres with at least 20 scorable bursts)

| Variant | sports (1439) AUC | Top-1 | portrait (57) AUC | Top-1 | snap (214) AUC | Top-1 | Unknown (88) AUC | Top-1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| shared | 0.732 | 56.6% | 0.676 | 52.6% | 0.602 | 47.2% | 0.631 | 46.6% |
| all, OFFSET_SHRINK 3 | 0.730 | 56.9% | 0.647 | 49.1% | 0.584 | 45.3% | 0.631 | 46.6% |
| all, OFFSET_SHRINK 10 | 0.733 | 57.2% | 0.661 | 52.6% | 0.584 | 44.9% | 0.626 | 46.6% |
| all, OFFSET_SHRINK 30 | 0.733 | 56.8% | 0.652 | 50.9% | 0.602 | 47.2% | 0.618 | 44.3% |
| all, OFFSET_SHRINK 100 | 0.730 | 56.4% | 0.662 | 52.6% | 0.603 | 47.2% | 0.620 | 45.5% |
| tech, OFFSET_SHRINK 30 | 0.730 | 56.4% | 0.618 | 47.4% | 0.598 | 47.2% | 0.651 | 50.0% |

### Paired per-burst difference (variant minus shared), bootstrap 95%

| Variant | Pooled | sports | portrait | snap | Unknown |
| --- | ---: | ---: | ---: | ---: | ---: |
| all, OFFSET_SHRINK 3 | -0.005 [-0.011, +0.002] | -0.002 [-0.008, +0.004] | -0.029 [-0.075, +0.005] | -0.018 [-0.053, +0.013] | +0.000 [-0.030, +0.032] |
| all, OFFSET_SHRINK 10 | -0.002 [-0.007, +0.004] | +0.001 [-0.005, +0.007] | -0.015 [-0.065, +0.021] | -0.018 [-0.042, +0.005] | -0.005 [-0.027, +0.015] |
| all, OFFSET_SHRINK 30 | -0.000 [-0.006, +0.005] | +0.001 [-0.004, +0.006] | -0.024 [-0.067, +0.003] | +0.001 [-0.026, +0.028] | -0.013 [-0.036, +0.007] |
| all, OFFSET_SHRINK 100 | -0.002 [-0.006, +0.002] | -0.001 [-0.005, +0.003] | -0.014 [-0.053, +0.008] | +0.001 [-0.016, +0.021] | -0.011 [-0.028, +0.002] |
| tech, OFFSET_SHRINK 30 | -0.002 [-0.006, +0.002] | -0.001 [-0.004, +0.001] | -0.058 [-0.115, -0.016] | -0.003 [-0.026, +0.020] | +0.020 [-0.007, +0.049] |

### Fallback: `snap` training bursts cut to 20

| Variant | Draw 0 | Draw 1 | Draw 2 | Mean | Minus shared |
| --- | ---: | ---: | ---: | ---: | ---: |
| shared | 0.589 | 0.571 | 0.590 | 0.583 | - |
| all, OFFSET_SHRINK 3 | 0.597 | 0.622 | 0.576 | 0.598 | +0.015 |
| all, OFFSET_SHRINK 10 | 0.559 | 0.576 | 0.607 | 0.581 | -0.003 |
| all, OFFSET_SHRINK 30 | 0.600 | 0.592 | 0.558 | 0.583 | +0.000 |
| all, OFFSET_SHRINK 100 | 0.581 | 0.592 | 0.592 | 0.588 | +0.005 |
| tech, OFFSET_SHRINK 30 | 0.610 | 0.583 | 0.598 | 0.597 | +0.014 |

### Decision

| Variant | Pooled minus shared (>= -0.002) | Fallback minus shared (\|x\| <= 0.005) | Result |
| --- | ---: | ---: | --- |
| all, OFFSET_SHRINK 3 | -0.005 | +0.015 | fail |
| all, OFFSET_SHRINK 10 | -0.002 | -0.003 | pass |
| all, OFFSET_SHRINK 30 | -0.000 | +0.000 | pass |
| all, OFFSET_SHRINK 100 | -0.002 | +0.005 | pass |
| tech, OFFSET_SHRINK 30 | -0.002 | +0.014 | fail |

Keep on: OFFSET_SHRINK = 30, offsets on all columns.

## Decision: keep on

- **`OFFSET_SHRINK` = 30**, offsets on **all `COLUMNS`** (technical,
  position and PCA). It passes with the highest pooled weighted AUC of the
  passing values: 0.709 against the shared model's 0.710 (paired
  difference -0.000, 95% [-0.006, +0.005]), Top-1 54.9% for both, and the
  fallback check on `snap` (the second-largest genre, 214 scorable bursts)
  reads 0.583 for both. 10 and 100 also pass; 3 fails both checks.
- **The technical-only shape** (Decision 2's) at 30 reads 0.707 (-0.002)
  and fails the fallback check (+0.014), so the all-columns shape is the
  one Step 2 builds.
- **No genre shows a significant gain.** At 30, sports is +0.001
  [-0.004, +0.006] and snap +0.001 [-0.026, +0.028]; portrait (57 bursts)
  reads -0.024 [-0.067, +0.003] and Unknown (88, which has no offset and
  moves only through the shared weights) -0.013 [-0.036, +0.007]. Neither
  interval excludes zero, but both lean negative; the bar does not require
  a gain, as the user chose.
- **Genre counts under the shipped rule**: 1439 sports / 214 snap / 57
  portrait / 3 landscape / 0 birds / animals / 88 Unknown scorable bursts
  (Decision 2's provisional softmax gave 1478 sports / 287 snap).

## Caveats for Step 2

- **`C` is chosen per fold here, fixed in the app.** The inner grouped
  choice of `C` moved toward stronger penalties once the offset columns
  were in (at 30, `C` 0.01 / 0.1 on 9 / 16 folds against 6 / 7 for the
  shared model), since one `C` scales both the shared weights and the
  offsets. `rank.rs` keeps the prior's `tau = 1 / (2 C)` with `C` = 0.1 and
  pulls the shared weights toward the prior, not toward zero, so the app's
  offsets will not sit exactly where this evaluation's did.
- **The fallback check is noisy.** The per-draw `snap` AUCs spread by
  about 0.04 for every variant, shared included; the means agreeing at 30
  says the offsets do not move a 20-burst genre more than that noise, not
  that they leave it exactly where the shared model is.
