# Learnings

## Step 1

- shellcheck flags the jq filter's `$t` as SC2016 (expression in single
  quotes). It is a jq variable, so the line carries a
  `# shellcheck disable=SC2016` with the reason rather than switching quotes.
- `shellcheck` and `actionlint` are not on the Git Bash `PATH` outside mise
  tasks; run them ad hoc with `mise exec -- shellcheck ...`.
- The workflow starts the branch with `git worktree add --orphan -b stats
  stats` (git 2.42+; `ubuntu-latest` ships newer). Checked locally that
  `git diff --cached --quiet` works on the unborn branch (0 before `git add`,
  1 after), so the "commit only when changed" guard survives the first run.
- The existing branch is fetched with `git fetch --depth=1 origin stats:stats`
  into the shallow checkout and added as a worktree; the push reuses the
  credentials `actions/checkout` persisted in the shared `.git/config`.
- Local verification against `minodisk/riffle`: 306 asset rows on
  2026-09-28; a second run gave a byte-identical file, and a hand-inserted
  row dated the day before survived a re-run in place.
- Every day adds new dated rows, so a scheduled run always commits; the
  no-change skip only matters for a second run on the same UTC day.

## Step 2

- `@csv` quotes strings, so the views / clones rows are built with jq string
  interpolation (`"\(.timestamp[:10]),\(.count),\(.uniques)"`) to keep the
  date column unquoted; the line-start `^<date>,` match that drops a day's
  rows depends on it. Referrer and path values keep `@csv` quoting.
- The per-file rewrite became one `upsert` helper that takes the dates to
  drop explicitly (today for the snapshot files, the window's dates for views
  and clones) rather than deriving them from the new rows, so an empty
  referrer or path list on a re-run still clears that day's earlier rows.
- Each file is now sorted as a whole (date first), not only the new block.
  For `downloads.csv` that is the same order Step 1 produced, since its older
  days were already sorted and precede today.
- All six API calls run before any file is written, so a 403 from the traffic
  endpoints aborts under `set -e` with every file untouched.
- Local verification against `minodisk/riffle` on 2026-09-28: two runs gave
  byte-identical files (307 / 15 / 15 / 2 / 11 / 2 lines for downloads,
  views, clones, referrers, paths, stars). A hand-edited in-window views day
  was restored from the API, an out-of-window day (2026-09-01) survived, and a
  referrer row dated the day before was kept.
- `paths.csv` rows are sorted by path, not by the API's rank; the rank is
  recoverable from `count`.
