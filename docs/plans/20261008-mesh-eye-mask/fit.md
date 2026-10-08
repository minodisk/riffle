# Step 2: refit on the eyelid contour mask

The fit behind the Decision in [plan.md](plan.md). The script is
[`fit.py`](fit.py) (pure Python 3, not part of the workspace), copied from
the archived
[`../_archived/20261008-mesh-eye-focus/fit.py`](../_archived/20261008-mesh-eye-focus/fit.py)
and extended with a loader for the mask dump. Its full output is
[`fit-out.txt`](fit-out.txt), and [`frozen.json`](frozen.json) holds the
selected cell at full precision.

```text
python fit.py D:\Photos\tests\2026-10-08-mesh-eye-mask D:\Photos\tests\2026-10-08-mesh-eye-focus --frozen frozen.json
```

## Data

- The mask dump of Step 1 (`training-mask.jsonl`, `heldout-mask.jsonl` in
  `D:\Photos\tests\2026-10-08-mesh-eye-mask\`, written by
  [`maskdump.patch`](maskdump.patch) with the face gate on): 406 labeled faced
  training frames (339 in focus, 67 off) and 400 held-out (342 / 58), the same
  frames as the archived fit. Per eye it holds the current 0.5-margin
  rectangle and, at dilations 0, 0.1, 0.25, 0.5 and 1.0, the mask's bounding
  window, pixel count, masked lap, masked edge width and the rectangle edge
  width of the mask's bounding window.
- The archived mesh dump (`training-dump.jsonl`, `heldout-dump.jsonl` in
  `D:\Photos\tests\2026-10-08-mesh-eye-focus\`), written before the face gate,
  for the pre-gate baseline.
- Check: both dumps list the same files in the same order (406 / 400 names
  aligned), and on every eye the mask dump meshes, its rectangle lap, edge
  width and longer side equal the archived dump's margin-0.5 contour region
  (0 mismatches on both sets).

## Method

The archived method, unchanged (see the archived
[`fit.md`](../_archived/20261008-mesh-eye-focus/fit.md), "Method"):

- Features per eye: `ln(lap + 1)` and `ln(edge_width / longer side)`, the
  longer side being the region's (for a mask, its bounding window's).
  Maximum-likelihood logistic regression by Newton's method, no
  regularization, on the training rows only; both eyes of a frame carry its
  label.
- The sharper eye (higher per-eye logit) scores the frame; a frame falls back
  to the window with the frozen window coefficients only when no eye counts.
  An eye counts when its region exists, has an edge width, and reaches the
  floor.
- The mesh threshold keeps the training coverage at 310 of 339 picks; the
  held-out set uses that same threshold. The pooled AUC shifts each fallback
  frame's window logit by `threshold - 1.2194`; ties count 0.5. The "meshed
  AUC mesh / window" columns compare the two logits on the frames that did not
  fall back.
- The control row refits the window's own two features on the frames the
  variant meshes, through the same threshold and pooling.

What is new in this fit:

- (b) the mask for both measures: masked lap and masked edge width.
- (c) the mask for the Laplacian only: masked lap and the rectangle edge
  width (`edge_width`) of the mask's bounding window.
- Both at each dilation x the longer-side floors 0, 8, 12, 16, 24 px, and x
  the pixel-count floors 0, 25, 50, 100, 200, 400 (the grid of pixel floors
  was chosen for this fit; the plan names only the rule). Floor 0 is the
  mask's own 3-pixel minimum.
- A masked edge width of 0 (a seed on the mask window's edge with nowhere to
  walk, on a window one or two pixels wide) has no logarithm; the fit counts
  it as no edge width. That happens on 8 training and 4 held-out eyes, all at
  dilation 0 or 0.1. The rectangle never gives 0 on these sets.
- Selection: the highest pooled training AUC over all mask cells, (b) and
  (c), of the floor rule (longer side or pixel count) whose best cell ranks
  higher; ties to the lower floor, then the smaller dilation. Training data
  only.
- Cells with no fit: when no eye reaches the floor, or the counting rows are
  separable (they sit at dilation 0 to 0.25 with large floors, where 0-2 off
  frames remain) so Newton's method overflows or does not converge within 100
  iterations (last step under 1e-12), the cell has no maximum-likelihood fit
  and is marked so.

## Results

### (a) Baseline: the rectangle, margin 0.5, floor 24

| baseline | train AUC | prec | cov | fallback | held AUC | prec | cov | fallback | held meshed AUC mesh / window |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| archived dump (pre-gate), refit | 0.882 | 93.9% | 91.4% | 249/406 | 0.800 | 88.6% | 95.9% | 227/400 | 0.876 / 0.848 |
| mask dump (post-gate), refit | 0.882 | 93.9% | 91.4% | 250/406 | 0.800 | 88.6% | 95.9% | 227/400 | 0.876 / 0.848 |
| mask dump (post-gate), shipped coefficients | 0.882 | 93.9% | 91.4% | 250/406 | 0.800 | 88.6% | 95.9% | 227/400 | 0.876 / 0.848 |
| control: window refit (post-gate) | 0.864 | 92.8% | 91.4% | 250/406 | 0.776 | 88.9% | 95.6% | 227/400 | 0.776 / 0.848 |

- The archived dump refits to the archived coefficients bit for bit
  (`c` = -8.158737592019197, `k1` = 1.9417143647766388, `k2` =
  -1.3161251379398846, threshold 0.8343419969086643), so 0.882 / 93.9% /
  91.4% and 0.800 / 88.6% / 95.9% are reproduced.
- Face gate delta: one training frame, `_DSC2748.ARW` (Reject, face 58.2 px,
  eye regions 24 and 20 px), counted before the gate (mesh logit -1.714,
  under the 0.834 threshold) and now falls back to the window (logit -0.716,
  under 1.2194). It stays off either way, so the post-gate baseline's
  precision and coverage are unchanged; the refit moves to `c` =
  -8.049349114108265, `k1` = 1.9283498240181896, `k2` = -1.2831167099140919,
  threshold 0.8524528415941095, training AUC 0.8821 -> 0.8819, held-out AUC
  0.80001 -> 0.80026 (328 / 370 and 328 / 342 both). No held-out frame
  changes.

### (b) and (c): the cells

Fallback counts are the frames with no eye that counts. Full rows with the
meshed AUC columns are in [`fit-out.txt`](fit-out.txt).

(b) mask for both measures, longer-side floor:

| d \ floor | 0 | 8 | 12 | 16 | 24 |
| --- | --- | --- | --- | --- | --- |
| 0 | 0.866 / 0.800 (133, 128) | 0.866 / 0.779 (181, 169) | 0.851 / 0.775 (294, 264) | 0.863 / 0.776 (362, 332) | no fit |
| 0.1 | 0.859 / 0.807 (112, 104) | 0.858 / 0.789 (134, 130) | 0.878 / 0.807 (190, 179) | 0.882 / 0.793 (305, 264) | no fit |
| 0.25 | 0.857 / 0.804 (108, 105) | 0.861 / 0.790 (121, 118) | 0.868 / 0.807 (149, 140) | **0.887** / 0.806 (211, 205) | 0.870 / 0.782 (354, 319) |
| 0.5 | 0.854 / 0.802 (107, 103) | 0.857 / 0.797 (113, 107) | 0.857 / 0.792 (127, 121) | 0.864 / 0.803 (149, 143) | 0.875 / 0.797 (269, 244) |
| 1.0 | 0.856 / 0.803 (106, 103) | 0.857 / 0.795 (108, 103) | 0.858 / 0.789 (113, 107) | 0.859 / 0.775 (120, 116) | 0.859 / 0.799 (150, 144) |

Each cell: training AUC / held-out AUC (training fallback, held-out
fallback).

(b) mask for both measures, pixel-count floor:

| d \ floor | 0 | 25 | 50 | 100 | 200 | 400 |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 0.866 / 0.800 | 0.879 / 0.797 | 0.866 / 0.773 | no fit | no fit | no fit |
| 0.1 | 0.859 / 0.807 | 0.865 / 0.791 | **0.889** / 0.797 | 0.868 / 0.777 | no fit | no fit |
| 0.25 | 0.857 / 0.804 | 0.864 / 0.790 | 0.867 / 0.798 | 0.886 / 0.795 | 0.875 / 0.777 | no fit |
| 0.5 | 0.854 / 0.802 | 0.854 / 0.797 | 0.861 / 0.785 | 0.860 / 0.795 | 0.879 / 0.811 | 0.879 / 0.794 |
| 1.0 | 0.856 / 0.803 | 0.855 / 0.804 | 0.854 / 0.791 | 0.856 / 0.775 | 0.856 / 0.782 | 0.872 / 0.800 |

(c) mask for the Laplacian only, longer-side floor:

| d \ floor | 0 | 8 | 12 | 16 | 24 |
| --- | --- | --- | --- | --- | --- |
| 0 | 0.859 / 0.765 | 0.861 / 0.764 | 0.856 / 0.774 | 0.853 / 0.775 | no fit |
| 0.1 | 0.869 / 0.777 | 0.861 / 0.780 | 0.862 / 0.791 | 0.879 / 0.791 | no fit |
| 0.25 | 0.864 / 0.792 | 0.866 / 0.792 | 0.866 / 0.805 | 0.886 / 0.807 | 0.871 / 0.780 |
| 0.5 | 0.861 / 0.806 | 0.861 / 0.803 | 0.863 / 0.799 | 0.872 / 0.810 | 0.876 / 0.796 |
| 1.0 | 0.854 / 0.800 | 0.853 / 0.796 | 0.854 / 0.795 | 0.856 / 0.784 | 0.859 / 0.802 |

(c) mask for the Laplacian only, pixel-count floor:

| d \ floor | 0 | 25 | 50 | 100 | 200 | 400 |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 0.859 / 0.765 | 0.884 / 0.766 | 0.869 / 0.769 | no fit | no fit | no fit |
| 0.1 | 0.869 / 0.777 | 0.866 / 0.777 | 0.880 / 0.796 | 0.860 / 0.777 | no fit | no fit |
| 0.25 | 0.864 / 0.792 | 0.866 / 0.792 | 0.865 / 0.796 | 0.886 / 0.798 | 0.877 / 0.778 | no fit |
| 0.5 | 0.861 / 0.806 | 0.862 / 0.803 | 0.864 / 0.800 | 0.865 / 0.802 | 0.882 / 0.815 | 0.880 / 0.795 |
| 1.0 | 0.854 / 0.800 | 0.854 / 0.800 | 0.852 / 0.797 | 0.855 / 0.784 | 0.852 / 0.786 | 0.874 / 0.803 |

Best cell of each measure rule and floor rule (training AUC):

| cell | train AUC | prec | cov | fallback | meshed AUC mesh / window | held AUC | prec | cov | fallback | meshed AUC mesh / window |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| (b) d 0.25, side 16 | 0.887 | 93.7% | 91.4% | 211/406 | 0.955 / 0.920 | 0.806 | 88.6% | 95.3% | 205/400 | 0.871 / 0.834 |
| (b) d 0.1, pixels 50 | 0.889 | 93.7% | 91.4% | 224/406 | 0.974 / 0.951 | 0.797 | 88.6% | 95.0% | 203/400 | 0.882 / 0.836 |
| (c) d 0.25, side 16 | 0.886 | 93.7% | 91.4% | 212/406 | 0.951 / 0.917 | 0.807 | 88.6% | 95.3% | 207/400 | 0.873 / 0.836 |
| (c) d 0.25, pixels 100 | 0.886 | 93.7% | 91.4% | 207/406 | 0.954 / 0.944 | 0.798 | 88.6% | 95.6% | 201/400 | 0.861 / 0.832 |

### Selected cell

The pixel-count floor ranks higher (best 0.8887 against 0.8868 for the
longer side), so the selected cell is **(b) the mask for both measures,
dilation 0.1, a 50-pixel floor**: `c` = -5.427550037562812, `k1` =
1.490862529732892, `k2` = -0.5154811321190877, threshold
1.4473909466077108.

| | train AUC | prec | cov | fallback | meshed AUC mesh / window | held AUC | prec | cov | fallback | meshed AUC mesh / window |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| selected | 0.889 | 93.7% (310 / 331) | 91.4% | 224/406 | 0.974 / 0.951 | 0.797 | 88.6% (325 / 367) | 95.0% (325 / 342) | 203/400 | 0.882 / 0.836 |
| control: window refit | 0.876 | 93.1% | 91.4% | 224/406 | 0.965 / 0.951 | 0.775 | 88.3% | 94.7% | 203/400 | 0.755 / 0.836 |
| baseline (post-gate) | 0.882 | 93.9% (310 / 330) | 91.4% | 250/406 | 0.948 / 0.946 | 0.800 | 88.6% (328 / 370) | 95.9% (328 / 342) | 227/400 | 0.876 / 0.848 |

### Adoption rule

Against both the post-gate baseline (held-out AUC 0.80026, 328 / 370,
328 / 342) and the archived numbers (0.80001, 328 / 370, 328 / 342), the
selected cell fails all three conditions:

- AUC 0.7971, not above 0.800 (-0.003);
- precision 325 / 367 = 88.56%, below 88.65% (328 / 370);
- coverage 325 / 342 = 95.0%, below 95.9% (3 fewer in-focus frames found).

The two baselines give the same outcome, so there is no stricter one to pick.

Had the floor stayed on the longer side (the rule
`tract-onnx-inference.md` describes), the selection would have been (b)
dilation 0.25, floor 16: held-out AUC 0.8056 passes, but precision 326 / 368
= 88.59% and coverage 326 / 342 = 95.3% both fall short. Post hoc, no cell of
the 110 in the four grids passes all three conditions on held-out (the
`fit-out.txt` section "Cells passing the rule" is empty).

## Reading

- On the training set the mask ranks the meshed frames better than the
  rectangle (meshed AUC 0.974 against the window's 0.951, where the
  rectangle reached 0.948 against 0.946), and the best mask cells beat the
  baseline's 0.882 by 0.004-0.007 pooled. That gain does not carry to the
  held-out set: the training-selected cells land at 0.797-0.807 held-out
  AUC, around the baseline's 0.800, and every one of them finds 1-3 fewer
  in-focus frames at the frozen threshold (325-327 of 342 against 328).
- Small dilations (0, 0.1) leave many masks too small for an edge width, so
  they send frames to the fallback and need a floor to rank well; the large
  floors there leave so few off frames that the fit separates. The mask
  analogue of the current rectangle (dilation 0.5, longer side 24) reaches
  0.875 / 0.797, under the rectangle's 0.882 / 0.800.
- Masking only the Laplacian, (c), peaks at the same cell as (b) (dilation
  0.25, longer side 16) within 0.001 AUC on both sets: there the edge width of
  the bounding window carries about as much as the masked one. At dilation 0
  and 0.1 with no floor it falls behind (b) on held-out by 0.03-0.04.
- The selected cell's held-out meshed frames hold 11 off frames (the
  baseline's 9), so the 0.003-0.007 AUC differences around 0.800 are well within noise;
  the coverage drop (3 frames) is the clearest of the three failures.
