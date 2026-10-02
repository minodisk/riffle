# Learnings

## Step 1

- `seq` is bumped in three places besides `show()` (`refilter`'s empty
  branch, `openDirectory`), so the failure flag is stored as the failing `seq`
  (`previewFailedSeq`) and compared against the current `seq` in
  `renderEmpty()`, in addition to being reset in `show()`. A bump that skips
  `show()` therefore cannot leave the overlay up for a different file.
- Both failure branches share one helper, `failPreview()` in `main.ts`
  (set the flag, `shown?.bitmap.close(); shown = null; draw();`). The
  following `setStatus(...)` calls `renderMeta()` → `renderEmpty()`, so the
  overlay appears without an extra render call.
- Read-through of the paths that could draw an old `shown` after the clear:
  `draw()` returns early on `shown === null`; `drawZoom()`'s placeholder and
  `drawFaceMarks` already require `shown.seq === seq`; the only writer of a
  non-null `shown` is the worker handler, which drops a stale `seq`.
- `mise run fmt` first failed with `Cannot find module
  .../node_modules/vite-plus/bin/vp`: a fresh worktree has no `node_modules`
  and `pnpm` is only on PATH through mise. `mise exec -- pnpm install
  --frozen-lockfile` fixed it.

## Deferred issues (todo candidates)

- Pending manual check (real app, Windows; Step 1's checkbox was ticked on
  the automated criteria only). Basis: plan.md Step 1 "Done when". Files:
  `crates/app/ui/src/main.ts`, `crates/app/ui/src/empty.ts`. Steps: open
  `D:\Photos\samples\NEF\`, view a normal NEF, then `NIKON_D70_Nikon.nef`.
  Expected: the main preview no longer shows the previous NEF and reads "The
  preview of this file could not be shown." with the error in the meta pane's
  status block; paging on to a normal file shows it; paging back and forth
  quickly across the bad file never leaves an older file's image on screen;
  `z` (1:1) and Compare on the bad file do not throw.
