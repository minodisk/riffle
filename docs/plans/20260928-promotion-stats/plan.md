<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Record promotion stats daily on a `stats` branch

## Purpose

GitHub only exposes release download counts as a running total, and keeps
repository traffic (views, clones, referrers, popular paths) for the last 14
days. Riffle v1 is about to be promoted (Reddit), and the effect of that
cannot be read off either of those after the fact. This work adds a scheduled
GitHub Actions workflow that takes a dated snapshot of the numbers every day
and commits it as CSV to a dedicated orphan branch, `stats`, in this
repository. `main` stays free of daily commits (its PR rules, CI and the
release-please history are untouched), and the history of every number
becomes a `git log` / spreadsheet away.

Decisions (taken with the user on 2026-09-28):

- **One small shell script of our own** (`gh api` + `jq`), not
  jgehrcke/github-repo-stats. The download counts need a script of our own
  in any case (github-repo-stats does not collect them), the traffic part is
  four more `gh api` calls into the same CSV layout, and one script on one
  branch is fewer moving parts than a Docker-based action with its own branch
  layout, report generator and token handling. No dashboard.
- **Per-asset rows.** The Tauri updater downloads `latest.json`, the
  `*.app.tar.gz` bundles and (because `updaterJsonPreferNsis: true`) the
  Windows `-setup.exe`, so an aggregate count would mix installs with
  auto-updates. Keeping the asset name in every row lets the installers
  (`.dmg`, `-setup.exe`, `.msi`, `.AppImage`, `.deb`, `.rpm`) be told apart
  from the updater artifacts and `.sig` files at read time; no classification
  is stored.
- **Cumulative counts, not deltas.** The CSV stores what the API returns
  (the running total per asset); a delta is a subtraction between two dated
  rows and is left to whoever reads the file.
- **Idempotent per day.** Every CSV is keyed by an ISO `date` column (UTC);
  the script drops today's rows before appending, so a re-run on the same
  day replaces rather than duplicates.
- **Two PRs.** Downloads work with the default `GITHUB_TOKEN` and should
  start recording before the promotion; traffic needs a PAT, so it is a
  separate step.
- **Traffic token: a new `STATS_TOKEN` repository secret.** The user created
  a fine-grained PAT for `minodisk/riffle` only, with `Contents: Read and
  write` and `Administration: Read-only`, and registered it as the Actions
  secret `STATS_TOKEN` (not `RELEASE_PLEASE_TOKEN`, so an expiry of one does
  not stop the other).
- **First run after each merge.** The agent may trigger the workflow with
  `workflow_dispatch` right after each merge and check that the `stats`
  branch and its CSVs appear.

## Steps

- [x] Step 1: Daily snapshot of release download counts to the `stats` branch
  - Done when:
    - `tools/stats/snapshot.sh` exists, runs locally with an authenticated
      `gh` (`GH_REPO` defaulting to `minodisk/riffle`), and writes
      `downloads.csv` into a directory given as its argument, with the header
      `date,tag,asset,download_count` and one row per asset of every
      release (drafts included, `--paginate`; releases are the unit the user
      promotes, so prereleases and drafts are not filtered). Re-running on the
      same day yields an identical file (today's rows are replaced, not
      appended twice). Rows are sorted so that the diff of a day is one
      contiguous block.
    - `.github/workflows/stats.yml` runs on `schedule` (one daily cron, UTC,
      off the hour to dodge GitHub's busy minute) and on `workflow_dispatch`,
      with `permissions: contents: write`, a `concurrency` group so two runs
      never race on the push, `runs-on: ubuntu-latest`, a `timeout-minutes`,
      and comments in the house style explaining why (see `ci.yml` /
      `release.yml`). It checks out `main` (for the script), checks out the
      `stats` branch into a subdirectory or creates it as an orphan when
      `git ls-remote --exit-code origin stats` finds none, runs the script,
      commits with a `chore(stats): snapshot YYYY-MM-DD` message only when
      the tree changed, and pushes. The commit author is the
      `github-actions[bot]` identity used elsewhere on GitHub.
    - `mise.toml`'s `lint` task passes shellcheck over `tools/stats/*.sh`
      as well, and `mise run ci` passes (actionlint, shellcheck, lychee).
    - `CONTRIBUTING.md` gets a short section (e.g. `## Promotion stats`)
      saying where the data lives (the `stats` branch, its CSVs, the columns,
      that counts are cumulative and updater artifacts are separate rows) and
      how to run the workflow by hand.
    - After the merge, a `workflow_dispatch` run creates the branch and the
      first snapshot.
  - Implementation approach (as far as it is known; omit if unknown):
    - Script: `set -euo pipefail`, `gh api --paginate
      "repos/$GH_REPO/releases" --jq '.[] | .tag_name as $t | .assets[] |
      [$t, .name, .download_count] | @csv'`, prefixed with the date. `jq`
      and `gh` are preinstalled on `ubuntu-latest`. Keep it shellcheck-clean
      (`-x`, same as `tools/git/*.sh`; reuse `tools/git/_lib.sh` only if it
      actually helps).
    - Idempotency: `grep -v "^$today,"` on the existing file (header kept),
      then append, then write back. Do not rewrite older rows.
    - Branch layout on `stats`: CSVs at the root plus a one-paragraph
      `README.md` pointing back at `CONTRIBUTING.md` on `main`. No workflow
      files, no CI on that branch (`ci.yml` only triggers on `main` and PRs).
    - Workflow: use `actions/checkout@v4` twice (the second with `ref: stats`,
      `path: stats`) guarded by a step that probes the remote branch, or a
      single checkout followed by `git worktree add`/`git switch --orphan`;
      pick whichever is shorter and survives the branch-missing case on the
      very first run. Only `main` is protected and there are no rulesets, so
      a push with `GITHUB_TOKEN` to `stats` is expected to work.
    - Do not add a `mise` task for the script; running it locally is
      `tools/stats/snapshot.sh <dir>`, documented in `CONTRIBUTING.md`.

- [x] Step 2: Persist traffic (views, clones, referrers, popular paths) and stars
  - Done when:
    - `tools/stats/snapshot.sh` also writes, in the same directory and with the
      same replace-today rule:
      - `views.csv` and `clones.csv`: `date,count,uniques`, one row per day
        from the 14-day window the API returns, upserted by date (the window
        overlaps from run to run; a re-run must not duplicate a day, and a
        missed run is filled by the next one).
      - `referrers.csv` and `paths.csv`: `date,referrer|path,count,uniques`
        — a dated snapshot of the rolling 14-day aggregate the API gives
        (there is no per-day breakdown for these).
      - `stars.csv`: `date,stargazers_count,forks_count` (from
        `repos/{owner}/{repo}`).
    - The workflow passes `secrets.STATS_TOKEN` to the script as `GH_TOKEN`
      (the download call works with it as well, so one token for the whole
      run).
    - After the merge, a `workflow_dispatch` run shows the traffic files on
      `stats` (the first proof that `STATS_TOKEN` can read the traffic API
      from inside Actions).
    - `CONTRIBUTING.md`'s section lists the new files, the meaning of each
      column, and that the workflow needs the `STATS_TOKEN` secret (a
      fine-grained PAT with `Contents: Read and write` and
      `Administration: Read-only` on this repository), and that a missing or
      under-scoped secret fails the run rather than silently skipping traffic.
    - `mise run ci` passes.
  - Implementation approach (as far as it is known; omit if unknown):
    - Step 1 is merged; this step extends its script and workflow, no new
      files.
    - Endpoints: `repos/{owner}/{repo}/traffic/views`, `.../traffic/clones`
      (`.views[]` / `.clones[]` with `timestamp`, `count`, `uniques`),
      `.../traffic/popular/referrers` and `.../traffic/popular/paths`
      (arrays of `referrer`/`path`, `count`, `uniques`), `repos/{owner}/{repo}`
      for stars. Quote referrer and path values for CSV (`@csv` does).
    - Do not swallow a 403 on the traffic endpoints: the workflow should
      fail loudly so the token problem is noticed before 14 days of data are
      lost.

## Trade-offs and risks

- **Schedule reliability.** GitHub runs scheduled workflows on the default
  branch only, may delay them under load, and disables them after 60 days of
  repository inactivity. A missed day leaves a gap in `downloads.csv` (the
  cumulative counts survive; a delta simply spans two days), while views and
  clones are back-filled from the 14-day window on the next run. Referrers
  and paths for a missed day are lost.
- **Idempotency by date (UTC).** A dispatch just before and just after
  00:00 UTC yields two dated rows, which is correct; a dispatch twice in the
  same UTC day replaces the earlier snapshot with the later one.
- **`stats` branch growth.** One commit and a few hundred bytes a day; no
  concern for years.
- **Token expiry.** A fine-grained PAT expires (at most a year). An expired
  `STATS_TOKEN` fails the run loudly (Step 2), and the downloads stop being
  recorded too because the run uses one token; the user renews it.
- **Release-shaping path.** Both PRs touch `.github/**`; merging needs the
  user's explicit OK per the pr/merge skill.

## Progress

- (2026-09-28) Step 1 complete
