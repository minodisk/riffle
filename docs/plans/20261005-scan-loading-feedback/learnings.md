# Learnings: scan loading feedback

## Step 1

- The bar's show / hide goes through one `setScanProgress(width | null)` in
  `main.ts`, called with `"0%"` in `openDirectory` only (not `startScan`, which
  every `resync()` also ends in), on each
  `scan-progress`, on `scan-done`, in `startScan`'s catch and in
  `openDirectory`'s reset. Hiding resets the indicator to `0%` so the next
  scan never flashes the previous fill.
- The `@keyframes pulse` only needs the `50% { opacity: 0.5 }` stop
  (shadcn's `animate-pulse`); the 0% / 100% frames default to the element's
  own opacity of 1.
- The first `mise run fmt` died with a Node.js stack trace (a
  `requireStack` error in the frontend formatter); re-running it passed
  with no change, so it was transient. `mise run ci` passed on the first run.

## Deferred issues (todo candidates)

- **Pending manual check** (Step 1, Windows): open a folder of 500+ files
  whose index is cold (or after Clear Cache) and confirm the `#scan-progress`
  bar appears before the first thumbnail, fills, and disappears at the end of
  the first pass, and that empty strip cells pulse until their thumbnail
  lands while failed (`.cell.failed`) cells do not. Expected: as described.
  The step's checkbox was ticked on the automated criteria; the item is
  already in `todo.md` as "App: the scan loading feedback's manual check is
  still open" (the plan's Done-when asked for it there). Files:
  `crates/app/ui/src/main.ts`, `crates/app/ui/style.css`,
  `crates/app/ui/index.html`.
