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
