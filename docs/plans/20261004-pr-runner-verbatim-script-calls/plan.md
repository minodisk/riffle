<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Make pr-runner run its scripts verbatim as standalone Bash calls

## Purpose

PR #671 taught `merger` (`.claude/agents/merger.md`, step 6) to run
`mise run git:main` and `delete_merged_branches.sh` exactly as written, each as
its own Bash call, because any pipe / `tail` / `;` variant misses the
`.claude/settings.json` allow rules and goes to the auto mode classifier.
`pr-runner` has the same hole, and the pr-runner transcript of PR #671 shows
its cost. The call
`bash .claude/skills/pr/scripts/wait-pr-actionable.sh 671 | tail -12; echo exit=${PIPESTATUS[0]}`
(timeout 600000) was issued at 04:39:31Z and returned at 05:52:09Z, about
73 minutes later. Back-calculating from the 600 s timeout, the script only
started at about 05:42:09Z, so roughly 62 minutes went by before it ran — the
`bash .claude/skills/pr/scripts/wait-pr-actionable.sh 671; echo exit=$?` call
just before it ran immediately, and the difference is the pipe and
`${PIPESTATUS[0]}`, which take the line out of the
`Bash(bash .claude/skills/pr/scripts/*)` allow rule. Then the script printed
`ACTION=ready` about 150 s after it started (visible in the background output
file), but the call did not return until the 600 s timeout and was moved to
the background: the pipe kept `tail` from seeing EOF (most likely a lingering
child holding the pipe open; not pinned down). A bare re-run returned `ready`
in 2 s. The exit code is already visible in the Bash tool result, so the
`; echo exit=$?` / `${PIPESTATUS[0]}` decoration buys nothing.

`pr-runner.md` §4 already forbids chaining `sleep` / `until` around the wait
script, but says nothing about pipes, `tail`, redirects or `; echo exit=$?`.
This plan extends that paragraph into the same verbatim, standalone rule
`merger` got, with the measured reason.

## Steps

- [x] Step 1: Extend `pr-runner.md` §4 with the verbatim, standalone script-call rule
  - Done when:
    - `.claude/agents/pr-runner.md` states, in the §4 paragraph that currently
      begins "Do not compose a compound command such as `until ...`", that
      every script under `.claude/skills/**/scripts` the agent runs — the
      `wait-pr-actionable` call above all — is run exactly as written in its
      code block, as a standalone Bash call: no pipes, no `tail`, no redirects
      (`2>&1`, `> /dev/null`), no `; echo exit=$?`, no `${PIPESTATUS[...]}`, no
      `;` / `&&` chaining (the existing `sleep` / `until` prohibition stays and
      folds into the same list)
    - The paragraph carries the measured reason in `merger.md` step 6's
      register: a shape outside the allow rule waits before it runs (PR #671:
      ~62 minutes, while the bare call just before ran immediately); a pipe can
      keep the call from returning after the script has finished (`ACTION=ready`
      at ~150 s, the call hit the 600 s timeout and was moved to the
      background, the bare re-run returned in 2 s); and the exit code is
      already in the tool result, so `; echo exit=$?` adds nothing
    - No other file changes; the surrounding §4 text is not reworded
    - `mise run ci` passes
  - Implementation approach:
    - Edit only the existing paragraph in §4; do not restate it elsewhere. Keep
      the `<wait-pr-actionable command>`-style placeholders the shared text
      uses; at most a one-line pointer may be added to the "Claude execution
      contract" at the end of the file if needed
    - Mirror `merger.md` step 6's wording and tone
    - Scope the rule to the scripts under `.claude/skills/**/scripts` that
      pr-runner runs, naming the wait script as the one where the cost was
      measured. Do not extend it to `git commit` / `git push` / `mise run ci`
      (decided by the user)
    - Do not assert more about the hung pipe than "most likely a lingering
      child holding the pipe open"

## Trade-offs and risks

- Scope: the user chose skill scripts only, where the cost was measured and the
  allow rule is exact. `git commit` / `git push` / `mise run ci` decorations ran
  fine in #671 and stay as they are.
- This is a prose fix; the real check is the next pr-runner run.

## Progress

- (none yet)
