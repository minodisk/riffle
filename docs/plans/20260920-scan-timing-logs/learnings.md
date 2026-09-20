# Learnings

## Step 1

- Duration format: **milliseconds** (`elapsed().as_millis()`), written as
  `... in {n}ms` at the end of every line. Step 2 must use the same so the
  lines are comparable and greppable.
- Line wording: `scan list` / `scan reconcile` / `scan sidecars` /
  `scan prepare` (the `scan_folder` summary) in `commands.rs`, and
  `scan extract` in `index::run_scan`. Step 2 uses `open ...` wording.
- `start_scan`: **skipped**, no wall-time line of its own. Its body is a
  `spawn_blocking` around `index::run_scan` plus two `emit` calls, so its wall
  time is `run_scan`'s time plus event emission; `run_scan` already logs the
  elapsed time together with `files` / `done` / `errors` / `threads` /
  `cancelled`, which is strictly more informative. A second, nearly identical
  number would only make the log longer (the 40 KB rotation budget) and invite
  mis-reading.
- `run_scan` now builds the `ScanSummary` into a local before returning it so
  the log line can read `total` / `errors` from it. The counts and the
  batching are unchanged.
- The sidecar phase logs after the `match dirty`, using a `dirty_count`
  captured in the `Ok` arm (the `Vec` is consumed by the writer loop), so a
  failed sidecar reconcile still produces a timing line with `dirty=0`.

## Deferred issues (todo candidates)

- (none)
