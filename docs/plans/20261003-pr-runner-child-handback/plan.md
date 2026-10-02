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

# pr-runner waits for a child's hand-back instead of polling

## Purpose

Since the `long-wait-timeouts` fix (#328, merged 2026-09-22) every
`wait-pr-actionable.sh` / `wait-post-merge-runs.sh` call in the 240 `pr-runner`
and 225 `merger` transcripts passed `timeout: 600000`, and the ~28 runs the
harness still moved to the background were `TaskStop`ped and re-run in the
foreground without improvised polling. That part of the fix is confirmed and
`merger` needs nothing.

What `pr-runner` still gets wrong is the child dispatch. The `Agent` tool
launches a child (`pr-conflict-resolver`, `pr-check-fixer`,
`pr-review-planner`, `pr-review-addresser`) asynchronously whatever
`run_in_background` says; the tool result is "Async agent launched
successfully". In about 27 of ~50 dispatches `pr-runner` ended its turn,
received the child's "[Subagent hand-back]" message and continued correctly.
In the rest it kept working while the child ran: it re-ran
`wait-pr-actionable.sh` (seeing a stale `ACTION=conflict`) or improvised
`timeout 280 bash -c 'while [ "$(git ls-remote ...)" = <sha> ]; do sleep 15; done'`,
`until [ "$(gh pr view N --json mergeable ...)" != CONFLICTING ]; do sleep 10; done`,
`while pgrep -f pr-conflict ...`, `sleep 240`, `sleep 90; git fetch ...`
(2026-09-30 to 2026-10-01 runs on PRs #596, #607, #627, #640, #648), plus
stray `sleep 45; wait-pr-actionable.sh` and `sleep 60` / `sleep 90` chains
around `head-reviewed-by-copilot.sh` / `rerequest-review.sh`.

The doc's rationale is also wrong where it says a subagent "exits the moment
its turn ends, leaving nobody to receive the completion notice": the
transcripts show a subagent does receive the child's hand-back message and a
background task's notification after ending its turn. After this work
`.claude/agents/pr-runner.md` states once that a child launch is async and
that the runner ends its turn and waits for the hand-back without any polling,
the dispatch sites reference that paragraph, the `sleep` ban covers bare
`sleep` chained to scripts, the foreground-only rule for the wait script and
`mise run ci` keeps its design but with a rationale that no longer contradicts
the child-wait rule, and `todo.md` swaps the confirmed long-wait check for a
check of this new behavior.

## Steps

- [x] Step 1: Rewrite the child-dispatch and waiting rules in `pr-runner.md` and update `todo.md`
  - Done when:
    - `.claude/agents/pr-runner.md` contains one shared paragraph stating that
      an `Agent` launch of a child returns at once as async ("Async agent
      launched successfully") regardless of `run_in_background`, and that
      after dispatching, `pr-runner` ends its turn and waits for the child's
      hand-back message: no `wait-pr-actionable.sh`, no `sleep` / `until` /
      `while` polling, and no branch / PR polling (`git ls-remote`, `git
      fetch`, `gh pr view`, `pgrep`) until the hand-back arrives; then it
      branches on the child's return value as already described
    - The four dispatch sites reference that paragraph instead of repeating
      it: "conflict → `pr-conflict-resolver`", "check_failed →
      `pr-check-fixer`", the review handling (planner in step 2 and addresser
      in step 5 of "changes_requested → review handling", and the planner
      line of the "Claude execution contract"), and the local-CI-failure
      `pr-check-fixer` path in "2. Local CI and push"
    - The existing rule "Do not compose a compound command such as
      `until ...; do sleep 30; done`" in §4 also forbids a bare `sleep`
      chained before or after a script (`sleep 45; wait-pr-actionable.sh`,
      `sleep 60` / `sleep 90` around `head-reviewed-by-copilot.sh` /
      `rerequest-review.sh`)
    - No sentence in `pr-runner.md` claims that a subagent exits the moment
      its turn ends or that nobody is left to receive a notification after the
      turn ends (the "same reason as §4" sentence in "2. Local CI and push"
      and the paragraph "Do not switch the waiting to `run_in_background:
      true`" in §4 are the two places)
    - The rules "Do not set `run_in_background: true`" for `mise run ci` and
      "Do not switch the waiting to `run_in_background: true`" for the wait
      script remain, with a rationale that does not contradict the child-wait
      rule (see the implementation approach)
    - The paragraph "If the harness nonetheless reports that a command was
      moved to the background ... `TaskStop`" stays as is
    - `todo.md`: the section "### Agents: confirm the long-wait timeout fix on
      a real `/pr` or `/merge` run" (title, background paragraph, `#### TODO`
      and its one checkbox) is deleted whole, and a new section in its place
      asks for a real `/pr` run to confirm `pr-runner` ends its turn after
      dispatching a child and waits for the hand-back without polling; no
      other `todo.md` item is touched
    - `.claude/skills/pr/SKILL.md`, `.claude/skills/merge/SKILL.md`,
      `.claude/skills/develop/SKILL.md`, `.claude/skills/develop/README.md`
      and `.claude/skills/develop/references/pr-merge-lifecycle.md` stay
      untouched (none states that `pr-runner` cannot receive notifications
      after its turn ends)
    - `mise run ci` passes
  - Implementation approach:
    - Files: `.claude/agents/pr-runner.md`, `todo.md`, plus this plan.md
    - Keep the doc's wording and style; change only what the rule needs.
      Keep `run_in_background: false` in the dispatch instructions (it is
      harmless and the skills pass it too) but say it does not make the call
      synchronous
    - Put the shared paragraph in a short subsection under "## About the
      child agents" (e.g. "### Dispatching a child and waiting for its
      hand-back") and reference it from every dispatch site
    - New rationale for the foreground-only rules (`mise run ci` in §2, the
      wait script in §4): the foreground run is what lets the counters
      (`wait_timeouts` etc.) tick on exit 2 and what makes the exit code
      observable; a background run would hand the watching to a task
      notification, and the one time this was tried the runner reported
      "waiting for a notice" and dropped the watching onto the caller (keep
      that "this actually happened" fact). Drop the sentence "The skill this
      was ported from expands in the main session and could receive notices;
      that premise does not hold for an agent." Do not reuse "nobody left to
      receive" anywhere
    - §4's sleep ban: extend the existing sentence rather than adding a new
      rule, e.g. "... such as `until ...; do sleep 30; done`, nor a bare
      `sleep` chained before or after a script (`sleep 45; <wait-pr-actionable
      command>`, `sleep 60` before `rerequest-review.sh`)". The permission
      prompt reason still applies; add that the script already does the
      waiting
    - `todo.md` new section: `### Agents: ...`, `#### Background`,
      `#### TODO`, one `- [ ]` item. Background: the long-wait fix was
      confirmed on 240 pr-runner / 225 merger transcripts (all calls passed
      `timeout: 600000`; backgrounded runs were `TaskStop`ped and re-run), and
      this plan added the child-hand-back rule to `pr-runner.md`, verified
      statically only. TODO: on the next real `/pr` run whose PR hits a
      conflict, a CI failure or a review round, read the pr-runner subagent
      transcript (the subagent `.jsonl` files under the Claude Code project
      directory for that session) and confirm that after each `Agent`
      dispatch the runner ended its turn, the next event is the "[Subagent
      hand-back]" message, and there is no `wait-pr-actionable.sh`, `sleep`,
      `until` / `while`, `git ls-remote` / `git fetch` or `gh pr view` call in
      between; if not clean, adjust `.claude/agents/pr-runner.md`
    - Do not create or merge any PR just to verify; verification is the
      `todo.md` item

## Trade-offs and risks

- The same "subagent exits the moment its turn ends" rationale appears in
  seven other agent files (`merger.md`, `pr-check-fixer.md`, `implementer.md`,
  `local-review-addresser.md`, `local-review-runner.md`,
  `pr-review-addresser.md`, `pr-conflict-resolver.md`). The user chose to
  change only `pr-runner.md`; their rule (foreground `mise run ci`) is still
  the intended design
- The rule is prose for a sonnet agent, and ~half of dispatches were already
  correct without it; whether the new wording fixes the other half is only
  testable on real runs, which is why the `todo.md` item exists

## Progress
