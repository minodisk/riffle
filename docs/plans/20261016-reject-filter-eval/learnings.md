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
