# Learnings

## Step 1: Backend reject collection over folders

- `trash::collect` takes the index rows through a closure
  (`Fn(&str) -> Result<HashMap<String, RowFlag>, String>`), so its tests
  feed hand-built rows and never open SQLite; `Index::row_flags` has its own
  test in `index.rs`.
- The flag rule mirrors `reconcile_sidecars` one step further than the plan
  spelled out: a clean row whose sidecar is gone is not a reject when its
  stored stat is set (the folder open would clear it), but is trusted when
  its stat is `(None, None)` (the folder open keeps such a row as it is).
- A dirty row wins even over a sidecar that changed, unlike the folder open
  (where the sidecar wins). The writer is drained before each collection, so
  a dirty row left at that point is a write that failed; the plan chose to
  trust the judgment the user made in the app.
- `trash_rejected` collects twice: once before the native dialog (for its
  count, without the `Scans` lock) and again under the lock for the move, as
  Step 4's preview / run split will. The per-folder `FolderPlan::dir` is only
  logged for now; the per-folder summary for the frontend is left to Step 4.
- `Summary.trashed` was replaced by `Summary.moved` (the RAW paths); the
  count is its length. Collection failures (an unreadable folder, a sidecar
  that does not parse, an index query error) go in `Summary.unread`, kept
  apart from `Summary.failed` (a move that was attempted and failed): nothing
  was tried on an unread path, so it must not read as a failed move or count
  toward `trashedStatus`'s "N failed".
- The flag `collect_folder` uses is decided from the configured
  `SidecarFormat`'s kinds only, the same ones the folder open reads; `Both`'s
  kinds are still gathered to know what sidecars move with a reject, but a
  leftover sidecar of the other format (e.g. a stale `.dop` from an earlier
  `Both` setting, or one PhotoLab keeps rewriting) never overrides the
  configured format's flag.
- The subfolder walk uses `DirEntry::file_type().is_dir()`, which is false for
  a symlink (and for a Windows junction, which std reports as a symlink), so a
  link is never followed; `folders::is_hidden` became `pub(crate)` for it.
- Windows check (on this machine, Windows 11): a throwaway test (not
  committed) put a `notify::recommended_watcher` `NonRecursive` watch on a
  canonical (`\?\`-prefixed) temp dir, as `watch.rs` does for the open
  folder, and ran `trash::run` with the real `TrashContext::default()` over a
  RAW and its `.xmp` inside it. Both went to the Recycle Bin with no failure,
  so trashing the rejects of the open (watched) folder needs no watcher
  change, and a folder that is not open has no watch at all. Contrary to the
  plan's note, renaming the watched directory itself also succeeded in that
  run (notify's `ReadDirectoryChangesW` handle is opened with
  `FILE_SHARE_DELETE`); `rename.rs` still releases the watcher first, which
  is harmless. The GUI check (the File menu item on an open folder, the
  status line and the refreshed strip) remains a manual check for the user.
