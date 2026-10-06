# Learnings: rename in-flight guard

## Step 1

- `RenamesInFlight` is reused for the strip's files with `ignoreCase =
  false`: the path checked is the very `files[index]` string `renameFile`
  passed to `renameStarted`, so no case folding is needed, and `relation`'s
  `/`-separated prefix test keeps a sibling such as `DSC0001.ARW.bak` out.
- `holdRename`'s `mark` is a `let` captured by both the run and the cancel
  closures. When the gate is idle, `whenIdle` runs `run` synchronously before
  `markPending` is reached, so the clear handed to `run` sees `mark === null`
  and is a no-op, which is right: nothing was drawn pending.
- The folder message moved into a `RENAMING` constant in `folders.ts`, shared
  by `openFolder` and `startRename`; the strip's note is its own string
  (`This file is being renamed; try again in a moment.`), routed through a new
  last `onError` parameter of `strip.init` that `main.ts` fills with
  `setStatus`.
- The worktree had no `node_modules`, so `pnpm exec vp test` could not run
  standalone before `mise run ci`.

## Deferred issues (todo candidates)

- Pending manual check (GUI, macOS and Windows): confirm a folder rename in
  the tree and a file rename in the strip while a scan runs, let the scan
  end, and while `rename_folder` / `rename_file` is still running (a large
  folder or a slow drive) try `Rename…` and the slow second click on the same
  row / cell. Expected: the editor does not open and the status line shows
  `A folder is being renamed; try again in a moment.` /
  `This file is being renamed; try again in a moment.`; once the rename
  settles, editing works again. Also re-check that a rename still held behind
  the scan can be re-edited (replace, cancel, keep) as before, in both views.
  Basis: plan Step 1 Done-when (the GUI check recorded at wrap-up). Files:
  `crates/app/ui/src/folders.ts`, `crates/app/ui/src/strip.ts`,
  `crates/app/ui/src/main.ts`. Step 1's checkbox was ticked on the automated
  criteria (Vitest and `mise run ci`).
