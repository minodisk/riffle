# Learnings: folder-sidecars

## Step 1

- The pure core of the rewrite is `foldersidecars::rewrite(dir, index, writer,
  format, names, bound)`, which takes the `Index` and a real `Writer` rather
  than the `AppHandle`, so the Rust tests run the actual writer thread over a
  temp folder (collecting `on_error` into a `Mutex<Vec<String>>` and asserting
  it is empty, per the `tauri-app.md` drain entry). The commands only add the
  guards (`Scans`, the first `DRAIN_TIMEOUT` flush, `canonicalize`).
- Failure count: chosen as "rows of the rewritten paths still dirty after the
  drain" (`dirty_rows(dir)` filtered by the paths marked). It needs no change
  to `Writer::flush` / `on_error`. It over-counts a row a judgment re-dirtied
  during the drain; from the UI that cannot happen (the dialog takes every
  key while it runs), but an MCP `set_rating` could. When the drain times out
  (`REWRITE_DRAIN`, 10 minutes) the rows still queued count as failed too,
  as the plan's Done-when says; `flushed` is still sent to the frontend, which
  does not phrase it differently.
- `Index::mark_dirty` returns only the rows of the given paths, not every
  dirty row of the folder: a dirty row whose RAW is no longer listed would
  otherwise be handed to the writer and mint a sidecar next to a missing RAW.
- The listing is read with `read_listing(dir, None)`, not `Some(format)`: the
  rewrite needs only the RAW list, not the sidecar names.
- "Judged" for the dialog's counts is `rating > 0 || flag != None || label`;
  a row with `rating = 0` (parsed from a sidecar holding `Rating 0`) counts
  as unjudged. Such a row is still handed to the writer as stored, so the
  `mark_written` guard (`rating IS ?`) matches.
- `baseName` in `main.ts` returns `""` for a root such as `C:\`; the dialog
  and status fall back to the full path there (`baseName(dir) || dir`), as the
  backend's `folder_name` does.
- The oversize-sidecar rule of `reconcile_sidecars_of` (a sidecar over
  `MAX_SIDECAR_BYTES` is never handed to the writer) is not applied by the
  rewrite: the user asked for the index to be pushed out. See the deferred
  issue below.

## Step 2

- `trash::redo`, read in the plan as "a leading non-RAW path is its own
  group", actually appended every later non-RAW path to that first group as
  its "sidecars": a redo of a sidecar-only run would have reported one moved
  file and, if the first sidecar was gone, skipped all the others. Fixed by
  letting a non-RAW path join the previous group only when that group's head
  is a RAW (existing rejected-trash runs group exactly as before). Covered by
  `a_redo_of_sidecars_alone_moves_each_on_its_own`; `trash::restore` needed
  no change (`raw_back` starts `true`), covered by
  `a_run_of_sidecars_alone_is_restored_whole`.
- The delete run reuses `trash::run` with one `Group` per sidecar (so the
  macOS `trashed_id` is taken as for a reject run), then rewrites each
  failure's path to its RAW's (`<sidecar name>: <message>`), so the meta
  pane's error list keys it by the file shown. The command returns the
  existing `trash::Summary` (`moved` lists the sidecars, `unread` is empty),
  so the frontend reuses `trashed()` and its `TrashSummary` type.
- `Index::clear_judgments` also sets `dirty = 0`, which the plan's UPDATE did
  not list: the writer is drained first, so a row still dirty is one whose
  write failed, and replaying it on the next open would mint the deleted
  judgment again.
- Frontend: `TrashEntry` gained an optional `what: "sidecars"`, kept by
  `mapTrashDirs` (spread). `undoneTrash` now takes the whole `TrashRestored`
  and counts through `trash.ts`'s `restoredCount` (`count - failed.length`
  for sidecars, since `restored` lists RAWs only). `trashed()` takes `what`
  and `rescan`: the delete run clears the strip and re-reads
  `folder_entries` (the backend cleared the rows), while a redo of a sidecar
  run clears the strip and resyncs, because `trash_rejected_redo` does not
  clear the index (it cannot tell which RAW a sidecar path belongs to); the
  resync's reconcile clears the rows (clean row, gone sidecar).
- The run button's class is set per kind (`button primary` / `button
  destructive`) in the shared `showSidecarDialog`, which the rewrite now goes
  through too.

## Deferred issues (todo candidates)

- **A redo of `Delete Sidecars…` leaves a closed folder's rows judged.**
  `trash_rejected_redo` (`crates/app/src/commands.rs`) moves the sidecars
  again but writes no index row; when the target folder is open the
  frontend's resync clears them, but when another folder is open the rows
  keep the judgments the undo's rescan restored until the folder is next
  opened (whose reconcile clears them). A `Rewrite Sidecars from Index…` in
  between would mint them again. Fixing it needs the run to remember each
  sidecar's RAW (e.g. in `trash::Trashed`). Basis: Step 2 implementation
  (`crates/app/src/foldersidecars.rs`, `crates/app/ui/src/main.ts` `trashed`).
- **Pending manual check (Step 2, any platform).** Not verified on a real
  app: right-click a single folder holding RAWs with sidecars of the current
  format, choose `Delete Sidecars…`; the dialog is titled `Delete Sidecars`,
  lists `N XMP sidecars (size)` (and `.dop` under "both"), ends with `Move N
  sidecars (size) of <name> to the Trash?`, and has a red `Move to Trash`
  and an outline `Cancel`; running it moves the files to the OS Trash, the
  status line reads `Moved N sidecars to the Trash`, and on the open folder
  the stars, flags and labels disappear at once; `Edit > Undo` restores them
  (`Restored N sidecars from the Trash`, the judgments come back after the
  rescan) and `Edit > Redo` moves them again; a folder with no sidecar of
  the format and a JPEG-only folder show a status-line refusal and no
  dialog; pressed during a scan, the meta pane shows `Delete Sidecars:
  waiting for the scan to finish`. The step's checkbox was ticked on the
  automated criteria (Rust and Vitest tests, `mise run ci`); Step 3 folds
  this into the `todo.md` manual check item.

- **Rewrite patches an oversize sidecar.** `rewrite_sidecars_run`
  (`crates/app/src/foldersidecars.rs`) hands every listed row to the writer,
  including a file whose sidecar is larger than `MAX_SIDECAR_BYTES`
  (`crates/app/src/commands.rs`), which a folder open deliberately keeps away
  from the writer because its contents were never read. Decide whether the
  rewrite should skip those too (and count them in the dialog). Basis: Step 1
  implementation, comparing with `reconcile_sidecars_of`.
- **Pending manual check (Step 1, any platform).** Not verified on a real
  app: right-click a single folder that was opened before, choose
  `Rewrite Sidecars from Index…`; the dialog shows the judged / unjudged /
  skipped rows and `Rewrite the XMP sidecars of N files in <name>?`;
  `Rewrite` disables both buttons and shows the running note, then the status
  line reads `Rewrote N sidecars in <name>`; with several folders selected
  the item is absent; on a never-opened folder and on a JPEG-only folder the
  status line shows the refusal and no dialog opens; pressed during a scan,
  the meta pane shows `Rewrite Sidecars from Index: waiting for the scan to
  finish` and the dialog opens when the scan ends. The step's checkbox was
  ticked on the automated criteria (Rust and Vitest tests, `mise run ci`);
  Step 3 folds this into the `todo.md` manual check item.
