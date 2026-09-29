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

## After the Step 2 merge

- The first `workflow_dispatch` after the Step 2 merge failed with
  `gh: To use GitHub CLI in a GitHub Actions workflow, set the GH_TOKEN
  environment variable` and the log showed `GH_TOKEN:` empty, although
  `gh secret list` listed `STATS_TOKEN`. The secret had been registered
  with an empty value (`read -rs T && printf '%s' "$T" | gh secret set ...`
  in Git Bash, with nothing pasted before Enter). `gh secret list` cannot
  tell an empty secret from a set one; the run log's empty `GH_TOKEN:` line
  is the tell. The user regenerated the fine-grained PAT and pasted it into
  the secret's page on github.com, and the next dispatch
  (run 36500846205) succeeded and wrote every CSV.
- Pushing a branch that adds `.github/workflows/*` needs the `workflow`
  scope on the gh token (`gh auth refresh -h github.com -s workflow`);
  without it the push is rejected with "refusing to allow an OAuth App to
  create or update workflow".

## Deferred issues (todo candidates)

- **Docs: add a guide for GitHub Actions workflows and `gh` secrets.**
  Change: create `docs/agents/github-actions-workflows.md` (read before
  writing or debugging a `.github/workflows/*.yml` that uses `gh`, secrets,
  or a dedicated branch as a data store, like `stats`). Points: an empty
  secret is indistinguishable in `gh secret list` and shows as an empty
  `GH_TOKEN:` in the run log (set secrets on github.com, or check the value
  is non-empty before `gh secret set`); pushing `.github/workflows/*` needs
  the `workflow` scope (`gh auth refresh -h github.com -s workflow`);
  `shellcheck` / `actionlint` run ad hoc through `mise exec --`;
  `git worktree add --orphan` (git 2.42+, present on `ubuntu-latest`) plus
  `git diff --cached --quiet` for a commit-only-when-changed guard.
  Rationale: no guide under `docs/agents/` covers workflows or secrets, so
  the next workflow change would rediscover these. Done when: the guide
  exists with these points and links this plan's archived learnings.md.
