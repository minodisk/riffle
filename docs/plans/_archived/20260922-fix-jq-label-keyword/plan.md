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

# Fix the jq compile error in wait-post-merge-runs.sh

## Purpose

`.claude/skills/merge/scripts/wait-post-merge-runs.sh` is the merger agent's
post-merge gate. After merging PR #292 (merge commit
e63b25ae653bac6738c4eb0ae8bcbc51e4152c57) it exited 3 with a jq syntax error
on two consecutive attempts, so the merger reported `run_list_failed` even
though both runs (CI, Release) had succeeded.

The cause is the jq variable name `$label` in `print_result()` (line 114).
`label` is a jq keyword; jq 1.6 (installed locally) refuses `$label` with
`syntax error, unexpected label, expecting IDENT`, while jq 1.7+ accepts
keywords after `$`. jq's compile-error exit status is 3, which `errexit`
propagates, colliding with the script's documented "`gh run list` failed"
code and sending the merger down the retry path. Every merge that produces at
least one run hits `print_result`, so on jq 1.6 the script has never
succeeded on that path; only `STATUS=no_runs` merges got through.

Once fixed, the merger reaches `STATUS=success` / `STATUS=failed` verdicts on
jq 1.6 as well as 1.7, and a test pins the table output so the next rename
cannot silently break it again.

## Steps

- [x] Step 1: Rename the `$label` jq variable and add a stub-`gh` test for the verdict paths
  - Done when:
    - `bash .claude/skills/merge/scripts/wait-post-merge-runs.sh --max-wait=240 e63b25ae653bac6738c4eb0ae8bcbc51e4152c57` exits 0 on jq 1.6 with a two-row `success` table, `TOTAL_COUNT=2`, `FAILED_COUNT=0`, `SUPERSEDED_COUNT=0`, `STATUS=success`
    - A new `.claude/skills/merge/scripts/wait-post-merge-runs.test.sh` passes locally and is run by `mise run lint` (so by `mise run ci`), covering at least: all success (exit 0, `STATUS=success`), one failure (exit 1, `STATUS=failed`, `FAILED_COUNT=1`, a `failure` row), cancelled with a newer success on the default branch (exit 0, a `superseded` row, `SUPERSEDED_COUNT=1`), and no runs (exit 0, `STATUS=no_runs`)
    - `mise run ci` passes (shellcheck on the new test file included, since `shellcheck -x .claude/skills/*/scripts/*.sh` globs it)
  - Implementation approach:
    - Script change is one token: rename `$label` to a non-keyword name (e.g. `$verdict`) in the `print_result()` jq program (both the `as $label` binding and the `\($label)` interpolation). Nothing else in the script needs to change; the `$sup | index($name)` construct is already correct. Do not touch the exit-code table in the header comment or the `STATUS=` lines; the merger (`.claude/agents/merger.md`) branches on them
    - Test file: follow `check-merge-approval.test.sh` (same header, `set -o errexit/pipefail/nounset`, `failures`/`cases` counters, `indent()` helper, final `N cases passed` line, exit 1 on any failure). Drive the script without modifying it by putting a stub `gh` first on `PATH` from a temp dir under `mktemp -d` (clean it up with a `trap`). The stub dispatches on its arguments: `run list --commit=*` prints the fixture runs JSON, `repo view --json defaultBranchRef ...` prints `main`, `run list --workflow=*` prints the fixture "recent runs" JSON, anything else exits 1 with a message. Pass fixtures via env vars the stub reads. Run the script with `POLL_INTERVAL=1 INITIAL_GRACE=0` so no case sleeps (and `--max-wait=0` where a timeout is not the point). Fixture JSON needs the same fields the script requests
    - Wire it into `mise.toml` `[tasks.lint]` next to the existing `bash .claude/skills/merge/scripts/check-merge-approval.test.sh` line
    - Verify during implementation that the unpatched script fails the new test on jq 1.6 (`jq --version`) before applying the rename, so the test is known to catch the bug
    - Note in the PR body that CI's `ubuntu-latest` ships jq 1.7.x, where `$label` was accepted, which is why CI never caught this

## Trade-offs and risks

- Exit-code collision: an internal jq/tooling failure still exits 3, the same code the merger reads as "gh failed, retry". Left as is to keep the contract untouched; a possible follow-up.
- CI's jq 1.7 cannot reproduce the original error; the test pins the observable output on both versions.
- The test relies on `gh` being resolved through `PATH`; if that changes, the stub's "unexpected" message makes the test fail loudly.

## Progress

- (2026-09-22) Step 1 complete
