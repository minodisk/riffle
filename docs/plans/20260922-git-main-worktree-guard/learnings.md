# Learnings

## Step 1

- Verified in a throwaway origin + clone + extra worktree: with the other
  worktree on `main`, the task leaves `main` unchanged, prints the notice and
  detaches at `origin/main`; with no worktree on `main` (including the
  self-detach case) `main` fast-forwards.
- The task body can be extracted for testing with
  `sed -n '/tasks."git:main"/,/^"""$/p' mise.toml` (the system Python lacks
  `tomllib`).
