# Learnings: trash-from-strip

## Step 1: the `trash_files` command

- `groups_of` checks that each path's parent is `dir` (canonicalized) and
  reports any other as a `Failure` ("not in the folder") without moving it,
  so a sidecar of `dir` never goes to the Trash with a same-stem file of
  another folder (review round 1).
- `trash_files` does not call `state.stop_embedding()`: like
  `trash_rejected_run`, it moves no index rows; the third pass reads each
  file by path and skips one that vanished, and the next rescan's reconcile
  drops the gone rows.
- `read_listing` returns `Err` for an unreadable folder, which `trash_files`
  passes on as the command's error (nothing moved). A folder with RAWs and
  JPEGs lists only the RAWs, so a path list is never mixed in practice;
  `groups_of` still decides per path (`is_raw_file`), so a JPEG never takes
  the `.xmp` of a same-stem RAW.
- `trash::restore` counted only RAWs as the "primary" files of a run:
  restoring a run of JPEGs (Decision 1, view-only folders) would have
  brought the files back but listed none in `Restored.restored`, so the
  frontend would say `Restored 0 files` and `restoredInto` would not place
  them. `restore` now treats a JPEG like a RAW (`is_jpeg_file`); `redo`
  needed no change, since a JPEG after a JPEG already forms its own group.
  The frontend comment in `crates/app/ui/src/trash.ts` ("`restored` lists
  RAWs alone") is left for Step 2 / 3.

## Step 2: `Move to Trash…` in the strip's menu

- The strip's confirmation rides the existing `TrashFlow`: `TrashTarget`
  gained an optional `paths` (the files of `dirs[0]`), `confirmTrashFiles`
  starts and marks the flow previewed at once (no preview command), and
  `runTrash` hands a target with `paths` to `trashFilesIn` after closing the
  dialog. The dialog's title (`#trash-title`) is now set per flow (`Move
  Rejected to Trash` / `Move to Trash`); Step 3 can fix it to the latter.
- Order of the idle gate: the dialog asks first, and the confirm goes
  through `whenIdle("Move to Trash", …)`, so a confirm during a scan waits
  with the status line saying so. Inside the held run, a changed
  `openDir` / `folderToken` drops the move before invoking. After the
  invoke, `trashed` runs regardless of the token (unlike the plan's "return
  early" note): the backend already recorded the run, so skipping would lose
  its undo entry; `trashed` refreshes the strip only when the open folder is
  the run's folder.
- In Compare the menu targets the active compared file alone, mirroring
  `judge`'s `comparing ? [current] : targets(...)`.
- The `trash.ts` comments that said `restored` lists RAWs alone now say RAWs
  and JPEGs (Step 1 made `restore` count JPEGs).

## Step 3: removing the folder tree's Move Rejected to Trash family

- `index.rs` `row_flags` / `RowFlag` are kept (Decision 4) but only tests
  call them once `collect_rejected` is gone, so `cargo build` warned them
  dead; both are now `#[cfg(test)]`.
- `Summary.unread` was only ever filled by the reject collection
  (`collect_rejected`'s unreadable folders and sidecars); `trash_files`,
  `trash_rejected_redo` and `delete_sidecars_run` never set it. It was
  removed from the Rust `Summary`, the `TrashSummary` type and `trashed`,
  together with `failureText`, whose only use was those rows.
- Nothing passes `recursive: true` any more, so it was dropped from
  `TrashEntry`, `trashed` and `opensTarget` (which now checks "same folder"
  only), as the plan allowed.
- `TrashFlow` lost its `previewing` phase (`idle` -> `asking` -> `running`)
  and its `isOpen` getter, which equalled `busy` once nothing is fetched
  before the dialog shows; `TrashTarget` is `{ dir, paths }`.
- The folder tree's right-click now always selects the clicked folder alone
  (`folders.ts` `menuTarget`), and the `targets` argument of the tree's
  context menu callback is gone (Decision 6).
- The removed `Move Rejected to Trash` bullet of `usage.md` / `usage.ja.md`
  carried the undo details (nothing overwritten, works with any folder
  open, redo without the dialog); they moved into the `Move to Trash…`
  bullet rather than being lost.
- `todo.md`: the macOS / Linux checks section was retitled to
  `Move to Trash` and its checks rewritten for the strip (the mover, "Put
  Back" and the undo collision still apply); the tree-only checks (the
  tree trash items, the per-folder dialog, the multi-folder pair, the
  un-reject-while-waiting preview check) were dropped; the wait-for-scan
  section was retitled to `Move to Trash and Rename` and
  `.claude/agents/todo-curator.md`'s quote of it follows.
- Python on Windows writes CRLF in text mode; open files with
  `newline=''` when scripting edits, or the working copy flips to CRLF.

## Deferred issues (todo candidates)

- **Pending manual check (user, Windows, `mise run tauri:dev`)** for Step 2
  of `docs/plans/20261013-trash-from-strip/plan.md` (the step was ticked on
  the automated criteria: `vp test` and `mise run ci`). With a copy of a
  folder under `D:\Photos\tests\<date>-trash-from-strip\`, verify:
  1. Right-click one file -> `Move to Trash…` -> dialog says `Move 1 file
     to the Trash?`; `Move to Trash` moves the RAW and its `.xmp` / `.dop`
     to the Recycle Bin, the strip shows the neighbour, the status line says
     `Moved 1 file to the Trash`.
  2. Select 3 files, right-click one of them -> the item reads `Move 3
     Files to Trash…`; confirming moves exactly those 3.
  3. `Cancel` and `Escape` leave everything untouched.
  4. Filter to rejects -> `Ctrl+A` -> `Move N Files to Trash…` moves only
     the filtered files.
  5. `Ctrl+Z` brings them back with their judgments; `Ctrl+Shift+Z` moves
     them again.
  6. Confirming during a scan waits (status line says so) and moves at the
     scan's end.
  7. In a JPEG-only folder the menu is `Select All` + `Move to Trash…`, and
     a JPEG moves alone; undo restores it (`Restored 1 file from the Trash`).
  Related files: `crates/app/ui/src/main.ts` (`confirmTrashFiles`,
  `trashFilesIn`), `crates/app/ui/src/context.ts`.
