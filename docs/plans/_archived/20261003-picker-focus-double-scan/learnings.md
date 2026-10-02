# Learnings

## Step 1

- `settleIdle` takes the whole `openFolder` chain including its `.catch`, the
  way the trash run's call does: passing a promise that can reject would leave
  the `promise.finally(...)` result an unhandled rejection.
- `openDirectory`'s `idle.discard()` only clears the held operation, not the
  in-flight count, so wrapping the picker in `settleIdle` is balanced: the
  count drops in `finally` after `openDirectory` settled, and its
  `drainResync()` is a no-op because `openDirectory` already cleared
  `resyncPending`.
- The `scan superseded` log drops the `Scans` lock before logging, so the
  early return does not hold it across the logger.

## Deferred issues (todo candidates)

- Pending manual check (Windows, by the user after merge; Step 1's checkbox
  was ticked on the automated criteria): run `mise run tauri:release:devtools`
  with timing logs on. With folder A open and its scan older than 5 s, File >
  Open Folder and pick folder B: expect exactly one `scan list` /
  `scan prepare` / `scan extract` set, for B, and no `scan superseded` line.
  Repeat with Settings > Clear Cache (one set, for the reopen) and a plain
  alt-tab back to the app (one set). Basis: plan Step 1 Done-when; files
  `crates/app/ui/src/main.ts` (`openFolder`, the `tauri://focus` listener),
  `crates/app/src/commands.rs` (`scan_folder`).
