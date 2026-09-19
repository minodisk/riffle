# Learnings

## Step 1

- The `mark_written` failure (an index error after a successful disk write)
  keeps its report-only behaviour: the sidecar is already on disk, so retrying
  the write would not help, and the dirty row is reconciled on the next open.
- The retry message passed to `on_error` appends `(retrying in Ns)`; the final
  failure and drain failures pass the bare error, so the status line needed no
  change.
- A drain (`now: None`) skips the requeue via `retry_delay(..).filter(|_|
  now.is_some())`, so `a_failed_write_leaves_the_row_dirty` still returns
  promptly.
- The retry tests take about 1 s each (the first retry delay); both passed well
  inside `eventually`'s 10 s budget locally.

## Deferred issues (todo candidates)

- (none)
