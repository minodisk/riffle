# Learnings

## Step 1

- The section sat at `todo.md` lines 1215-1228 exactly as the plan said; deleting that range (heading through the trailing blank line) left the Step 3 and Step 5 sections separated by one blank line with no other edits. Nothing else referenced the heading.
- A fresh worktree has no `node_modules`, so `mise run fmt` failed with `Cannot find module ...\node_modules\vite-plus\bin\vp`. `pnpm` is not on the Git Bash PATH; `mise exec -- pnpm install --frozen-lockfile` fixed it.
