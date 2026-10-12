# Learnings: trash-from-strip

## Step 1: the `trash_files` command

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
