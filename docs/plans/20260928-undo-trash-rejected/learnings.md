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
