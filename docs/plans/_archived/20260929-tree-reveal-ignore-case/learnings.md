# Learnings

## Step 1

- `respell` has to be applied level by level after each listing, not once:
  `rebase` joins the remaining segments in the caller's spelling, so only
  the level just matched against a listing takes the listing's case; the
  deeper ones are corrected when their own parent is listed.
- In `reveal` the `for...of` became an index loop with `chain !== null` in
  its condition, so TypeScript narrows `chain` inside the body while it is
  reassigned there.
- A worktree-isolated agent cannot run a Python heredoc through Bash that
  edits files (the command is refused as unverifiable); use the Edit tool.

## Deferred issues (todo candidates)

- `relation`, `rebase` and `renameFolder` in `crates/app/ui/src/tree.ts`
  still compare paths case-sensitively (apart from the drive letter), so a
  rename or watcher event whose path differs in case from the tree's keys on
  macOS / Windows would not match. Basis: the plan's "Scope of
  case-insensitivity" trade-off, left out of this step to keep it to the
  todo item. Files: `crates/app/ui/src/tree.ts`,
  `crates/app/ui/src/folders.ts`.
