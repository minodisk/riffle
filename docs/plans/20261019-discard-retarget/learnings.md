# Learnings: discard-retarget

## Step 1: `burstfeat.py`

- **Run**: `python -I -B burstfeat.py D:\Photos\tests\2026-10-12-rank-app
  D:\Photos\tests\2026-10-19-discard-retarget`, 67 s; it wrote
  `burst_features.tsv` and `burstfeat.out.md` there. It read only the dump:
  no label and no model score.
- **Interpreter substitution**: `%TEMP%\embedvenv` was wiped on 2026-10-19
  (every `.py` file under it gone; the packages imported as empty namespace
  packages, `cannot import name 'roc_auc_score' from 'sklearn.metrics'
  (unknown location)`). With the user's approval a new venv was built at
  `D:\Photos\tests\.venv-eval\Scripts\python.exe` with the same versions
  (numpy 2.5.4, scikit-learn 1.9.1, scipy 1.18.1, joblib 1.6.0, threadpoolctl
  3.7.0). plan.md's "What is known" now names this path as the
  interpreter for the later steps.
- **No scipy in the script**: the first run failed on `scipy.stats.spearmanr`
  (an early symptom of the wiped venv), so the Spearman correlation is a
  local average-rank helper over numpy; the script needs only numpy plus
  the archived harnesses.
- **Counts**: 32,459 frames, 6,969 bursts, 1,943 singles. Frames with a
  value: `nn_cos`, `sharp_z`, `sharp_rank` 30,516 (94.0%, NaN exactly on the
  singles, so every frame of a multi-frame burst has a sharpness); `ef_gap`, `ef_rank` 26,523
  (81.7%); `ear_gap`, `ear_rank` 19,718 (60.7%).
- **Quantiles** (q0 / q5 / q25 / q50 / q75 / q95 / q100; mean):

  | Column | q0 | q5 | q25 | q50 | q75 | q95 | q100 | Mean |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
  | `nn_cos` | 0.7067 | 0.9339 | 0.9562 | 0.9662 | 0.9748 | 0.9867 | 0.9992 | 0.9639 |
  | `sharp_z` | -4.6231 | -1.6345 | -0.8207 | 0.1145 | 0.8163 | 1.4064 | 3.7875 | 0.0000 |
  | `sharp_rank` | 0 | 0 | 0.1818 | 0.5 | 0.8182 | 1 | 1 | 0.5000 |
  | `ef_gap` | -10.7777 | -5.9454 | -3.6698 | -0.6228 | 0 | 0.4850 | 8.9428 | -1.6717 |
  | `ef_rank` | 0 | 0 | 0 | 0.5 | 0.8 | 1 | 1 | 0.4608 |
  | `ear_gap` | -2.1826 | -0.4138 | -0.1969 | -0.0826 | -0.0162 | 0.0795 | 2.1220 | -0.1178 |
  | `ear_rank` | 0 | 0 | 0.1667 | 0.5 | 0.8333 | 1 | 1 | 0.5000 |

  Share at 0 ("no sharper / higher frame in the burst"): `sharp_rank`
  16.5%, `ef_rank` 27.2%, `ear_rank` 18.1%. `ef_gap` has its q75 at exactly 0: ties at
  the burst top are common (probably the `gate.logit` clip at 1 - 1e-3).
- **Spearman correlation** with `G0`'s burst-relative base columns (frames
  where both exist):

  | Column | `tech[:, 0]` | log size | position | first |
  | --- | ---: | ---: | ---: | ---: |
  | `nn_cos` | +0.076 | +0.085 | +0.054 | -0.092 |
  | `sharp_z` | +0.694 | -0.002 | -0.115 | +0.083 |
  | `sharp_rank` | -0.652 | +0.000 | +0.109 | -0.079 |
  | `ef_gap` | +0.359 | -0.467 | -0.061 | +0.206 |
  | `ef_rank` | -0.145 | +0.035 | +0.077 | -0.051 |
  | `ear_gap` | +0.247 | -0.461 | +0.016 | +0.163 |
  | `ear_rank` | +0.004 | +0.000 | +0.006 | -0.014 |

  The sharpness pair largely restates `tech[:, 0]` (|rho| about 0.65-0.69);
  `nn_cos`, `ef_rank` and `ear_rank` are nearly independent of the base
  columns; the two gaps fall with burst size (rho about -0.46), as a max
  over more mates does.
- **Checks, all pass**: every (`folder`, `file`) of the dump has exactly one
  row; `sharp_rank` is 0 exactly where `tech[:, 0]` is 0 among the frames
  with a `sharp_rank`; `nn_cos` is NaN exactly on the singles.

## Deferred issues (todo candidates)

- None from Step 1.
