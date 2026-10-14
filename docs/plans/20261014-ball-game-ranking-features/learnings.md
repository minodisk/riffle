# Learnings: ball-game-ranking-features

The plan with the pre-registered criteria landed on `main` as
`6138670662d69713559bf0df76b3f517dbe5adf8` (PR #804, 2026-10-14T12:50:57+09:00),
before any prompt vector, detector output or fit existed.

## Step 1

- **Prompts written blind.** `ball_prompts.json` was written from the four
  definitions in it before any text vector or cosine was computed and
  without opening a frame. It has the archived `genre_prompts.json` shape
  (`label`, `parent`, `prompts`, plus a `definition`), so the archived
  `genre_text.py` tokenizes it unchanged; Step 2's `ball_text.py` can be
  that thin.
- **`ballgate.py` extends by import.** It puts
  `../_archived/20261012-image-embeddings` on `sys.path` with
  `sys.dont_write_bytecode` (as `rank_prior.py` does) and uses `gate.load`,
  `build`, `scorable`, `design(..., "shared")`, `choose_c`, `fit`,
  `burst_metrics`, `summarize`, `per_folder`, `spread`, and `gate.lofo` for
  the reference itself. A candidate's design is `gate.design`'s shared
  matrix with the candidate's columns appended, imputed with the training
  mean and standardized on the training folders (column-wise, so it equals
  standardizing the stacked matrix). Its own `run_fold` / `lofo` / `curve`
  mirror gate.py's (same folds, same curve seeds) only because gate.py
  hard-codes the design variant; joblib's loky workers re-import `gate`
  fine from the inserted path. They do not inherit
  `sys.dont_write_bytecode`, so a run without `-B` leaves
  `_archived/20261012-image-embeddings/__pycache__/gate.cpython-313.pyc`
  behind (untracked, not ignored; removed before the commit); run it as
  `python -I -B ballgate.py`.
- **Bar 4's column rule.** The criterion says "its `ball` column"; the
  script reads the candidate's first column whose name starts with `ball`
  (for the combination, the first file's, i.e. Candidate 1's when given as
  `extras_c1.tsv,extras_c2.tsv`). Every column's AUC against both labels is
  printed anyway, so Step 3 can quote Candidate 2's for the combination if
  the order matters. Name the columns accordingly in Steps 2 and 3
  (`ball visible ...`, `ball present ...`).
- **Sports subset.** By the burst's argmax of the mean top-level softmax
  (`d["genre"]`, the burst average of `genres.tsv`'s softmax columns), as
  gate.py's provisional-genre table does: 1,478 sports / 323 other scorable
  bursts. The sports-gated reading aid multiplies each candidate column
  (imputed with its overall mean) by that burst's sports probability before
  the standardization and runs the same LOFO.
- **The sample.** `ball_sample.py` (seed 20261014) drew 240 frames from 24
  folders into `D:\Photos\tests\2026-10-14-ball-labels\`: `frames.tsv`,
  `img/` (240 upright previews at 1600 px through the archived
  `genre_sample.previews`, its `EDGE` raised from 800 so a small ball is
  visible; the ARW preview is about that size), `labels.tsv` (the empty
  template) and `label.html`. The page's UI text is Japanese, as the
  archived `genre_sample.py`'s was, since the user labels in Japanese.
  Label values: `ball` `visible` / `none` / `unsure`, `contest` `yes` /
  `no` / `na`.
- **Runtime.** The dry run (reference, one 2-column candidate, the gated
  aid and the learning curve) took 2 min 44 s with 8 jobs, so the full
  Step 2 / 3 runs fit in one background job without trouble.
- **The `--labels` path was exercised on random labels** (a scratch copy
  of `frames.tsv`), with a combination spec, and the missing-row guard
  (460 of 32,459 missing) stops the run as intended. Those numbers are
  not results and are not recorded.

### Dry run (random noise)

A two-column extras file of uniform noise (`ball noise`, `noise 2`, seed 7)
for all 32,459 frames, run with `--curve noise`. The reference reproduces
`ranking.md` exactly, **0.710 weighted AUC / 54.9% Top-1** (unweighted
0.763, Top-3 87.7%, `C` per fold 6 / 7 / 5 / 8), and the noise candidate
fails every bar:

| Ranking | Bursts | Weighted pairwise AUC | Per-folder SD | Unweighted pairwise AUC | Top-1 | Per-folder SD | Top-3 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Reference (shared model) | 1801 | 0.710 | 0.054 | 0.763 | 54.9% | 8.2% | 87.7% |
| noise | 1801 | 0.709 | 0.061 | 0.763 | 54.0% | 8.4% | 87.5% |

| Candidate | 1. AUC minus reference (>= 0.010) | Met | 2. Top-1 minus reference, points (>= 2.0) | Met | 3. Folders at or above (>= 13) | Met | 4. `ball` column AUC vs label (>= 0.75) | Met | Verdict |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `noise` | -0.001 | no | -0.9 | no | 9 of 20 | no | no `--labels` | - | fail |

Reading aids: each noise column alone 0.497 / 0.515 weighted AUC; gated by
P(sports) 0.708 (-0.002), 53.9%; sports bursts 0.729 -> 0.731, other 0.621
-> 0.610; pick vs reject (3 bursts) 0.926 -> 0.933; learning curve 0.690 /
0.698 / 0.703 / 0.709 at 5 / 10 / 20 / 26 folders; the fit on every folder
gives the noise columns +0.007 and +0.060 at `C` = 0.1. Folders with 20 or
more scorable bursts: 20, as the criterion assumed.

## Deferred issues (todo candidates)

- **Pending manual step (the user's): label the 240-frame sample before
  Step 2's fit.** Open `D:\Photos\tests\2026-10-14-ball-labels\label.html`
  in a browser (file://), answer `ボール` and `競り合い` for each frame
  (keys 1/2/3 and Q/W/E, Enter for the next; clicking the photo shows it at
  full size), then `TSV を書き出す` and replace that folder's `labels.tsv`
  with the download. Expected: at least 200 frames with a `ball` label
  (Step 2's precondition). Basis: plan.md Step 1 / Step 2 precondition;
  files `docs/plans/20261014-ball-game-ranking-features/ball_sample.py`,
  `D:\Photos\tests\2026-10-14-ball-labels\`. Step 1's checkbox was ticked on
  the automated criteria (the sample is drawn; labeling is the user's).
