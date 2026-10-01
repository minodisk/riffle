# Learnings

## Step 1: the analysis pass in `riffle-core`

- `extract_faces` / `extract_faces_all` were renamed to `extract_analysis` /
  `extract_analysis_all`, returning `Analysis { cue, sharpness }`. The app
  (`crates/app/src/index.rs`, `run_faces_scan`) had to follow the rename in
  this step to keep building, although the plan listed only `scan.rs` and the
  CLI; it reads `.cue.eye_focus` and ignores the score until Step 2. Stale
  mentions of `extract_faces` in `docs/agents/tauri-app.md` and
  `docs/agents/tract-onnx-inference.md` were renamed too.
- To keep the score bit-identical, the whole-image face search and the
  guarded `score_preview` call were pulled out of `extract_unless` into two
  private helpers (`whole_image_faces`, `score`) that both `extract_unless`
  and `extract_analysis_unless` call. Step 2 deletes the call from
  `extract_unless` and the helpers stay with the analysis pass.
- Interim cost until Step 2: the second pass now also scores the rows it
  runs on (those with a trusted AF point; `faces_todo` still marks no-AF rows
  done), about 3 ms per file on top of the cue, and the result is dropped.
- The equality test (`the_analysis_score_is_the_one_extract_gives`) covers
  the no-AF-point branch only (a flat and a textured 64x48 preview): the
  `scan.rs` fixture is a bare TIFF shell with no MakerNote, so it carries no
  AF point. The AF-point branch shares the same `score` helper and passes the
  same `trusted_focus` / `eye_af_frame` / empty faces, so it is identical by
  construction.
- A JPEG file returns `Analysis::default()` before `read_preview`, so a
  missing `.jpg` path is `Ok` (asserted in the JPEG test).

## Step 2: the score moves to the second pass in the app

- `faces_todo`'s JPEG rule is matched in SQL (`path LIKE '%.jpg' OR path
  LIKE '%.jpeg'`); SQLite's `LIKE` ignores ASCII case, so `.JPG` / `.JPEG`
  match too, the way `is_jpeg_file` does. The same `UPDATE` now also clears
  `sharpness`, so an error row or a JPEG row is marked done with neither
  value.
- Step 1's equality test (`the_analysis_score_is_the_one_extract_gives`)
  compared `extract` with `extract_analysis`; with the score gone from
  `extract` it had nothing left to compare and was removed. The app's
  `the_faces_pass_writes_and_reports_every_path` now checks that the first
  pass leaves every score `NULL` and the second fills in exactly
  `extract_analysis`'s score, and the new
  `a_row_the_old_first_pass_scored_keeps_its_score_and_a_new_one_gets_it_from_the_faces_pass`
  covers the no-bump migration.
- `run_faces_scan` now keeps the whole `Analysis` per pending file instead
  of just the probability; an unreadable file is stored as
  `Analysis::default()` (no probability, no score).
- `riffle-cli scan` no longer includes the score (nor the whole-image face
  search for no-AF files) in its per-file time, so its numbers are not
  comparable with the "scan" figures in `docs/performance.md` measured
  before this step; Step 7 rewrites that section.
- The frontend ignores `FaceReady.sharpness` until Step 3; until then the
  scores show at once when `faces-done` re-reads the entries
  (`refreshOnFacesDone`), not file by file.

## Step 3: the score fills in as the second pass runs

- The patching lives in a sibling of `applyFaceReady`,
  `applySharpnessReady(entries, sharpness, ready)` in `focus.ts`, rather than
  in `applyFaceReady` itself: `applyFaceReady` skips a row with no focus
  point, but a no-AF file now gets a score from the pass too. It sets or
  deletes the map entry (even for a path with no row), writes the row's
  `sharpness`, and returns whether any score changed.
- `relativeSharpness` over 5000 synthetic files (a throwaway vitest run,
  capture-ordered, a mix of bursts and singles) took about 4 ms per call. At
  `PROGRESS_INTERVAL` (100 ms) that is ~4% of the main thread while the pass
  runs, so `applySharpness()` is called on every tick that changed a score,
  not throttled or limited to the files in `files`. The real-folder figure is
  still to be read from `refreshTimingLine`'s `sharpness` field.
- The status text while the second pass runs changed from `focus N / M` to
  `analyzing N / M` (a sibling of the first pass's `scanning N / M`, and the
  meta pane's Analysis group holds both values the pass fills in). The focus
  mark and Sharpness cue entries in `docs/usage.md` say so.

## Step 4: the scan's workers below normal OS priority

- `thread-priority` 3.1.1 has no macOS QoS API (its macOS path is
  `pthread_setschedparam` on `SCHED_OTHER`), so it is a Windows-only
  dependency (`WinAPIThreadPriority::BelowNormal` / `Lowest` through
  `set_current_thread_priority(ThreadPriority::Os(level.into()))`) and the
  unix branches call `libc` directly: macOS
  `pthread_set_qos_class_self_np(QOS_CLASS_UTILITY | QOS_CLASS_BACKGROUND, 0)`
  (both exported at the `libc` root in 0.2.189, `qos_class_t` a `#[repr(u32)]`
  enum), Linux (and Android) `setpriority(PRIO_PROCESS, 0, max(current, 5 | 10))`, which sets
  the calling thread's nice alone. The crate's Linux mapping of
  `Crossplatform(0..=99)` onto nice is a truncating float formula, so the
  explicit nice values read better than the magic priority numbers it would
  need. The `windows` 0.62 crate it pulls was already in the lock.
- The priority error travels back as the `Ok(Some(msg))` of `extract_all` /
  `extract_analysis_all` (`Result<Option<String>, String>`): each worker
  sets its priority before `thread.run()` and records the first failure in a
  shared `OnceLock`, read once the pool has finished. The app logs it with
  `log::warn!` once per pass; the CLI passes `Priority::Normal`, which makes
  no call at all. A worker that never started (fewer files than threads)
  reports nothing, but it also ran nothing.
- `every_worker_runs_at_the_priority_it_was_given` reads the level back
  inside `per_file` (Windows `get_current_thread_priority`, Linux
  `getpriority`); macOS has no such check here since this machine cannot
  build it, only the `Ok(None)` assertion of `every_index_is_delivered_once`
  run on CI's macOS job.

## Step 5: the shared work queue and `set_scan_focus`

- `for_each_path` runs one worker loop per pool thread with
  `ThreadPool::broadcast` (rayon 1.12) rather than
  `(0..threads).into_par_iter()`: `broadcast` guarantees each thread runs the
  loop once, while a `par_iter` over the thread count may run two items on
  one thread (the second then finds the queue empty) and leave a thread idle.
  The loop holds the queue's lock only for `take_next`, never across
  `per_file` / `on_item`.
- `WorkQueue` keeps a `taken: Vec<bool>` and a `next` cursor instead of a
  `BTreeSet` of pending indices: a hot take marks its index taken out of
  order and the cursor skips it later, so a take is O(hot list) plus an
  amortized O(1). The hot list lives in `ScanFocus` (an
  `Arc<Mutex<Vec<String>>>`), locked inside the queue's lock; `set` takes only
  the focus lock, so the two never deadlock.
- `ScansState.running` became a 4-tuple (`RunningScan`, with the
  `ScanFocus` third) and `set_scan_focus` goes through
  `ScansState::set_focus`, which the `commands.rs` test drives directly (the
  command itself needs an `AppHandle`); the test reads the handle back by
  running `extract_all` over three missing paths on one thread and checking
  the delivery order.
- The command is a plain (synchronous) `#[tauri::command]`, so it runs on
  the main thread. Several commands hold the `Scans` lock for their whole
  run, all only while no scan is running: `clear_index` (the drain, the
  `VACUUM` and the WAL checkpoint), `trash_rejected_run`,
  `trash_rejected_undo` and `trash_rejected_redo` (moves to and from the OS
  Trash, seconds for a large folder on the Windows Recycle Bin), and
  `spawn_eviction`'s `VACUUM` (~0.2 s). A blocking lock could freeze the UI
  for a call fired just after `faces-done`, so `set_scan_focus` takes the lock
  with `try_lock` and drops the call when it is busy (a busy `Scans` means no
  scan is taking the focus); a poisoned lock is recovered as `index::lock`
  does. The alternative, making the command `async`, was not chosen.
- `capabilities/default.json` lists only core and plugin permissions; the
  app's own commands need no entry, so it is unchanged.
- The first `mise run ci` failed on clippy's `too_many_arguments` (8/7) for
  `run_scan` and `run_faces_scan` once they took the focus handle; they got
  `#[allow(clippy::too_many_arguments)]`, as `sidecar.rs` already does, rather
  than a parameter struct.
- `the_analysis_pass_delivers_every_index_once_and_stops_on_cancel` takes
  about 70-80 s in `mise run ci` here (YuNet over 200 previews); only its
  arguments changed in this step, and its time before the step was not
  measured.

## Step 6: the frontend sends the on-screen files

- `scanFocusPaths(files, current, first, last)` (`scanfocus.ts`) takes the
  strip's range as `strip.visibleRange()` returns it (the visible cells plus
  `RANGE_MARGIN` on each side, the same range `render` keeps cells for, now
  computed in one place). The current file leads even when the user has
  scrolled it out of the range; the range is then ordered by distance from it,
  so it fills from the side nearest the current file.
- `sendScanFocus()` in `main.ts` is called from `show()`, from the strip's
  scroll listener (a new `onScroll` argument of `strip.init`) and once
  `start_scan` resolves: `ScansState.running` (and so the focus handle) is
  set inside `start_scan`, so a call before it resolves would be dropped by
  the id check, and without this call a folder reopened at its remembered
  file would not tell the scan where it is until the user moved.
- The timer reads `scanId`, `files`, `index` and the range when it fires, not
  when it was armed, and does nothing once `scanRunning` is false;
  `newFolderToken` clears it. The paths sent are `files` entries, the
  `list_arw` strings, which are built with the same `to_string_lossy` as the
  scan's `FileStat` paths, so the backend's queue matches them.

## Step 7: the documentation

- Steps 4 and 6's hand measurements are still pending, so `docs/performance.md`
  gained a "Which pass carries which cost" subsection (a table of which pass
  each cost moved to, the priority levels, and a note that the page-latency
  and wall-time measurements are pending) instead of new numbers. The older
  sections ("Sharpness scoring cost", "Face detection cost", the RAF note)
  keep their figures with a sentence saying they predate the move, since
  their `riffle-cli scan` "after" columns include costs that pass no longer
  carries. Once the pending checks are done, their numbers go in that
  subsection.
- `docs/usage.md` already said the score fills in during `analyzing N / M`
  (Step 3); this step added the on-screen-first order and the lowered
  priority to the Filmstrip entry, and that the Analysis rows come from the
  second pass.
- The README sentence on priority avoids promising that paging "stays
  responsive" (not measured yet); it says the viewer comes first when the
  passes compete for the CPU.

## Deferred issues (todo candidates)

- Pending manual check (Step 3, `crates/app/ui/src/main.ts` `faces-progress`
  handler, `crates/app/ui/src/focus.ts` `applySharpnessReady`): on the desktop
  app (any platform), clear the cache (settings modal, `Clear Cache`), open a
  folder of a few hundred RAWs with bursts, and watch: the thumbnails appear
  with no sharpness bars while the status shows `scanning N / M`, then the
  bars (and the pick-colored best frame) fill in while it shows
  `analyzing N / M`, and the meta pane's `Sharpness` row appears for the
  current file once its score arrives. With the debug log on, a
  `refresh entries` line's `sharpness` field on a large folder (~5000 files)
  should stay a few milliseconds; if it is much larger, throttle
  `applySharpness()` in the handler. The step's checkbox was ticked on the
  automated criteria (unit tests and `mise run ci`).
- Pending manual check (Step 4, `crates/core/src/scan.rs` `for_each_path` /
  `Priority`, `crates/app/src/index.rs` `run_scan` / `run_faces_scan`): on
  the Windows machine, clear the cache (settings modal, `Clear Cache`), open
  a large RAW folder (a few thousand files) in a development build with the
  settings modal's `Timing logs` on, and page through it with the arrow keys
  while `scanning N / M` and then `analyzing N / M` run. Compare the `page
  invoke=.. decode=.. total=.. keypressToPixels=..` lines in `Riffle.log`
  (the plan's "page latency"; there is no line by that name)
  with the same run on the previous release (and with paging after the scan
  ends): during the passes they must not be worse and should be closer to
  the idle value. Note the wall time of the `scan extract` / `scan faces`
  summary lines of both runs to see what the lowered priority costs while
  paging. No `ran at normal priority` warning should appear in the log. On a
  Mac, repeat once to see that the analysis pass (`QOS_CLASS_BACKGROUND`,
  which also throttles disk IO) does not crawl; if it does, move it to
  `QOS_CLASS_UTILITY`. The step's checkbox was ticked on the automated
  criteria (unit tests and `mise run ci`).
- Pending manual check (Step 6, `crates/app/ui/src/main.ts` `sendScanFocus`,
  `crates/app/ui/src/scanfocus.ts`, `crates/app/ui/src/strip.ts`
  `visibleRange`): on the Windows machine, clear the cache (settings modal,
  `Clear Cache`), open a large RAW folder (a few thousand files) in a
  development build with `Timing logs` on, and at once jump to the middle of
  the strip (drag the strip's scrollbar, then click a cell). Expected: the
  cells around the current file get their thumbnails during `scanning N / M`,
  and then their focus marks and sharpness bars during `analyzing N / M`,
  before the cells at the folder's start do (by eye, or from the
  `scan-progress` / `faces-progress` `ready` lists in the webview devtools).
  Note the wall time of the `scan extract` / `scan faces` summary lines in
  `Riffle.log` against Step 4's run of the same folder, so the queue's own
  overhead shows. The step's checkbox was ticked on the automated criteria
  (`scanfocus.test.ts` and `mise run ci`).
- Todo grouping (from the wrap-up's learnings extraction): file the Step 4
  and Step 6 checks above as one todo item, "Manual check of the scan
  priority and on-screen-first order on a large RAW folder (Windows, plus one
  Mac run)" (they share the setup). Done when: the measured page-latency and
  `scan extract` / `scan faces` numbers, with their conditions, are in
  `docs/performance.md` ("Which pass carries which cost"); the README wording
  on paging is revisited from that result; and, if the Mac analysis pass
  crawls, it is moved to `QOS_CLASS_UTILITY` and the priority entry in
  `docs/agents/tauri-app.md` is updated. File the Step 3 check as its own
  item, "Manual check of the sharpness bars filling in during the second
  pass". Done when: the `refresh entries` `sharpness` field on a ~5000-file
  folder is recorded in `docs/performance.md`, and `applySharpness()` is
  throttled if it is much more than a few milliseconds.
