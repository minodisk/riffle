# Learnings

## Step 1

- `herdr worktree list` (read-only) confirmed `.result.source.source_checkout_path`
  on herdr 0.9.1; it also returns `source_workspace_id` and per-worktree
  `open_workspace_id`. The shape of `herdr worktree create`'s output was not
  re-verified (running it would create a worktree), so the skill tells the
  model to read field names from the actual response, including the workspace
  ID, whose exact path the plan did not pin down.
- `--help` of `herdr worktree create`, `herdr agent start`, and
  `herdr agent prompt` matched the flags recorded in the plan.
- In a fresh worktree the first `mise run fmt` failed with `Command "vp" not
  found` (the frontend dependencies were not installed yet); `mise run ci`
  installs them, and `mise run fmt` passed on re-run.
