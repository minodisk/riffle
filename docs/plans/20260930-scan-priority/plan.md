<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Defer the sharpness score to the second scan pass, lower the scan's priority, and process the on-screen files first

## Purpose

The folder scan's first pass (`riffle_core::scan::extract`, driven by
`Index::run_scan`) computes a file's thumbnail, metadata and sharpness score
together, and, when the file has no trusted AF point, runs YuNet on the whole
preview first so the score can land on the eyes of a face. A row is not
written, and the thumbnail not shown, until all of that is done. The eye
focus cue (`extract_faces`, `run_faces_scan`) already runs as a separate
second pass after `scan-done`. Both passes run at normal OS priority on
`scan_threads()` (cores - 2) workers, competing with the viewer's `preview`
command for CPU, and both take their files in file-name order whatever the
user is looking at.

After this work:

- The first pass computes only the thumbnail and metadata and writes the
  row at once, so thumbnails appear as fast as the preview can be read. The
  second pass computes both the eye focus cue and the sharpness score from
  one `read_preview` per file. For an HDR PQ CR3 the HEVC `PRVW`
  (`crates/core/src/hevc.rs`, ~100 ms) is decoded twice per file
  (thumbnail, analysis) instead of up to three times. The UI shows the
  sharpness bar, the meta pane's `Sharpness` row and the compare order as
  the second pass fills them in, the way the focus mark already turns from
  white.
- The scan's worker threads run below normal OS priority (the first pass
  slightly below, the analysis pass lowest), so the viewer's preview decode
  and the UI win under contention while the scan keeps every core when the
  machine is otherwise idle.
- Both passes take the files the user is looking at first: the frontend
  tells the backend the current file and the strip's visible range, and
  the workers pull those from the shared work queue before the rest.

What was measured before (`docs/performance.md`): the score itself adds
~3 ms per file on one thread (a grayscale decode plus a Laplacian window),
whole-image face detection ~17 ms, and the HEVC decode 65-125 ms. The cold
first scan on Windows is disk-bound (~16-19 ms per file), so on a cold folder
the gain of deferring is mostly the CPU share and the earlier row write; on a
warm folder and on HDR PQ CR3 folders it is most of the per-file time.

Out of scope: speeding up or retuning the HEVC decode; the Sony ARW,
Fujifilm RAF and OM System ORF parsers (worked on in other sessions).

The user's decisions at planning time: OS thread priority (not a smaller
thread count) for the passes; a live on-screen-first work queue for both
passes; `FACES_VERSION` / `faces_extractor` keep their names (no schema
bump); the `focus N / M` status text is Step 3's call.

## Steps

- [x] Step 1: Add the analysis pass to `riffle-core`: one `read_preview`
      that yields both the focus candidate cue and the sharpness score
  - Done when:
    - `crates/core/src/scan.rs` has a per-file function (working name
      `extract_analysis` / `extract_analysis_unless`, with an
      `extract_analysis_all` driver over `for_each_path`) that returns an
      `Analysis { cue: Cue, sharpness: Option<f64> }` from a single
      `read_preview`. It replaces `extract_faces` / `extract_faces_all`
      (rename, do not keep both); `crates/cli/src/main.rs`'s `candidates`
      reads `.cue` from it.
    - For a RAW with a trusted AF point (`sharpness::trusted_focus`): the
      cue is what `focus_cue_unless` gives today, and the score is
      `score_preview(preview, focus, eye_af_frame, &[])`, exactly what the
      first pass computes today.
    - For a RAW without one (no AF point or manual focus): the cue is
      `Cue::unknown()` with no detection, faces are detected on the whole
      preview (`detect_around(.., None)`, any failure or panic is no face,
      as `extract_unless` does today) and the score is
      `score_preview(preview, None, None, &faces)`.
    - For a JPEG file (`is_jpeg_file`): `Analysis` with an unknown cue and
      no score, without reading the file.
    - An unreadable file is `Err`; a decode, detection or scoring failure
      or panic is an unknown cue / `None` score, never an error (the rules
      `extract_faces_unless` and `extract_unless` follow today).
    - Cancellation is checked between the read, the detection and the
      scoring, as in `extract_faces_unless`; the existing cancel tests in
      `scan.rs` cover the new function the way they cover
      `extract_faces_unless` today.
    - `extract` / `Entry` are unchanged in this step (the first pass still
      scores), so the app builds and behaves as before; `mise run ci`
      passes.
  - Implementation approach:
    - Keep `candidate.rs` and `sharpness.rs` untouched: `focus_cue_unless`
      decodes RGB itself and `score_preview` decodes grayscale itself, so
      the analysis of one file does two JPEG decodes of the preview
      (~5-10 ms). The first pass did the same two decodes before, so this is
      not a regression; sharing one decode is a later optimization, to be
      measured before doing (see "Trade-offs and risks").
    - The score must be bit-identical to what `extract` computes today for
      the same file, because Step 2 relies on it to avoid a version bump:
      same `trusted_focus`, same `eye_af_frame`, same faces argument. Add a
      test that runs `extract` and `extract_analysis` on the same fixture
      and compares the scores (the `scan.rs` fixtures in `tests` give a
      scorable 64x48 preview).
    - Tests: move the sharpness assertions of
      `a_preview_that_cannot_be_scored_still_yields_a_thumbnail` and
      `a_jpeg_file_yields_its_exif_and_a_thumbnail_but_no_score` to the
      analysis function (they will leave `extract` in Step 2); extend
      `the_faces_pass_delivers_every_index_once_and_stops_on_cancel` and
      `a_file_canceled_mid_pipeline_is_not_delivered` to the renamed driver.
    - Files: `crates/core/src/scan.rs`, `crates/cli/src/main.rs`.

- [x] Step 2: Move the score out of the first pass and into the second in
      the app's index and scan
  - Done when:
    - `Entry` has no `sharpness` field and `extract_unless` does neither
      face detection nor scoring: read, thumbnail, done. `write_batch`
      writes `sharpness` as `NULL` (the `Ok` arm; the `Err` arm already
      does).
    - `Index::faces_todo` no longer marks rows without an AF point or with
      manual focus as done: only error rows and JPEG files are marked done
      at `FACES_VERSION` with no cue and no score in the same transaction,
      every other row of the folder not at `FACES_VERSION` is listed. The
      existing test `a_scanned_jpeg_has_its_exif_and_thumbnail_and_skips_the_faces_pass`
      still holds; the `faces_todo` tests around index.rs lines 2210-2290
      are updated for the no-AF rows now being listed.
    - `Index::write_faces` stores `(path, eye_focus, sharpness)` at
      `FACES_VERSION`; `FaceReady` (the `faces-progress` item) carries
      `sharpness: Option<f64>` next to `eye_focus` and `candidate`.
    - `run_faces_scan` drives `extract_analysis_all` and writes both
      values; `run_scan` is unchanged apart from the `Entry` field. The
      doc comments on `run_scan`, `run_faces_scan`, `start_scan`
      (commands.rs ~1339), `EXTRACTOR_VERSION` and `FACES_VERSION` say
      which pass owns what.
    - Neither `SCHEMA_VERSION`, `EXTRACTOR_VERSION` nor `FACES_VERSION` is
      bumped (see below), and a test proves the migration story: a row
      written by the old first pass (sharpness set, `faces_extractor` at
      `FACES_VERSION`) is not listed by `faces_todo` and keeps its score;
      a row written by the new first pass (sharpness `NULL`,
      `faces_extractor` 0) is listed and gets its score from
      `run_faces_scan`.
    - `a_sharpness_score_round_trips` goes through `write_faces` instead of
      `write_batch`; `entries` still returns the score.
    - `mise run ci` passes.
  - Implementation approach:
    - Why no version bump: the score's algorithm does not change (Step 1's
      equality test), so a row that already has one is correct whichever
      pass wrote it. An old row with `faces_extractor = 0` (its faces pass
      never ran) is re-analyzed and gets the same score. A row whose score
      failed (`NULL` at `FACES_VERSION`) stays `NULL`, as today. Bumping
      `EXTRACTOR_VERSION` would re-extract every folder, which is the slow
      path the user just reported; bumping `FACES_VERSION` would re-run the
      pass on every folder for no new information.
    - The JPEG rule in `faces_todo`: the table has no file-type column, so
      either select the folder's paths and filter with
      `riffle_core::scan::is_jpeg_file` before the `UPDATE`, or match the
      extension in SQL (`LOWER(path) LIKE '%.jpg' OR ... '%.jpeg'`). Either
      is fine; keep it in one place, since `scan_folder`'s idle check
      (commands.rs ~1179) calls `faces_todo` too.
    - Update the `tauri-app.md` entry "Bump `EXTRACTOR_VERSION`, not
      `SCHEMA_VERSION`, when extraction output changes" in this step, since
      it lists `sharpness.rs` under `EXTRACTOR_VERSION`: the score now
      belongs to `FACES_VERSION` (bump that on a change to
      `sharpness::score_preview` or to what `extract_analysis` feeds it).
      The column and constant keep their names (the user's decision; a
      rename would need a `SCHEMA_VERSION` bump), so the doc comments carry
      the wider meaning.
    - Files: `crates/core/src/scan.rs` (drop the scoring from
      `extract_unless`, `Entry.sharpness`), `crates/app/src/index.rs`,
      `crates/app/src/commands.rs` (doc comments; `run_faces_pass` is
      otherwise unchanged), `docs/agents/tauri-app.md`.

- [x] Step 3: Show the score as the second pass fills it in
  - Done when:
    - The `faces-progress` handler in `crates/app/ui/src/main.ts` patches
      each ready item's `sharpness` into the `sharpness` map (set when
      non-null, delete otherwise) and into `entries.get(path).sharpness`,
      then calls `applySharpness()` so the strip bars and the compare
      order update; the meta pane's `Sharpness` row and the MCP
      `get_view` / `get_photo` answers (`companion.ts`, which read the
      map) follow without further change.
    - `applyFaceReady` (`focus.ts`), or a sibling helper next to it, does
      the patching so it is unit-tested without the DOM (`focus.test.ts`).
    - The scan status text while the pass runs is reconsidered: today it
      is `focus N / M` and the pass now covers every RAW; if the wording
      changes, `docs/usage.md` (line ~156) changes with it in this step.
    - `mise run ci` passes.
  - Implementation approach:
    - `relativeSharpness` runs over `allFiles` in capture order on every
      call; at 10 ticks/s on a 5000-file folder measure it with the
      existing `refresh entries` timing (`refreshTimingLine`'s
      `sharpness` field shows what one run costs). If it is not
      negligible, only call `applySharpness()` when a ready path is in
      `files` or throttle it, rather than dropping the per-tick update.
    - No change to the null state is needed: a missing score already
      hides the bar (`strip.setSharpness(at, null)`), sorts last in
      `comparisonCandidates` and shows nothing in the meta pane. Verify
      this by opening a folder with a cleared index and watching the
      bars appear after the thumbnails.
    - Files: `crates/app/ui/src/main.ts`, `crates/app/ui/src/focus.ts`,
      `crates/app/ui/src/focus.test.ts`, possibly `docs/usage.md`.

- [ ] Step 4: Run the scan's workers below normal OS priority, the
      analysis pass lowest
  - Done when:
    - `scan::for_each_path` (and so `extract_all` and
      `extract_analysis_all`) takes a priority parameter (a small enum in
      `scan.rs`, e.g. `Priority::BelowNormal` / `Priority::Lowest`) and
      builds its rayon pool with `ThreadPoolBuilder::spawn_handler`, so
      each worker sets its own OS priority once at spawn before running
      rayon's `run`. The thread count stays `scan_threads()`.
    - `run_scan` runs at `BelowNormal`, `run_faces_scan` at `Lowest`; the
      CLI benchmarks (`riffle-cli scan`, `candidates`) run at normal
      priority as today, so their numbers stay comparable with
      `docs/performance.md`.
    - A failure to set the priority is logged once per pool (`log::warn!`
      in the app; `riffle-core` has no logger, so the handler returns the
      error to the caller or the pool builder records it) and the scan
      runs anyway at normal priority.
    - A unit test in `scan.rs` shows every index is still delivered once
      and cancel still stops early through the custom spawn handler
      (extend the existing `every_index_is_delivered_once` /
      `canceling_stops_the_scan_early` to the new parameter).
    - Hand measurement, recorded in `learnings.md` and checked by the user
      on the Windows machine: open a large RAW folder with a cleared index
      and page through it during the analysis pass; compare the `page
      latency` lines in `Riffle.log` (already written by `log_timing`)
      with the same runs on the previous release. The page latency during
      the pass must not be worse, and should be closer to its idle value.
      Note the wall time of `scan extract` / `scan faces` too, to see what
      the lowered priority costs when the user is paging.
    - `mise run ci` passes on all three CI targets (the macOS and Linux
      code paths are compiled and their tests run only there).
  - Implementation approach:
    - Why the first pass is lowered too: OS priority only matters under
      contention. When the machine is idle a below-normal thread still
      gets a full core, so throughput is unchanged; when the user pages
      during the first pass, the `preview` command's `read_preview`
      (running on tokio's blocking pool at normal priority) and the
      webview's decode win the contended cores, which is what keeps the
      viewer responsive. Thumbnails stay above the analysis pass by
      running at the higher of the two lowered levels. On Windows the
      two levels map to `THREAD_PRIORITY_BELOW_NORMAL` and
      `THREAD_PRIORITY_LOWEST`; do not use `THREAD_PRIORITY_IDLE` (a
      disk-bound pass at idle priority can starve behind any other
      process).
    - Crate: `thread-priority` (crates.io, cross-platform;
      `set_current_thread_priority` / `ThreadPriority::Os(..)` under
      `cfg(windows)` for the two Windows levels, and its
      `ThreadPriority::Crossplatform` or the unix policy API elsewhere).
      Verify the exact enum names against the version resolved in
      `Cargo.lock` before writing them. On macOS the sound mechanism is a
      QoS class (`pthread_set_qos_class_self_np`: `QOS_CLASS_UTILITY` for
      the first pass, `QOS_CLASS_BACKGROUND` for the analysis pass);
      check whether the crate exposes it, otherwise call it through
      `libc` under `cfg(target_os = "macos")`. `QOS_CLASS_BACKGROUND` also
      throttles disk IO on macOS, which is acceptable for the analysis
      pass (its reads are of files the first pass just read) but not for
      the first pass, hence `UTILITY` there. This machine cannot compile
      the macOS target (tauri-app.md, "This machine cannot compile the
      macOS target"), so the macOS branch is verified by CI's macOS job
      and by the user on a Mac later; keep the branch small.
    - Add the dependency to `crates/core/Cargo.toml` (the pool is built in
      `scan.rs`); check its license against the workspace's `cargo deny` /
      license list if one is configured.
    - Files: `crates/core/src/scan.rs`, `crates/core/Cargo.toml`,
      `Cargo.lock`, `crates/app/src/index.rs` (pass the priority),
      `crates/cli/src/main.rs` (pass `Normal`).

- [ ] Step 5: A shared work queue both passes pull from, and the
      `set_scan_focus` command that reorders it
  - Done when:
    - `scan::for_each_path` no longer walks `paths.par_iter()`; it spawns
      `threads` workers on the pool that each loop: check `cancel`, take
      the next index from a shared `WorkQueue`, run `per_file`, hand the
      result to `on_item`, until the queue is empty. Every index is
      delivered exactly once; once `cancel` is set no further index is
      taken and files in flight stop at their next stage, as today.
    - `WorkQueue` is a small type in `scan.rs` (or a sibling module) that
      holds the pending indices and a "hot" list of paths; `take_next`
      returns the first hot path that is still pending, in hot-list order,
      else the lowest pending index. The hot list is replaced whole by
      `set_focus(paths)`, callable from another thread while the workers
      run (`Arc<Mutex<..>>` or an atomic-swapped `Arc<Vec<String>>`;
      the choice is the implementer's, but a `take_next` must be O(hot +
      log n) or better, not O(n), since it runs once per file).
    - `extract_all` and `extract_analysis_all` take the queue's shared
      handle (working name `ScanFocus`, `Clone`, `Send + Sync`) so the app
      can hand the same handle to both passes; the CLI passes a handle that
      is never set.
    - `ScansState` keeps the handle of the running scan next to `running`
      (`(scan_id, cancel, focus, handle)`), and a new Tauri command
      `set_scan_focus(scan_id, paths: Vec<String>)` replaces the hot list
      when `scan_id` is the running scan's id and is a no-op otherwise
      (a stale id is not an error). It is a synchronous command: it only
      swaps a small list under a lock. It is registered in `main.rs`'s
      handler list and allowed by `capabilities/default.json` if that file
      lists commands.
    - Tests in `scan.rs`: with one worker and a hot list set before the run,
      the hot paths are delivered first in hot order and the rest in index
      order; a hot list set from `on_item` mid-run is honored by the next
      take; a hot path that is not in the list or already done is skipped;
      every index once; cancel stops early. Tests in `commands.rs`'s
      `tests`: `set_scan_focus` with the running id reaches the handle, a
      stale id does not.
    - `mise run ci` passes. The frontend does not call the command yet
      (Step 6), so behavior is unchanged for the user.
  - Implementation approach:
    - "Nearest to the current file first" is decided by the frontend, which
      knows the strip's order, filter and sort; the backend only honors the
      order of the list it is given (current file first, then the visible
      range outward, Step 6). This keeps the backend free of any notion of
      the strip and makes the queue trivially testable.
    - Keep rayon for the pool (Step 4's `spawn_handler` lives there) and
      use `pool.scope` / `broadcast` to run one worker loop per thread;
      `pool.install(|| (0..threads).into_par_iter().for_each(worker))`
      is the simplest form. The `on_item` panic rule (must not panic) is
      unchanged and stays in the doc comment.
    - The first pass's `paths` are `FileStat`s in `run_scan` and `String`s
      in `run_faces_scan`; the queue keys on the path string
      (`to_string_lossy`, the same conversion `write_batch` keys rows with)
      so the frontend's paths match. Build the `HashMap<String, usize>` of
      pending paths once at queue creation.
    - `scan_folder` cancels and joins the previous scan; the handle it
      stores for the new scan is fresh, so a late `set_scan_focus` with the
      old id is ignored by the id check. Nothing else in the cancel path
      changes.
    - Files: `crates/core/src/scan.rs`, `crates/app/src/index.rs`
      (`run_scan` / `run_faces_scan` take the handle),
      `crates/app/src/commands.rs` (`ScansState`, `start_scan`,
      `set_scan_focus`), `crates/app/src/main.rs`,
      `crates/app/capabilities/default.json` if needed,
      `crates/cli/src/main.rs`.

- [ ] Step 6: The frontend sends the current file and the visible range
      while a scan runs
  - Done when:
    - While `scanRunning` is true, `show()` (the current file changing) and
      the strip's scroll send `set_scan_focus(scanId, paths)` with the
      current file first, then the strip's visible cells outward from it
      (nearest first, alternating sides), capped at a small count (e.g.
      the visible cells plus `RANGE_MARGIN` on each side, as `strip.ts`'s
      own request range does), debounced at ~100 ms so a held arrow key or
      a wheel scroll sends one call per settle, not one per event. Nothing
      is sent when no scan is running or the strip is empty.
    - The list builder is a pure function in its own module (e.g.
      `crates/app/ui/src/scanfocus.ts`) tested in `scanfocus.test.ts`:
      current first, outward order, cap, no duplicates, empty when there is
      no current file.
    - `strip.ts` exposes the visible index range it already computes for
      its own virtualization (~line 369) rather than the caller
      recomputing it from `scrollLeft`.
    - Hand measurement, recorded in `learnings.md`: open a large RAW folder
      with a cleared index, jump to the middle of the strip at once, and
      note from the `scan-progress` `ready` lists (or by eye) that the
      cells around the current file get their thumbnails, then their marks
      and bars, before the folder's start does. Note the total
      `scan extract` / `scan faces` wall time against Step 4's numbers so
      the queue's own overhead is visible.
    - `mise run ci` passes.
  - Implementation approach:
    - Hook points: `show()` in `main.ts` already runs on every current-file
      change; the strip's scroll listener in `strip.ts` (its request-range
      logic) is where the visible range changes. Both call one debounced
      `sendScanFocus()`; a folder switch (`newFolderToken`) cancels a
      pending send, and `scanId` is captured at send time so a late timer
      never targets a newer scan.
    - The strip's own thumbnail requests keep their current priority logic
      (`strip.ts` ~line 274, "nearest the middle of the viewport first");
      this step only tells the backend which rows to produce first.
    - Files: `crates/app/ui/src/main.ts`, `crates/app/ui/src/strip.ts`,
      `crates/app/ui/src/scanfocus.ts`, `crates/app/ui/src/scanfocus.test.ts`.

- [ ] Step 7: Update the documentation that describes the scan
  - Done when:
    - `README.md` and `README.ja.md` (the sharpness cue and the "second
      pass" sentences around lines 95-110), `docs/usage.md` (the second
      pass around line 154, the Analysis rows around line 212, and the
      scan's behavior: below-normal priority, the on-screen files first),
      `docs/performance.md` ("Sharpness scoring cost" and "Face detection
      cost" say which pass carries the cost now; add Step 4's and Step 6's
      hand measurements under "Real folders on Windows" or a new
      subsection) and `CLAUDE.md`'s layout paragraph (`run_scan`
      "(thumbnail, metadata, sharpness)", `run_faces_scan`, and the new
      `set_scan_focus` command / `ScanFocus` handle) describe the first
      pass as thumbnail + metadata and the second as cue + sharpness, both
      below normal priority and pulling the on-screen files first.
    - `docs/agents/tauri-app.md` gains one entry on the scan pool: the
      priority levels per pass and why the first pass is not `IDLE` /
      `BACKGROUND`, and that `set_scan_focus` ignores a stale `scan_id` by
      design, with the source pointing at this plan's `learnings.md`.
    - `mise run ci` (lychee included) passes.
  - Implementation approach:
    - Doc-only PR; keep the wording changes to the sentences that name the
      passes. `README.md` and `README.ja.md` change in the same PR.

## Trade-offs and risks

- **macOS priority is unverified here.** The development machine is
  Windows and cannot compile the macOS target; Step 4's macOS branch (QoS
  class or the crate's unix policy) is checked by CI's macOS job for
  compilation and by the user on a Mac for effect. If `thread-priority`'s
  macOS support turns out to be the plain pthread priority (no QoS), it
  may have little visible effect there; the plan accepts that and notes it
  in `learnings.md` rather than adding a second mechanism blind.
- **`QOS_CLASS_BACKGROUND` throttles IO on macOS.** Chosen only for the
  analysis pass, whose files were just read by the first pass; if the hand
  measurement on a Mac shows the pass crawling, drop it to `UTILITY` as
  well.
- **The queue replaces rayon's work splitting with a mutex per take.** At
  one take per file (a 1-30 ms unit of work) the lock is uncontended in
  practice; the Step 6 measurement of total wall time is what confirms it.
  If it shows up, batch takes (two or four indices per lock) before
  reaching for a lock-free structure.
- **A hot list only helps rows still pending.** With the first pass at
  ~1000 files/s warm, the visible range is often already done by the time
  the frontend's debounce fires; the benefit is on cold folders and on the
  analysis pass, which is where the user waits today.
- **`set_scan_focus` traffic.** One call per 100 ms at most while paging;
  the command does no IO. The frontend must not send while no scan runs
  (the id check makes a late call harmless, but it is still an IPC round
  trip).
- **`FACES_VERSION` semantics widen without a rename** (the user's
  decision). The column `faces_extractor` and the constant now also cover
  the sharpness score; the doc comments and `tauri-app.md` carry the new
  meaning.
- **Two JPEG decodes per file in the analysis pass** (RGB for faces,
  grayscale for the score) as before in the first pass. Sharing one decode
  would mean `score_preview` taking a gray plane (`candidate::luma` of the
  RGB) instead of the JPEG bytes; the saving is ~3 ms per file on a JPEG
  preview. Measure with `riffle-cli` before doing it, and do it in its own
  PR.
- **Rows at `faces_extractor = 0` after an interrupted old scan** are
  re-analyzed and re-scored; that is idempotent and costs the analysis of
  those files once.
- **The score arrives later than today** for a file with an AF point: on a
  folder where the first pass is disk-bound, the second pass starts only
  after `scan-done`, so the bars fill in after every thumbnail instead of
  along with them. That is the requested priority; the strip's null state
  covers the gap, and Step 6 makes the on-screen bars come first.
- **The `scanning` status text.** `focus N / M` no longer names all the
  pass does; changing it touches `docs/usage.md` and any test asserting on
  it (Step 3 decides).

## Progress

- (2026-09-30) Step 1 complete
- (2026-10-01) Step 2 complete
- (2026-10-01) Step 3 complete
