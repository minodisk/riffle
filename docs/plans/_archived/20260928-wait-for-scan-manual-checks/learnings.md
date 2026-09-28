# Learnings

## Step 1

- Every code reference in the new todo item was checked against the current
  tree before writing: the status text is built in `renderMeta`
  (`crates/app/ui/src/main.ts`) as
  `` `${idle.waiting}: waiting for the scan to finish` ``, `idle.drain()` runs
  on both `faces-done` and the scan's error path, `idle.discard()` runs in
  `openDirectory`, and `SCAN_RUNNING` still guards `clear_index`,
  `trash_rejected_preview` and `trash_rejected_run` in
  `crates/app/src/commands.rs` and `rename_folder` / `rename_file` in
  `crates/app/src/rename.rs`.
- The archived plan path is written in backticks rather than as a link, since
  `todo.md` sits at the repository root and a plain path reads fine there.

## Deferred issues (todo candidates)

- (none)
