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

# Agent files: correct the foreground-only rationale

## Purpose

The `pr-runner-child-handback` plan
(`../_archived/20261003-pr-runner-child-handback/plan.md`) established that a
subagent does receive a child's hand-back and a background task's notification
after its turn ends, and rewrote the rationale for the foreground-only rules in
`.claude/agents/pr-runner.md` accordingly. The same inaccurate claim ("a
subagent exits the moment its turn ends, leaving nobody to receive the
completion notice") still appears in seven other agent files. The rules
themselves (run `mise run ci` and the wait scripts in the foreground with
`timeout: 600000`) are the intended design; only the stated reason is wrong, and
a wrong reason invites an agent to "correct" the rule once it notices the
premise does not hold. This work replaces the reason with the one `pr-runner.md`
now gives, so every file states the rule for the right reason.

## Steps

- [x] Step 1: Replace the "exits the moment its turn ends" rationale in the seven agent files
  - Done when:
    - `grep -rn "exits the moment\|moment its turn ends\|you exit the moment\|nobody left to receive\|nobody to receive" .claude/agents` returns nothing
    - In `merger.md`, `pr-check-fixer.md`, `implementer.md`,
      `local-review-addresser.md`, `local-review-runner.md`,
      `pr-review-addresser.md` and `pr-conflict-resolver.md`, every
      "Do not set / switch to `run_in_background: true`" rule and every
      `timeout: 600000` instruction is still present, each with a reason
      consistent with `pr-runner.md`
    - `local-review-runner.md` no longer justifies the foreground
      `commit-review-history.sh` run by analogy with the Address phase's
      `Agent` launch
    - `todo.md`'s section "### Agents: fix the inaccurate "subagent exits the
      moment its turn ends" rationale in the other agent files" is the item
      this PR resolves (deleting it is left to the wrap-up phase's
      todo-curator; say so in the PR description)
    - `mise run ci` passes
  - Implementation approach:
    - Files: the seven files under `.claude/agents/` listed above, plus this
      plan.md. Do not touch `pr-runner.md` (already correct) or the skills
      under `.claude/skills/`
    - Docs-only, English, surgical: change only the parenthetical / paragraph
      that carries the wrong reason; keep each file's wording and style
      (list item vs paragraph, bold on the rule) and the surrounding
      `timeout: 600000` sentences as they are
    - For the `mise run ci` rule in `pr-check-fixer.md` (line ~64, which also
      says "The same applies to the local reproduction in step 1": keep that
      sentence), `implementer.md` (~66), `local-review-addresser.md` (~82),
      `pr-review-addresser.md` (~73) and `pr-conflict-resolver.md` (~154), use
      the reason from `pr-runner.md` "2. Local CI and push" (line ~96-98):
      the foreground run is what makes the exit code observable to branch on,
      while a background run would hand the watching to a task notification.
      These files have no counters, so do not mention counters there
    - For `merger.md`'s step-1 waiting loop (paragraph at ~91-96): merger
      holds `pr_wait_timeouts` / `pr_status_failures` and branches on exit 2
      like `pr-runner` §4, so use the §4 reasoning (`pr-runner.md` ~262-273):
      the foreground run is what lets the counters tick on exit 2 and what
      makes the exit code observable to branch on; a background run would hand
      the watching to a task notification, and the one time this was tried the
      runner handed "waiting for a notice" back to the caller, dropping the
      watching onto the caller (keep that "this actually happened" fact, which
      the current text already carries as "a trap `pr-runner` actually hit").
      Keep the closing instruction "Express a long wait by ticking it off in
      the foreground while counting". Leave the post-merge paragraph
      (~211-217, "backgrounding is impossible for the same reason as step 1")
      alone: it already gives the exit-code reason and references step 1,
      which becomes correct
    - For `local-review-runner.md` (~91-95): drop the "same reason as starting
      children in the foreground in '4. Address phase'" cross-reference
      entirely (an `Agent` launch returns async regardless of
      `run_in_background`, see `pr-runner.md` "Dispatching a child and waiting
      for its hand-back", so the Address phase is not an instance of the same
      mechanism) and state the `mise run ci` reason directly, as in the five
      files above. Do not change the Address phase itself or the "Claude
      execution contract" lines (`run_in_background: false` for the
      children is harmless and matches the skills)
    - Per the archived plan, do not reuse the phrase "nobody left to receive"
      (or "nobody to receive") anywhere
    - Verify with the grep in "Done when" and with `mise run ci` (prettier /
      markdownlint / lychee cover `.claude/agents/*.md`)

## Trade-offs and risks

- `merger.md`'s post-merge paragraph says backgrounding is "impossible for the
  same reason as step 1". "Impossible" is stronger than the corrected reason
  (it is ruled out, not impossible). Left untouched here to keep the change
  surgical.
- `local-review-runner.md`'s "Claude execution contract" says to start the
  children "in the foreground, too". That is a rule (`run_in_background:
  false`), not a reason, and is harmless, so the plan keeps it.
- The "this actually happened" incident is placed only in `merger.md`, whose
  waiting loop is the structural twin of `pr-runner.md` §4; the five
  `mise run ci` files get the short §2 reason, as `pr-runner.md` itself does.

## Progress

- (none yet)
