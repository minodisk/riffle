# Learnings

## Step 1

- Docs only. The note sits right after the "A sidecar created by other software
  is edited in place" paragraph in both READMEs, since the cause is the sidecar
  naming and it covers both Rename… and Move Rejected to Trash.
- The todo.md section was removed whole (heading through its TODO list); the
  surrounding sections were left untouched.
- In a fresh worktree `mise run fmt` fails with `Cannot find module
  .../node_modules/vite-plus/bin/vp`; `pnpm` is not on the Bash PATH, so run
  `mise exec -- pnpm install --frozen-lockfile` first.
