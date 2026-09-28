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
