# Learnings

## Step 1: focus / manual rescan

- No backend index or scan change was needed: `scan_folder` + `start_scan`
  already are the diff, so `resync()` is the same invoke chain
  `openDirectory` runs, factored out into `startScan(folder, token)`.
- `refilter` gained a second parameter (`keepScroll`) rather than a separate
  list-replacement function, and `strip.setFiles` a matching optional
  argument; it is the only place that touches `strip.scrollTop`, and the
  offset is clamped to `files.length * CELL_HEIGHT - clientHeight`.
- `entries` is not pruned by `resync`: the `folder_entries` read on
  `scan-done` clears and refills the map, so rows of files that are gone drop
  out there. Kept it that way instead of adding a prune pass.
- Deferral state is two flags: `scanRunning` (set when `start_scan` is
  invoked, cleared in the `scan-done` listener) and `resyncPending`, drained
  from `scan-done` via `drainResync()`. `resyncInFlight` keeps one `list_arw`
  outstanding, the way `refreshEntries` does for `folder_entries`.
- `tauri://focus` was used as decided (decision 2); it was **not** verified at
  runtime here, since GUI automation does not work on this Mac. If it turns
  out not to reach the global `event.listen`, the fallback is
  `WindowEvent::Focused(true)` emitting `folder-changed` (see the plan's
  Trade-offs).
- The `Reload Folder` menu item is a plain `MenuItem` with a fixed
  `CmdOrCtrl+R`; no `cfg` split, since it carries no icon.

## Deferred issues (todo candidates)

- A rescan whose `scan_folder` result lands with a stale folder token leaves
  `resyncPending` set until the next `scan-done` drains it. Harmless (at worst
  one extra rescan is skipped until then), so it was not worth a third flag.
  From the Step 1 implementation; `crates/app/ui/src/main.ts`
  (`startScan` / `drainResync`).
- The cost of a rescan on an unchanged large folder (stat of every file plus
  the sidecar reconcile, plus the `folder_entries` read on `scan-done`) was
  not measured; the plan's "Other risks" asks for a number on a real folder.
  `crates/app/src/commands.rs` (`scan_folder`), `crates/app/src/index.rs`.
