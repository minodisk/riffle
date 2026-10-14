# Learnings: per-file-viewed-ranking

## Step 1: Remove Culling Finished and fit on per-file views

- The new age-rule exemption broke an existing test: `eviction_keeps_the_views_and_clear_drops_them`
  evicted an old folder holding a `views` row through the age rule, which now spares it. It evicts
  through the size cap (`max_bytes: 0`) instead, which still shows that eviction keeps `views`.
- `Index::set_rating` always writes `dirty = 1`, and `evict_folder` keeps dirty `ratings` rows, so a
  test fixture that needs a reject the eviction can drop has to insert a clean `ratings` row with SQL.
- `Index::clear` only walks the `folders` rows: a `write_batch` without a `folders` row (no
  `reconcile` / `set_opened_at`) survives a `clear` in a test.
- The `RANK_DIRS` SQL fragment is a `const &str` shared by `rank_dirs` (`{RANK_DIRS} ORDER BY dir`)
  and `evict` (`dir IN ({RANK_DIRS})`), so the training set and the age exemption cannot drift.
- `refit_in_background` lost its last caller with `set_culling_finished` / `finish_culling`, so it
  was removed here to keep clippy clean; Step 2 reintroduces a background refit for its triggers.
- Between this step and Step 2 nothing refits automatically; `usage.md` says `Refit Ranking` is the
  refit for now, and Step 2 rewrites that sentence.
- Git Bash refused one long `python - <<'EOF'` heredoc ("unexpected EOF while looking for matching
  `'`"); writing the script to the scratchpad and running it worked.

## Step 2: Refit at startup and on folder switch

- Coalescing over skipping: `AppRank.queue` (a `Queue { running, pending }`) runs one background
  fit at a time and turns every request made while it runs into one rerun after it, so the
  judgments made during a fit are never left out for longer than one more fit. The state machine
  (`request` / `finish`) is pure, so it is unit-tested without threads. The rerun checks
  `aiEnabled` again and skips the fit when it was turned off meanwhile.
- The settings' `Refit Ranking` still calls `refit` directly; `AppRank.fit` already serializes it
  against a background fit, so it never runs two fits in parallel either.
- The switch rule lives in `viewed.ts` as `switchesFolder(previous, next, ignoreCase)` on top of
  `tree.ts`'s `relation` (so a case-only difference on macOS / Windows is the same folder);
  `openDirectory` checks it against `openDir` just before reassigning it. The reopens through
  `openDirectory(openDir, …)` (`sidecar-format`, `index-cleared`) therefore do not refit.
- A rename of the open folder reopens it under its new path, which `switchesFolder` counts as a
  switch and so refits once. Harmless (the fit is seconds at most), left as is.
- The startup refit is kicked from `setup` right after `AppRank` is managed; `ranking-updated` may
  fire before the main window registers its listener, which is fine since `ranking_of` reads the
  stored weights whenever the frontend asks.
- A stray `cat > file` with no heredoc in a chained Bash command blocked on stdin until the 120 s
  timeout; kill the `cat` and remove the empty file it created.

## Deferred issues (todo candidates)

- Pending manual check (Step 2, Windows 11, already added to `todo.md` under "App: real-device
  check of the personal ranking" as the plan's Done-when asks; the step's checkbox was ticked on
  the automated criteria only): with AI analysis on, judge frames in folder A, open folder B
  without `Refit Ranking` -> a `ranking fit:` line in `Riffle.log` and A's Recommended order
  changed on reopen; restart Riffle -> a fit line at startup; with AI analysis off, start and move
  between folders -> no fit line. Files: `crates/app/src/rank.rs` (`refit_in_background`),
  `crates/app/ui/src/viewed.ts` (`switchesFolder`), `crates/app/ui/src/main.ts`
  (`openDirectory`).
