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

# Folder open: one directory listing, and a measured `refreshEntries`

## Purpose

Two follow-ups left open by docs/plans/_archived/20260925-folder-open-off-main-thread (todo.md, "Cross-cutting / other"):

1. Opening a folder lists the same directory twice: `openDirectory` /
   `resync` in `crates/app/ui/src/main.ts` invoke `list_arw`, then
   `startScan` invokes `scan_folder`, whose `list_folder_in` reads the
   directory again (both go through `list_dir` in
   `crates/app/src/commands.rs`). Under disk contention (a scan reading the
   same drive) each listing is the slow part of an open. After this work
   `scan_folder` reuses the listing `list_arw` just produced and only lists
   on its own when it has nothing fresh to reuse.
2. `refreshEntries` (`main.ts` ~1110) re-reads every indexed row of the
   folder and redoes all per-row derived state (entries/ratings,
   `groupBursts`, `rebuildExifMenu`, `renderMeta`, `draw`, `applySharpness`,
   `applyBursts`, `refilter`). It runs three times on every open or resync of
   an already-indexed folder (from `openDirectory`, `scan-done` and
   `faces-done`), and during a scan it re-runs on every `scan-progress` tick
   while the current file's row is missing. Its cost has never been
   measured. After this work its total and sub-phases land in `Riffle.log`
   when timing logs are on, and the redundant runs that code reading
   already shows are gone, with no visible change.

The two items are independent (backend vs frontend), so they are two steps
and two PRs.

## Steps

- [x] Step 1: Reuse `list_arw`'s listing in `scan_folder`
  - Done when:
    - Opening a folder (tree click, picker, drop, reopen on
      `sidecar-format` / `index-cleared`, last-folder reopen) and `resync`
      read the opened directory with `read_dir` once on the
      `list_arw` -> `scan_folder` path. `Riffle.log` shows it: the
      `scan list:` line reports that the listing was reused (e.g.
      `reused=true`) instead of a fresh listing time.
    - `scan_folder` still lists on its own when there is no reusable
      listing (a different canonical dir, a different sidecar format, a
      changed directory mtime, or a listing already consumed), so a
      `scan_folder` without a preceding `list_arw` keeps working. The
      `scan list:` line says `reused=false` in that case.
    - The stale-listing window is closed with a directory-mtime check (see
      approach), documented in the doc comment of the cache and in
      `learnings.md`.
    - `commands.rs` unit tests cover the reuse decision as a pure function
      (same dir + same format + same mtime -> reused once, then gone;
      different dir -> not reused; different format -> not reused; changed
      directory mtime -> not reused). The existing listing tests
      (`one_listing_finds_the_raw_files_and_the_sidecars_of_each_format`
      and the others around `commands.rs:3022-3104`) still pass.
    - `mise run ci` passes. Manual check for the user (cannot be automated
      here): with timing logs on, open a large folder from the tree and
      confirm one `open list:` line and one `scan list: ... reused=true`
      line per open; then add a RAW to the open folder and confirm the
      watcher's resync picks it up.
  - Implementation approach (as far as it is known):
    - Keep the frontend call sequence as it is (`list_arw` then
      `scan_folder`); the frontend needs `allFiles` before `scan_folder`
      returns, because `scan_folder` first joins the previous scan (seconds
      after a cancel). Share the listing on the backend instead: a managed
      state in `commands.rs` (e.g. `pub struct AppListing(Mutex<Option<Listing>>)`,
      registered with `app.manage` in `crates/app/src/main.rs` next to
      `Scans`), written by `list_arw` and `take()`n by `scan_folder`.
    - `list_arw` gains an `app: tauri::AppHandle` parameter (the frontend
      invoke arguments `{ dir }` do not change) and reads the current
      `AppSidecarFormat` so the one listing also keeps the sidecar entries;
      everything stays inside its `spawn_blocking` closure as PR #449 left
      it. Keep the `open list:` log line format.
    - Do not move the per-sidecar `index::stat` into `list_arw`: PR #449
      removed a per-entry stat from this exact path because it costs seconds
      under contention, and `list_arw` is what the first paint waits on.
      Split `list_dir` so the cached `Listing` holds the sorted RAW paths and
      the sidecar entries' `(lower-cased name, path)` without stats, and
      `scan_folder` stats the sidecars (the `SidecarStat` map
      `reconcile_sidecars_of` takes) after taking the listing, in the
      `spawn_blocking` block that lists today. `list_folder_in` can stay as
      the composition of the two for the tests and the fallback path.
    - Key the cached listing by the canonical dir (`canonicalize`, which
      `list_arw` already computes), the format it was listed for, and the
      directory's mtime (`std::fs::metadata(dir).modified()`) taken with the
      listing; `scan_folder` stats the directory once and takes the listing
      only when all three match, otherwise it re-lists. Snapshot the
      decision, not the lock, across the `spawn_blocking` (guide: "Snapshot
      the state you decided on, not just the lock").
    - Staleness: `scan_folder` sets the watcher before it joins the previous
      scan and lists (`watch::set`, `commands.rs:1021`), so a file added
      after that point already triggers `folder-changed` -> `resync`. The
      window a reused listing adds is between `list_arw`'s `read_dir` and
      `watch::set`; the mtime check closes it for one `stat`. Its only
      weakness is coarse mtime resolution on FAT-like file systems, which the
      watcher covers.
    - Log the reuse on the existing `scan list:` line rather than adding a
      new line, so the open/scan log stays one line per phase.
    - Out of scope: the tree's `list_subfolders` of the opened folder from
      `folders.reveal` (its RAW-count badge needs its own `read_dir`; sharing
      it would couple the tree to the open path) and the per-RAW
      `index::stat` in `reconcile` (needed for the mtime/size diff).

- [x] Step 2: Time `refreshEntries` and drop its redundant runs
  - Done when:
    - With timing logs on, each `refreshEntries` run writes one line to
      `Riffle.log` through `debugLog` -> `log_timing` (never the log
      plugin's JS API, per the guide) with the row count, the
      `folder_entries` invoke time, the time of each sub-phase (entries
      rebuild incl. `applyRating`, `groupBursts`, `rebuildExifMenu`,
      `renderMeta`, `draw`, `applySharpness`, `applyBursts`, `refilter`,
      and whether `refilter` called `strip.setFiles`) and the total. The
      line is cheap enough to leave in the `scan-progress` path: at most
      one per completed read, ~200 bytes, against the 1 MB
      `max_file_size` in `main.rs`.
    - The `scan-progress` handler no longer re-runs `refreshEntries` on
      every tick while the current file's row is missing: it re-runs when
      the tick's `ready` contains the current file, or when the current
      file changed since the last progress-triggered refresh (so a file
      paged to that landed earlier still gets its row), and otherwise waits.
      The visible result (the focus mark appearing once the row lands) is
      unchanged.
    - The `faces-done` refresh is skipped when it cannot have changed
      anything: the pass wrote nothing (`payload.total === 0`) and the
      preceding `scan-done` for the same `scan_id` also had `total === 0`.
      Every other `faces-done` still refreshes (it is what drops
      `faceCache` after a resync that changed files).
    - Any further reduction (e.g. skipping `rebuildExifMenu`'s DOM rebuild
      when the per-group label sets are unchanged) is done only if the
      user's measurement or code reading shows it is a clear cost; a
      skipped one is recorded in `learnings.md` with the number that
      justified skipping it.
    - The decision logic is pure and unit-tested under vitest's `node`
      environment (a small module such as `crates/app/ui/src/refresh.ts`
      with `refresh.test.ts`: the progress-tick decision, and the
      `faces-done` skip decision; the timing line formatter too if it is
      extracted).
    - `mise run ci` passes. Manual check for the user, written out in the
      PR: turn on `Timing logs` in the settings modal, open a folder of
      3000+ RAWs that is already indexed, and read the `refresh entries`
      lines (open, `scan-done`, `faces-done` if not skipped) and the
      backend `open entries:` line next to them; then open a folder that
      needs a full scan and confirm the `refresh entries` lines during the
      scan are one per current-file landing, not one per tick. Report the
      numbers with their conditions (folder size, cold/warm, scan running
      or not) per the guide's "Write a performance number with its
      measurement conditions".
  - Implementation approach (as far as it is known):
    - Timing: `performance.now()` marks inside the `.then` of
      `refreshEntries`, one `debugLog(...)` call at the end; `debugLog`
      joins its arguments with `String`, so a label plus numbers formats
      correctly. Keep the `entriesInFlight` / `entriesPending` coalescing
      exactly as it is; it is what serializes overlapping reads.
    - `refilter` already returns early when the ordered/filtered list is
      unchanged (`main.ts:824`), so `strip.setFiles` is not called on a
      no-op refresh today; the timing line should record whether it was,
      rather than adding a second guard.
    - Progress gating: the `scan-progress` payload's `ready: string[]` lists
      exactly the paths committed since the previous emit (guide:
      "`scan-progress` carries flushed paths"). Track the path the last
      progress-triggered refresh was for (a `let` in `main.ts`, reset in
      `openDirectory` with the other per-folder state) and refresh when
      `!entries.has(current) && (ready.includes(current) || current !== lastRefreshedFor)`.
      Do not touch the `scan-done` / `faces-done` refresh triggers' order.
    - `faces-done` skip: remember the `scan-done` total per `scanId` (a
      `let` next to `scanErrors`), and in the `faces-done` listener call
      `refreshEntries` unless both totals are zero. `setScanRunning(false)`,
      the status update and `drainResync()` must still run in that case.
    - Consistency: the guide's "A derived-state refresh has to run even when
      `refilter` short-circuits" still holds; the sub-phase calls stay in
      the same order, only their timing is added.

## Trade-offs and risks

- Step 1 moves the sidecar entries into `list_arw`'s listing without stats.
  Statting sidecars in `list_arw` would be simpler code but slow the first
  paint by one stat per sidecar, the class of cost PR #449 removed.
- Step 1 alternative not taken: one command returning the RAW list and
  starting the scan. Rejected because `scan_folder` joins the previous scan
  before listing and the frontend must paint the list before that join.
- Step 2 numbers can only come from the user's machine; the PR ships the
  instrumentation and the two code-reading reductions, and records the
  numbers in `learnings.md` when the user reports them.
- Step 2 `faces-done` skip: if a future backend change makes the faces pass
  write rows with `total === 0`, the skip would hide them; the doc comment on
  the skip must state the assumption (a `Done.total` of zero means no
  `write_faces` happened).
- Step 2 progress gating changes when the refresh starts (on the landing tick
  instead of every tick); it never delays the mark beyond one tick after the
  row lands, which is what happens today.
- Out of scope: the tree's third `read_dir` of the opened folder from
  `folders.reveal` (`list_subfolders` for the RAW-count badge).

## Progress

- (2026-09-26) Step 1 complete
- (2026-09-26) Step 2 complete
