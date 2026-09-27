# Learnings

## Step 1: backend folder rename

- `Scans`' inner mutex, `ScansState`, `ScansState::scanning` and
  `SCAN_RUNNING` became `pub(crate)` so `rename.rs` can hold the `Scans` lock
  across the rename and the index write, as `trash_rejected` does.
- `folder_target` runs inside `spawn_blocking` (first thing, before the
  writer flush) rather than before it as the plan lists: it canonicalizes and
  stats paths, and blocking IO on an async runtime worker is what
  `docs/agents/tauri-app.md` "Synchronous commands run on the main thread"
  warns against. The refusal still happens before anything on disk changes.
- `Index::rename_dir` first deletes any rows already stored under the new
  path: the rename's target did not exist on disk, so such rows are stale
  cache of a vanished folder, and without the delete the `UPDATE` would fail
  on the `path` / `dir` primary keys. `old == new` returns early so that
  delete can never wipe the moved rows.
- `watch::release` (the lock-free core of `release_under`) matches "under"
  with `Path::starts_with`, which is component-wise, so `photos2` does not
  count as under `photos`. The test builds a real `notify` watcher on a temp
  dir; the Windows handle release itself is not asserted, since notify's
  Windows watcher stops its thread asynchronously on drop.
