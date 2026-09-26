# Learnings

## Step 1: Reuse `list_arw`'s listing in `scan_folder`

- `list_dir` split into `read_listing` (one `read_dir`: sorted RAW paths plus
  the format's sidecars as `(lower-cased name, path)`, no stats) and
  `stat_sidecars` (the `SidecarStat` map `reconcile_sidecars_of` takes).
  `list_arw` keeps only the unstatted listing, so the first paint pays no
  per-sidecar `stat` (the cost PR #449 removed); `scan_folder` stats the
  sidecars after taking the listing, in the `spawn_blocking` that used to list.
- The cache is `AppListing(Mutex<Option<CachedListing>>)` in `commands.rs`,
  registered next to `Scans` in `main.rs`, keyed by canonical dir, sidecar
  format and the directory's mtime. `take_listing` is the pure reuse decision:
  it takes the listing only on a full match (so it is reused once), and leaves
  a non-matching one in place for the `scan_folder` it may belong to. A failed
  directory stat (`None` mtime) never matches, and `list_arw` caches nothing
  when it cannot read the mtime.
- Staleness: the directory mtime is read just before `list_arw`'s `read_dir`
  (not after), so a file added or removed during or after the listing bumps
  the mtime past the cached value and `scan_folder` re-lists. That closes the
  window between `list_arw`'s `read_dir` and `scan_folder`'s `watch::set`;
  changes after `watch::set` reach the watcher (`folder-changed` -> resync).
  The remaining weakness is a coarse mtime (FAT's 2 s), where a change in the
  same tick can slip past the check. Documented on `AppListing`.
- After the split, `list_arw_in` and `list_folder_in` are only used by tests,
  so both are `#[cfg(test)]` (otherwise the non-test build warns about dead
  code). `list_folder_in` stays as the composition of the two halves.
- The `scan list:` log line gains `reused=true|false`; its `ms` now covers the
  sidecar stats plus, when not reused, the `read_dir`.
- `cargo` is not on the Git Bash PATH here; `mise exec -- cargo test -p
  riffle-app <filter>` works for a quick run.

## Step 2: Time `refreshEntries` and drop its redundant runs

- The decisions and the timing line formatter live in
  `crates/app/ui/src/refresh.ts` (`refreshOnProgress`, `refreshOnFacesDone`,
  `refreshTimingLine`), tested in `refresh.test.ts` under the `node`
  environment. `main.ts` only takes `performance.now()` marks and wires them.
- The timing line is `refresh entries: rows=... invoke=... entries=...
  bursts=... exif=... meta=... draw=... sharpness=... apply_bursts=...
  candidates=... refilter=... set_files=... total=...`, ~190 bytes for a
  4-digit row count (the test asserts < 220). `applyCandidates` runs between
  `applyBursts` and `refilter` today, so it is timed as its own `candidates`
  phase even though the plan's list left it out. `invoke` spans the
  `folder_entries` round trip; `total` spans invoke start to the end of
  `refilter`, so it includes the `.then` scheduling delay.
- `refilter` now returns whether it called `strip.setFiles` (false only on
  its no-op early return), which is how `set_files` is recorded without a
  second guard. Its other callers ignore the return value.
- Progress gating keeps `progressRefreshedFor` (reset in `openDirectory` with
  `scanDone`), set whenever a progress tick triggers a refresh. The first tick
  after an open refreshes (nothing recorded yet), then the handler waits until
  the tick whose `ready` contains the current file, or until the current file
  changes. When a gated refresh coalesces into an in-flight read,
  `entriesPending` still re-runs it, so the landing tick is never lost.
- `faces-done` skip: `scanDone` holds `{ scanId, total }` of the last
  `scan-done`; the skip requires the same `scan_id` and both totals zero.
  `emit_empty_scan_events` (a no-op `start_scan`) produces exactly that pair,
  as does a resync of an unchanged, fully indexed folder. The assumption (a
  `faces-done` total of zero means no `write_faces`) is on the doc comment of
  `refreshOnFacesDone`. `setScanRunning(false)`, the status and
  `drainResync()` still run on a skip.
- Not done: skipping `rebuildExifMenu`'s DOM rebuild when the per-group label
  sets are unchanged. There is no measurement yet (the numbers come from the
  user's machine with `Timing logs` on), and the plan allows it only once a
  measurement shows it is a clear cost; the new `exif=` field of the timing
  line is what would justify it.

## Deferred issues (todo candidates)

- Conditional `rebuildExifMenu` in `refreshEntries` (skip the DOM rebuild
  when the per-group label sets are unchanged). Basis: Step 2 of this plan
  deferred it pending the user's measurement of the `exif=` phase of the new
  `refresh entries:` timing line. Files: `crates/app/ui/src/main.ts`
  (`refreshEntries`, `rebuildExifMenu`).
