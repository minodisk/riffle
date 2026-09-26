# Learnings

## Step 1

- Migration guard: `folders` was introduced at v8 and a v2 to v7 database
  gets it from the `CREATE TABLE IF NOT EXISTS`, already carrying
  `last_viewed`. So the `ALTER TABLE folders ADD COLUMN last_viewed` guard is
  `(8..17).contains(&version)`, not `version < 17`, which would add a
  duplicate column to a v2-v7 database and discard the cache. The other
  guards needed no change: every one is keyed to a fixed range or a single
  version. The accepted-versions list in `prepare` needed `16` added, and
  every migration test's `assert_eq!(version, 16)` moved to 17.
- The test helper `set_opened_at` writes with `INSERT OR REPLACE`, which
  deletes and reinserts the row and so resets `last_viewed` to `NULL`. A test
  that combines the two has to call `set_opened_at` first.
- `anchorAfterFilter` took `string[]`; `resumeTarget` takes
  `readonly string[]` as planned, so the parameter was widened to
  `readonly string[]` (no caller changes).
- `last_viewed` reads through `AppIndexReader` (the read-only WAL
  connection), so it does not wait for the writer lock a running scan's
  `write_batch` holds; only `set_last_viewed` does, off the main thread.
- The first `mise run ci` failed 8 migration tests (v8 to v15): their
  fixtures are built with `open`, so they now carry `last_viewed`, and the
  new guard's `ADD COLUMN` failed on the duplicate, which discarded the
  whole cache. Each of those fixtures now also runs
  `ALTER TABLE folders DROP COLUMN last_viewed;` (the "faking an older
  version means dropping every column added since" rule in
  `docs/agents/tauri-app.md`). A pnpm install was needed first for
  `mise run fmt` in this fresh worktree (`vp` not found).
