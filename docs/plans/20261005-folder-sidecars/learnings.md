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

## Deferred issues (todo candidates)

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
