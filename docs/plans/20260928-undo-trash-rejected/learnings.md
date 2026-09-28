# Learnings: undo-trash-rejected

## Step 1

- `os_limited::list()` on this machine's Recycle Bin (Windows 11, 22 items,
  release build of a scratch binary against `trash` 5.2.9): 65 ms on the
  first call in the process (COM start-up), then 3.5-3.8 ms per call, about
  0.16 ms per item. One listing per undo is cheap even for a bin of
  thousands of items (well under a second), so the "list once after the run,
  match by original path" design stays; no before/after diff.
- RAW-vs-sidecar failure rule chosen: when a RAW does not come back (its
  original path is taken, it is gone from the Trash, or the restore itself
  fails) its sidecars stay in the Trash, each reported as "left in the
  Trash: its RAW could not be restored". This mirrors `trash::run`'s rule
  (sidecars only move after their RAW) and avoids restoring a reject sidecar
  next to a different file that took the RAW's name, which would pin the
  reject on that file. The user can still restore them by hand from the
  Trash.
- Deviations from the plan's sketch, both within its "e.g.":
  - Instead of `restore_plan` returning `(Vec<(Trashed, Item)>,
    Vec<Failure>)`, one pure `trash::restore(run, in_trash, exists, mover)`
    plans and performs the restore. The rule above depends on whether the
    RAW's actual restore succeeded, which a plan computed before any move
    cannot know. `in_trash` takes the whole `Trashed` (not only its path) so
    Step 2's macOS closure can check `trashed_at`.
  - `Runs` is `Mutex<RunsState { next_id, runs }>` rather than a bare
    `Mutex<Vec<TrashRun>>`, so a forgotten run's id is never minted again.
- A RAW vs sidecar in the flat `TrashRun.moved` list is told apart with
  `riffle_core::scan::is_raw_file` (sidecars are `.xmp` / `.dop`), keeping
  `Trashed` to the two fields the plan names.
- The macOS stub marks `restore`, `trash_key`, `newest_by_path` and the
  failure texts `#[cfg_attr(target_os = "macos", allow(dead_code))]`; Step 2
  should drop the attributes it no longer needs (`restore` and the texts
  become used there).

## Step 2

- The macOS compile is verified only by CI's macOS job. This Windows
  machine cannot `cargo check --target aarch64-apple-darwin` the app
  (`objc2-exception-helper`'s build script needs a C compiler for the
  target). As a partial check, the new `trash_file` / `percent_encode`
  code was copied into a scratch crate depending on `objc2-foundation`
  0.3.2 (the version `trash` 5.2.9 pins in `Cargo.lock`) and
  `percent-encoding`, which `cargo check` and `cargo clippy -D warnings`
  pass for `aarch64-apple-darwin`. The rest of the macOS cfg (the
  `commands.rs` mover and `restore_run`, the remaining `dead_code`
  attributes) is not compiled here.
- `objc2-foundation` 0.3.2's signature, read from the registry after
  `cargo fetch` (which downloads every platform's crates):
  `pub fn trashItemAtURL_resultingItemURL_error(&self, url: &NSURL,
  out_resulting_url: Option<&mut Option<Retained<NSURL>>>) -> Result<(),
  Retained<NSError>>`. It is a safe fn in 0.3 (no `unsafe` block), and
  `NSURL::path()` is `Option<Retained<NSString>>`. Letting `None` infer
  the out pointer's type avoids a direct `objc2` dependency for
  `Retained`.
- `percent-encoding` is added as a macOS-only dependency too (the crate's
  own path handling uses it; already in `Cargo.lock` through `trash`).
- On Windows / Linux the mover is now `::trash::delete` (the default
  context) and `trash_context()` is gone. `restore`, the three failure
  texts and the new `restore_recorded` are used on macOS; `trash_key`
  and `newest_by_path` keep their macOS `allow(dead_code)`, and
  `restore_recorded` gets the mirror attribute off macOS (it is used
  there only by its test). `Restored::none` was missed in this pass: its
  only non-test caller left after this commit is the
  `#[cfg(not(target_os = "macos"))]` `restore_run`, so it also needs
  `#[cfg_attr(target_os = "macos", allow(dead_code))]` (added in review
  round 1; this Windows machine cannot compile the macOS cfg to catch it
  itself).
- A file existing at the recorded `trashed_at` is not enough to call it
  "still in the Trash": the Trash can hand a freed name to an unrelated
  file trashed later from elsewhere (camera file names like
  `DSC00001.ARW` repeat across cards). `Trashed` now also records
  `trashed_id`, the `(dev, ino)` of the file at `trashed_at` taken right
  after the move (`std::os::unix::fs::MetadataExt`, `None` on
  non-Unix), and `restore_recorded`'s `in_trash` requires it to still
  match before renaming back (added in review round 1).
- The `#[cfg(target_os = "macos")]` test `ns_file_manager_tells_where_the_file_went`
  trashes a real temp file; it is not `#[ignore]`d. If the CI runner's
  Trash turns out to be unusable, mark it `#[ignore]` and record it here.
- Manual check on a Mac (not possible on this machine): trash rejects in a
  folder, then `Edit > Undo` once Step 3 lands; the files come back and a
  file with the same name at the original location is reported, not
  overwritten.

## Step 3

- The trash entry is not popped when `Ctrl+Z` is pressed: `step` peeks
  (`History.peek`, new) and `undoTrash` removes the entry only once
  `trash_rejected_undo` has returned. A refusal (a scan that slipped in
  under the lock) therefore leaves the entry where it was, even if a
  judgment was pushed above it meanwhile, and `openDirectory`'s
  `idle.discard()` of a held undo loses nothing. The one error that drops
  the entry is the backend's `this run can no longer be undone`
  (`stillUndoable` in `trash.ts`); keeping it would make every later
  `Ctrl+Z` hit the same error and never reach the judgments beneath.
- While the restore is out (`restoring`), `step` does nothing, for undo and
  redo alike: a second `Ctrl+Z` would otherwise pop the judgment batch of
  the trashed files beneath while they are not yet in `allFiles`, and
  `step` drops a batch whose files are all missing.
- `trash_rejected_undo` returns the recorded paths in the backend's
  canonical spelling (`\\?\C:\...` on Windows), which is how `list_arw`
  spells `allFiles`, but `openDir` may come from the tree without the
  prefix. `relation` (tree.ts) does not strip the verbatim prefix, so
  `restoredInto` compares through `shownPath` on both sides.
- Backend check against the real Recycle Bin (Windows 11, a throwaway
  `#[ignore]` test in `commands.rs`, not committed): three RAW + `.xmp`
  pairs in a canonicalized temp folder were moved with `trash_one`, then
  `DSC2.ARW` was recreated by hand and `DSC3.xmp` purged from the bin with
  `os_limited::purge_all`. `restore_run` took 61 ms and returned
  `DSC1.ARW` and `DSC3.ARW` restored; `DSC2.ARW` "already exists at the
  original location" (the hand-made file kept its content), `DSC2.xmp`
  "left in the Trash: its RAW could not be restored", `DSC3.xmp` "not in
  the Trash (emptied or restored by hand)". `DSC1.xmp` came back next to
  `DSC1.ARW`, and the run was gone from `Runs` afterwards.
- Manual GUI checks on Windows, pending for the user (a subagent cannot
  drive the app):
  - Trash rejects in the open folder, `Ctrl+Z`: the files come back, the
    strip shows them again with the reject flag (from the sidecars), and
    the folder tree's count follows on its next re-list.
  - The same from the tree for a folder that is not open: only the status
    line changes ("Restored N files from the Trash").
  - A multi-folder run undone as one `Ctrl+Z`.
  - A conflict (copy a trashed RAW back by hand first): the status line
    says "…, N failed" and the error list shows "could not restore from the
    Trash: already exists at the original location" (backend side verified
    above).
  - An emptied Recycle Bin: every file is reported "not in the Trash"
    (backend side verified above for one purged item).
  - After a restore, the index picks the reject judgments back up on the
    `resync` (the plan's "no index code" assumption for Step 1).

## Deferred issues (todo candidates)

- A folder renamed (tree `Rename…`) after a `Move Rejected to Trash` run
  leaves the recorded run pointing at the old path, so undoing it restores
  into the old, now missing, folder (on Windows the Recycle Bin may
  recreate the folder). The frontend's rename handler rewrites judgment
  entries (`mapJudgments`) but not a trash entry's `dirs`, and the
  backend's `TrashRun` is not rebased either. Basis: Step 3
  implementation. Files: `crates/app/ui/src/main.ts` (rename handler),
  `crates/app/src/rename.rs`, `crates/app/src/trash.rs` (`Runs`).
