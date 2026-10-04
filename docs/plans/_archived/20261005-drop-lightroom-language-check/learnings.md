# Learnings

## Step 1

- The section sat at `todo.md` lines 1215-1228 exactly as the plan said; deleting that range (heading through the trailing blank line) left the Step 3 and Step 5 sections separated by one blank line with no other edits. Nothing else referenced the heading.
- A fresh worktree has no `node_modules`, so `mise run fmt` failed with `Cannot find module ...\node_modules\vite-plus\bin\vp`. `pnpm` is not on the Git Bash PATH; `mise exec -- pnpm install --frozen-lockfile` fixed it.

## User intervention

- While the PR was open, #677 (`lightroom-label-presets-all-languages`) added two real-device checks for the 16-language preset dropdowns to the section this plan deletes. The conflict resolution kept the deletion and dropped them too. The user chose to keep those two checks, so they were re-added under their own heading ("App: real-device check of the 16-language Lightroom label preset dropdowns").
