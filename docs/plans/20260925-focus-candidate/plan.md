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

# Focus candidate cue, computed in a second scan pass

## Purpose

The face-catch state shipped by `docs/plans/_archived/20260924-face-catch-state/`
answers "did the AF land on a face?", which is not what culling needs: a
Sony face-tracked frame is `caught` even when the AF sat on the background
(`_DSC2188`), and a face under the AF point can still be soft. The user
hand-labeled 500 ILCE-7M5 frames in focus / off focus and found a better cue:
the Laplacian variance of the luma over a window between the eyes of the face
nearest the AF point ("eye sharpness"). With a fixed threshold of 80, the
frames at or above it ("focus candidates") were in focus 93% of the time
(310/335) and covered 80% of the in-focus frames; frames below it were in focus
48% of the time. AUC of eye sharpness was 0.82 against 0.67 for the existing
sharpness score. In-focus rate by eye sharpness: <40 39%, 40-80 45%, 80-150
85%, 150-300 91%, >=300 96%; no face near the AF point 53%. A folder-relative
threshold (percentile / Otsu) was worse than the fixed 80.

This plan replaces the face-catch state with that cue and adds a filter that
shows only candidates. The existing sharpness score and its strip bar do not
change.

Definition (all in the embedded preview's stored coordinates, 1616x1080 on
the α7 V):

- Faces come from the existing `faces::detect_around` (480 crop around the
  trusted AF point). Sony eye-AF frames (`eye_af_frame`, `AFTracking == 1`)
  are **not** trusted any more: they get the crop detection too, because the
  camera's claim does not say whether the face is sharp.
- The face nearest the AF point is the one minimizing (distance from the AF
  point to the closer of its two eyes) / (the face box's long side).
- Eye sharpness = `sharpness::laplacian_variance` over the square
  `window_at` window centered at the midpoint of that face's eyes, side = the
  face box's long side, at least 24 px, on the luma of the preview.
- Candidate: eye sharpness >= 80 (one named constant). Not a candidate: a
  face near the AF point with eye sharpness < 80. Unknown: no trusted AF
  point, no face in the crop, or any decode / detection failure or panic.

Known limits (documented, not solved here): AF on a background person gives a
sharp face and a false candidate; YuNet false positives on shirt logos
(`todo.md`); back-of-head and upturned faces give unknown.

Scan cost: the cue needs YuNet on every trusted-AF frame, eye-AF frames
included, which is ~70% more scan time on a Sony folder (6.9 s -> 11.7 s for
2134 files at 24 threads). Thumbnails must keep appearing as fast as today, so
extraction becomes two passes: pass 1 (thumbnail, metadata, sharpness) with
unchanged latency, and pass 2 (eye sharpness) that runs right after it on the
same task and streams its progress like the scan does.

Ground truth for the manual check (XMP sidecars written by Riffle,
`xmpDM:good="True"` in focus, `"False"` off focus), under `/mnt/d/Photos/2026/`:
`2026-09-19-focus-sample`, `2026-09-19-focus-sample-2`,
`2026-07-31-focus-sample`, `2026-09-13-a-focus-sample`,
`2026-06-05-focus-sample`. The labels are for the files in those sample folders
(copies of the originals).

Constraints: pass 1 latency must not grow; YuNet at most once per file per
pass; any detection failure or panic gives unknown, never an error for the
file; everything committed in English, except `README.ja.md`, which every
step that changes `README.md` updates in Japanese in the same PR; `mise run
ci` passes for every step.

## Steps

- [x] Step 1: Compute the focus candidate state and eye sharpness in `riffle-core`, with a CLI check against the labeled folders
  - Done when:
    - A core module (suggested: new `crates/core/src/candidate.rs`, registered
      in `lib.rs`; `faces.rs` stays about detection) has:
      - `pub const CANDIDATE_THRESHOLD: f64 = 80.0` and
        `pub const CANDIDATE_WINDOW_MIN: usize = 24` (not `EYE_WINDOW_MIN`,
        which `sharpness.rs` already uses for 128), each with a doc comment
        citing the validation numbers above.
      - `pub enum FocusCandidate { Candidate, NotCandidate, Unknown }`
        (`Copy`, `Debug`, `PartialEq`, `Eq`, `Default = Unknown`).
      - `pub fn nearest_face<'a>(faces: &'a [Face], point: (usize, usize)) ->
        Option<&'a Face>`: the face minimizing the distance from `point` to
        the closer eye divided by `width.max(height)`; a face whose long side
        is not positive is skipped; `None` for no faces. Tests: chosen by eye
        distance, not box center; the normalization (a small face with a
        near eye beats a large one whose eye is farther in absolute pixels but
        closer relative to its size, and the reverse); empty slice.
      - `pub fn eye_window(width, height, face: &Face) -> Window`:
        `window_at` on the eye midpoint (clamped into the image) with side
        `long_side.max(CANDIDATE_WINDOW_MIN)`. Tests: side and center on a
        plain face; a tiny face gets the 24 px minimum; a face at a corner is
        clamped.
      - `pub fn luma(rgb: &[u8], width, height) -> Vec<u8>` computing
        `(299 R + 587 G + 114 B) / 1000` per pixel in integer arithmetic.
        **This is the luma the 500-frame analysis used**, so the 80 threshold
        carries over exactly; the mozjpeg grayscale decode `sharpness::score`
        uses is not used here (see Trade-offs). Test: a pure red / green /
        blue pixel gives 76 / 149 / 29 and white gives 255.
      - `pub fn eye_sharpness(gray: &[u8], width, height, face: &Face) -> f64`
        = `laplacian_variance(gray, width, eye_window(...))`.
      - `pub fn candidate(eye_sharpness: Option<f64>) -> FocusCandidate`:
        `None` -> `Unknown`, `>= CANDIDATE_THRESHOLD` -> `Candidate`, else
        `NotCandidate`. Tests: `None`, 79.99, 80.0, 300.
      - `pub struct Cue { pub state: FocusCandidate, pub eye_sharpness:
        Option<f64>, pub face: Option<Face>, pub detection: Option<Detection> }`
        (names at the implementer's discretion) and the entry point
        `pub fn focus_cue(preview: &[u8], orientation: u16, focus:
        Option<FocusLocation>) -> Result<Cue>`: `focus == None` returns
        `Unknown` with no decode and no detection; otherwise it decodes the
        preview **once** (`decode_rgb`), takes the luma of the stored RGB,
        runs the crop detection, picks `nearest_face` on the stored-coordinate
        faces and the stored-coordinate AF point that `Detection` already
        carries, and scores the eye window on the stored luma. No eye-AF
        special case: `eye_af_frame` is not consulted.
    - `faces::detect_around` is split so the cue does not decode twice:
      `pub fn detect_around_rgb(rgb: &[u8], width, height, orientation,
      focus) -> Result<Detection>` holds today's body after the decode, and
      `detect_around` becomes `decode_rgb` + `detect_around_rgb`. `faces_of`
      and the CLI keep calling `detect_around`; existing tests still pass.
    - `crates/core/src/scan.rs`: `pub fn extract_faces(path: &Path) ->
      Result<Cue, String>`: `read_preview`, then `focus_cue` inside
      `catch_unwind` (like `extract` does for the thumbnail); a decode /
      detection error or panic is `Ok(Cue::unknown())`, and only an unreadable
      file is `Err`. `extract` itself is untouched in this step. Test: on the
      synthetic TIFF fixtures already in `scan.rs`, a file with no AF point is
      `Unknown` and a non-JPEG preview is `Unknown`, not a panic.
    - `crates/cli/src/main.rs`:
      - `riffle-cli candidates <dir> [threads]` runs `extract_faces` over the
        folder's RAW files on a pool (reuse the pool shape of `scan_dir`) and
        prints one line per file: name, state, eye sharpness (or `-`), the
        nearest face's box, and, when an XMP sidecar sits beside the file
        (`riffle_core::xmp::sidecar_path` + `xmp::read_flag`), its flag. It
        ends with the wall time and, when any flag was read, the counts the
        acceptance criterion needs: candidates / candidates in focus (`Pick`),
        in-focus frames / in-focus frames that are candidates, so the check
        below is one run per folder. Step 3 reuses the timing as the pass-2
        cost measurement.
      - `riffle-cli faces <file> <out.png>` also prints the cue (state, eye
        sharpness, nearest face) and draws the eye window, so single files
        can be eyeballed. The face-catch line stays until Step 2 removes it.
    - Manual check recorded in `learnings.md`: `riffle-cli candidates` on each
      of the five labeled folders and pooled; expected roughly 93% of
      candidates in focus and 80% of in-focus frames covered. A large
      deviation means the implementation differs from the analysis (luma,
      window, nearest-face rule): investigate before going on, do not tune
      the threshold.
    - `mise run ci` passes.
  - Implementation approach:
    - Faces and the AF point from `Detection` are already in stored
      coordinates; the window is a square and the long side is rotation
      invariant, so nothing needs the upright image except the detector.
    - Keep `sharpness.rs`'s own `eye_window` / `chosen_face` as they are: they
      serve the score's no-AF path with different clamps (128..256) and must
      not change, or the score changes.

- [x] Step 2: Persist the eye sharpness in the index and remove the face-catch state
  - Version check at implementation time: this branch still had
    `SCHEMA_VERSION = 14` and `EXTRACTOR_VERSION = 4`, so the numbers below
    apply unchanged (v15, extractor kept at 4).
  - Done when:
    - `crates/app/src/index.rs`:
      - `SCHEMA_VERSION` is 15. `files` gains `eye_sharpness REAL` (`NULL` =
        none) and `faces_extractor INTEGER NOT NULL DEFAULT 0`, and loses
        `face_catch`. Migration: `(10..15).contains(&version)` adds the two
        columns with `ALTER TABLE`; `version == 14` also runs `ALTER TABLE
        files DROP COLUMN face_catch` (SQLite `DROP COLUMN` is already used by
        the v11 migration); the `(10..14)` face-catch guard is deleted; `14`
        joins the `prepare` whitelist; the doc comment describes v15. Audit
        every remaining guard per `docs/agents/tauri-app.md` "Bumping
        `SCHEMA_VERSION` can strand an old per-version column guard".
      - `pub const FACES_VERSION: i64 = 1` (name at the implementer's
        discretion), the version of what `scan::extract_faces` produces, with
        a doc comment like `EXTRACTOR_VERSION`'s: bump it when the cue's
        computation changes (threshold, window, detector), and pass 2 re-runs
        without redoing pass 1.
      - `EXTRACTOR_VERSION` stays 4 **if** what `extract` stores is
        byte-identical after this step (see below); the doc comment says so.
        Verify by reasoning on `extract`: faces are already ignored by
        `score_preview` whenever a trusted AF point exists (Step 1 of the
        previous plan), so dropping the crop detection there changes neither
        the thumbnail, the metadata nor the sharpness. If anything stored does
        change, bump to 5 instead.
      - `write_batch` inserts `faces_extractor = 0` and `eye_sharpness = NULL`
        on both the success and the error row (a re-extracted file starts
        pass 2 over, which is right since the file changed).
      - `pub fn faces_todo(&mut self, dir: &str) -> Result<Vec<String>,
        String>`: in one transaction, first `UPDATE files SET faces_extractor
        = FACES_VERSION, eye_sharpness = NULL WHERE dir = ?1 AND
        faces_extractor != FACES_VERSION AND (error IS NOT NULL OR focus_w IS
        NULL OR manual_focus != 0)` (rows that can only be unknown: no trusted
        AF point, i.e. exactly `sharpness::trusted_focus == None`, or a failed
        extraction), then returns the paths still at another
        `faces_extractor`, sorted by file name like `entries`.
      - `pub fn write_faces(&mut self, rows: &[(String, Option<f64>)]) ->
        Result<(), String>`: one transaction of `UPDATE files SET
        eye_sharpness = ?2, faces_extractor = FACES_VERSION WHERE path = ?1`.
      - `Focus` loses `face_catch` and gains `eye_sharpness: Option<f64>` and
        `candidate: FocusCandidate`, serialized as `"candidate" |
        "not_candidate" | "unknown"` through a `serialize_with` function like
        `flag`; `entries` derives it with `candidate::candidate(eye_sharpness)`
        (the state is not stored; see Trade-offs). A row whose pass 2 has not
        run yet is `unknown` with `None`.
      - `face_catch_name` / `_code` / `_from_code` / `serialize_face_catch`
        and the `FaceCatch` import are removed.
      - Tests: a round trip (`write_batch` of three entries: trusted AF,
        manual focus, no AF; `faces_todo` returns only the trusted-AF path and
        marks the other two done; `write_faces` with 120.0 / `None`; `entries`
        reads `candidate` / `unknown` and the value; a second `faces_todo` is
        empty; an `INSERT OR REPLACE` of the same path puts it back on the
        list). Migration: a new v14 fixture (built by `open`, then `PRAGMA
        user_version = 14`, `DROP COLUMN eye_sharpness`, `DROP COLUMN
        faces_extractor`, `ADD COLUMN face_catch INTEGER NOT NULL DEFAULT 0`)
        gains the two columns, loses `face_catch` and keeps `files` /
        `ratings`; the existing v10-v13 fixtures drop `eye_sharpness` /
        `faces_extractor` instead of `face_catch`; every `user_version`
        assert moves from 14 to 15. The old face-catch tests are deleted.
    - `crates/core`: `FaceCatch`, `face_catch` and `Entry.face_catch` are
      removed; `scan::extract` runs `detect_around` only when
      `trusted_focus` is `None` (the whole-image faces the score needs; see
      Trade-offs) and never with an AF point; the module doc and the
      `faces.rs` module doc no longer mention the state. `CATCH_CROP` /
      `CATCH_CONFIDENCE` keep their names (`faces_of` and the cue use them).
      The `riffle-cli faces` face-catch line is removed.
    - `crates/app/src/commands.rs` `read_faces` doc comment no longer refers
      to the stored face-catch state.
    - Frontend, minimal so it compiles and keeps working: `Focus` in
      `main.ts` and `MarkFocus` / `FocusMark` in `focus.ts` swap `face_catch`
      / `faceCatch` for `candidate` (`"candidate" | "not_candidate" |
      "unknown"`) and `eye_sharpness: number | null`; `FOCUS_MARK_COLORS` is
      keyed by the candidate state with the same three colors (green
      candidate, orange not a candidate, white unknown); `metaGroups`' third
      argument becomes the candidate state and the row is `Focus`:
      `Candidate` / `Not a candidate`, omitted when unknown; `focus.test.ts` /
      `meta.test.ts` updated. Final rows and the filter come in Step 4.
    - `CLAUDE.md`'s layout paragraph replaces the face-catch mentions with the
      candidate module and the index's `eye_sharpness` / `faces_extractor`
      columns filled by the second pass (Step 3 refines the wording).
    - `mise run ci` passes. Until Step 3 lands nothing fills the columns, so
      every mark is white; say so in the PR body.
  - Implementation approach:
    - Follow the shape of the previous plan's Step 3 and its `learnings.md`
      (fixtures faking older versions must drop every column added since).
    - Removing the crop detection from `extract` makes pass 1 faster for
      trusted-AF, non-eye-AF files (428 of 2134 in the Sony folder); Step 3
      measures it.

- [ ] Step 3: Run the eye-sharpness pass after the scan and stream it to the frontend
  - Done when:
    - `crates/core/src/scan.rs`: the rayon pool loop of `extract_all` is
      shared with the new `pub fn extract_faces_all<F>(paths, threads,
      on_item: F, cancel)` (e.g. a private generic `for_each_path(paths,
      threads, per_file, on_item, cancel)` both call), with the same
      "`on_item` must not panic" contract and the same `threads == 0` error.
      The existing `extract_all` tests still pass; one test shows
      `extract_faces_all` delivers every index once and stops on cancel.
    - `crates/app/src/index.rs`: `pub fn run_faces_scan<P>(index, dir, paths:
      &[String], threads, cancel, progress_interval, progress: P) ->
      ScanSummary` mirroring `run_scan`: results pending in `BATCH`-sized
      batches written with `write_faces`, a `ready: Vec<FaceReady { path,
      eye_sharpness: Option<f64>, candidate: FocusCandidate }>` list pushed
      only after the batch's write returned `Ok`, a rate-limited `progress`
      plus the final unconditional call after the trailing flush (the reason
      is in `docs/agents/tauri-app.md` "`scan-progress` carries flushed
      paths"), an `Err` from `extract_faces` counted as an error and written
      as `None` (unknown, not retried until `FACES_VERSION` moves), and a log
      line `scan faces: dir= files= done= errors= threads= canceled= in ms`.
      Tests: every path is written and reported; cancel stops early and the
      trailing flush still reports the written ones; an unreadable path lands
      as `NULL` at `FACES_VERSION` and counts as one error.
    - `crates/app/src/commands.rs` `start_scan`: the spawned task, after
      emitting `scan-done`, and unless `cancel` is set, calls
      `faces_todo(&dir)` under the writer lock and runs `run_faces_scan` with
      the **same** `cancel` flag and `scan_threads()`, emitting
      `faces-progress { dir, scan_id, done, total, ready }` and then
      `faces-done { dir, scan_id, total, errors }`, and only then does its
      `finish(scan_id)` + `scan-state` emit under the lock as today. Every
      path that emits an empty `scan-done` (`emit_empty_scan_done`, the
      cancel-before-pass-2 case) also emits an empty `faces-done`, so the
      frontend's flag always clears. Consequences to keep and document in
      the command's doc comment: `ScansState::scanning()` is true through
      pass 2 (so `clear_index` refuses and the settings modal shows scanning);
      `scan_folder` cancels and joins pass 2 through the one `running` handle,
      and rows not yet written keep `faces_extractor = 0`, so the next scan of
      the folder resumes them; a resync's `reconcile` runs only after the
      join, so `write_faces` never races a row deletion. The `commands.rs`
      `tests` module gets a case that a finished pass 2 leaves nothing in
      progress (extend the existing scan-state tests).
    - Frontend (`crates/app/ui/src/main.ts`): a `faces-progress` listener
      (guarded by `payload.scan_id === scanId`) patches `entries.get(path)
      .focus` with `candidate` / `eye_sharpness` for each ready item, sets
      the status text to a second phase (e.g. `focus ${done} / ${total}`),
      and, when the current file is among them, calls `renderMeta()` and
      `draw()`; it calls `refilter()` when the candidate filter (Step 4) is
      on. `scanRunning` now clears on `faces-done`, not `scan-done`
      (`scan-done` keeps `strip.refresh()` + `refreshEntries()`); `faces-done`
      sets the final status, `renderMeta()`, `refreshEntries()` (the
      authoritative re-read) and `drainResync()`. The in-place patch is a
      small pure helper (e.g. in `focus.ts`: apply ready items to a
      `Map<string, IndexedFile>`, returning whether a given path was touched)
      with a unit test, leaving only the listener in `main.ts`.
    - `docs/performance.md` "Face detection cost" gains a "Focus candidate
      pass" subsection with, on the 2134-file Sony folder, runs alternated,
      threads and machine stated (per the guide's "Write a performance number
      with its measurement conditions"): `riffle-cli scan` before (origin/main
      at the plan's start) and after Step 2 (pass 1 must not rise; it should
      drop for the 428 crop-path files), and `riffle-cli candidates` wall time
      as the pass-2 cost, plus the `scan extract` / `scan faces` log lines of
      one app open of that folder.
    - `CLAUDE.md`'s layout paragraph says the index is filled in two passes
      (`run_scan`, then `run_faces_scan` on the same task) and names the
      `faces-progress` / `faces-done` events.
    - `mise run ci` passes; a manual check in the app (listed for the user:
      open the Sony folder, thumbnails appear at the old speed, marks turn
      from white to green / orange while the status shows the second phase,
      alt-tab during pass 2 does not restart it) is recorded in
      `learnings.md`, or reported as not verified.
  - Implementation approach:
    - Assumes Step 2 is merged (columns, `faces_todo`, `write_faces`).
    - The `ready` payload carries the results so the frontend does not
      re-read `folder_entries` ten times a second during pass 2; the
      `faces-done` `refreshEntries()` is the safety net (see Trade-offs).
    - Emit `faces-done` before `finish(scan_id)`, under no lock; the
      `scan-state` emit stays under the lock as the guide requires.

- [ ] Step 4: Meta rows, the candidate filter, and the documentation
  - Done when:
    - `crates/app/ui/src/meta.ts`: the Riffle section shows `Focus`
      (`Candidate` / `Not a candidate`, omitted when unknown) and `Eye
      sharpness` (`toFixed(1)`, omitted when null) after `Sharpness`;
      `metaGroups` takes both (or the `Focus` object); `meta.test.ts` covers
      candidate, not a candidate, unknown-with-no-value.
    - `crates/app/ui/src/filter.ts`: `FilterState` gains `candidatesOnly:
      boolean` (or a one-member `Set` mirroring the other groups; the
      implementer picks whichever keeps `filterChanged` / reset uniform), and
      `passes` takes the file's candidate state; when on, only `"candidate"`
      passes. `filter.test.ts` covers on / off and the unknown state.
    - `crates/app/ui/index.html`: after the orientation group, an `<hr />`
      and one `menuitemcheckbox` `Focus candidates` (`data-candidate`);
      `main.ts` wires it like `data-orientation` (the `filterItems` selector,
      `filterActive`, `filterChanged`, the click handler, `filter-reset`), and
      `passes(path)` passes `entries.get(path)?.focus?.candidate`. The Step 3
      `faces-progress` listener refilters while it is on, so the strip fills
      in as pass 2 runs.
    - `focus.test.ts` asserts the color mapping for the three states (if
      Step 2 did not), and the `drawFocusMark` comment describes the new
      meaning.
    - Docs, all in the same PR: `README.md` and `README.ja.md` ("Focus mark":
      green = a focus candidate, the face nearest the AF point has sharp eyes;
      orange = a face near the AF point whose eyes are not sharp; white =
      unknown: no AF point, manual focus, no face near the point, or not
      computed yet; the meta pane shows the state and the eye sharpness; the
      cue is computed in a second pass after the thumbnails; the camera's face
      tracking no longer colors the mark; "Filter menu" gains `Focus
      candidates`), `docs/usage.md` (the same three places: Focus mark, Meta
      pane, Filter menu), `docs/cameras.md` if its "Face tracking" wording
      implies the mark's color, `CLAUDE.md` (`meta.ts` / filter sentence),
      `todo.md` (the face/eye section: replace the face-catch item with the
      candidate cue and its validation numbers, reword the logo item to "a
      false candidate", add the known limits as items: AF on a background
      person, back-of-head / upturned faces unknown, and drop the
      "de-duplicate the eye-AF rule" item since the rule is gone).
    - `mise run ci` passes; the GUI check (marks, meta rows, the filter with
      pass 2 running) is listed for the user and recorded in `learnings.md`,
      or reported as not verified.
  - Implementation approach:
    - Assumes Steps 2 and 3 are merged.
    - Keep the state decision in pure modules (`filter.ts`, `meta.ts`,
      `focus.ts`) and only the DOM wiring in `main.ts`, as today.

## Trade-offs and risks

- **Store the state or derive it from `eye_sharpness` (Step 2).** Planned:
  derive (`candidate(eye_sharpness)`) at read time in `entries`, so the index
  holds one `REAL` and the threshold lives in one core constant; a later
  threshold change takes effect on the next open without a pass-2 re-run.
  Alternative: a `candidate INTEGER` column written by pass 2, which makes a
  threshold change a `FACES_VERSION` bump and lets the state disagree with
  the value. Unknown and not-yet-computed are indistinguishable either way
  unless a `faces_extractor`-based "pending" is surfaced; not planned.
- **Which luma (Step 1).** Planned: `(299R+587G+114B)/1000` from the RGB
  decode, the formula the analysis used, so the validated 80 carries over and
  the preview is decoded once for detection and scoring. mozjpeg's grayscale
  decode (what `sharpness::score` uses) applies the same BT.601 weights in
  fixed point and would differ by rounding only, but would cost a second
  decode per file; not chosen.
- **Whole-image detection on the no-AF path (Step 2).** Planned: it stays in
  pass 1, exactly as today, because the sharpness score of manual-focus /
  DNG / SIGMA files depends on it and moving it would make the score change
  after pass 2 (the bar would move later) or force an `EXTRACTOR_VERSION`
  bump. Alternatives: move it to pass 2 (pass 1 gets faster for DNG folders;
  the score would have to be recomputed in pass 2, a bigger change) or drop
  it (those files score `tile_max`; a separate behavior change). If the caller
  wants the drop, Step 2's `extract` loses its detection entirely and
  `score_preview` its `faces` parameter.
- **Pass 2 on the same task vs a separate command (Step 3).** Planned:
  chained on the `start_scan` task so one `cancel`, one `JoinHandle` and one
  `finish` cover both passes and `scan_folder`'s cancel-and-join needs no
  change. Cost: `scanning()` is true through pass 2, so Clear Cache is refused
  and the settings modal shows scanning until the cue is done. A separate
  `start_faces_scan` command would let the frontend decide when to run it
  but duplicates the `Scans` bookkeeping.
- **`scanRunning` until `faces-done` (Step 3).** Planned, so the existing
  rule "a resync waits for the scan instead of restarting it" holds for pass
  2 too; a focus change during a long pass 2 is deferred. Alternative: clear
  it on `scan-done` and let a resync cancel pass 2 (it resumes on the next
  scan, since unwritten rows stay at `faces_extractor = 0`), at the cost of
  re-listing and re-reconciling on every focus change.
- **`faces-progress` carries results vs re-reading `folder_entries`.**
  Planned: the payload carries `(path, eye_sharpness, candidate)` and the
  frontend patches `entries` in place, with `refreshEntries()` on
  `faces-done` as the authoritative refresh. Re-reading on every tick would
  be simpler but is a full-folder read at up to 10/s.
- **No `EXTRACTOR_VERSION` bump (Step 2).** Existing caches keep their
  thumbnails and only pass 2 runs on them, which is the point of the split.
  It rests on `extract`'s stored output being identical; the step verifies
  that and bumps if not. If in doubt, bumping costs one rescan per folder.
- **Sony eye-AF is no longer trusted.** Back-of-head and upturned faces that
  were green become white (no face) or orange; `_DSC2188`-style frames
  become not-a-candidate instead of green. This is the intended change; the
  docs say the camera's tracking no longer colors the mark.
- **Migration risk.** `DROP COLUMN face_catch` on a v14 database: SQLite
  refuses to drop an indexed or constrained column; `face_catch` is neither.
  The fixture pitfall from the previous plan applies (fake older versions by
  dropping every newer column).
- **Two extractor-like versions.** `EXTRACTOR_VERSION` and `FACES_VERSION`
  are independent on purpose; the `docs/agents/tauri-app.md` item on bumping
  should gain a line for `FACES_VERSION` in Step 3.
- **Filter representation (Step 4).** A boolean is the simplest; the menu's
  other groups are `Set`s. Either is fine; the step picks the one that keeps
  `filterChanged` / reset uniform.

## Progress

- (2026-09-25) Step 1 complete
