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
