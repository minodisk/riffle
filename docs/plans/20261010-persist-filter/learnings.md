# Learnings

## Step 1

- `filter_setting` checks `stars` with `Value::as_u64`, which is `None` for a
  float such as `2.5` and for a string such as `"3"`, so both drop without
  extra cases; a float like `5.0` drops too, which is fine because the UI
  sends integers.
- `cargo` is not on the Git Bash `PATH` in this worktree; run it through
  `mise exec -- cargo ...`.

## Step 2

- `main.ts` now keeps one `filterState` object over the `shown*` sets, which
  `passes`, `toStored` and `applyStored` all share, so the restore mutates the
  same sets the strip filters by.
- `filterChanged()` is split into `mirrorFilter()` (checks + lit button) and
  the change path (mirror, `refilter()`, `set_filter`). The EXIF item clicks
  go through the same change path, so they also save; the saved value is the
  same then, since `toStored` drops `exif`.
- The reopen of the last folder now waits on
  `Promise.allSettled([sortLoaded, filterLoaded])`; `sortLoaded` lost its
  `.finally` but still resolves the same way for the `loadRoots` join.

## Deferred issues (todo candidates)

- **Pending manual check: the filter survives a relaunch** (from Step 2's
  Done when; files `crates/app/ui/src/main.ts`, `crates/app/src/commands.rs`).
  On a desktop build (Windows or macOS): (1) open a folder, check e.g.
  `Picked` and `Good` in the filter menu, quit, relaunch -> the same files are
  hidden, the menu shows those checks and the filter button is lit;
  (2) press `Reset`, quit, relaunch -> no filter, button not lit; (3) quit,
  set `"filter": "bogus"` (or a value with unknown states) in the app's
  `settings.json`, relaunch -> launches unfiltered without error. Step 2's
  checkbox was ticked on the automated criteria (vitest + `mise run ci`).
