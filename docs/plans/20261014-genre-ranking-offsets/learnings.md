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

## Deferred issues (todo candidates)

- **Portrait leans negative under the offsets.** At `OFFSET_SHRINK` 30 the
  portrait bursts' (57) paired difference is -0.024 [-0.067, +0.003] and
  the technical-only shape's is significantly negative (-0.058
  [-0.115, -0.016]). Not a bar failure (no per-genre bar), but worth
  re-checking once Step 2 ships and more non-sports judgments accumulate,
  e.g. by re-running `genre_offsets.py` on a fresh dump. Basis: Step 1's
  run, `docs/plans/20261014-genre-ranking-offsets/offsets.md`; related
  `crates/core/src/rank.rs`.
