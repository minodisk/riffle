<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Carry the flushed paths in `scan-progress`

## Purpose

`scan-progress` carries only `{dir, scan_id, done, total}` and arrives ~10/s
during a scan. Because it does not say which files became readable, `refresh`
in `crates/app/ui/src/strip.ts` re-requests every visible not-yet-fetched cell
through the `thumbnail` command on every event and treats an `Err` as "not
yet". That is bounded by `MAX_IN_FLIGHT = 4` and throttled to one run per
`REFRESH_INTERVAL = 1000`, which is avoidable IPC and makes thumbnails appear
up to a second late while the scan runs.

A done-index high-water mark does not work: `run_scan`
(`crates/app/src/index.rs`) drives `extract_all` with `scan_threads()` rayon
workers, `done` is an unordered `AtomicUsize`, and rows land in the index in
`BATCH`-sized transactions, so a file counted in `done` is not necessarily
answerable by `thumbnail` yet. The only thing the backend can promise is
"these paths have been committed to the index since the previous emit". This
plan makes `run_scan` report exactly that, carries it in the `scan-progress`
payload as `ready`, and narrows the strip to request only those cells. The
`scan-done` refresh stays as the authoritative catch-all, and the existing
`Err` = "not yet" path stays as the fallback for the watcher-rescan and
folder-switch paths.

Closes two `todo.md` items: "App: the filmstrip re-requests every visible
placeholder on each `scan-progress` event" and "App: `scan-progress` carries
no way to tell what became available".

## Decisions settled before implementation

These three were put to the user at approval time and are settled; do not
revisit them mid-implementation. The reasoning is kept in Trade-offs below.

- **No cap on the `ready` array**, and no `ready: null` fallback field.
- **Full paths**, not names relative to `dir`.
- **`REFRESH_INTERVAL` is removed entirely** in Step 3, not shortened.

## Steps

- [x] Step 1: Make `run_scan` report the paths each flush committed
  - Done when:
    - The `progress` callback of `index::run_scan` receives, besides
      `(done, total)`, the paths whose rows `write_batch` committed since the
      previous callback (both successful extractions and extraction errors,
      since an error row is also authoritative: `Index::thumbnail` answers
      "no cached thumbnail" for it). Paths of a batch whose `write_batch`
      failed are not reported.
    - The trailing flush after `extract_all` (the last partial batch, on a
      normal finish and on cancel alike) is covered by a callback, so no
      committed path is ever left unreported.
    - Tests in `index::tests`, in the style of
      `a_scan_writes_every_file_and_reports_progress`:
      - concatenating the reported path lists over all callbacks equals the
        set of files the scan wrote (`entries(dir)`), with no path reported
        twice and none missing, for a file count that is not a multiple of
        `BATCH` and with `threads = 2`;
      - a scan where some files fail extraction (a bad fixture) still reports
        the failed files' paths (their rows exist with `error` set);
      - the cancel test (`cancelling_after_the_first_batch_keeps_what_was_written`)
        still passes, and the paths reported up to the end equal the rows
        persisted.
    - `cargo test -p riffle-app` and `mise run ci` pass.
  - Implementation approach (as far as it is known; omit if unknown):
    - Keep the existing shape of `run_scan`: add a `Mutex<Vec<String>>`
      (`ready`, say) next to `pending`; `flush` pushes the batch's paths into
      it only after `write_batch` returned `Ok`. The path string should be
      the same form `write_batch` stores (`file.path.to_string_lossy()`), so
      it matches what `list_arw` hands the frontend.
    - When `on_item` decides an emit is `due`, `std::mem::take` the `ready`
      vector under its lock and pass it to `progress`. Because `flush` runs
      before the `done` increment in `on_item`, a full batch is always
      handed over by the callback that follows it or a later one.
    - Move the "always on the last file" emit: drop the `done == total`
      special case inside `on_item` and instead call `progress` once after
      the trailing `flush(std::mem::take(&mut *lock(&pending)))`, with the
      final `done` and whatever `ready` still holds. This one unconditional
      final call covers both the normal end and a cancel. Keep the
      "always on the first file" behaviour (the `last.is_none()` branch) as
      is. Update the doc comment above `run_scan`.
    - Callback signature: `P: Fn(usize, usize, Vec<String>) + Send + Sync`
      (or a small `ScanProgress { done, total, ready }` struct if clippy
      complains about the arity; either is fine, pick one and use it in both
      call sites). The only other caller is `start_scan` in `commands.rs`;
      adapt it minimally in this step (ignore the third argument) so the
      crate compiles, and leave the payload change to Step 2.
    - Do not change `BATCH` or `PROGRESS_INTERVAL`.
    - Commit as `feat(app): report the flushed paths from run_scan` or
      similar.

- [ ] Step 2: Carry `ready` in `scan-progress` and request only those cells
  - Done when:
    - The `scan-progress` payload is `{dir, scan_id, done, total, ready: string[]}`
      where `ready` is the list Step 1's callback handed over (full paths).
    - `strip.ts` exports a new entry point (e.g. `ready(paths: string[])`)
      that maps each path to its index in the strip's current `files` array
      (paths not in the current list, e.g. filtered out, are ignored),
      removes those indices from `missing`, and calls `pump()`. Cells that
      are on screen and were "not yet" are therefore requested at once; cells
      not on screen are simply no longer marked `missing`, and are requested
      by `render` when scrolled into view as today.
    - The `scan-progress` listener in `main.ts` calls the new entry point
      with `payload.ready` instead of `strip.refresh()`. The `refreshEntries`
      call in the same listener stays as it is.
    - The `scan-done` listener still calls `strip.refresh()` (authoritative
      re-request of everything still `missing`), and the watcher-rescan
      (`resync` -> `startScan`) and folder-switch (`setFiles`) paths behave
      as before.
    - A request that was already in flight when its path was reported ready
      and comes back `Err` (the query ran before the batch committed) is not
      stuck in `missing` until `scan-done`: it is retried by the next `pump`.
    - Verified by hand in `mise run tauri:dev` on a folder with a cold index:
      thumbnails fill in while the scan runs (not only at `scan-done`), and
      the log / devtools show no burst of `thumbnail` invokes per event
      beyond the newly ready cells. Note the observed behaviour in
      `learnings.md`.
    - `mise run ci` passes.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Step 1 is merged.
    - `commands.rs`: add `ready: Vec<String>` (or `&[String]`) to `Progress`
      and pass the callback's list through in `start_scan`. Update the
      `scan-progress` doc comments on `scan_folder` / `start_scan` if they
      describe the payload.
    - `strip.ts`: on `setFiles`, build a `Map<string, number>` from path to
      index alongside `files` (the strip's order is the UI's sorted and
      filtered order, so never assume positional correspondence with the
      scan's `todo`). Clear it and any new set in `setFiles` like the other
      per-folder state. The `generation` guard already drops late responses
      of an old folder, so `ready` needs no extra scan-id logic (the listener
      in `main.ts` already checks `scan_id`).
    - For the in-flight race: keep a `ready: Set<number>` (indices the scan
      has reported committed). In `request`'s `.catch`, add to `missing` only
      when the index is not in that set; then `finally`'s `pump()` picks it
      up again (it is in neither `requested`, `inFlightIndices`, `missing`
      nor `failed`). A genuine "no cached thumbnail" still goes to `failed`
      regardless. Keep this to the minimum that closes the race; do not add
      retry counters.
    - Do not touch `REFRESH_INTERVAL` in this step; `refresh` is still the
      `scan-done` path and the throttle is removed separately in Step 3.
      The two comments on `REFRESH_INTERVAL` and `missing` that say the
      re-request "cannot be narrowed" become wrong here; fix those comments
      in this step.
    - Payload size: no cap and no fallback field (settled; see Decisions).
      Each path is sent exactly once across the whole scan, so the total is
      the sum of path lengths (~0.5 MB for 5000 files at ~100 bytes each),
      and at the measured ~1000 files/s and 10 emits/s a single event holds
      ~100 paths (~10 KB). If manual verification in this step shows an
      event visibly stalling the webview, record the numbers in
      `learnings.md` and raise it with the user rather than adding a cap
      unilaterally.
    - Frontend tests: `strip.ts` and `main.ts` are not importable under the
      `environment: "node"` vitest setup (see the xmp-pick-gate plan), and
      the path->index mapping is one `Map` lookup, so no new `*.test.ts` is
      required; add one only if a pure helper falls out naturally.
    - Commit as `feat(app): request only the thumbnails scan-progress reports ready`
      or similar.

- [ ] Step 3: Drop the `refresh` throttle now that progress no longer drives it
  - Done when:
    - `REFRESH_INTERVAL`, `lastRefresh` and `refreshTimer` are removed from
      `strip.ts` and `refresh()` runs immediately (clear `missing`, `pump()`).
      The `setFiles` reset no longer needs the timer bookkeeping.
    - `refresh` is called only from `scan-done` (verify with a grep; if
      Step 2 left any other caller, report it rather than silently keeping a
      throttle, and note it in `learnings.md`).
    - Manual check in `mise run tauri:dev`: `scan-done` on a large folder
      does not visibly starve the IPC channel (it issues at most
      `MAX_IN_FLIGHT` invokes for the still-missing visible cells, once).
    - The comment in `crates/app/src/main.rs` on `max_file_size` that says
      `folder_entries` is refreshed "on every `scan-progress` event" is
      checked against the current `refreshEntries` gating; correct it only
      if it is wrong (it describes `folder_entries`, not the strip, so it
      may already be accurate).
    - `mise run ci` passes.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Step 2 is merged. Small, self-contained PR so it can be
      reverted alone if the throttle turns out to still be needed.
    - Full removal, not a shortened interval (settled; see Decisions).
    - Commit as `refactor(app): drop the filmstrip refresh throttle` or
      similar.

- [ ] Step 4: Close the todos and record the design
  - Done when:
    - The two headings and their TODO lists are removed from `todo.md`:
      "App: the filmstrip re-requests every visible placeholder on each
      `scan-progress` event" and "App: `scan-progress` carries no way to tell
      what became available". If the first item's remark about reading
      `has_thumb` from `folder_entries` is still a live idea, fold it into the
      residual note rather than keeping a heading with no TODO.
    - `docs/agents/tauri-app.md` gains a short entry (tagged Measured or
      Inferred, following the file's format) stating why the payload carries
      flushed paths rather than a done-index high-water mark (unordered
      rayon completion + batched flush), and that a path is reported only
      once its transaction committed, so the trailing flush is covered by a
      final callback. Place it near the existing `scan-progress` log entry.
    - `learnings.md` in this plan folder holds the measured payload sizes and
      anything the manual checks in Steps 2 and 3 turned up.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Steps 1-3 are merged. Docs-only PR (`docs(app): ...`). Do not
      do the archive move or the auto memory update here; that is the
      wrap-up.

## Trade-offs and risks

- **No cap on `ready` vs. a cap with fallback.** Settled: no cap. A path is
  reported exactly once over a scan, so total bytes are bounded by the
  folder's path lengths, not by the emit rate; per event it is throughput /
  10, ~100 paths at the measured 5000 files in 5.55 s. A cap (e.g. 500 paths,
  or `ready: null` meaning "too many, re-request all") would keep the old
  behaviour as a runtime fallback but adds a second code path in `strip.ts`
  that is hard to exercise.
- **Full paths vs. names relative to `dir`.** Settled: full paths. They match
  `list_arw`'s strings exactly, so the frontend map is a plain lookup.
  Relative names would need a join that has to agree with the backend's
  separator handling on Windows.
- **Including error rows in `ready`.** Reporting a failed extraction's path
  makes the strip request it and receive the definitive "no cached
  thumbnail", so the cell shows `failed` during the scan instead of at
  `scan-done`. Excluding them would be a smaller list but would leave those
  cells in `missing` until the end. Chosen: include, since "ready" means
  "the index can answer", not "has a thumbnail".
- **Removing the throttle (Step 3) vs. shortening it.** Settled: remove.
  With progress no longer calling `refresh`, the throttle's only caller fires
  once per scan, so an interval adds latency for nothing. The risk is a path
  that still calls `refresh` repeatedly (none found; `resync` goes through
  `setFiles`), which is why Step 3 re-checks with a grep.
- **Callback signature change in `run_scan`.** Adding a third argument is a
  breaking change for the two call sites (`start_scan` and the tests); a
  struct is slightly more ceremony but future-proof. Either is acceptable;
  Step 1 decides.
- **Moving the final emit after the trailing flush** changes when the last
  `scan-progress` fires relative to the last `on_item`; the frontend does not
  depend on that ordering (it only compares `scan_id` and then `scan-done`
  follows immediately), and the existing `progress.last() == (8, 8)` test
  still holds. On cancel the final callback reports `done < total`, which is
  what today's behaviour already does through the throttled emits.
- **In-flight race in `strip.ts`.** Without the `ready` set, a request that
  was answered `Err` just before its batch committed would be stuck until
  `scan-done`; with it, the retry is one extra `thumbnail` invoke for that
  cell. This is the only place the plan adds state beyond the map, and it
  should stay that small.

## Progress

- (none yet)
