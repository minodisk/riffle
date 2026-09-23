# Learnings

## Step 1

- The plan's `ALTER TABLE files ADD COLUMN extractor` guard,
  `version != 0 && version < 12`, would also fire for v2-v9, where `files` has
  just been dropped and recreated *with* the column. The `ALTER` would then fail
  with a duplicate column, and `prepare` would discard the whole cache,
  dirty ratings included. The guard is keyed as `(10..12).contains(&version)`
  instead, with a comment. The v3/v6/v7/v8/v9 migration tests cover it.
- The v11 `ratings.flag` migration was guarded by `version != SCHEMA_VERSION`,
  which is the stranded-guard pitfall from `docs/agents/tauri-app.md`: bumping
  to 12 would have made it fire for v11 databases that already have `flag`.
  It is now keyed to `version < 11`. Every `SCHEMA_VERSION` bump needs a sweep
  of all `!= SCHEMA_VERSION` guards, not just the new one.
- A migration test fixture built with `open` gets the *current* schema, so a
  test that fakes an older version must also remove every column added since
  then. The v10 fixture now drops `files.extractor` too. Otherwise the in-place
  `ALTER` fails, the cache is discarded, and the "files are kept" assertion
  fails.
- An interrupted session left a truncated test binary in
  `target/debug/deps/` ("Exec format error"). Deleting it and rebuilding fixed
  it.
- Rule of thumb, for Step 2's guide text: bump `EXTRACTOR_VERSION` when what
  `riffle_core::scan::extract` produces changes (`arw.rs`, `scan.rs`,
  `sharpness.rs`, `faces.rs`). Bump `SCHEMA_VERSION` only when the table
  layout changes. An extractor bump re-extracts every row, error rows
  included, on the next scan of each folder, and keeps `ratings`.

## Step 2

- Documentation only. The new `docs/agents/tauri-app.md` section and the
  additions to the stranded-guard section cite the plan's learnings under
  `docs/plans/_archived/...`, the path it will have after the wrap-up move, in
  code spans (not links), so lychee does not check them before the move.
