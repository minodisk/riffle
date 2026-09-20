# Learnings

## Step 1

- Chose the tuple callback `Fn(usize, usize, Vec<String>) + Send + Sync` over a
  `ScanProgress` struct; clippy did not complain about the arity.
- `flush` returns early when `write_batch` fails, so a failed batch's paths
  never reach `ready`. The `errors` accounting in that branch is unchanged.
- `IndexedFile` has no `failed` field; an extraction error shows up as
  `exif: None` (the `error IS NOT NULL` column), so the bad-fixture test
  asserts on that.
- `summary.total` now reads a `done_count` captured before the final
  `progress` call, so the unconditional last notification and the summary
  report the same number.

## Step 2

- `Progress` in `crates/app/src/commands.rs` gained `ready: Vec<String>`; the
  callback's list is moved straight into the payload, so no clone.
- `strip.ts` keeps `indexOf: Map<string, number>` rebuilt in `setFiles`
  (alongside the other per-folder resets) and a `ready: Set<number>`. The new
  export is `markReady(paths)` (`ready` was already taken by the set).
- The in-flight race is closed in `request`'s `.catch`: `requested.delete` runs
  as before, and the `ready` bypass is one-shot (`!ready.delete(index)`), so a
  failure right after the scan reports the path goes unmarked once and
  `finally`'s `pump()` retries it, but a repeat failure on the same index (the
  row is missing, the DB query keeps failing, etc.) settles into `missing`
  instead of spinning forever on the IPC channel. Found in review round 1.
- The stale comments on `REFRESH_INTERVAL` and `missing` ("the re-request
  cannot be narrowed") were rewritten; `REFRESH_INTERVAL` itself is untouched,
  it goes in Step 3.

## Deferred issues (todo candidates)

- Manual GUI check of this step is outstanding: `mise run tauri:dev` on a
  folder with a cold index, confirming thumbnails fill in while the scan runs
  (not only at `scan-done`) and that devtools shows no burst of `thumbnail`
  invokes per `scan-progress` beyond the newly ready cells, plus the observed
  payload sizes. Basis: Step 2's "Done when" hand check; this session has no
  GUI access. Files: `crates/app/ui/src/strip.ts`, `crates/app/ui/src/main.ts`,
  `crates/app/src/commands.rs`.
