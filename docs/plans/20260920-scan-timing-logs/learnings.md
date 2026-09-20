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

## Step 2

- Wording: `open list` (`list_arw`) and `open entries` (`folder_entries`),
  both `log::info!`, both ending in `in {n}ms` as Step 1 does.
- `list_arw` now canonicalizes into a local so the line can name the same
  directory string the other lines use.
- Log chattiness: **chose (b), raising `max_file_size`** to 1_000_000 on the
  log plugin builder in `main.rs`. The numbers: `refreshEntries`
  (`crates/app/ui/src/main.ts`) keeps at most one `folder_entries` invoke in
  flight and is driven by `scan-progress`, capped by `PROGRESS_INTERVAL` =
  100ms, so a ~30s scan of 5000 files yields up to ~300 `open entries` lines.
  With the plugin's prefix (`[date][time][riffle_app::commands][INFO] `) and a
  Windows path, one line is ~120 bytes: ~36 KB, which on its own all but fills
  the 40 KB `DEFAULT_MAX_FILE_SIZE` and would rotate the `scan` lines of the
  same run out of the current file.
- Option (a) (log only when the folder is fully indexed) was rejected:
  `folder_entries` does not know the listed count — only `scan_folder` does —
  so it would need cross-command state, which is more machinery than the
  measurement warrants. Raising the cap also keeps the mid-scan rows=N
  progression, which shows how fast the index fills.

## Deferred issues (todo candidates)

- (none)
