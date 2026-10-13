<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Move to Trash from the filmstrip, replacing Move Rejected to Trash

## Purpose

Deleting is done where the culling happens: the filmstrip's right-click menu
gets a `Move to Trash` item that moves the right-clicked file, or every
selected file when the right-clicked one is among a multi-selection, to the
OS Trash together with its sidecars (`.xmp` and `.dop`, whichever exist),
after a count-only confirmation. The strip already has multi-selection
(`Cmd`/`Ctrl+click`, `Shift+click`, `Shift+Arrow`) and `Select All`
(`CmdOrCtrl+A`, `Edit > Select All`, the strip menu), and `Select All` selects
exactly the files that pass the strip's filter (`crates/app/ui/src/main.ts`
`selectAllFiles` takes `files`, the filtered list). So "filter to rejects,
select all, Move to Trash" does what the folder tree's `Move Rejected to
Trash…` family did, with one mechanism instead of two. That family (the folder
tree's single- and multi-folder items, the `trash_rejected_preview` /
`trash_rejected_run` commands, the preview flow and the reject collection
behind them) is removed. The `Runs` record and `trash_rejected_undo` /
`trash_rejected_redo` stay: `Delete Sidecars…` runs on them
(`crates/app/src/foldersidecars.rs`), and the new delete is undone and redone
through them too. The `#trash-dialog` markup stays, reused as the new
delete's confirmation.

Investigation notes (what exists already, so nothing new is needed for it):

- `crates/app/ui/src/selection.ts` `targets(selection, files, focused)`
  returns the selected files in strip order when the focused file is part of
  a multi-selection, else the focused file alone. The strip's `contextmenu`
  handler (`crates/app/ui/src/strip.ts`, cell listener) hands the cell index
  to `main.ts` `openContextMenu`, which reads `files[index]` as the focused
  file.
- `main.ts` `trashed(summary, dirs, recursive, what, rescan)` prunes the
  open folder's state for the moved RAWs, pushes a `kind: "trash"` entry onto
  the undo history with the backend `run_id`, reports `failed` / `unread`
  into the meta pane's errors, sets the status line and calls
  `resync("trash")`. `resync` passes its pre-listing `files` as `previous`
  to `refilter`, whose `anchorAfterFilter` moves the focus to the deleted
  file's neighbour. The index rows of the gone files are dropped by that
  rescan's reconcile, as they are for a file deleted outside Riffle.
- `crates/app/src/trash.rs`: `Group`, `run`, `Trashed`, `TrashRun`, `Runs`,
  `restore`, `redo`, `restore_recorded`, `create_missing`, `remove_created`,
  `relocate`, `trash_file`, `trash_key`, `newest_by_path`, `sidecars_of` and
  the constants are shared with `foldersidecars.rs` and the undo / redo
  commands and stay. `collect`, `visit`, `collect_folder`, `flag_of`,
  `read_flag`, `subfolders`, `Collection`, `FolderPlan`, `preview`,
  `Preview`, `FolderCount`, `bytes`, `nothing_to_trash` serve only the
  removed feature. `index.rs` `row_flags` / `RowFlag` serve only
  `collect_rejected` and two `foldersidecars.rs` tests.
- `crates/app/src/mcp.rs` has no trash tool; its `INSTRUCTIONS` and the
  docs' "Nothing is deleted" MCP lines stay true.

## Decisions

1. `Move to Trash` is offered in a view-only (JPEG) folder too (user's
   choice): its menu becomes `Select All` + `Move to Trash`; a JPEG moves
   alone, without sidecars.
2. A count-only in-window confirmation dialog, reusing `#trash-dialog`
   (`Move N files to the Trash?`, `Move to Trash` / `Cancel`, Escape) (user's
   choice). No size, no preview command.
3. No keyboard shortcut; right-click only.
4. `index.rs::row_flags` / `RowFlag` are kept (two `foldersidecars.rs` tests
   use them).
5. `trash_rejected_undo` / `trash_rejected_redo` keep their names.
6. The folder tree's multi-folder selection mechanism stays; a right-click on
   a multi-selection selects the right-clicked folder alone and shows its
   single-folder menu.

## Steps

- [x] Step 1: Add the `trash_files` command (backend)
  - Done when: a `trash_files` Tauri command moves the given files of one
    folder and the sidecars that exist for each to the OS Trash through the
    existing mover, records the run in `trash::Runs` so
    `trash_rejected_undo` / `trash_rejected_redo` restore and redo it
    unchanged, and returns `trash::Summary`; the grouping is covered by a
    Rust unit test; `mise run ci` passes. No frontend change in this step.
  - Implementation approach:
    - `crates/app/src/commands.rs`: `pub async fn trash_files(app, dir: String,
      paths: Vec<String>) -> Result<trash::Summary, String>`, registered in
      `crates/app/src/main.rs` next to the trash commands. Guards exactly as
      `trash_rejected_run`: `spawn_blocking`, drain the sidecar writer
      (`AppWriter.flush(DRAIN_TIMEOUT)`), take the `Scans` lock, refuse with
      `SCAN_RUNNING` while scanning, hold the lock across the moves. Check
      whether `state.stop_embedding()` (as `rename_file` in
      `crates/app/src/rename.rs` does) is needed; the third pass reads files
      by path and a vanished file is skipped there, so probably not.
    - Canonicalize `dir` with `commands::canonicalize` as `collect_rejected`
      did. `paths` are the strings `list_arw` / `folder_entries` hand the
      frontend, i.e. already the canonical spelling.
    - Sidecars: `read_listing(&dir, Some(SidecarFormat::Both))` +
      `stat_sidecars(&listing.sidecars)`, then per path
      `trash::sidecars_of(SidecarFormat::Both, path, &stats)` for a RAW
      (`riffle_core::scan::is_raw_file`), the same rule `collect_folder`
      used ("both `.xmp` and `.dop` when both are there, whatever format is
      selected"); a JPEG moves alone (Riffle never reads or writes a sidecar
      next to a JPEG). Put the pure grouping in `trash.rs` (e.g.
      `pub fn groups_of(dir: &Path, paths: &[String], sidecars: &HashMap<String, SidecarStat>) -> (Vec<Group>, Vec<Failure>)`,
      a path outside `dir` coming back as a `Failure` instead of a group)
      so it is unit-tested like `collect_folder`'s tests were; keep the path
      order the frontend gives (strip order).
    - `trash::run(groups, trash_one)`, `summary.run_id = Runs::record(moved)`,
      `summary.unread` stays empty, `log::info!` the summary as
      `trash_rejected_run` does. A path no longer on disk fails in the mover
      and lands in `failed`; no pre-check.
    - Tests: `trash.rs` grouping (a RAW with both sidecars, a RAW with one, a
      RAW with none, a JPEG with a stray `.xmp` that is not taken), in the
      style of `a_reject_is_grouped_with_the_sidecars_of_both_formats`.
- [x] Step 2: Add `Move to Trash` to the filmstrip's right-click menu (frontend)
  - Done when: right-clicking a strip cell shows a `Move to Trash…` item
    (`Move N Files to Trash…` when the right-clicked file is in a
    multi-selection of N) as the last group, after `Rename…`, in a view-only
    (JPEG) folder too; choosing it opens the count-only confirmation, and
    confirming moves exactly `targets(selection, files, index)` through
    `trash_files`, which after `Select All` under a filter is exactly the
    files passing the filter; afterwards the strip shows the next file (the
    deleted file's neighbour), the selection holds no gone path, the status
    line says `Moved N files to the Trash`, failures are listed per file, and
    one `Edit > Undo` restores the run (`Edit > Redo` moves it again); pressed
    during a scan it waits in the idle gate with the status line saying so;
    `vp test` covers the menu item, its label and the view-only menu;
    `mise run ci` passes; README.md / README.ja.md and `docs/humans/usage.md`
    / `docs/humans/usage.ja.md` describe the item (in this step add, do not
    remove).
  - Implementation approach:
    - Requires Step 1 merged.
    - `crates/app/ui/src/context.ts`: a new last section `[["trashFiles", …]]`
      in `SECTIONS`; `contextMenuGroups` takes the target count (e.g. a
      `count` argument, default 1) and labels the item `Move to Trash…` for 1,
      `Move N Files to Trash…` for N > 1. The `viewOnly` slice keeps
      `Select All` and `Move to Trash…`. Tests in `context.test.ts`
      (`contextMenuGroups` describe block).
    - `main.ts` `openContextMenu`: compute `targets(selection, files, index)`
      once, pass its length to `contextMenuGroups`, and on `trashFiles` open
      the confirmation for those paths (captured at click time).
    - Confirmation: reuse `#trash-dialog` / `TrashFlow` / `trashKeydown` with a
      count-only total line (`Move N files to the Trash?`, `Move 1 file to the
      Trash?`) and the existing `Move to Trash` / `Cancel` buttons, no
      preview command, no per-folder rows. Confirm calls a new
      `trashFilesIn(paths)` that mirrors `trashRejectedIn`:
      `whenIdle("Move to Trash", …)` (hold while a scan runs), bail when
      `openDir` is null or `sidecarFlow` / `sequenceFlow` is busy as the other
      flows do, `invoke<TrashSummary>("trash_files", { dir: openDir, paths })`,
      then `trashed(summary, [dir], false)`; wrap the invoke in `settleIdle`
      as the renames do, so a rescan queues behind it. Guard the result with
      the folder token (`dir !== openDir || token !== folderToken` returns
      early) as `renameFile` does.
    - `undoTrash` / `redoTrash` label strings: `"Move Rejected to Trash"` ->
      `"Move to Trash"` (the idle-gate status text).
    - Verify by hand on Windows (`mise run tauri:dev`) with a copy under
      `D:\Photos\tests\<date>-trash-from-strip\`: single delete, a
      multi-selection delete, Cancel does nothing, filter to rejects ->
      `Ctrl+A` -> delete moves only the filtered files, `Ctrl+Z` brings them
      back with judgments, `Ctrl+Shift+Z` moves them again, the focus lands on
      the neighbour, and a JPEG-only folder deletes too.
    - Docs: README.md "Filmstrip" bullet and README.ja.md; `usage.md`
      Filmstrip / right-click menu paragraph (including the view-only menu)
      and a `Move to Trash` feature bullet (what moves, sidecars, the
      confirmation, undo / redo, scan wait), and `usage.ja.md`.
- [x] Step 3: Remove the folder tree's Move Rejected to Trash family and update the docs
  - Done when: the folder tree's menu no longer offers `Move Rejected to
    Trash…`, `… Including Subfolders…` or the `Move Rejected in N Folders…`
    pair; `trash_rejected_preview`, `trash_rejected_run`, `collect_rejected`
    and the reject collection in `trash.rs` are gone with their tests; the
    preview flow (per-folder rows, size, the preview command) is gone while
    `#trash-dialog` stays for Step 2's confirmation; a right-click on a
    multi-folder selection shows the right-clicked folder's single-folder
    menu; no orphaned helper, type, test or import remains; CLAUDE.md,
    README.md / README.ja.md, `docs/humans/usage.md` / `usage.ja.md`,
    `docs/agents/*.md` lines that describe the removed commands as current
    code, and the `todo.md` sections that are only manual checks of the
    removed feature are updated; `cargo build` warns of no dead code;
    `mise run ci` passes.
  - Implementation approach:
    - Requires Step 2 merged (so the app never lacks a way to trash).
    - Rust: `commands.rs` drop `trash_rejected_preview`, `trash_rejected_run`,
      `collect_rejected` (keep `trash_rejected_undo`, `trash_rejected_redo`,
      `restore_run`, `trash_one`, `canonicalize`, `format_bytes` /
      `SIZE_BASE` if `foldersidecars.rs` still uses them — check);
      `main.rs` registration; `trash.rs` drop `collect`, `visit`,
      `collect_folder`, `flag_of`, `read_flag`, `subfolders`, `Collection`,
      `FolderPlan`, `preview`, `Preview`, `FolderCount`, `bytes`,
      `nothing_to_trash`, `MAX_SIDECAR_BYTES` if unused, the
      `RowFlag` / `SidecarFormat` imports as they fall out, and their tests
      (`index_rows_alone_give_the_rejects` through
      `nothing_to_trash_names_the_folder_or_counts_them`); update the
      module doc comments that name `Move Rejected to Trash` (`trash.rs`
      head, `foldersidecars.rs`, `index.rs` `clear_judgments` /
      `row_flags` docs, `rename.rs`). Keep `index.rs` `row_flags` /
      `RowFlag` (Decision 4).
    - Frontend: `context.ts` drop the two trash groups and the `count > 1`
      branch of `folderMenuGroups` (Decision 6); `context.test.ts`
      accordingly; `main.ts` drop `trashRejectedIn`, `showTrashPreview`, the
      preview parts of the dialog flow, the `trashRejected` /
      `trashRejectedTree` cases, and make a tree right-click on a
      multi-selection select the clicked folder alone; `trash.ts` drop
      `TrashPreview`, `TrashFolder`, `folderRows`, `emptyFoldersLine`, the
      size parts of `totalLine`, `TrashTarget` and whatever else only the
      preview used (keep `TrashSummary`, `TrashRestored`, `TrashFailure`,
      `trashedStatus`, `restoredStatus`, `restoredCount`, `restoredInto`,
      `shownPath`, `stillHeld`, `opensTarget`, the confirmation's flow;
      simplify `opensTarget` / `TrashEntry.recursive` only if nothing passes
      `recursive: true` any more and the simplification is small);
      `trash.test.ts` / `undo.test.ts` accordingly; `undo.ts` comments;
      `index.html` drop the preview-only parts of `#trash-dialog`;
      `docs/agents/ui-styling.md` rows naming `#trash-dialog` updated.
    - Docs: README.md folder tree bullet (drop "move its rejects to the
      Trash" and the multi-folder sentence) and the whole `Move Rejected to
      Trash` bullet; README.ja.md likewise; `usage.md` folder tree menu order
      and the multi-selection paragraph, the feature bullet, Delete Sidecars'
      "Like Move Rejected to Trash…", Undo / Redo, the keys table, and
      `usage.ja.md` likewise; CLAUDE.md's `src/trash.rs` description (now:
      the `trash_files` command, the `Runs` record and undo / redo) and
      `src/idle.ts`'s "(Move Rejected to Trash, the renames)";
      `docs/agents/tauri-app.md` ("the same pattern `trash_rejected_run`
      uses" -> `trash_files`) and other lines that present the removed
      commands as the current pattern; historical "Source:" lines stay.
      `todo.md`: drop the sections that are only manual checks of the removed
      feature and reword the ones that stay (Delete Sidecars' undo note, the
      rename-then-undo check, which still applies to `trash_files` runs);
      `.claude/agents/todo-curator.md` quotes a section title — keep it
      consistent.

## Risks

- `trashed()` keys its "is the open folder affected" check on
  `opensTarget(openDir, dirs, recursive)`; passing `[openDir]` makes it true
  by construction. Keep the canonical-spelling caveat in mind (`openDir` may
  lack the `\\?\` prefix the backend uses; `opensTarget` already handles
  that through `relation` / `shownPath`).
- The idle gate holds one operation: a delete pressed during a scan replaces
  a held rename, as today's trash did. Acceptable and already documented.
- A delete of a file whose sidecar judgment is still inside the writer's
  debounce window: the backend drains the writer first, as
  `trash_rejected_run` did, so no orphan sidecar is minted after the move.
- Undo after a folder rename: `Runs::rename_dir` and `mapTrashDirs` keep
  working unchanged since the new runs are recorded the same way.
- The docs pairs (README / README.ja, usage / usage.ja) must change in the
  same PR; Step 2 adds, Step 3 removes, each in both languages.

## Progress

- (2026-10-13) Step 1 complete
- (2026-10-13) Step 2 complete
