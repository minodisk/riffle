# Learnings

## Step 1

- `sort.ts` compares file names with JavaScript `<` (UTF-16 code units). This
  matches `Path::file_name` byte order for every name without characters
  outside the BMP, which covers camera file names.
- `subsec` is compared by right-padding the shorter string with `"0"`, so
  `"5"` (.5) sorts after `"122"` (.122); a missing `subsec` counts as `.0`.
- A rating of `0` is treated as unrated, like an absent entry.

## Step 2

- `IndexedFile` in `main.ts` did not declare `capture_time` / `subsec`,
  contrary to the plan; the backend already serialises them, so only the
  TypeScript interface gained the two fields.
- The filter and sort dropdowns sit in a new `#tools` flex row; the sort menu
  reuses the filter menu's rules by adding `#sort-*` selectors beside them.
- `openDirectory()` now clears `entries` / `ratings` before building `files`,
  so the first order uses no stale facts from the previous folder.
- First-scan re-sort cost (reasoned, not measured in a GUI):
  `scan-progress` only calls `refreshEntries()` while the current file's row
  is still missing, and `refreshEntries()` keeps one read in flight, so the
  re-order (and its `strip.setFiles`) happens a handful of times per first
  scan, then once more on `scan-done` — not 10/s. `refilter()` skips
  `setFiles` when the order is unchanged. Judged tolerable; no scan-done-only
  fallback, so the README should say the order settles while a folder is
  scanned for the first time.
- Manual GUI confirmation (rollover / two-body folder, rating grouping,
  paging / click / `n / N` following the order, filter keeping the order,
  judging under `Rating` keeping the cursor) is left for the user.

## Step 3

- The stored value is parsed by `parse_sort_order` in `commands.rs` (unit
  tested); `set_sort_order` normalises through it too, so the store never
  holds an unknown value.
- `main.ts` awaits `sort_order` (errors ignored) before `reopenLastFolder()`,
  so the reopened folder is ordered once, with no re-sort flash.
- The capability file does not list commands; only `main.rs` needed the two
  new handlers.

## Step 4

- Closed todo sections are removed outright (as in the last wrap-up commit).
- No new pitfalls for `docs/agents/tauri-app.md`; CLAUDE.md untouched (no new Rust file).
