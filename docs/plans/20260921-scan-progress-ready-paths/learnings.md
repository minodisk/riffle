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
