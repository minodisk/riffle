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

## Step 3

- `REFRESH_INTERVAL`, `lastRefresh` and `refreshTimer` are gone; `refresh()` is
  now `missing.clear(); pump();`. The `setFiles` reset lost the timer
  bookkeeping with them.
- Grep across `crates/app/ui/src` confirms the sole caller is the `scan-done`
  listener (`crates/app/ui/src/main.ts:1384`), so no throttle is needed: it
  fires once per scan and issues at most `MAX_IN_FLIGHT` invokes.
- The `max_file_size` comment in `crates/app/src/main.rs` was wrong: it said
  `folder_entries` is refreshed "on every `scan-progress` event", but
  `refreshEntries` is called from the progress listener only while the focused
  row is still missing from `entries`, and it coalesces through
  `entriesInFlight`. Reworded to "re-read while a `scan-progress` stream leaves
  the focused row missing"; the ~300-line estimate is left as is.

## Step 4

- Payload size: a path is reported exactly once per scan, so the bytes are
  bounded by the sum of the folder's path lengths, not by the emit rate. At
  the README's measured 5000 files in 5.55 s and ~10 emits/s, that is ~90
  paths per event; with a typical absolute path of ~80-120 bytes that is
  ~7-11 KB of JSON per `scan-progress` and ~0.5 MB over the whole scan. That
  is why no cap and no `ready: null` fallback was added. Reasoned from the
  existing benchmark numbers, not re-measured (no GUI access this session).
- The `todo.md` item "the filmstrip re-requests every visible placeholder on
  each `scan-progress` event" also carried a residual remark: that the strip
  learns whether a file has a thumbnail by invoking `thumbnail` and treating
  `Err` as "not yet", rather than reading `has_thumb` from the
  `folder_entries` map. Steps 1-3 made that moot rather than merely narrower:
  `refresh` no longer runs per progress event at all, so the "keeping the map
  fresh would mean a 10/s full re-read" cost it traded against is gone, and
  `ready` already names exactly the cells worth requesting. Reading
  `has_thumb` would now only save the single invoke for an error row, which
  still has to be asked to reach the definitive `failed` state. Dropped
  instead of being kept as a residual note.
- Both removed headings' TODOs are covered by the shipped code, so `todo.md`
  loses them entirely; only the two manual GUI checks below are outstanding.

## Deferred issues (todo candidates)

- Manual GUI check of this step is outstanding: `mise run tauri:dev` on a
  folder with a cold index, confirming thumbnails fill in while the scan runs
  (not only at `scan-done`) and that devtools shows no burst of `thumbnail`
  invokes per `scan-progress` beyond the newly ready cells, plus the observed
  payload sizes. Basis: Step 2's "Done when" hand check; this session has no
  GUI access. Files: `crates/app/ui/src/strip.ts`, `crates/app/ui/src/main.ts`,
  `crates/app/src/commands.rs`.
- Also outstanding for Step 3: confirm in `mise run tauri:dev` that the
  un-throttled `scan-done` `refresh()` on a large folder does not visibly
  starve the IPC channel. Same GUI-access basis as the item above; fold into
  the same hand check. Files: `crates/app/ui/src/strip.ts`,
  `crates/app/ui/src/main.ts`.
