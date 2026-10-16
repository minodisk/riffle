# Learnings: reject-filter-eval

## Step 1: pre-registration

- Before writing `reject.md`, only headers were read: the `scores.tsv` header
  (`folder`, `file`, then the 14 variant columns `W0` … `W0 +c1+c2`) and the
  `features/<folder>.tsv` header (which holds the raw `ear` column the rule
  filters may need). No bottom-oriented number was computed.
- The plan's "ten cue variants" count `W0+off` among them (the genre offsets
  on `W0`); `scores.tsv` has nine further cue columns. `reject.md` lists
  `W0+off` with the cue variants and names it explicitly in the genre
  criterion against `W0`, next to the other three genre columns.
- The `rule count` score is the number of the four rules of `any of` that
  fire (`eyes closed`, `eye focus low`, `face away`, `sharpness bottom 10%`);
  `not good` is not one of them, as the plan's list implies.
- Every link in `reject.md` targets `../_archived/...` (the five archived
  plans' scripts and result docs), so no in-flight plan's archive move can
  break lint here.

## Step 2: rejectfilter.py

- **Import workaround (copy this for a later plan).** `rejectfilter.py`
  puts the five archived folders (`20261012-image-embeddings`,
  `20261014-folder-wide-pick-model`, `20261014-ball-game-ranking-features`,
  `20261014-genre-ranking-offsets`, `20261015-folder-wide-ranking-eval`) on
  `sys.path`, imports `gate`, `folder_gate`, `ballgate`, `genre_offsets`, then
  `folderwide as fw` (whose own stale `sys.path` entries then point at
  nothing and are harmless), and sets `fw.GENRES = folder_gate.GENRES` to the
  repository's `crates/core/genres/siglip2.json` before `folder_gate.extend`.
  It finds the archive as `HERE/../_archived`, or `HERE/..` when its own
  folder already sits under `_archived/`, so it keeps importing after this
  plan's archive move (Step 3 imports it by path).
- **Reference checks, all as expected.** Frame AUC from `scores.tsv`:
  `W0` 0.666, `F0+ctx` 0.666 (25 folders). Random (seed 0): `RN@95` 5.8%,
  `pAUC95` 0.504. `R0`'s fold function at pick weight 1 with the AUC inner CV
  (`folder_gate.choose_c`) reproduces `F0+ctx` to 5.00e-06, the file's
  6-significant-digit rounding, and the same `C` counts (20 x 1, 7 x 10).
  The run is deterministic: a rerun printed identical output.
- **Deviation: `eyes closed` reads the raw EAR.** The plan said `gate.load`'s
  `eo` is the 0–100 openness that is 0 at or below `EYES_CLOSED_EAR`; in the
  dump `eyes_open` is a 0–1 value whose minimum is 0.020, so `eo == 0` fired
  on no frame (the first run showed the rule removing nothing). The
  pre-registration defines the rule as "the EAR at or below
  `EYES_CLOSED_EAR`", so the script reads the `ear` column of
  `features/<folder>.tsv` (`load_ear`) and removes `ear <= 0.137` (2,385 of
  the 21,563 frames with a judgment). This also changes `any of` and
  `rule count`.
- **Deviation: `G0`'s weights are rescaled to mean 1 per training fold, and
  it matters.** The plan's implementation approach says `sample_weight =
  d["weight"]`; `folder_gate.extend`'s weights sum to 1 per folder (about
  6e-4 per frame), so `HistGradientBoostingClassifier`'s `l2_regularization`
  1.0 and `min_hessian_to_split` act against leaf hessian sums of about
  0.01 and shrink every leaf about a hundredfold; the fixed setting was
  meant for unit-scale weights. The script therefore passes `w * len(idx) /
  w.sum()` (a constant per fold, so the weighted objective is the same up to
  the regularization), decided before the first run. The literal-plan fit is
  a committed check row, `G0 on the raw weights (check)` (`g0_fold(...,
  rescale=False)`), printed in `reject.md`'s Results: `RN@95` 17.6%, `RN@99`
  9.0%, `pAUC95` 0.551, frame AUC 0.649, against the rescaled 22.0% / 11.5% /
  0.568 / 0.670; its paired difference against `W0` is +1.0 points (p =
  0.2891, 13 of 25 folders, 0 of 3 bars) and against `F0+ctx` -0.9 (p =
  0.7546, 11 of 25, 0 of 3). So `G0`'s pass of the comparison bar rests on
  this deviation; Step 3's Decision should read it with that said. (An
  earlier scratch check with sklearn's default early stopping on is not
  reproduced here and is dropped.) Reported in `reject.md`'s Reading.
- **Choice: `G0` gets the missing values as NaN.** The pre-registration says
  "missing values handled natively so the indicator columns stay"; the
  implementation approach says `folder_gate.design(d, mask, "full")`, which
  imputes the means. The script takes that design and puts NaN back into the
  11 technical columns wherever `d["tech"]` is NaN, keeping the indicator
  columns. `early_stopping=False` so the fit runs exactly the pre-registered
  200 iterations (sklearn turns it on by default above 10,000 samples, with
  a random 10% hold-out, which would be untracked tuning); `random_state` 0.
- **Choices the text left open:**
  - `need(r, picks)` is `ceil(r * picks / 100)` in integer arithmetic
    (`0.95 * 20` in floats is 19.000000000000004, whose ceil is 20).
  - A "bottom X%" or "bottom q" share removes `floor(X * n / 100)` frames
    (the reading aids, `sharpness bottom q`, the `quantile@r` cut).
  - `quantile@r`: `q_r` is searched on a grid of 1/2000 of the folder; the
    largest grid share whose mean recall over the other 24 qualifying
    folders is >= r (a 1e-12 tolerance). `absolute@r`: `t_r` is the
    (`M - ceil(r M / 100)` + 1)-th lowest pooled pick score of the other
    folders; frames strictly below it are removed (ties at `t_r` kept).
  - `sharpness bottom q` ranks only the frames with a sharpness value, ties
    by the seed-0 draw, and removes `floor(q * folder frames / 100)` of them.
  - `pAUC95` uses `-score` with `-inf` (missing sharpness / `eye_focus`)
    replaced by the folder's lowest finite score minus 1; it averages ties
    instead of breaking them by the seed-0 draw (it is threshold-free).
  - The frame AUC column is recomputed from the same scores rather than
    copied; it reproduces `folderwide.md`'s values row for row. A binary
    rule's frame AUC is that of its keep / remove split.
  - For a binary rule, the `quantile@95` cell carries the rule's own point
    (it needs no threshold) and `absolute@r`, `pAUC95` and the picks-lost
    columns are "-". An extra "Rule points" table gives each rule's point and
    the folders where it reaches 95% / 99%.
  - `R0`'s inner CV averages `RN@95` over the inner validation folders that
    qualify (>= 50 frames, >= 5 picks); a tie keeps the smaller `C`.
  - The main table is printed as two tables over the same rows (the bottom
    metrics, then the reading aids and the 99% label-free pairs) to stay
    readable; the 99% label-free pairs are extra.
  - Rejects are counted as `reject & ~pick`: all 58 are non-picks, all in
    qualifying folders.
- **Wall time.** 63 s on 20 jobs (load 4 s, the `R0` check 11 s, `R0` 10 s,
  `G0` 17 s, the `G0` raw-weight check 20 s, metrics 1 s). No `__pycache__/` appeared in the archived
  folders or the data folders.

## Step 3: rejectablate.py and Decision 2

- **Import.** `rejectablate.py` puts its own folder on `sys.path`, imports
  `rejectfilter as rf` (which sets up the five archived folders and the
  `GENRES` override, see Step 2), then `import ablate`: its `import
  folderwide as fw` hits the already-loaded module, so the archived
  `ablate.specs` / `ablate.task` run unchanged. The setup mirrors
  `ablate.main` (`dw` keeps a copy of `d["tech"]` taken before
  `folder_gate.extend`; `W0` folds are the 26 folders with a scorable burst).
- **Reference check.** The unablated reruns reproduce the stored held-out
  scores to 5.00e-06 (the files' rounding) for `F0+ctx` and `W0`
  (`scores.tsv`) and for `G0` (`scores_reject.tsv`), with identical `RN@95` /
  `pAUC95`.
- **Choice: the `G0` ablations run all 11 drop-one rows of `F0+ctx`'s
  `ablate.specs`** (the 7 groups plus `|yaw|`, `|pitch|`, `|roll|` and
  `ln burst size`), not only the 7 `REASONS` groups. The pre-registration
  says "the drop-one groups only", read as "the drop-one rows, not the
  each-alone ones"; the per-axis rows were cheap and Decision 2's item 2
  asks per head-pose axis. A reader holding to the literal 7 groups can
  ignore the 4 extra rows; none of them contributes.
- **`G0` drop-one fold** is `rejectfilter.g0_fold` (weights rescaled to mean
  1 per training fold, NaN put back into the technical columns) with the
  design's columns masked; written beside it in `rejectablate.py` rather
  than adding a parameter to the Step 2 script.
- **Each alone** runs on `F0+ctx` and `W0` (both bases, as the archived
  ablations did); the table reports `RN@95`, `RN@99`, `pAUC95`, the
  `quantile@95` pair and the frame AUC (the last reproduces the archived
  each-alone frame AUCs, e.g. Look 0.629 / 0.636).
- **`scores_ablate.tsv`** columns are `<base> <row label>` with the Markdown
  escapes removed (`F0+ctx drop |yaw|`), 55 columns (22 `F0+ctx`, 21 `W0`,
  12 `G0`), `nan` for the `W0` family on `2026-07-23`.
- **Wall time.** 1,464 held-out fits in 523 s on 20 jobs (load 4 s, metrics
  2 s, total 530 s); the archived 1,140 linear fits took 356 s, the 324
  `G0` fits are the rest. No `__pycache__/` appeared in the archived folders,
  this plan folder or the data folders.
- **Decision 2 outcome.** `G0` is the best filter at 95% and 99% and passes
  the comparison bar against `W0` only with the per-fold weight rescale (on
  the raw weights it fails); no filter passes the worth-a-filter bar (`G0`'s
  `quantile@95` 94.7% recall / 16.8% `RN` against the 25% floor), so no app
  change is recommended. Genre's "drop for the ranking" stands.

## Deferred issues (todo candidates)

- **Core: update the genre-drop todo with Decision 2's confirmation.** The
  todo "Core: a plan to drop the genre from the personal ranking" should note
  that the reject-filter criterion (Decision 2, item 3, in
  `docs/plans/20261016-reject-filter-eval/plan.md`) confirms Decision 1's
  "drop for the ranking": none of the four genre comparisons passes on
  `RN@95` (best `W0 +genre cols` +1.0 points, p = 0.064, 1 of 3 bars). Basis:
  Step 3 of this plan; the plan names this as the wrap-up's job.
