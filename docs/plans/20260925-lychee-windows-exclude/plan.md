<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Make the lychee review-history exclusion separator-independent

## Purpose

`mise run lint` (and so `mise run ci`) fails on Windows: lychee 0.24.2 reports
input paths with backslashes (`docs\plans\review-history\...`), and the
`--exclude-path docs/plans/review-history` value in `mise.toml`, which lychee
treats as a regular expression, only matches forward slashes. The
review-history files, which are meant to be excluded, are then link-checked
and one of them fails on a stale `#running-the-app` fragment. Once fixed,
`mise run ci` passes locally on Windows and keeps behaving identically on
the Linux GitHub Actions runner.

## Steps

- [x] Step 1: Replace the literal exclusion path with a separator-agnostic regex in the `lint` task
  - Done when:
    - `mise.toml` line 73 uses `--exclude-path 'docs.plans.review-history'` (regex `.` matches both `/` and `\`) instead of `docs/plans/review-history`
    - `mise run lint` passes on Windows with the review-history files present (lychee reports 0 errors; the total link count drops by exactly the review-history link, from 498 to 497, and the 51 OK links are unchanged)
    - CI (`mise run lint` on ubuntu in `.github/workflows/ci.yml`) is green on the PR
    - The PR description explains the backslash cause in one or two sentences so the pattern is not "simplified" back to a literal path later
  - Implementation approach:
    - Only `mise.toml` changes (the single lychee line in `[tasks.lint]`). Do not touch the review-history file or the other lint steps.
    - Do not use a backslash character class (`docs[/\\]plans[/\\]review-history`): Git Bash/MSYS argument conversion collapses `\\` to `\` before lychee sees it, and lychee then aborts with `regex parse error: invalid character class range`. `.` avoids quoting problems in every shell.
    - Keep the value single-quoted like the neighboring glob arguments.
    - Verify locally with the task shell, i.e. `mise run lint` (not only `mise x -- lychee ...`), so the `bash -c` path mise uses on Windows is exercised. The `pnpm install` / `vp check` steps run first; if `vp` is missing in a fresh worktree, `mise exec -- pnpm install` fixes it.
    - Optional sanity check that is not committed: run the same lychee flags against a scratchpad Markdown file with a broken fragment (`[x](#nope)`) and confirm it reports 1 error, showing fragment checking is intact.
    - Commit as `fix(mise): match the lychee review-history exclusion on Windows paths` (repo precedent: `fix(mise): ...` in #330).

## Trade-offs and risks

- `docs.plans.review-history` versus the bare `review-history`: both produce the same result today. The longer form keeps the intent of excluding exactly that directory; the bare form would also exclude any future file or directory named `review-history` elsewhere.
- Restructuring the inputs so review-history is never fed to lychee (e.g. enumerating `docs/` subdirectories instead of `docs/**/*.md`) was rejected: it needs several globs that must be kept in sync as `docs/` grows, whereas the regex change is one token.
- `.` also matches any single character other than a separator, so `docsXplansXreview-history` would be excluded too. No such path exists and none is plausible; the small over-match is accepted for the sake of shell-independent quoting.
- Other `mise.toml` lint steps (`shellcheck` globs, `actionlint`, the merge-skill test scripts) pass forward-slash paths to bash, which Git Bash handles; no separator issue was found there, so nothing else changes.

## Progress

- (none yet)
