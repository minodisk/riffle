# Learnings: pr-runner-child-handback

## Step 1

- The shared rule lives in a new subsection "Dispatching a child and waiting
  for its hand-back" under "## About the child agents" in
  `.claude/agents/pr-runner.md`; the four dispatch sites (conflict,
  check_failed, review handling steps 2 and 5, the local-CI `pr-check-fixer`
  path) and the planner line of the "Claude execution contract" point to it
  by name.
- The review handling step 2 used to say the planner runs "in the
  foreground"; that wording contradicted the async launch, so it was replaced
  with the hand-back reference rather than kept.
- The foreground-only rationale for `mise run ci` and the wait script now
  rests on the exit code being observable and the counters ticking on exit 2,
  not on a subagent being unable to receive notifications.
- The phrase "there is nobody to redo it" in the review-plan verification
  step is unrelated (a converted plan JSON has no planner to re-run) and was
  left alone.
- A fresh worktree has no `node_modules`, so `mise run fmt` failed with
  `Cannot find module ...vite-plus/bin/vp`; `pnpm` is only on the PATH through
  mise, so `mise exec -- pnpm install --frozen-lockfile` fixed it.

## Deferred issues (todo candidates)

- The "a subagent exits the moment its turn ends, leaving nobody to receive
  the completion notice" rationale still appears in `.claude/agents/merger.md`,
  `pr-check-fixer.md`, `implementer.md`, `local-review-addresser.md`,
  `local-review-runner.md`, `pr-review-addresser.md` and
  `pr-conflict-resolver.md`. Basis: this plan's "Trade-offs and risks"; the
  user chose to change only `pr-runner.md`, and the rule itself (foreground
  `mise run ci`) is still intended, only its stated reason is inaccurate.
- Pending manual check (already recorded in `todo.md` by this step): on a
  real `/pr` run with a conflict, CI failure or review round, confirm in the
  pr-runner subagent transcript that each `Agent` dispatch is followed by the
  end of the turn and then the "[Subagent hand-back]" message, with no
  polling in between. The step's checkbox was ticked on the static criteria.
