# Learnings

## Step 1

- **main's post-merge CI lint went red on this plan's link.** The plan PR
  (#827) linked the similar-scene-groups plan at
  `../20261015-similar-scene-groups/plan.md`. Another session's PR #826
  (stop and archive similar-scene-groups) moved that folder to
  `docs/plans/_archived/20261015-similar-scene-groups/` and merged first, so
  after both merged lychee (`mise run lint`) failed on main with a missing
  file. Each PR passed on its own; the break only exists in their
  combination. Fixed in the Step 1 PR by pointing the link at
  `../_archived/20261015-similar-scene-groups/plan.md` and saying the plan
  was stopped and archived in #826. Lesson: a plan that links a sibling plan
  in flight can break when that plan is archived; prefer linking archived or
  stable targets, or re-run lint after rebasing onto a main that moved plans.
- `folderwide.md`'s pre-registration was written from file lists, the
  archived harnesses' function names and the extras TSVs' header lines only;
  no label-dependent number was computed in this step.

## Step 2

- **Reference checks all reproduce exactly**: `W0` 0.710 weighted AUC /
  54.9% Top-1 within bursts (archived gate 0.710 / 54.9%), `F0+ctx` 0.666
  mean frame AUC over the 25 qualifying folders (archived folder-wide gate
  0.666), `W0+off` 0.709 (`offsets.md` 0.709). Nothing to investigate.
- **Wall time** 456 s on 20 jobs (Windows 11): `W0+off` 152 s, `P0` 86 s,
  `F0+ctx +genre off` 46 s, every other variant 9–25 s. Far below the plan's
  estimate (`genre_offsets.py`'s 49 min was mostly its fallback reruns).
  stdout is buffered to the end, so the script writes one progress line per
  variant to stderr (`folderwide.err`) for polling.
- **The archived harnesses' own import paths are stale.** `folder_gate.py`,
  `ballgate.py` and `genre_offsets.py` insert
  `<their folder>/../_archived/20261012-image-embeddings` (written before
  they were archived, so now `_archived/_archived/...`), and
  `folder_gate.GENRES` resolves to `docs/crates/...`. They import only
  because `folderwide.py` puts all four archived folders on `sys.path` and
  imports `gate` first, and it overrides `folder_gate.GENRES` with the real
  `crates/core/genres/siglip2.json` before `extend`. No archived script was
  modified.
- **`folder_gate.extend` replaces `d["tech"]`** with 11 columns (adding
  absolute `ln sharpness`), which `gate.design` would then take as `TECH`.
  The `W0` family, `F0` and `P0` therefore run on a shallow copy of `d`
  holding the original 10 technical columns; `F0+ctx` and its cue variants
  use `folder_gate.design(d, mask, "full")` on the extended `d`, as the
  archived gate did.
- **`W0` folds are 26, not 27.** `gate.lofo` holds out only folders with a
  scorable burst; `2026-07-23` (18 frames, 4 picks) has none, so the `W0`
  family gives it no score (`-` in the per-folder table, `nan` in
  `scores.tsv`). It is not qualifying, so no mean, interval or test changes.
- **The burst-centered `W0` score is not near chance folder-wide** (frame
  AUC 0.662 against `W0`'s 0.666), contrary to the pre-registration's
  "expected near chance". It is a reading aid, so nothing turns on it; the
  number is reported as measured.
- **`F0+ctx +genre off` chose `C` = 10, the top of `gate.CS`, in all 27
  folds**, so a larger `C` might fit it better. The grid is pre-registered and
  was not extended.
- Choices the pre-registered text left open (none changes a bar):
  - `+genre off` on the pointwise model adds `hier_design`'s columns only
    (`[burst genre is g] * x / sqrt(30)` for each top-level genre, over every
    design column); the intercept stays shared, with no per-genre intercept
    column, matching the shipped offsets' form.
  - `P0`'s pairs are drawn without replacement from the folder's pick x
    non-pick product (`rng.choice(p * q, 20000)`, seed = folder index,
    sorted), weighted `2 / len` per flipped pair as `gate.pairs` does (its
    "pairs total 1" in the doubled-entry reading). 383,072 pairs over the
    folders, 17 folders capped; the all-folder design is 129 MiB, so the cap
    was not lowered. `P0`'s inner CV uses `folder_gate.choose_c`'s
    `GroupKFold` splits and fits on the pairs of the inner training folders.
  - "Pooled" is the frame-weighted mean of the per-folder values over the
    qualifying folders (not one AUC across folders, which would compare
    uncentered scores between folders).
  - R-precision's ties at the cut break by the same seed-0 uniform draw that
    breaks the lift's ties and is the random baseline.
  - The folder bootstrap uses a fresh `np.random.default_rng(0)` per
    comparison and metric, so every comparison is read on the same 1,000
    draws of folder indices. The within-burst interval is
    `genre_offsets.boot` on the per-burst weighted AUC; the Top-1 difference
    is reported without an interval.
  - "Folders at or above" counts ties as at or above, per the bar's wording.
  - Pick vs explicit reject covers only the 3 folders holding a rejected
    non-pick.
- **`sys.dont_write_bytecode` does not reach the joblib workers.** The loky
  worker processes import `gate` and `genre_offsets` again when unpickling
  the tasks, without the flag, and left `__pycache__/` in two archived
  folders. They were deleted by hand before committing. Step 3 should run
  with `python -I -B` (`-I` ignores `PYTHONDONTWRITEBYTECODE`) and check
  `git status` for stray `__pycache__/` afterwards.

## Step 3

- **Wall time** 356 s on 20 jobs (Windows 11) for 1,140 held-out fits
  (22 `F0+ctx` models x 27 folds + 21 `W0` models x 26 folds) plus 3 s for
  the two all-folder fits, far below the plan's one-hour budget. Run with
  `python -I -B`; no `__pycache__/` was left this time.
- **The unablated rerun matches `scores.tsv`** to 5e-6 (the file's
  6-significant-digit rounding) on both bases, so the column-mask path is
  the Step 2 model and every drop-one difference is against the same
  held-out score.
- **`python -I` drops the script's own folder from `sys.path`**, so
  `ablate.py` inserts its folder before `import folderwide`; the loky
  workers inherit the parent's `sys.path` and import it too.
- Choices the pre-registered text left open (none changes a bar):
  - Each ablation masks the columns of the base's per-fold design after it
    is built and standardized. Per-column standardization makes that the
    same as building it without them; the PCA is still fit on the training
    folders as before. Every masked model reruns the inner CV over
    `gate.CS`.
  - On `F0+ctx`, the `Sharpness` group is the relative `ln rel sharpness`
    and the absolute `ln sharpness` (as `folder_gate.GROUPS`), and
    `Burst position` is `position` and `first frame` only; `ln burst size`
    is its own drop-one row and is not in "burst position alone".
  - Dropping `|yaw|`, `|pitch|` or `|roll|` alone keeps `no pose`.
  - "Alone" on `W0` is the within-burst pairwise fit on those columns,
    on `F0+ctx` the weighted pointwise fit; c1, c2 and the genre labels
    alone are their block mean-imputed and standardized on the training
    folders (`folderwide.append_block` on an empty design).
  - "Contributes" is read on each metric separately (frame AUC,
    R-precision, within-burst wAUC); the pre-registration says "the primary
    metric", which has two parts.
  - The each-alone table gives the held-out means without intervals (the
    pre-registration asks for intervals only on the drop-one changes).
  - The fitted weights: `W0` on every scorable burst, `F0+ctx` on every
    frame, `C` by the same inner grouped CV over all 27 folders (`W0` 0.1,
    `F0+ctx` 1). Look is reported as the L2 norm of its 32 PCA weights.
- Decision 1 cites one non-pre-registered number: `F0+ctx +genre off`
  against `W0`, taken as the difference of the two table means (+0.012
  frame AUC, -0.9 points R-precision), with no interval or test, to show it
  would not pass against the reference either.

## Deferred issues (todo candidates)

- **The shipped model scores singles at chance folder-wide.** Basis: Step 2
  results and Step 3's Decision 1 (reading aid, not a bar). Held out, `W0`
  reads singles AUC 0.508 against 0.611 for `F0+ctx` (13 folders with >= 5
  single picks and 5 single non-picks). The app ranks only within bursts
  today, so nothing shows it; if a later plan ever ranks or marks singles
  (a folder-wide "best" view), the within-burst weights should not be used
  for them as is. Files: `crates/core/src/rank.rs`,
  `docs/plans/20261015-folder-wide-ranking-eval/folderwide.md`.

- **The archived harnesses cannot run standalone any more.** Basis: Step 2
  implementation (the import paths found while importing them).
  `docs/plans/_archived/20261014-folder-wide-pick-model/folder_gate.py`,
  `docs/plans/_archived/20261014-ball-game-ranking-features/ballgate.py` and
  `docs/plans/_archived/20261014-genre-ranking-offsets/genre_offsets.py`
  insert `../_archived/20261012-image-embeddings` relative to their own
  folder, which after the archive move is `_archived/_archived/...`, and
  `folder_gate.py`'s `GENRES` points at `docs/crates/core/genres/siglip2.json`.
  Running any of them directly fails at `import gate` (or at `extend`). They
  still work when imported after `gate`, as `folderwide.py` does. Either fix
  the paths in the archived scripts or note in the archive convention that a
  plan's scripts are frozen as run.
