# Learnings

## Step 1

- The deferral is `IdleGate` in `crates/app/ui/src/idle.ts`. `main.ts` wraps
  it in two helpers: `whenIdle(label, run)` (request plus a `renderMeta()`, so
  the waiting line shows at once) and `settleIdle(promise)` (sets
  `idle.inFlight` and, in `.finally`, clears it and calls `drainResync()`).
- `renameFolder`'s success handler now returns the reopen's
  `openDirectory(...)` promise, so `settleIdle`'s `.finally` runs after the
  reopen. Otherwise a `resync()` deferred during the invoke would drain before
  the reopen landed and run `list_arw` on the old, renamed-away path.
- `idle.drain()` runs the held closure synchronously, so in `faces-done` it
  sets `idle.inFlight` before `drainResync()` runs; a pending rescan then
  stays pending until the operation settles, which is the intended order.

## Step 2

- `settings.ts` owns its own `IdleGate` (`clearGate`). `setScanRunning(false)`
  drains it before the scan-end `index_size` refresh, so a drained clear sets
  `clearInFlight` first and that refresh is skipped in favor of the clear's
  own.
- The button stays enabled while a clear is held; a second press just
  replaces the held clear (same one-slot rule as Step 1).
- Concurrent-work check: only the backend half of trash-folders
  (`trash_rejected`'s `dirs` / `recursive`, #518) is on main. There is no
  folder-menu `Move Rejected to Trash` in the frontend yet, and no other
  `scanRunning` refusal remains in `main.ts`, so nothing else was routed
  through the gate. When the folder-menu entry lands it should call
  `whenIdle` like `trashRejected`.

## Deferred issues (todo candidates)

- The inline rename edit now always starts, even during a scan, and a
  confirmed name waits silently in the edit's place apart from the status
  line; the strip / tree cell shows its old name until the rename runs.
  Whether it should show the pending name is an open UX question. Basis:
  Step 1 implementation. Files: `crates/app/ui/src/main.ts`,
  `crates/app/ui/src/strip.ts`, `crates/app/ui/src/folders.ts`.
- The `renameAllowed` parameter of `folders.init` is now always `() => true`
  and can be removed once the tree-watch work in `folders.ts` lands. Basis:
  plan "Trade-offs and risks". Files: `crates/app/ui/src/folders.ts`,
  `crates/app/ui/src/main.ts`.
- The folder tree's upcoming `Move Rejected to Trash` entry (trash-folders
  work; only the backend landed in #518) must go through `whenIdle` /
  `settleIdle` with run-time reads rather than a `scanRunning` refusal.
  Basis: plan "Trade-offs and risks", Concurrent work; Step 2 check. Files:
  `crates/app/ui/src/main.ts`, `crates/app/ui/src/folders.ts`.
