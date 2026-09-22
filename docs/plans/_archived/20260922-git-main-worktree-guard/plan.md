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

# Keep `git:main` from moving `main` under another worktree

## Purpose

The `git:main` mise task (`mise.toml`) fast-forwards local `main` with
`git fetch origin main:main`. With the local git (2.34.1) that command only
refuses when the *current* worktree has `main` checked out. The merger agent
runs the task from `~/.herdr/worktrees/riffle/...` worktrees, so
`refs/heads/main` advances under the primary worktree that has `main` checked
out, leaving its index and working tree stale (staged reverse diffs, and
`git pull` fails with "Your index contains uncommitted changes"). The task's
own description and `.claude/agents/merger.md` already promise "unless another
worktree holds it"; this makes the task actually honour that promise.

## Steps

- [x] Step 1: Skip the `main` fast-forward when any worktree has `main` checked out
  - Done when:
    - Running `mise run git:main` from worktree B while worktree A has `main`
      checked out leaves `refs/heads/main` unchanged and prints the existing
      notice (`Notice: local main was not moved ...`) to stderr; HEAD still ends
      detached at `origin/main`
    - With no worktree holding `main` (including the case where the current
      worktree had `main` and was detached by the task itself), `main` is
      fast-forwarded as before
    - `mise run ci` passes
  - Implementation approach:
    - File: `mise.toml`, `[tasks."git:main"]` `run` body only. Keep
      `set -euo pipefail`, the initial `git fetch origin main`, the self-detach,
      and the final `git checkout --detach origin/main` untouched.
    - After the self-detach block, check
      `git worktree list --porcelain | grep -qx 'branch refs/heads/main'`
      (porcelain output has one `branch refs/heads/<name>` line per worktree).
      If it matches, print the existing notice and skip the fetch; otherwise run
      `git fetch origin main:main || echo "<same notice>"` as today (the `||`
      fallback still covers the not-a-fast-forward case). Use one `if/else` so
      the notice text stays a single string (or hoist it into a variable).
    - The check must run *after* the self-detach so a worktree that had `main`
      checked out itself does not block its own fast-forward.
    - Verify manually before opening the PR: from a secondary worktree with the
      primary on `main`, run the task and confirm `git rev-parse main` is
      unchanged and the notice appears; then with no worktree on `main`, confirm
      `main` moves.
    - Docs: `.claude/agents/merger.md` and the task `description` already
      describe this behaviour accurately; no change needed.
    - Commit: `fix(mise): skip fast-forwarding main when any worktree has it checked out`

## Trade-offs and risks

- Detection via `git worktree list --porcelain` is stable and line-exact
  (`grep -x`), unlike the plain output.
- Stale worktree entries with `main` checked out also cause the skip. This is a
  safe failure mode (main simply is not moved; the notice says why), and
  `git worktree prune` resolves it; not handled in the task.
- Alternative not taken: `git -C <primary> pull --ff-only` to update the
  primary's checkout too. Rejected as it writes to another worktree's working
  tree and index, which is exactly the class of side effect the fix removes.

## Progress

- (2026-09-22) Step 1 complete
