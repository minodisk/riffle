# Learnings

## Step 1

- The section sat at lines 590–606 of `todo.md` on 2bc9c39b, the same range as
  on 971a9b96; deleting the heading through the blank line before the next
  `###` heading left exactly one blank line between the neighbours.
- A fresh worktree has no `node_modules`, so `mise run fmt` failed on the
  missing `vite-plus` bin; `mise exec -- pnpm install --frozen-lockfile` (pnpm
  is not on the plain Git Bash PATH) fixed it.
