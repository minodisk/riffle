<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Undo "Move Rejected to Trash"

## Purpose

`Move Rejected to Trash` (the File menu item, the folder tree's items and the
multi-folder selection; see
[the archived plan](../_archived/20260928-trash-rejected-from-tree/plan.md))
moves hundreds of files in one confirmed run. The user asked whether a run
can be taken back. Today the only way is the OS Trash's own "Restore" /
"Put Back", file by file, and Riffle's undo history forgets the judgments of
the trashed files (`history.removeWhere` in `main.ts`).

This makes one run a single undo unit: `Edit > Undo` (`CmdOrCtrl+Z`) moves
every RAW and sidecar the run trashed back to where it came from, never
overwriting, reports what could not come back (a file with the same name at
the original location, a Trash that was emptied, a file already restored by
hand) in the status line and the error list, and refreshes the open folder so
the strip and the index get the files and their reject judgments back. Redo
moves the same files to the Trash again without the dialog.

Platform facts that shape the design (read from `trash` 5.2.9's sources in
the cargo registry):

- Windows / Linux: `trash::os_limited::list()` returns every item in the
  Trash with `original_parent`, `name`, `time_deleted` (seconds) and an
  opaque `id`; `restore_all(items)` moves them back. On Windows it first
  checks every item's `original_path().exists()` and returns
  `Error::RestoreCollision` for the first hit *without restoring anything*,
  then runs one `IFileOperation` for all items with `FOFX_EARLYFAILURE`, so
  a single item gone from the Recycle Bin fails the whole batch. On Linux it
  restores item by item with `create_new` and fails at the first collision
  with the remaining items. Windows reports `original_parent` without the
  verbatim `\\?\` prefix the app's canonical paths carry.
- macOS: the crate has no listing or restore; `DeleteMethod::NsFileManager`
  calls `trashItemAtURL_resultingItemURL_error(&url, None)`, discarding the
  URL the file got inside the Trash. The app has to make that call itself
  (through `objc2-foundation`, already a dependency of the crate on macOS)
  with an out pointer, keep the resulting URL, and move the file back with
  `std::fs::rename` (the Trash of a volume lives on that volume, so the
  rename does not cross devices).

## Steps

- [x] Step 1: Record each run, and `trash_rejected_undo` for Windows / Linux
  - Done when:
    - `trash_rejected_run` records the run: a `TrashRun { id: u64, moved:
      Vec<Trashed> }` where `Trashed` holds the original path (RAW or
      sidecar; the RAW's sidecars come right after it, in the order moved)
      and, for later use by Step 2, an optional `trashed_at: PathBuf`
      (`None` on Windows / Linux). Runs live in an app state
      (`trash::Runs`, a `Mutex<Vec<TrashRun>>` managed in `main.rs` next to
      `Sequences`), bounded to the newest N (match the frontend's history
      limit of 100). `trash::Summary` gains `run_id: Option<u64>` (`None`
      when nothing moved, so the frontend pushes no undo entry).
    - New command `trash_rejected_undo(run_id: u64) -> Result<Restored,
      String>` in `commands.rs`: refuses a running scan under the `Scans`
      lock (the same `scanning()` guard `trash_rejected_run` uses), holds
      the lock across the moves as the guide's "index/scan mutation" rule
      requires, and returns `Restored { restored: Vec<String> (the RAW
      paths that came back), failed: Vec<Failure> (every path, RAW or
      sidecar, with the reason) }`. An unknown `run_id` is an `Err` ("this
      run can no longer be undone"). A run already undone is removed from
      `Runs` (Step 4's redo mints a new run).
    - The restore is planned by a pure function in `trash.rs` (e.g.
      `restore_plan(run: &TrashRun, in_trash: impl Fn(&Path) ->
      Option<Item>, exists: impl Fn(&Path) -> bool) -> (Vec<(Trashed,
      Item)>, Vec<Failure>)`) that yields, per recorded path: a failure
      "already exists at the original location" when `exists` holds
      (never overwrite), a failure "not in the Trash (emptied or restored
      by hand)" when the Trash holds no item for it, and otherwise the item
      to restore. The RAW is restored before its sidecars; a RAW that fails
      does not stop its sidecars (their judgment is worth keeping on disk
      for the user to pair by hand) — or the reverse; pick one, state it
      in the doc comment, and test it.
    - Windows / Linux mover-side: the command lists the Trash once with
      `os_limited::list()` after taking the lock, keys the items by their
      original path normalized the way the app compares paths on that
      platform (strip the `\\?\` / `\\?\UNC\` prefix from the recorded
      path before comparing, compare case-insensitively on Windows; see
      `shownPath` in `crates/app/ui/src/trash.ts` for the prefix rule), and
      for a path with several items takes the newest `time_deleted`. Each
      item is restored with its own `restore_all([item])` call so one
      missing or colliding item cannot fail the batch (the crate's batch
      semantics above); the crate's `RestoreCollision` on a race is mapped
      to the same "already exists" failure text.
    - `Trashed` is recorded from what `trash::run` actually moved, so
      `run` (or its mover closure) reports the moved sidecar paths too, not
      only `Summary.moved`'s RAWs. Keep `run` pure: the mover returns
      `Result<Option<PathBuf>, String>` (the trashed location when the
      platform knows it) and `run` collects the `Trashed` list next to the
      `Summary`.
    - Rust unit tests in `trash.rs`: the restore plan for a full run, a
      run whose RAW is back already (its sidecar still restored or not,
      per the rule chosen), a run whose sidecar is missing from the Trash,
      a run where nothing is in the Trash; the path normalization for the
      Windows verbatim prefix and case (`#[cfg(windows)]` where the
      behavior is Windows-only); `Runs` keeps the newest N and forgets an
      undone run. Tests never call the real Trash: the plan takes closures.
    - `CLAUDE.md`'s layout paragraph for `trash.rs` mentions the run record
      and the restore.
    - `mise run ci` passes. No user-visible change yet (the frontend ignores
      `run_id` until Step 3).
  - Implementation approach:
    - Model the command on `trash_rejected_run` / `rename_folder`: `async`,
      `spawn_blocking`, refusal under the `Scans` lock before anything on
      disk changes. No sidecar writer drain is needed (nothing pending can
      target a file that is in the Trash), but say so in the doc comment.
    - No index write: the frontend's `resync()` re-lists the folder, and
      `Index::reconcile` re-adds the `files` rows while `reconcile_sidecars`
      trusts a `ratings` row whose stored stat still matches the restored
      sidecar (the Trash keeps mtime on all three platforms) or parses the
      sidecar again; a folder never opened has no rows and parses. Verify
      this in Step 3's manual check rather than adding index code.
    - `os_limited` is `cfg`-gated in the crate to Windows and freedesktop
      Unix; put the listing / restoring code behind
      `#[cfg(not(target_os = "macos"))]` and leave a `todo`-free macOS stub
      that returns every path as "not supported yet" until Step 2 lands
      (CI builds macOS, so it must compile).
    - Do not modify `crates/app/src/watch.rs`. Moving files back into a
      watched folder is what the watcher is for; its `folder-changed` for
      the open folder coalesces with the explicit `resync`.

- [x] Step 2: macOS: keep the trashed URL and move the file back
  - Done when:
    - On macOS the mover in `commands.rs` no longer goes through
      `TrashContext::delete`; a `trash::macos` (or a `cfg`'d block in
      `trash.rs`) calls `NSFileManager::defaultManager()
      .trashItemAtURL_resultingItemURL_error(&url, Some(&mut out))` and
      returns the resulting URL's path, which `run` records as
      `Trashed::trashed_at`. The path-to-`NSString` handling (UTF-8 as is,
      else percent-encoded) copies the crate's `delete_using_file_mgr`.
      `trash_context()` and its `DeleteMethod::NsFileManager` setting go
      away if nothing else uses them.
    - `trash_rejected_undo` on macOS restores from the recorded
      `trashed_at`: a failure when the original path exists (never
      overwrite), a failure when `trashed_at` no longer exists (Trash
      emptied or the file put back by hand; "not in the Trash"), else
      `std::fs::rename(trashed_at, original)`. The same `restore_plan` of
      Step 1 is reused by handing it an `in_trash` closure that checks
      `trashed_at.exists()`.
    - `crates/app/Cargo.toml` adds `objc2-foundation` (the version the
      `trash` crate pins, with the `NSFileManager`, `NSURL`, `NSString`,
      `NSError`, `std` features) under
      `[target.'cfg(target_os = "macos")'.dependencies]`.
    - Rust tests: the macOS restore's rename path is exercised on every
      platform by a test that builds a `TrashRun` with `trashed_at` set to
      a temp file and restores it (the rename logic is not macOS-specific;
      only the `NSFileManager` call is). The `NSFileManager` call itself is
      covered by a `#[cfg(target_os = "macos")]` test that trashes a temp
      file and asserts the returned URL exists, marked `#[ignore]` if the CI
      runner's Trash proves unusable (record the finding in
      `learnings.md`).
    - `mise run ci` passes on all three CI targets. Manual check on a Mac
      (cannot be done on this Windows machine; list it for the user):
      trash rejects in a folder, then `Edit > Undo` after Step 3 lands.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - Check the exact `objc2-foundation` 0.3 signature in the registry on a
      machine that has it downloaded
      (`~/.cargo/registry/src/*/objc2-foundation-0.3*/src/generated/NSFileManager.rs`);
      it is expected to be `unsafe fn
      trashItemAtURL_resultingItemURL_error(&self, url: &NSURL,
      out_resulting_url: Option<&mut Option<Retained<NSURL>>>) ->
      Result<(), Retained<NSError>>`. This machine (Windows) does not have
      the crate; CI's macOS job is the compile check, so push early and
      watch it.
    - `std::fs::rename` on Unix replaces an existing target; the `exists()`
      pre-check is the guard (a TOCTOU window is accepted; see Trade-offs).
    - `NSURL::path()` gives the file system path of the trashed item; keep
      it as a `PathBuf`, not a URL string.

- [ ] Step 3: `Edit > Undo` restores the run; the open folder refreshes
  - Done when:
    - The undo history holds two kinds of entry: the existing judgment
      batch and a trash run (`{ kind: "trash", runId, count, dirs,
      recursive }`). The `Judgment[]`-typed `history` / `redoable` in
      `main.ts` become `History<Entry>` with the kind split in `undo.ts`
      (pure helpers such as `isJudgments(entry)` so `map` / `removeWhere`
      call sites stay short).
    - `runTrash`'s success handler pushes a trash entry when
      `summary.run_id` is set, clears `redoable` (as a judgment does), and
      no longer calls `history.removeWhere` / `redoable.removeWhere` for
      the trashed files: the trash entry sits above those judgments, so
      undoing reaches them only after the files are back, and `step()`'s
      existing `allFiles.includes` filter skips a file that did not come
      back.
    - `step()` handles a trash entry: it works with `openDir === null` too
      (a run from the tree needs no open folder), invokes
      `trash_rejected_undo` through `whenIdle` + `settleIdle` like the run,
      refuses while a modal is open the way `modalOpen()` already gates the
      menu event, and on return: adds each failure to `errors` ("could not
      restore from the Trash: …"), sets the status line from a pure
      `restoredStatus(result)` in `trash.ts` ("Restored 148 files from the
      Trash", "…, 3 failed"), and when the open folder is among the run's
      `dirs` (reuse `opensTarget`) patches `allFiles` in place with the
      restored RAW paths (the guide's "patch the affected arrays at resolve
      time" rule), `refilter`s, and calls `resync()` so the scan re-adds the
      rows and the strip gets thumbnails, flags and sharpness back. The
      entry moves to `redoable` only when Step 4 lands; until then an undone
      trash entry is dropped (say so in the code comment).
    - The trash entry survives what clears judgment entries: `openDirectory`
      (folder switch, `sidecar-format`, `index-cleared`) and
      `setViewOnly(true)` (a folder whose RAWs were all trashed turns
      view-only on the very resync that follows the run) drop only
      judgment entries (`removeWhere(isJudgments)`), keeping trash entries.
    - A trash entry's undo while a scan runs waits like the run does
      (`whenIdle("Undo Move Rejected to Trash", …)`), the status line
      saying so; the backend still refuses under the lock if a scan slipped
      in, and the error shows in the status line with the entry put back
      on `history` (not lost).
    - TS tests: `undo.test.ts` for the kind split (pruning judgments keeps
      trash entries, `map` leaves trash entries alone); `trash.test.ts`
      for `restoredStatus` (singular / plural / with failures) and the
      "put the entry back on failure" helper if it is pure.
    - `docs/usage.md` (`Move Rejected in This Folder to Trash…` and the
      `Undo` / `Redo` bullets, the keys table row for `CmdOrCtrl+Z`),
      `README.md` and `README.ja.md` (the trash bullet: "one `Edit > Undo`
      brings them back") describe the undo, including that a file with the
      same name at the original location, an emptied Trash or a file
      already restored by hand is reported and left as it is.
    - Manual GUI checks (Windows, this machine) listed in `learnings.md`
      with their results: trash rejects in the open folder, `Ctrl+Z`
      restores them, the strip shows them again with the reject flag
      (from the sidecars) and the folder tree's count follows on its next
      re-list; the same from the tree for a folder that is not open (only
      the status line); a multi-folder run undone as one; a conflict
      (copy a trashed RAW back by hand first) shows in the status line
      and the error list; an emptied Recycle Bin reports every file.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged (Step 2 only adds macOS support; the
      frontend is platform-agnostic).
    - `step(from, to, verb)` today pops, filters by `allFiles`, and pushes
      the pre-state onto `to` synchronously. Branch on the entry kind at
      the top: the judgment path stays as it is; the trash path is
      asynchronous, so it must not push onto `to` before the command
      returns.
    - The trash entry does not move the current file or the selection.
    - `idle.ts`'s `IdleGate` holds one operation; an undo pressed while a
      trash run is held replaces it (existing one-slot rule) — acceptable,
      note it in `docs/usage.md` only if the reviewer asks.
    - Keep the `errors` wording parallel to the run's ("could not move to
      the Trash" / "could not restore from the Trash").

- [ ] Step 4: `Edit > Redo` moves the restored files to the Trash again
  - Done when:
    - New command `trash_rejected_redo(run_id)` (or `trash_paths`): moves
      the exact RAW and sidecar paths the undone run restored back to the
      Trash (no re-collection, no dialog; a path missing meanwhile is a
      failure), under the same guards as `trash_rejected_run`, and records
      a new `TrashRun` (new `run_id`), returning the same `Summary`.
    - `step()` pushes the undone trash entry (with the restored paths)
      onto `redoable`; redo invokes the new command, pushes a fresh trash
      entry (new `run_id`) onto `history`, prunes the open folder's maps
      and `allFiles` by the moved paths as `runTrash` does, and `resync`s.
      A judgment made after the undo still clears `redoable`, as today.
    - Rust test: the redo run over paths of which one is gone reports it
      and moves the rest (reusing the mover test double). TS test: the
      undone entry that goes onto `redoable` carries the restored paths
      only (a file that failed to come back is not re-trashed).
    - `docs/usage.md`'s `Redo` bullet and the README trash bullets say what
      redo does after an undone trash run.
    - `mise run ci` passes; manual check on Windows: undo, redo, undo again.
  - Implementation approach:
    - Assumes Step 3 is merged.
    - Build the groups for `trash::run` from the entry's restored paths in
      the recorded order (RAW then its sidecars), so the existing "RAW
      first, sidecars only if the RAW went" rule holds.

## Trade-offs and risks

- **Redo semantics (decided by the user: re-trash).** Redo re-trashes the
  restored files (Step 4) for symmetry with every other editor: an
  accidental `Ctrl+Z` after a big run is one `Ctrl+Shift+Z` away from being
  put right. The rejected alternative was "an undone trash run is not
  redoable" (the user re-runs the dialog, which re-collects the same rejects
  since their sidecars still say reject).
- **Matching trashed items on Windows / Linux.** Listing after the run and
  matching by original path (newest `time_deleted` per path) is one listing
  per undo and copes with a Recycle Bin the user emptied meanwhile. The
  alternative, `list()` before and after the run and diffing item ids, is
  exact (no ambiguity when the same path was trashed twice within a second)
  but lists twice on every run, and `list()` costs several COM calls per
  item on a Recycle Bin that may hold thousands of unrelated files. Measure
  `list()` on this machine's Recycle Bin in Step 1 and record the number;
  switch to the diff if matching proves ambiguous in practice.
- **Where the trash entry lives.** One mixed history stack, with the trash
  entry surviving a folder switch and a view-only flip, keeps `Ctrl+Z`
  meaning "the last thing I did" whatever it was, and lets a tree run with
  no folder open be undone. The alternative (keep the per-folder history as
  it is; a trash entry dies with the folder) is simpler but surprising: a
  run started from the tree while another folder is open, then a folder
  switch, and the run can no longer be undone. A middle ground (the trash
  entry only, kept in a separate stack, with `Ctrl+Z` preferring it while
  it is newer) needs an ordering key across the two stacks; not worth it.
- **Judgment entries of trashed files are kept.** Today they are pruned
  (`removeWhere`) so an undo cannot `set_rating` a gone path. With the trash
  entry above them they are reachable only after the run is undone; if the
  undo fails for some file, `step()`'s `allFiles` filter skips it, and if
  the user presses `Ctrl+Z` again before the `resync` after a restore has
  re-listed the folder, the in-place `allFiles` patch of Step 3 is what
  keeps that judgment reachable. Not patching `allFiles` and relying on the
  rescan alone would silently drop such a judgment.
- **macOS restore is not atomic.** `std::fs::rename` replaces an existing
  file on Unix; the `exists()` pre-check prevents overwriting in every case
  but a race with something creating the file in the same instant.
  `renameat2(RENAME_NOREPLACE)` / `renamex_np(RENAME_EXCL)` would close it
  at the cost of `libc` calls per platform; not taken.
- **macOS cannot be verified here.** This machine is Windows; the
  `objc2-foundation` call and the restore path are compiled by CI's macOS
  job and must be checked by hand on a Mac. Step 2 is ordered before
  Step 3 so the frontend lands on a backend that works on all three
  platforms, but Steps 2 and 3 are independent and can be swapped if the
  Mac check has to wait.
- **The Trash's own "Restore" / "Put Back" and Riffle's undo coexist.** A
  file put back by hand is reported as "not in the Trash" and counted as a
  failure, per the requirement. On Windows the Recycle Bin's own restore
  leaves no trace to tell that apart from an emptied bin, so both share one
  message.
- **Index rows during the gap.** Between the run and its undo the open
  folder's `resync` dropped the `files` rows of the trashed RAWs (the
  `ratings` rows stay, since `reconcile_sidecars` only sees listed files),
  so the restore costs a re-extraction of those files on the next scan.
  Accepted; no index code changes for undo.
- **`watch.rs` untouched.** Restoring into a watched folder needs no
  watcher change (files move under the watched directory, which Windows
  allows; the run already showed that direction). If the tree-watch
  session's per-node watchers are in place, they are `NonRecursive` too.

## Progress

- (2026-09-28) Step 1 complete
- (2026-09-29) Step 2 complete
