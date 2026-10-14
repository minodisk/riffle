# Learnings

## Step 1

- **Precondition held.** The per-file-viewed-ranking plan is archived on
  `main` (#806, #810, #813, wrap-up #814). folder-wide-pick-model (#812) and
  the in-flight ball-game-ranking-features did not touch `gate.py`, the app
  dump or `rank.rs`'s 44 columns, and the shared model still reads 0.710 /
  54.9% Top-1, so the baseline did not move.
- **Python environment.** The system Python has no scikit-learn; the
  image-embeddings plan's venv at `%TEMP%\embedvenv` (scikit-learn 1.9.1,
  joblib, numpy) runs `gate.py` and `genre_offsets.py`.
- **Cost.** 27 folds x (shared, four all-columns values, one
  technical-only value) plus the fallback check (three draws x the same six
  variants) took 49 minutes on 20 joblib workers. The script prints only at
  the end (stdout to a file is block-buffered), so there is no progress
  line; a re-run should expect the same wait.
- **Result: keep on**, `OFFSET_SHRINK` = 30, offsets on all columns (see
  [offsets.md](offsets.md)). No genre shows a significant gain; portrait and
  Unknown lean negative at 30 with intervals that include zero.
- **The evaluation's `C` is not the app's.** gate.py chooses `C` per fold,
  and with offset columns in, it moved toward stronger penalties; the app
  keeps the prior's `C` and pulls the shared weights toward the prior, not
  zero. Step 2's tests check the mechanism, not this number.
- **Shell escaping tripped the Markdown edits.** Python heredocs run from
  Git Bash turned `\t` and `\20` in a Windows path into a tab and a control
  character; building the path with `chr(92)` fixed it.

## Step 2

- **`rank.rs` on `main` matched the plan's assumptions.** `Labeled` (with
  `viewed`) and `pairs` (with `REJECT_WEIGHT`) were as the plan expected;
  `pairs` only gained the burst's genre index per pair and returns the
  bursts per genre. No deviation to record.
- **`UNKNOWN` must be excluded in `Weights::of`, not only at parse.** The
  first draft looked the burst's genre up in `offsets` directly, so a
  `Weights` carrying an `Unknown` key (the test builds one on purpose)
  shifted unknown bursts. `of` now ignores `genre::UNKNOWN`; the app's
  `Stored::parse` also drops it as a non-top-level label.
- **Test vectors of one genre.** The label vectors themselves turn every
  label on (the text-text cosines are far above the image-text thresholds
  of 0.05 to 0.10), so `genre::vector_of` removes the projection onto every
  other label's vector and scales the rest to a dot of 1 with its own: it
  turns on that label alone (and its parent for a sub-genre).
- **The offset-at-work test passes with the shipped `OFFSET_SHRINK` = 30.**
  300 contradicting bursts in a second genre against 300 in the first turn
  that genre's burst to the blurrier frame while the shared weights still
  prefer the sharper one; 2 contradicting bursts do not move it. The pick
  alternates between the two positions so the position columns cancel.
- **The exact-zero offset is exact by construction.** A genre with no pair
  has zero gradient and a Hessian block uncoupled from the rest, so the
  Cholesky solve gives exactly 0.0 for its rows; the test compares with
  `==`.
- **Heredocs with long Rust bodies failed in Git Bash** (`unexpected EOF
  while looking for matching`), so the edits went through Python scripts
  written to the scratchpad with the Write tool.

## Step 3

- **The suffix shows the raw label.** The genre data file's labels are
  lowercase (`sports`, `birds / animals`), so the row reads `· as sports`
  as the plan's example does, while the usage docs name the genres
  capitalized (`Sports`) in prose. No mapping was added.
- **`Ranked.genre` is set even when the burst has one scored frame.** It
  names the label the weights came from; `recommend.ts` ignores the field,
  so the sparkles icon is unchanged (its test fixture only gained
  `genre: null`).

## Deferred issues (todo candidates)

- **Portrait leans negative under the offsets.** At `OFFSET_SHRINK` 30 the
  portrait bursts' (57) paired difference is -0.024 [-0.067, +0.003] and
  the technical-only shape's is significantly negative (-0.058
  [-0.115, -0.016]). Not a bar failure (no per-genre bar), but worth
  re-checking once Step 2 ships and more non-sports judgments accumulate,
  e.g. by re-running `genre_offsets.py` on a fresh dump. Basis: Step 1's
  run, `docs/plans/20261014-genre-ranking-offsets/offsets.md`; related
  `crates/core/src/rank.rs`.
- **Pending manual check (Step 2): the fit line lists bursts per genre and
  `ranking.json` holds `offsets`.** Already added to `todo.md`'s
  "App: real-device check of the personal ranking" section by this step,
  as the plan's Done-when asks; the wrap-up need not add it again. On
  Windows 11, with AI analysis on and a `ranking.json` from before this
  change, start Riffle; expected: the startup `ranking fit:` line in
  `Riffle.log` ends with `per genre: sports N, ...`, and `ranking.json` then
  has `"rankVersion":2` and an `offsets` object keyed by the five top-level
  genres, 44 numbers each. The step's checkbox was ticked on the automated
  criteria. Files: `crates/app/src/rank.rs`, `crates/core/src/rank.rs`.
- **Pending manual check (Step 3): the `Recommended` row's genre suffix.**
  Already added to `todo.md`'s "App: real-device check of the personal
  ranking" section by this step, as the plan's Done-when asks; the wrap-up
  need not add it again. On Windows 11, with AI analysis on, select a frame
  of a sports burst in a folder embedded by pass 3; expected: the meta
  pane's `Recommended` row reads `N of M in its burst · +0.xx · as sports`,
  and a burst whose frames show no genre has no suffix. The step's
  checkbox was ticked on the automated criteria. Files:
  `crates/core/src/rank.rs`, `crates/app/ui/src/meta.ts`.
