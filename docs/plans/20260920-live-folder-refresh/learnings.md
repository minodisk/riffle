# Learnings

## Step 1: focus / manual rescan

- No backend index or scan change was needed: `scan_folder` + `start_scan`
  already are the diff, so `resync()` is the same invoke chain
  `openDirectory` runs, factored out into `startScan(folder)`.
- `refilter` gained a second parameter (`keepScroll`) rather than a separate
  list-replacement function, and `strip.setFiles` a matching optional
  argument; it is the only place that touches `strip.scrollTop`, and the
  offset is clamped to `files.length * CELL_HEIGHT - clientHeight`.
- `entries` is not pruned by `resync`: the `folder_entries` read on
  `scan-done` clears and refills the map, so rows of files that are gone drop
  out there. Kept it that way instead of adding a prune pass.
- Deferral state is two flags: `scanRunning` (set at the top of `startScan`,
  before `scan_folder` is invoked, so the prepare phase is covered too;
  cleared in the `scan-done` listener; `startScan`'s own `.then` and `catch`
  exits key their ownership check off a per-call monotonic `scanSeq` captured
  at the top of `startScan` against the module-level `currentScan`, so a
  later call — a different folder or a deliberate re-open of the same one —
  supersedes this one; the superseded exit leaves `scanRunning` alone because
  the newer `startScan` already owns it) and `resyncPending`, drained
  from `scan-done` via
  `drainResync()`. `resyncInFlight` keeps one `list_arw` outstanding, the way
  `refreshEntries` does for `folder_entries`.
- `tauri://focus` was used as decided (decision 2); it was **not** verified at
  runtime here, since GUI automation does not work on this Mac. If it turns
  out not to reach the global `event.listen`, the fallback is
  `WindowEvent::Focused(true)` emitting `folder-changed` (see the plan's
  Trade-offs).
- The `Reload Folder` menu item is a plain `MenuItem` with a fixed
  `CmdOrCtrl+R`; no `cfg` split, since it carries no icon.

## Step 2: folder watcher

- `notify` resolved to 8.2.0 (`cargo add` needs the network, so it had to run
  with the sandbox disabled).
- `triggers` had to special-case the empty path list: `[].iter().all(..)` is
  `true`, which would have made a pathless event *not* trigger. The unit test
  caught it on the first run.
- The debounce state is a single `Option<(dir, deadline)>` rather than
  `sidecar::run`'s deadline map: only one folder is open at a time, and a
  message for another folder means the old one is gone, so its pending event
  is dropped on purpose.
- The debounce thread is driven through its channel in the test (`run` takes
  the receiver and an `emit` closure), so no real file-system event or FSEvents
  latency is involved and CI cannot flake on it.
- `watch::set` is called from `scan_folder` right after `canonicalize`, so it
  also runs on every rescan; the "same folder, leave it alone" check is what
  keeps a rescan from unwatching and rewatching (which on Windows would drop
  and retake the directory handle).
- Not verified here (GUI automation does not work on this Mac): that a copied
  file really appears within ~1 s, that a ten-file copy produces one rescan,
  and that no `scan list` lines follow sidecar writes while culling. All of
  that is the plan's manual check.

## Deferred issues (todo candidates)

- The cost of a rescan on an unchanged large folder (stat of every file plus
  the sidecar reconcile, plus the `folder_entries` read on `scan-done`) was
  not measured; the plan's "Other risks" asks for a number on a real folder.
  `crates/app/src/commands.rs` (`scan_folder`), `crates/app/src/index.rs`.
- A file picked up mid-copy is extracted from a partial read and corrected
  only by the event the copy's completion fires; the plan accepts this, but it
  was not observed either way here. `crates/app/src/watch.rs`,
  `crates/app/src/commands.rs`.
