# Step 2: refit on the mesh eye regions

The fit behind the Decision in [plan.md](plan.md). The script is
[`fit.py`](fit.py) (pure Python 3, not part of the workspace); `frozen.json`
holds the chosen variant at full precision.

## Data

- The training set is the 406 labeled faced frames of the 5 training folders
  (339 in focus, 67 off). The held-out set is the 400 of the 4 held-out folders
  (342 / 58). Both are listed in plan.md's "What is known". Every frame has a
  window edge width, so the AUC pools all of them.
- The Step 1 `candidates` lines only carry the 0.25-margin regions and no AF
  point, which variants (c) and (d) need. So `fit.py` reads one JSON line per
  faced file written by a scratch `riffle-cli meshdump` subcommand
  ([`meshdump.patch`](meshdump.patch), applied to the Step 1 commit
  `d509e555`, built in release, then reverted). For each file the line holds
  the label, the window's lap and edge width, the AF point, the pose, and per
  eye the contour bounding box center plus the contour and iris regions at
  margins 0, 0.25, 0.5 and 1.0 (`eye_region`'s code with the margin as a
  parameter, measured with `eye_measures`), all at full precision. The files
  are `training-dump.jsonl` and `heldout-dump.jsonl` in
  `D:\Photos\tests\2026-10-08-mesh-eye-focus\`, next to Step 1's
  `training.txt` / `heldout.txt`. `fit.py` checks the 0.25-margin window size
  and lap of every eye against the Step 1 lines (0 mismatches on both sets),
  so the dump measures exactly what Step 1 measured. The script's full output
  is saved as [`fit-out.txt`](fit-out.txt).
- The held-out frames are the same 400 the current model was compared on, so
  the comparison is fair but not fresh. The reserved folders are still
  unlabeled.

## Method

- Features per eye: `ln(lap + 1)` and `ln(edge_width / longer side)` over the
  eye's contour region, the current model's form with the region's longer
  side in place of the window side. Variant (e) adds `ln(iris lap + 1)`. The
  fit is plain maximum-likelihood logistic regression (Newton's method, no
  regularization), fitted on the training rows only.
- An eye counts when its contour region exists, has an edge width, and its
  longer side is at least the floor. A frame with no eye that counts falls
  back to the current window with the frozen coefficients, so every variant is
  compared over the same 406 / 400 frames.
- Threshold: the fallback frames keep their current state
  (`logit >= 1.2194`). The mesh threshold is then set so the training
  coverage stays at 310 of 339 picks (91.4%), both paths counted. It sits
  midway between the boundary pick's mesh logit and the next lower mesh logit,
  so no training frame lies on it. The held-out set uses that same threshold.
- AUC: pooled over all frames, with each fallback frame's window logit
  shifted by `mesh threshold - 1.2194`. That puts both paths on one threshold,
  which is plan.md's Trade-off option A (the fallback intercept moves, its
  state boundary does not). Ties count 0.5. The "meshed AUC mesh / window"
  columns give the AUC over only the frames that did not fall back, of the
  mesh logit and of the current window logit on the same frames. They show
  whether the region itself ranks better, apart from the pooling.
- Rules: (b) the eye with the higher logit, fitted on both eyes, each eye
  carrying its frame's label. (c) the eye whose contour center is nearer the
  AF point, fitted on that eye only; if that eye does not count, the frame
  falls back. (f) above the |yaw| cut, the eye the yaw sign names (yaw > 0:
  the image-right eye); at or below the cut, or with no pose, as (b). The model
  is (b)'s fit. (g) the same scoring as (f), but refitted with the far eye
  dropped from the training rows above the cut. In (f) and (g), if the named
  eye does not count, the other eye is used. The yaw cuts tried are 15, 30, 45
  and 60 deg.
- A control row ("window refit") scores the frames that (b) would mesh with
  the window's own two features refitted on those frames, through the same
  threshold and pooling. It measures how much of a gain comes from giving
  part of the frames their own model, regardless of the region.

## Results

### (a) Baseline

```text
train: AUC 0.852  precision 93.1%  coverage 91.4%  (333 candidates, 310 hits, 339 picks)
held: AUC 0.754  precision 89.1%  coverage 95.3%  (366 candidates, 326 hits, 342 picks)
```

This reproduces the frozen numbers: training 0.852, 93.1% / 91.4%; held-out
0.754, 89.1% / 95.3%.

### Region sizes

```text
margin 0.0 train: p10 / median / p90 [5, 11, 18]
margin 0.0 held: p10 / median / p90 [4, 11, 24]
margin 0.25 train: p10 / median / p90 [8, 16, 26]
margin 0.25 held: p10 / median / p90 [8, 17, 35]
margin 0.5 train: p10 / median / p90 [11, 21, 35]
margin 0.5 held: p10 / median / p90 [11, 22, 47]
margin 1.0 train: p10 / median / p90 [15, 32, 52]
margin 1.0 held: p10 / median / p90 [15, 32, 69]
iris (margin 0.25) train: 778 regions, p10 / median / p90 [4, 8, 12], >= 12 px 80
iris (margin 0.25) held: 769 regions, p10 / median / p90 [4, 8, 16], >= 12 px 152
```

The iris regions (median 8 px on their longer side, under 12 px on 80-90% of
eyes) are too small for (e) "on most faces", the plan's condition for using
the iris. (e) is reported but is not eligible.

### Floor: AUC by bucket

(b) is fitted with no floor at each margin. The frames are bucketed by the
larger contour region side of the two eyes, and the mesh logit and the window
logit are compared on the same frames in each bucket ("n (off)": frames and,
of them, off frames).

Margin 0.0:

| larger side | train n (off) | mesh | window | held n (off) | mesh | window |
| --- | --- | --- | --- | --- | --- | --- |
| 0-7 | 16 (5) | 0.964 | 0.836 | 11 (3) | 0.500 | 0.542 |
| 8-11 | 53 (8) | 0.750 | 0.769 | 54 (8) | 0.780 | 0.772 |
| 12-15 | 82 (9) | 0.976 | 0.992 | 62 (7) | 0.862 | 0.764 |
| 16-23 | 50 (7) | 0.963 | 0.910 | 56 (3) | 0.912 | 0.906 |
| 24+ | 7 (1) | 1.000 | 1.000 | 40 (0) | - | - |

Margin 0.25:

| larger side | train n (off) | mesh | window | held n (off) | mesh | window |
| --- | --- | --- | --- | --- | --- | --- |
| 0-7 | 12 (3) | 0.370 | 0.593 | 6 (2) | 0.750 | 0.500 |
| 8-11 | 53 (15) | 0.758 | 0.728 | 59 (14) | 0.687 | 0.649 |
| 12-15 | 78 (10) | 0.791 | 0.812 | 72 (14) | 0.711 | 0.722 |
| 16-23 | 155 (15) | 0.948 | 0.933 | 131 (14) | 0.831 | 0.741 |
| 24+ | 63 (9) | 0.903 | 0.926 | 95 (2) | 0.984 | 0.989 |

Margin 0.5:

| larger side | train n (off) | mesh | window | held n (off) | mesh | window |
| --- | --- | --- | --- | --- | --- | --- |
| 0-7 | 6 (1) | 0.800 | 0.800 | 4 (1) | 1.000 | 0.000 |
| 8-11 | 33 (9) | 0.657 | 0.565 | 32 (10) | 0.632 | 0.577 |
| 12-15 | 62 (15) | 0.746 | 0.756 | 53 (13) | 0.690 | 0.681 |
| 16-23 | 135 (15) | 0.827 | 0.820 | 124 (20) | 0.755 | 0.701 |
| 24+ | 157 (19) | 0.944 | 0.949 | 173 (9) | 0.825 | 0.848 |

Margin 1.0:

| larger side | train n (off) | mesh | window | held n (off) | mesh | window |
| --- | --- | --- | --- | --- | --- | --- |
| 0-7 | 1 (0) | - | - | 0 (0) | - | - |
| 8-11 | 16 (5) | 0.545 | 0.855 | 15 (4) | 0.727 | 0.477 |
| 12-15 | 21 (6) | 0.633 | 0.722 | 27 (10) | 0.700 | 0.588 |
| 16-23 | 76 (19) | 0.759 | 0.700 | 65 (16) | 0.611 | 0.681 |
| 24+ | 288 (34) | 0.882 | 0.903 | 292 (27) | 0.779 | 0.773 |

The buckets are small (1-19 off frames each) and do not order cleanly. Under
8 px the mesh is erratic at every margin. From 8 px up neither region wins
every bucket. So the floor is taken from the pooled training AUC over the
floors 0 (none: the 3 px region minimum), 6, 8, 10, 12, 16 and 24 px in the
next table.

### (b) Sharper eye and (d) the margin, by floor

| variant | train AUC | train prec | train cov | train fallback | train meshed AUC mesh / window | held AUC | held prec | held cov | held fallback | held meshed AUC mesh / window |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| b m0.0 floor 0 | 0.830 | 90.9% | 91.4% | 198/406 | 0.917 / 0.908 | 0.763 | 88.5% | 94.7% | 177/400 | 0.841 / 0.801 |
| b m0.0 floor 6 | 0.833 | 91.4% | 91.4% | 204/406 | 0.917 / 0.905 | 0.760 | 88.3% | 95.0% | 180/400 | 0.835 / 0.805 |
| b m0.0 floor 8 | 0.837 | 91.7% | 91.4% | 226/406 | 0.931 / 0.923 | 0.775 | 88.3% | 94.7% | 192/400 | 0.861 / 0.820 |
| b m0.0 floor 10 | 0.854 | 92.3% | 91.4% | 254/406 | 0.968 / 0.978 | 0.777 | 88.4% | 95.6% | 226/400 | 0.904 / 0.851 |
| b m0.0 floor 12 | 0.857 | 92.5% | 91.4% | 284/406 | 0.976 / 0.981 | 0.768 | 88.6% | 95.0% | 250/400 | 0.919 / 0.840 |
| b m0.0 floor 16 | 0.865 | 93.4% | 91.4% | 352/406 | 0.993 / 0.958 | 0.768 | 88.8% | 95.0% | 315/400 | 0.964 / 0.976 |
| b m0.0 floor 24 | 0.853 | 93.1% | 91.4% | 399/406 | 1.000 / 1.000 | 0.767 | 89.1% | 95.3% | 361/400 | - |
| b m0.25 floor 0 | 0.862 | 93.7% | 91.4% | 45/406 | 0.866 / 0.856 | 0.791 | 88.5% | 92.7% | 37/400 | 0.791 / 0.766 |
| b m0.25 floor 6 | 0.864 | 93.9% | 91.4% | 50/406 | 0.870 / 0.857 | 0.791 | 88.8% | 92.7% | 39/400 | 0.793 / 0.765 |
| b m0.25 floor 8 | 0.868 | 93.7% | 91.4% | 62/406 | 0.876 / 0.862 | 0.779 | 88.5% | 92.4% | 52/400 | 0.775 / 0.770 |
| b m0.25 floor 10 | 0.864 | 93.7% | 91.4% | 85/406 | 0.868 / 0.857 | 0.776 | 87.8% | 92.7% | 76/400 | 0.785 / 0.787 |
| b m0.25 floor 12 | 0.869 | 93.4% | 91.4% | 113/406 | 0.889 / 0.898 | 0.782 | 88.7% | 93.9% | 106/400 | 0.777 / 0.783 |
| b m0.25 floor 16 | 0.876 | 93.4% | 91.4% | 188/406 | 0.928 / 0.927 | 0.797 | 88.8% | 95.0% | 174/400 | 0.834 / 0.816 |
| b m0.25 floor 24 | 0.877 | 93.9% | 91.4% | 344/406 | 0.996 / 0.939 | 0.775 | 88.8% | 95.3% | 305/400 | 0.984 / 0.989 |
| b m0.5 floor 0 | 0.870 | 93.9% | 91.4% | 13/406 | 0.858 / 0.837 | 0.774 | 88.0% | 94.2% | 14/400 | 0.770 / 0.748 |
| b m0.5 floor 6 | 0.870 | 93.9% | 91.4% | 13/406 | 0.858 / 0.837 | 0.775 | 88.0% | 94.4% | 15/400 | 0.772 / 0.748 |
| b m0.5 floor 8 | 0.872 | 94.2% | 91.4% | 21/406 | 0.863 / 0.839 | 0.765 | 88.4% | 93.3% | 19/400 | 0.769 / 0.763 |
| b m0.5 floor 10 | 0.871 | 93.7% | 91.4% | 40/406 | 0.865 / 0.843 | 0.758 | 88.8% | 93.0% | 34/400 | 0.752 / 0.755 |
| b m0.5 floor 12 | 0.867 | 93.4% | 91.4% | 54/406 | 0.877 / 0.869 | 0.774 | 88.2% | 94.2% | 53/400 | 0.759 / 0.754 |
| b m0.5 floor 16 | 0.872 | 93.7% | 91.4% | 114/406 | 0.896 / 0.901 | 0.792 | 89.1% | 95.3% | 103/400 | 0.779 / 0.783 |
| b m0.5 floor 24 | 0.882 | 93.9% | 91.4% | 249/406 | 0.951 / 0.949 | 0.800 | 88.6% | 95.9% | 227/400 | 0.876 / 0.848 |
| b m1.0 floor 0 | 0.842 | 93.4% | 91.4% | 4/406 | 0.838 / 0.847 | 0.762 | 89.0% | 94.2% | 1/400 | 0.758 / 0.750 |
| b m1.0 floor 6 | 0.842 | 93.4% | 91.4% | 4/406 | 0.837 / 0.847 | 0.762 | 89.0% | 94.2% | 1/400 | 0.758 / 0.750 |
| b m1.0 floor 8 | 0.843 | 93.4% | 91.4% | 7/406 | 0.837 / 0.847 | 0.751 | 88.6% | 93.6% | 2/400 | 0.751 / 0.756 |
| b m1.0 floor 10 | 0.848 | 94.2% | 91.4% | 12/406 | 0.843 / 0.846 | 0.743 | 88.9% | 93.3% | 10/400 | 0.742 / 0.756 |
| b m1.0 floor 12 | 0.858 | 94.2% | 91.4% | 22/406 | 0.852 / 0.846 | 0.736 | 89.2% | 92.1% | 18/400 | 0.745 / 0.770 |
| b m1.0 floor 16 | 0.858 | 94.8% | 91.4% | 42/406 | 0.853 / 0.848 | 0.746 | 88.8% | 92.7% | 43/400 | 0.746 / 0.761 |
| b m1.0 floor 24 | 0.862 | 93.7% | 91.4% | 118/406 | 0.883 / 0.903 | 0.779 | 89.5% | 95.0% | 108/400 | 0.764 / 0.773 |

Selection on the training data only: the highest pooled training AUC of (b)
over the 28 margin x floor cells (ties: the lower floor, then the smaller
margin). That is **margin 0.5, floor 24 px** (0.882). Margin 0 sends about
half the frames to the fallback: most of its eyes have no edge width (a box
without margin is too thin for `edge_width`, 413 of 812 training eyes) or
are under 3 px (99). Margin 1.0, the variant added for the small regions, lets
enough brow and cheek in to fall behind 0.25 and 0.5 at every floor on the
training set.

At margin 0.5 a 24 px floor sends 61.3% of the training frames and 56.8% of
the held-out frames to the window fallback (a frame counts when either eye's
region reaches 24 px). The floor sits at the window's own minimum
(`CANDIDATE_WINDOW_MIN`). Every frame that counts has a face box of at least
58.2 px (training) / 65.3 px (held-out), and 105 / 103 fallback frames have
faces under 60 px (`EYES_MIN_FACE`); one counting training frame (58.2 px)
is under 60 px, none held-out. In all, 106 of 406 training (26.1%) and 103
of 400 held-out (25.8%) labeled faces are under 60 px. `fit.py` prints these
under "Face sizes" in `fit-out.txt`.

### Eye rules

The chosen margin and floor, followed by two reference settings: the plan's
starting margin with no floor, and the chosen margin with no floor.

Margin 0.5, floor 24 (chosen):

| variant | train AUC | train prec | train cov | train fallback | train meshed AUC mesh / window | held AUC | held prec | held cov | held fallback | held meshed AUC mesh / window |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| b sharper | 0.882 | 93.9% | 91.4% | 249/406 | 0.951 / 0.949 | 0.800 | 88.6% | 95.9% | 227/400 | 0.876 / 0.848 |
| c AF-nearest | 0.888 | 93.9% | 91.4% | 275/406 | 0.996 / 0.969 | 0.786 | 88.8% | 95.0% | 259/400 | 0.948 / 0.830 |
| e iris | 0.884 | 93.9% | 91.4% | 249/406 | 0.976 / 0.949 | 0.795 | 88.6% | 95.9% | 227/400 | 0.871 / 0.848 |
| f yaw 15 | 0.883 | 93.9% | 91.4% | 249/406 | 0.952 / 0.949 | 0.795 | 88.6% | 95.0% | 227/400 | 0.920 / 0.848 |
| g yaw 15 | 0.883 | 93.9% | 91.4% | 249/406 | 0.955 / 0.949 | 0.790 | 88.8% | 95.0% | 227/400 | 0.917 / 0.848 |
| f yaw 30 | 0.882 | 93.9% | 91.4% | 249/406 | 0.951 / 0.949 | 0.793 | 88.6% | 95.3% | 227/400 | 0.862 / 0.848 |
| g yaw 30 | 0.882 | 93.9% | 91.4% | 249/406 | 0.952 / 0.949 | 0.791 | 88.8% | 95.3% | 227/400 | 0.860 / 0.848 |
| f yaw 45 | 0.882 | 93.9% | 91.4% | 249/406 | 0.951 / 0.949 | 0.796 | 88.6% | 95.6% | 227/400 | 0.869 / 0.848 |
| g yaw 45 | 0.882 | 93.9% | 91.4% | 249/406 | 0.951 / 0.949 | 0.796 | 88.6% | 95.6% | 227/400 | 0.869 / 0.848 |
| f yaw 60 | 0.882 | 93.9% | 91.4% | 249/406 | 0.951 / 0.949 | 0.796 | 88.6% | 95.6% | 227/400 | 0.869 / 0.848 |
| g yaw 60 | 0.882 | 93.9% | 91.4% | 249/406 | 0.951 / 0.949 | 0.796 | 88.6% | 95.6% | 227/400 | 0.869 / 0.848 |
| control: window refit | 0.865 | 92.8% | 91.4% | 249/406 | 0.957 / 0.949 | 0.777 | 88.9% | 95.6% | 227/400 | 0.781 / 0.848 |

Margin 0.25, floor 0:

| variant | train AUC | train prec | train cov | train fallback | train meshed AUC mesh / window | held AUC | held prec | held cov | held fallback | held meshed AUC mesh / window |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| b sharper | 0.862 | 93.7% | 91.4% | 45/406 | 0.866 / 0.856 | 0.791 | 88.5% | 92.7% | 37/400 | 0.791 / 0.766 |
| c AF-nearest | 0.862 | 92.0% | 91.4% | 85/406 | 0.873 / 0.868 | 0.777 | 87.7% | 93.6% | 70/400 | 0.783 / 0.766 |
| e iris | 0.876 | 93.7% | 91.4% | 48/406 | 0.885 / 0.859 | 0.797 | 88.2% | 93.6% | 39/400 | 0.798 / 0.762 |
| f yaw 15 | 0.846 | 92.0% | 91.4% | 45/406 | 0.846 / 0.856 | 0.784 | 88.4% | 93.9% | 37/400 | 0.788 / 0.766 |
| g yaw 15 | 0.846 | 92.3% | 91.4% | 45/406 | 0.847 / 0.856 | 0.781 | 88.5% | 94.2% | 37/400 | 0.784 / 0.766 |
| f yaw 30 | 0.844 | 91.7% | 91.4% | 45/406 | 0.844 / 0.856 | 0.773 | 88.2% | 93.9% | 37/400 | 0.773 / 0.766 |
| g yaw 30 | 0.844 | 91.7% | 91.4% | 45/406 | 0.845 / 0.856 | 0.772 | 88.2% | 93.9% | 37/400 | 0.772 / 0.766 |
| f yaw 45 | 0.855 | 92.5% | 91.4% | 45/406 | 0.856 / 0.856 | 0.778 | 88.5% | 94.2% | 37/400 | 0.778 / 0.766 |
| g yaw 45 | 0.856 | 92.3% | 91.4% | 45/406 | 0.857 / 0.856 | 0.779 | 88.2% | 94.2% | 37/400 | 0.779 / 0.766 |
| f yaw 60 | 0.860 | 92.8% | 91.4% | 45/406 | 0.863 / 0.856 | 0.783 | 88.3% | 92.7% | 37/400 | 0.783 / 0.766 |
| g yaw 60 | 0.861 | 92.5% | 91.4% | 45/406 | 0.863 / 0.856 | 0.783 | 88.1% | 93.0% | 37/400 | 0.782 / 0.766 |
| control: window refit | 0.853 | 93.1% | 91.4% | 45/406 | 0.856 / 0.856 | 0.761 | 89.1% | 95.3% | 37/400 | 0.765 / 0.766 |

Margin 0.5, floor 0:

| variant | train AUC | train prec | train cov | train fallback | train meshed AUC mesh / window | held AUC | held prec | held cov | held fallback | held meshed AUC mesh / window |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| b sharper | 0.870 | 93.9% | 91.4% | 13/406 | 0.858 / 0.837 | 0.774 | 88.0% | 94.2% | 14/400 | 0.770 / 0.748 |
| c AF-nearest | 0.855 | 92.0% | 91.4% | 41/406 | 0.848 / 0.847 | 0.762 | 87.7% | 93.6% | 34/400 | 0.747 / 0.738 |
| e iris | 0.876 | 93.4% | 91.4% | 19/406 | 0.867 / 0.839 | 0.775 | 88.2% | 94.2% | 17/400 | 0.774 / 0.751 |
| f yaw 15 | 0.859 | 92.0% | 91.4% | 13/406 | 0.846 / 0.837 | 0.764 | 88.0% | 94.4% | 14/400 | 0.761 / 0.748 |
| g yaw 15 | 0.860 | 91.7% | 91.4% | 13/406 | 0.846 / 0.837 | 0.756 | 87.7% | 94.2% | 14/400 | 0.752 / 0.748 |
| f yaw 30 | 0.859 | 92.3% | 91.4% | 13/406 | 0.845 / 0.837 | 0.754 | 88.0% | 94.2% | 14/400 | 0.750 / 0.748 |
| g yaw 30 | 0.858 | 92.0% | 91.4% | 13/406 | 0.844 / 0.837 | 0.749 | 88.0% | 94.2% | 14/400 | 0.745 / 0.748 |
| f yaw 45 | 0.859 | 92.5% | 91.4% | 13/406 | 0.846 / 0.837 | 0.754 | 87.7% | 94.2% | 14/400 | 0.749 / 0.748 |
| g yaw 45 | 0.859 | 92.5% | 91.4% | 13/406 | 0.846 / 0.837 | 0.754 | 87.7% | 94.2% | 14/400 | 0.749 / 0.748 |
| f yaw 60 | 0.864 | 92.5% | 91.4% | 13/406 | 0.851 / 0.837 | 0.764 | 87.5% | 94.4% | 14/400 | 0.759 / 0.748 |
| g yaw 60 | 0.864 | 92.5% | 91.4% | 13/406 | 0.851 / 0.837 | 0.764 | 87.5% | 94.4% | 14/400 | 0.759 / 0.748 |
| control: window refit | 0.850 | 93.1% | 91.4% | 13/406 | 0.835 / 0.837 | 0.736 | 88.6% | 95.3% | 14/400 | 0.727 / 0.748 |

### Yaw buckets and the pose-chosen eye

"above (off)": frames with a pose whose |yaw| exceeds the cut, and the off
frames among them. The last column counts the frames where both eyes count
and the eye the yaw names is not the sharper one (by (b)'s logit), with
their labels. A phantom far eye that reads sharp on an off frame would show
up there as a reject.

Margin 0.5, floor 24:

| cut | set | above (off) | at or below (off) | no pose | pose eye != sharper (pick / reject) |
| --- | --- | --- | --- | --- | --- |
| 15 | train | 291 (44) | 115 (23) | 0 | 8 (6 / 2) |
| 15 | held | 268 (39) | 129 (18) | 3 | 15 (14 / 1) |
| 30 | train | 199 (31) | 207 (36) | 0 | 1 (1 / 0) |
| 30 | held | 173 (22) | 224 (35) | 3 | 7 (7 / 0) |
| 45 | train | 109 (11) | 297 (56) | 0 | 0 (0 / 0) |
| 45 | held | 93 (13) | 304 (44) | 3 | 3 (3 / 0) |
| 60 | train | 53 (6) | 353 (61) | 0 | 0 (0 / 0) |
| 60 | held | 45 (6) | 352 (51) | 3 | 2 (2 / 0) |

Margin 0.25, floor 0:

| cut | set | above (off) | at or below (off) | no pose | pose eye != sharper (pick / reject) |
| --- | --- | --- | --- | --- | --- |
| 15 | train | 291 (44) | 115 (23) | 0 | 104 (95 / 9) |
| 15 | held | 268 (39) | 129 (18) | 3 | 106 (95 / 11) |
| 30 | train | 199 (31) | 207 (36) | 0 | 68 (64 / 4) |
| 30 | held | 173 (22) | 224 (35) | 3 | 65 (59 / 6) |
| 45 | train | 109 (11) | 297 (56) | 0 | 35 (34 / 1) |
| 45 | held | 93 (13) | 304 (44) | 3 | 22 (20 / 2) |
| 60 | train | 53 (6) | 353 (61) | 0 | 5 (5 / 0) |
| 60 | held | 45 (6) | 352 (51) | 3 | 8 (7 / 1) |

With no floor, the pose names a different eye than the sharper one on a
third of the frames turned past 30 deg. Those frames are nearly all picks
(64 / 4 training, 59 / 6 held-out), so a sharper far eye does not mostly mark
off frames. With no floor, following the pose lowers the AUC at every cut, on both
sets and at both margins.
At the 24 px floor the far eye rarely reaches the floor, so the rules nearly
coincide (0-15 differing frames). Neither 30 nor 45 deg falls in an empty
stretch of yaw. Since no pose rule beats (b), no cut is needed.

## Outcome

Chosen: **(b) the sharper eye, margin 0.5, floor 24 px, no yaw cut**. The
pose rules (f) / (g) do not beat (b) on the held-out AUC at the chosen
setting (best 0.796 against 0.800), so the plan's recommendation falls back
to the sharper eye.

| | train | held-out |
| --- | --- | --- |
| AUC (window: 0.852 / 0.754) | 0.882 | 0.800 |
| precision (window: 93.1% / 89.1%) | 93.9% (310 / 330) | 88.6% (328 / 370) |
| coverage (window: 91.4% / 95.3%) | 91.4% (310 / 339) | 95.9% (328 / 342) |
| frames on the window fallback | 249 / 406 | 227 / 400 |

Coefficients (`frozen.json`): `c` = -8.158737592019197, `k1` (ln(lap + 1)) =
1.9417143647766388, `k2` (ln(edge / longer side)) = -1.3161251379398846,
mesh threshold 0.8343419969086643, fallback threshold 1.2194 (unchanged).

Caveats, for the Decision:

- Part of the pooled gain comes from scoring the larger-eyed frames with a
  model of their own, not from the region. The window-refit control on the
  same split reaches 0.865 / 0.777. The mesh region adds 0.017 (training)
  and 0.023 (held-out) on top of it.
- On the frames that do not fall back, the region barely ranks better than
  the window on the training set (0.951 against 0.949). On held-out it does
  (0.876 against 0.848), but over only 9 off frames.
- Held-out precision (88.6%) is below the window's 89.1% and coverage (95.9%)
  above its 95.3%, so the rule's "not both below" condition holds by the
  coverage alone.
- The selection compared 28 margin x floor cells on the training set, and the
  eye rule compared held-out AUCs as the plan asks. With 58 off frames
  held-out, differences of ~0.01-0.02 AUC are within noise.
