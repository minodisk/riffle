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

# Rewrite or delete a folder's sidecars from the folder tree's context menu

## Purpose

The folder tree's right-click menu can move a folder's rejects to the Trash
but cannot touch its sidecars as a whole. Two cases need that: the sidecars
on disk no longer match what Riffle holds (a write that failed and ran out of
retries, a sidecar another program clobbered, a folder judged under one
sidecar format and now wanted in the other), where the user wants the
index's judgments pushed back out; and a folder whose sidecars should go
away entirely (a re-export, handing the folder to someone else, a mess of
stale `.xmp` / `.dop` files), where deleting them one by one in the file
manager is tedious and loses nothing Riffle cannot bring back.

This adds `Rewrite Sidecars from Index…` and `Delete Sidecars…` to the folder
menu, both behind a confirmation dialog, and closes the `todo.md` section
"App: rewrite or delete a folder's sidecars from the folder tree's context
menu" (deferred from `docs/plans/_archived/20260927-folder-menu-copy/plan.md`).

The decisions taken, each agreed with the user on 2026-10-05 (each has an
alternative under "Trade-offs and risks"):

- **Rewrite patches, it never regenerates.** Every write goes through the
  existing sidecar writer (`crates/app/src/sidecar.rs`), whose `write_kind`
  splices only `xmp:Rating` / `xmpDM:good` / `xmp:Label` /
  `photoshop:LabelColor` (XMP) and `Rating` / `ShouldProcess` / `ColorLabel`
  plus the two timestamps (`.dop`) into an existing file, so Lightroom's
  `crs:` develop settings and PhotoLab's corrections stay byte-for-byte. A
  file with no sidecar gets a fresh one only when its row holds a judgment
  (the writer's existing rule; a fresh `.dop` gets PhotoLab's Uuids and the
  RAW's Orientation as today). A row with no judgment patches an existing
  sidecar to unrated / unflagged / no label and mints nothing. A listed RAW
  with no `ratings` row is skipped (the index knows nothing about it).
- **Delete moves the whole sidecar file to the OS Trash** and records the run
  in `trash::Runs`, so `Edit > Undo` restores it through the existing
  `trash_rejected_undo` and `Edit > Redo` moves it again through
  `trash_rejected_redo`. The judgments of the folder are cleared in the index
  and the strip, as they are when a sidecar disappears from under the app
  (the sidecar is the source of truth); an undo brings them back through the
  normal reconcile parse.
- **Only the current `SidecarFormat`'s kinds** are rewritten or deleted
  (`Both` covers both). The app never reads or writes the other format's
  files when one format is selected, and this keeps that rule.
- **The clicked folder only, one folder at a time.** No subfolder option and
  the items are not offered on a multi-folder selection.
- **Pressed during a scan, both wait for the scan** through the existing
  `whenIdle` hold (`crates/app/ui/src/idle.ts`), the same way `Move Rejected
  to Trash…` does, and the backend refuses with `SCAN_RUNNING` under the
  `Scans` lock as a second line.
- **A folder never opened has no rows**: the rewrite preview reports "No
  judgments indexed for `<name>`; open the folder first" in the status line
  (the `nothing_to_trash` pattern) and does nothing. Delete needs no index
  and works on any folder.
- **Both items confirm** in a dialog, rewrite included: a rewrite overwrites
  judgments other software may have changed since the folder was last opened.

## Steps

- [x] Step 1: `Rewrite Sidecars from Index…` end to end (backend commands, folder menu item, confirmation dialog)
  - Done when:
    - Right-clicking a single folder in the tree offers `Rewrite Sidecars
      from Index…` in a new group after the trash group and before
      `Sequence JPEG Timestamps…` (not offered with several folders
      selected); choosing it opens a dialog that names the folder and says
      how many files will be written: rows with a judgment, rows without one
      (whose existing sidecar is patched to "no judgment"), and how many
      listed RAWs have no index row and are skipped; `Rewrite` runs it,
      `Cancel` / `Escape` closes it, and the buttons are disabled while it
      runs (the sequence dialog's running note pattern).
    - The run marks every `ratings` row of the folder dirty, hands each to
      the sidecar writer with no debounce (`Writer::set_now`, as the folder
      open replays dirty rows), and waits for the writer to drain; each
      write patches the existing sidecar of each current-format kind or
      mints one when the row has a judgment and none exists; a write that
      fails reports through the writer's existing `on_error` path (the
      `sidecar-error` event the meta pane's error list already shows) and
      the row stays dirty for the next folder open to retry. The status line
      says `Rewrote N sidecars in <folder>` (plus `, M failed` when the
      drain reported errors or timed out).
    - A folder with no `ratings` row at all, and a JPEG-only (view-only)
      folder, are refused by the preview with a status-line message and no
      dialog.
    - A running scan: the item waits (`<label>: waiting for the scan to
      finish` in the meta pane, as the trash item does) and the preview runs
      when it ends; the backend refuses `SCAN_RUNNING` if one slipped in.
    - Rewriting the open folder changes nothing in the strip (the index was
      already the truth); the rows' stored sidecar stat is updated by
      `mark_written` so the next open does not re-parse what was just
      written.
    - Rust tests cover: a folder with a judged row and no sidecar mints one;
      a row with a judgment and an existing Lightroom-style XMP (with a
      foreign `crs:` attribute) and a PhotoLab-style `.dop` (with a
      `Settings` block) is patched with the foreign content intact; an
      unjudged row with an existing sidecar clears its judgment fields and
      mints nothing for one without; a listed RAW without a row is skipped;
      a `ratings` row whose RAW is no longer listed is not written (no
      sidecar next to a missing RAW); under `Both` both kinds are written;
      the no-row folder is an `Err`. Frontend tests cover the menu group and
      the dialog's text helpers and flow class. `mise run ci` passes.
  - Implementation approach:
    - New module `crates/app/src/foldersidecars.rs` (registered in
      `main.rs`'s `mod` list and `generate_handler!`) holding the pure parts
      and the two commands `rewrite_sidecars_preview(dir)` /
      `rewrite_sidecars_run(dir)`, shaped on `trash_rejected_preview` /
      `trash_rejected_run` in `crates/app/src/commands.rs`: `async`,
      `spawn_blocking`, flush the writer first (`DRAIN_TIMEOUT`), refuse
      `SCAN_RUNNING` (preview checks `Scans` without holding it, run holds
      the `Scans` lock across the index write and the drain, per
      `docs/agents/tauri-app.md` "An index/scan mutation ... holds the
      `Scans` lock"), `canonicalize(dir)` before touching the index (rows are
      keyed by the canonical spelling).
    - Listing: `read_listing(dir, Some(format))` (pub(crate) in
      `commands.rs`); refuse when `files` holds no RAW (`is_raw_file`), which
      also covers a JPEG-only folder. Intersect the rows with the listed
      RAWs so a stale row never mints a sidecar next to a RAW that is gone.
    - Index: add `Index::rows_of(dir) -> Vec<DirtyRow>`-style read of every
      `ratings` row (`path, rating, flag, label, label_known`) for the
      preview counts, and `Index::mark_dirty(dir, paths) -> Vec<DirtyRow>`
      (one transaction: `UPDATE ratings SET dirty = 1 WHERE dir = ?1 AND
      path IN (...)`, then the same SELECT `dirty_rows` runs) for the run.
      Keep `xmp_size` / `xmp_mtime_ns` as they are: `mark_written` overwrites
      them, and a row whose write fails keeps the stat the next open's
      reconcile compares against, so it is replayed rather than re-parsed.
      Each row travels with its own `label_known` (a row created before the
      label was learned must not strip a sidecar's label; see
      `Index::set_rating`'s doc).
    - Writer: `set_now` per row with the current `AppSidecarFormat` and
      `AppLabelNames`, then `writer.flush(bound)`. `DRAIN_TIMEOUT` (2 s) is
      for quit; a 5000-file folder on Windows needs minutes (each write is a
      read, a temp write, `sync_all` and a rename with retries), so use a
      separate generous bound (a constant in the new module, e.g. 10
      minutes) and report `flushed: false` to the frontend when it elapses.
      Because every write goes through the one writer thread, a keypress on
      the open folder during the run cannot write the same path
      concurrently.
    - Result payload: `{ written: usize, failed: usize, flushed: bool }`.
      `failed` can be counted by passing a counter into the writer only if
      `Writer::spawn`'s `on_error` is extended; simpler is to count the rows
      still dirty after the drain (`dirty_rows(dir).len()`), which is also
      what the next open will retry. Decide in the step; the count must not
      include rows a keypress dirtied during the drain if that is cheap to
      exclude, otherwise document the over-count.
    - Frontend: `crates/app/ui/src/context.ts` `folderMenuGroups` gains the
      group `[rewriteSidecars]` (Step 2 adds `deleteSidecars` to the same
      group) after the trash group; update the doc comment and
      `context.test.ts`. New `crates/app/ui/src/sidecars.ts` with the text
      helpers (`rewriteRows`, `rewriteTotalLine`, `rewrittenStatus`) and a
      `SidecarFlow` class modeled on `TrashFlow` (`idle` / `previewing` /
      `previewed` / `running`, with a `kind: "rewrite" | "delete"` so Step 2
      reuses it), tested in `sidecars.test.ts`. `main.ts` wires
      `rewriteSidecarsIn(path)` through `whenIdle`, gates on
      `formatDialog.hidden`, `settings.isOpen`, `trashFlow.busy`,
      `sequenceFlow.busy` (and the new flow's `busy` must be added to the
      gates those three check: `startSequence`, `trashRejectedIn`,
      `open-settings`, `modalOpen()` at the keydown gate around line 3647),
      shows the dialog, and runs it under `settleIdle` so a focus rescan
      waits (the dialog's close refocuses the window, per the
      `tauri://focus` entry in `docs/agents/tauri-app.md`).
    - Dialog: one new `#sidecar-dialog` in `crates/app/ui/index.html` next to
      `#trash-dialog`, same `dialog` / `dialog-box` / `dialog-title` /
      `dialog-actions` classes, with `#sidecar-title`, `#sidecar-rows`
      (`ul`), `#sidecar-total` (`p`), `#sidecar-running` (`p`, hidden until
      the run), `#sidecar-run` and `#sidecar-cancel`. Per
      `docs/agents/ui-styling.md`, the run button is `button primary` for
      rewrite (the sequence `Run` precedent) and `button destructive` for
      delete (Step 2 swaps the class and label per `kind`); no new colors or
      id rules that set colors. Keys: a `SettingsModal` instance for
      Escape / Tab as `trashKeys` does, dispatched from the keydown handler
      where `trashKeydown` is.
    - Labels and copy: menu item `Rewrite Sidecars from Index…`; dialog title
      `Rewrite Sidecars from Index`; rows like `12 files with a judgment`,
      `3 files without one (their sidecars are cleared)`, `5 files not in
      the index (skipped)`; total `Rewrite the XMP sidecars of 15 files in
      <name>?` (naming the format: `XMP`, `.dop`, or `XMP and .dop`);
      status `Rewrote 15 sidecars in <name>`.

- [ ] Step 2: `Delete Sidecars…` end to end (backend commands, menu item, the shared dialog, undo / redo through the trash runs)
  - Done when:
    - The folder menu's new group holds `Rewrite Sidecars from Index…` then
      `Delete Sidecars…`; choosing the latter opens the shared dialog
      titled `Delete Sidecars`, listing `N XMP sidecars (size)` / `N .dop
      sidecars (size)` lines for the current format's kinds and ending with
      `Move N sidecars (size) of <name> to the Trash?`, with a red
      destructive `Move to Trash` and an outline `Cancel`.
    - The run moves each sidecar to the OS Trash (`trash_one`), records the
      moved files as one `trash::Runs` run, clears the judgments of those
      files in the index, and returns `{ moved, failed, run_id }`; the
      status line says `Moved N sidecars to the Trash` (`, M failed`), and
      each failure joins the meta pane's error list keyed by the RAW's path.
    - When the open folder is the target, the strip drops the stars, flags
      and labels of its files at once (and `touched` forgets them, or the
      next `folder_entries` refresh would keep what the keyboard set this
      session), and `folder_entries` is re-read.
    - `Edit > Undo` restores the sidecars from the Trash (status `Restored N
      sidecars from the Trash`, `, M failed`), and when the open folder is
      the target a rescan re-parses them so the judgments come back;
      `Edit > Redo` moves them again. The entries sit on the same history as
      the trash runs and outlive a folder switch, as those do.
    - A folder with no sidecar of the current format's kinds, and a
      JPEG-only folder, are refused by the preview with a status-line
      message and no dialog. A running scan: waits, then refused by the
      backend if one slipped in, as in Step 1.
    - Rust tests cover: the collection (only sidecars of listed RAWs, only
      the current format's kinds, `Both` takes both, a case-differing
      `FOO.XMP` is included, a sidecar of an unlisted RAW is not); the run
      through an injected mover (a failure leaves the file and is reported,
      the moved list is recorded in order); the index clear; a JPEG-only
      folder is an `Err`. Frontend tests cover the menu group, the delete
      text helpers, the undo / redo entry mapping and the status lines.
      `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged (the module, the dialog and `SidecarFlow`).
    - Backend in `crates/app/src/foldersidecars.rs`:
      `delete_sidecars_preview(dir)` / `delete_sidecars_run(dir)`, same
      guards as Step 1's commands (flush the writer first: a judgment still
      in its debounce window would otherwise re-mint a sidecar right after
      the delete; `Scans` lock held across the moves and the index write).
      Collection: `read_listing(dir, Some(format))` + `stat_sidecars` from
      `commands.rs`, keep only the sidecars `format.kinds()` name for a
      listed RAW (the same lookup `trash::sidecars_of` does; make that
      helper `pub(crate)` or copy its three lines), with sizes for the
      dialog (`format_bytes(bytes, SIZE_BASE)` is private in `commands.rs`;
      make it `pub(crate)`). Mover: `commands::trash_one` (make it
      `pub(crate)`), recording `Trashed { path, trashed_at, trashed_id,
      restore_to: None }` as `trash::run` does; `Runs::record(moved)` gives
      the `run_id`. `trash::restore`'s "a sidecar stays in the Trash when
      its RAW did not come back" rule starts with `raw_back = true`, so a run
      of sidecars only restores cleanly; `trash::redo` groups a leading
      non-RAW path as its own group, so redo moves them again. Both were
      read, not run: cover them with a unit test on a sidecar-only run.
    - Index: add `Index::clear_judgments(dir, paths)` running the same
      UPDATE `reconcile_sidecars` applies to a gone sidecar (`rating = NULL,
      flag = 0, label = NULL, label_known = 1, xmp_size = NULL, xmp_mtime_ns
      = NULL`) for the RAWs whose sidecar was moved, under the `Scans` lock.
      Without it a closed folder's rows would keep the judgments until its
      next open, and a `Rewrite` pressed in between would re-mint them.
      Since the rows then already match "no sidecar", a later rescan's
      `changed` is 0 and the frontend must refresh the strip itself (next
      bullet) instead of relying on `refreshOnScanDone`.
    - Frontend (`main.ts`): `deleteSidecarsIn(path)` through `whenIdle`,
      the shared dialog with `kind: "delete"`, the run under `settleIdle`.
      On success when `openDir` is the folder (`relation(openDir, dir) ===
      "same"` from `tree.ts`, as `opensTarget` uses): for every path in
      `allFiles`, `touched.delete(path)` and `applyRating(path, null,
      "none", null)`, then `refreshEntries()`. Push a history entry for
      undo: reuse `TrashEntry` (`kind: "trash"`, `runId`, `count`, `dirs:
      [dir]`, `recursive: false`) so `step` / `undoTrash` / `redoTrash` work
      unchanged, but give the entry a way to phrase its status as sidecars
      (e.g. an optional `what: "sidecars"` field read by `restoredStatus` /
      `trashedStatus`, with `restored = count - failed.length` since
      `TrashRestored.restored` lists RAWs only), and `redoable.clear()` as
      the trash run does. `restoredInto` adds nothing for sidecar paths
      (they are not RAWs in `allFiles`), and `resync("restore")` re-parses
      them; add a `"sidecars"` value to `RescanTrigger` in `refresh.ts` only
      if a rescan is triggered from the delete itself (it is not, per the
      previous bullet; the watcher's `triggers` drops sidecar-only events,
      so no rescan comes from the deletes either).
    - Dialog copy: title `Delete Sidecars`; rows `12 XMP sidecars (1.2 MB)`
      / `12 .dop sidecars (340 KB)`; total `Move 24 sidecars (1.5 MB) of
      <name> to the Trash?`; run button `Move to Trash` (`button
      destructive`). Refusal messages: `No XMP sidecars in <name>` (naming
      the format) and, for a JPEG folder, the same `raw_only`-style text
      Step 1 uses.

- [ ] Step 3: Documentation, the `todo.md` close and the manual check item
  - Done when:
    - `docs/humans/usage.md` and `docs/humans/usage.ja.md` describe the two
      items where the folder menu is listed (the paragraph after `Move
      Rejected to Trash, Including Subfolders…` and before `Sequence JPEG
      Timestamps…`) and in a new bullet next to **Move Rejected to Trash…**
      covering the dialog, the running wait, the open-folder refresh, the
      undo / redo of a delete, what rewrite preserves and that it needs the
      folder to have been opened once; `README.md` / `README.ja.md` mention
      the items in the folder-tree sentence that lists the right-click
      actions, in sync.
    - `CLAUDE.md`'s Layout paragraph names `crates/app/src/foldersidecars.rs`
      and `crates/app/ui/src/sidecars.ts` in the style of the other entries;
      `docs/agents/tauri-app.md` gets an entry only if Steps 1 or 2 hit
      something (the drain bound or the sidecar-only trash run, say), per
      their learnings.
    - The `todo.md` section "App: rewrite or delete a folder's sidecars from
      the folder tree's context menu" is removed, and a new manual check
      item is added in the style of the existing `On Windows ...` items: on
      a copy under `D:\Photos\tests\<date>-<topic>\` (the manual test folder
      location) with a Lightroom-written `.xmp` and a PhotoLab-written
      `.dop`, rate a few files, `Rewrite Sidecars from Index…` and confirm
      in Lightroom (`Metadata > Read Metadata from File`) and PhotoLab that
      the judgments show and the develop settings / corrections survived;
      `Delete Sidecars…`, confirm the strip clears, `Undo` brings the stars
      back, `Redo` moves them again; press both during a scan and see the
      wait note; and try the rewrite on a folder never opened.
    - `mise run ci` passes (lychee on the new links).
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged. Docs only; keep the Japanese pair in
      the same PR (CLAUDE.md "Language").

## Trade-offs and risks

Each open question, with the option the plan takes first.

1. **Rewrite: patch or regenerate; rows without a judgment.** Taken: patch
   through the existing writer (`write_kind`), so other software's content
   survives; a row without a judgment patches an existing sidecar to "no
   judgment" and mints nothing; a RAW with no row is skipped. Alternatives:
   (a) regenerate from scratch (`write_rating(None, ...)` templates) — new
   code path, loses `crs:` / PhotoLab corrections, and a fresh `.dop` for a
   registered image needs PhotoLab's Uuids again (`photolab.rs`), which the
   writer already handles; only worth it as a repair for an unparseable
   sidecar, which today is reported, not fixed. (b) Skip rows without a
   judgment too (write only judged rows) — less destructive, but then
   "rewrite from the index" does not make the sidecars agree with the index
   for files the user cleared; the plan's choice is one line in the row
   filter either way. (c) Remove the judgment fields (`xmp:Rating` removed
   rather than `0`) for unjudged rows — the writer writes `Rating 0` today
   and Lightroom / PhotoLab read `0` as unrated, so no change is proposed.
2. **Delete: fields or file; Trash or permanent; undo.** Taken: the whole
   file to the OS Trash, recorded in `trash::Runs` so the existing undo /
   redo commands and the frontend's `TrashEntry` history work on it (a
   sidecar-only run restores because `trash::restore`'s `raw_back` starts
   `true`). Alternatives: (a) patch the judgment fields out and keep the
   file — that is Step 1's rewrite with every row cleared, keeps other
   software's edits, needs no Trash, but is not "delete" and leaves the files
   the user wanted gone; could be a third item later. (b) Permanent delete —
   less code (no `Runs` entry) but irreversible, and the project's existing
   destructive item goes to the Trash with undo; not recommended. Risk: the
   `TrashEntry` reuse makes `restoredStatus` count RAWs (`restored` lists
   RAWs only), hence the `what: "sidecars"` phrasing field; if that grows
   messy, a separate `SidecarEntry` kind that calls the same two commands is
   the cleaner shape at the cost of a second branch in `step`.
3. **Formats.** Taken: the current `SidecarFormat`'s kinds (`Both` → both).
   Alternative: both kinds regardless, as `Move Rejected to Trash` does when
   it moves a RAW's sidecars — reasonable for delete ("clean the folder"),
   but it breaks the invariant that a single-format setting never touches
   the other format's files, and a `.dop` with PhotoLab corrections deleted
   under an XMP-only setting is a surprise; for rewrite it would mint
   PhotoLab files the user never chose. The plan reads the format once per
   command from `AppSidecarFormat`, so switching to "both" is a one-line
   change in the collection.
4. **Subfolders and multi-selection.** Taken: the clicked folder only, not
   offered with several folders selected. Alternatives: an `..., Including
   Subfolders…` pair reusing `trash::subfolders` (would need to become
   `pub(crate)`) and the `recursive` walk, and a `... in N Folders…` form on
   a selection — straightforward extensions (the commands would take `dirs:
   Vec<String>, recursive: bool` like the trash ones), left out to keep the
   first PRs small.
5. **During a scan; the open folder.** Taken: `whenIdle` hold plus a backend
   `SCAN_RUNNING` refusal, exactly as the trash flow; rewrite leaves the
   strip alone; delete clears the strip's judgments itself and re-reads
   `folder_entries` (a rescan would report `changed = 0` because the index
   rows are cleared in the command). Alternative for delete: leave the
   index rows alone and let the resync's `reconcile_sidecars` clear them
   (gone sidecar + clean row) — no index method, but a closed folder keeps
   stale judgments until its next open, and a rewrite in between would
   re-mint them. Risk: a `set_rating` landing during the delete run (a key
   pressed on the open folder) re-dirties a row the command clears; the
   `Scans` lock does not stop `set_rating`, so the writer would mint a
   fresh sidecar right after the delete. The window is one keypress wide;
   accept it, or have the frontend ignore judgment keys while the dialog is
   open (it does: every key stops at the dialog's keydown handler).
6. **A folder never indexed.** Taken: rewrite refuses with a status message
   ("open the folder first"); delete works regardless. Alternative: run the
   folder's sidecar reconcile first (`reconcile_sidecars_of` reads the
   sidecars into rows) and then rewrite — but that just writes back what
   was read, which is a no-op in intent; opening the folder is the honest
   answer.

Other decisions the steps will face:

- **Orphan sidecars** (a `.xmp` whose RAW is gone) are neither rewritten
  (no row survives without a listed RAW) nor deleted (collection is keyed
  by listed RAWs, as the trash collector is). Deleting them too is one
  filter change in Step 2's collection; left out so delete cannot remove a
  sidecar of something that is not a RAW (a `.jpg.xmp`, say).
- **Failure count of the rewrite.** The writer reports failures through
  `on_error` events, not a return value; counting rows still dirty after
  the drain is the cheap measure and over-counts a keypress made during the
  drain. If exact is wanted, extend `Writer::flush` to return the number of
  failed entries of that drain.
- **Progress.** A 5000-file rewrite may run for minutes with only the
  dialog's running note; progress events (as `sequence-progress`) are out
  of scope and can be added later from the writer's `flush` loop.
- **`has_sidecar` after delete** becomes false in `folder_entries`
  (`xmp_size IS NOT NULL`); nothing in the frontend keys off it visibly
  today, so no change is planned.

## Progress

- (2026-10-05) Step 1 complete: `rewrite_sidecars_preview` / `rewrite_sidecars_run` rewrite a folder's sidecars from the index, with the confirmation dialog's text and flow in `sidecars.ts`. The failure count is the number of entries still waiting after the writer's drain (the cheap measure noted above). The manual check is pending (see `learnings.md`).
