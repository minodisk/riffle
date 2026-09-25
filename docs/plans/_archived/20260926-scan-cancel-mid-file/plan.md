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

# Cancel a scan inside a file's pipeline

## Purpose

Opening one folder after another cancels the running scan, but the cancel flag
is only read between files (`for_each_path` in `crates/core/src/scan.rs`), so
every file already dispatched to the `cores - 2` rayon workers runs its whole
pipeline (RAW read, JPEG decode, thumbnail encode, whole-image face detection
when there is no AF point, sharpness; and in the faces pass, decode plus
detector inference) before the scan task ends. `scan_folder`
(`crates/app/src/commands.rs`) joins that task before listing and reconciling
the new folder, and strip thumbnails only come from the index the new scan
writes, so the new folder stays blank for the 8-15 s Riffle.log shows
(`scan extract ... canceled=true in 15728ms`, even `files=1 ... in 8030ms`),
while the old workers compete with `preview` for disk and CPU.

After this work a canceled scan stops between pipeline stages of each in-flight
file and drops that file without writing a row, so the join in `scan_folder`
returns after at most one stage per worker and the next scan of the folder
redoes the dropped files. Closes the `todo.md` item "App: canceling a scan
still lets in-flight files finish (8-15s)" (the todo entry itself is removed
in the wrap-up).

## Steps

- [x] Step 1: Check the cancel flag between pipeline stages and drop abandoned files without a row
  - Done when:
    - `riffle_core::scan` checks `cancel` inside each file: in the first pass
      after `read_preview`, before the thumbnail encode, before the
      whole-image `detect_around`, and before `score_preview`; in the faces
      pass after `read_preview`, after the RGB decode, before
      `detect_around_rgb`, and before `eye_sharpness`.
    - A file abandoned mid-pipeline is never handed to `on_item`, so
      `run_scan` / `run_faces_scan` write no row for it (no `files` row in the
      first pass; `faces_extractor` untouched in the second), it is not in
      `ready`, and it is not counted in `done` / `ScanSummary.total`. The
      `canceled=true` log line therefore reports only files actually written
      or failed; document that in the `run_scan` / `run_faces_scan` doc
      comments and in the `extract_all` doc comment (replace "files already
      running finish").
    - The public signatures `scan::extract(&Path)`, `scan::extract_faces(&Path)`,
      `scan::extract_all`, `scan::extract_faces_all` and
      `candidate::focus_cue` keep their current shapes so `crates/cli`
      compiles unchanged (verify with `cargo build --workspace`).
    - Unit tests in core: with the flag set before a stage (set it in a test
      the way the existing `canceling_stops_the_scan_early` does, or via a
      test that calls the cancel-aware variant with an already-set flag),
      the file yields no `on_item` call and `extract_all` /
      `extract_faces_all` return promptly; the existing cancel tests
      (`canceling_stops_the_scan_early`,
      `the_faces_pass_delivers_every_index_once_and_stops_on_cancel`) still
      pass. Unit tests in app: `run_scan` and `run_faces_scan` with the flag
      set from `progress` still satisfy `written.len() == summary.total`
      (the existing `canceling_after_the_first_batch_keeps_what_was_written`
      and `canceling_the_faces_pass_still_reports_what_was_written`), and a
      new assertion that no row exists for a path that was abandoned (e.g.
      the paths not in `entries("d")` have no row at all, and in the faces
      pass their `faces_extractor` is still `0`, so `faces_todo` lists them).
    - `EXTRACTOR_VERSION` and `FACES_VERSION` are not bumped: what a
      *completed* extraction produces is unchanged.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/core/src/scan.rs`: change `for_each_path`'s `per_file` to
      `Fn(&Path, &AtomicBool) -> Option<Result<T, String>>`; `None` means
      abandoned and skips `on_item`. Keep the existing pre-dispatch check.
      Add private `extract_unless(path, cancel)` and
      `extract_faces_unless(path, cancel)` (naming is free; keep them
      `pub(crate)` or private) and make `extract` / `extract_faces` call them
      with a never-set flag, mirroring how `extract_faces_all` mirrors
      `extract_all`. A small helper such as
      `fn canceled(cancel: &AtomicBool) -> bool` keeps the checks one line
      each; `Ordering::Relaxed`, as elsewhere.
    - Cancel checks for the first pass live in `extract_unless` between the
      existing calls (`read_preview` -> check -> `thumbnail_jpeg` -> check ->
      `detect_around` (only on the `None` focus branch) -> check ->
      `score_preview`). Note that `detect_around` and `score_preview` each
      decode the preview again; do not restructure that, it is out of scope.
    - Faces pass: `focus_cue` in `crates/core/src/candidate.rs` decodes and
      detects inside one function, so add a cancel-aware variant there
      (e.g. `focus_cue_unless(preview, orientation, focus, cancel)`, checking
      after `decode_rgb`, before `detect_around_rgb`, and before
      `eye_sharpness`) and keep `focus_cue` as the wrapper with a never-set
      flag so the CLI's `candidate::focus_cue` call and the existing candidate
      tests are untouched. `scan::cue` wraps the variant in the same
      `catch_unwind` as today; a panic or `Err` still maps to
      `Cue::unknown()`, an abandoned file maps to `None`.
    - Return shape for abandoned: `Option<Result<T, String>>` (not a sentinel
      error string, which `run_scan` would persist as an error row that is not
      retried until an `EXTRACTOR_VERSION` bump).
    - `crates/app/src/index.rs`: no logic change should be needed in
      `run_scan` / `run_faces_scan` because abandoned files never reach
      `on_item`; update the doc comments (done semantics on cancel) and add
      the new assertions to the two existing cancel tests (or one new test
      each). `crates/app/src/commands.rs`: update the `run_faces_pass` /
      `start_scan` doc comments only if their wording about "rows it had not
      written yet keep their old `faces_extractor`" needs the in-flight case
      spelled out; keep `scan_folder`'s cancel-then-join unchanged.
    - Granularity note to record in `learnings.md` during implementation:
      the log's `files=1 ... canceled=true in 8030ms` means a single file's
      pipeline took 8 s, so one stage (a cold `read_preview` on a contended
      disk, or the whole-image YuNet inference / mozjpeg decode under 22
      competing threads) can still be seconds long. Stage-level checks bound
      the wait to one stage per worker, not zero; going inside mozjpeg / the
      ONNX inference is out of scope.
    - Update `README.md` / `README.ja.md` only if they describe cancel
      behavior (a quick grep for "cancel" in both; currently unlikely).
  - Manual check (left to the user; the implementer cannot drive the GUI):
    open a large folder, then another; `scan extract ... canceled=true in Nms`
    in Riffle.log is much shorter than before, and the new folder's
    thumbnails appear sooner.

## Trade-offs and risks

- **Abandoned files and `done`**: not counted (they never reach `on_item`),
  which keeps `written.len() == summary.total` and the `ready` contract
  intact.
- **Where the faces-pass checks live**: a cancel-aware variant in
  `candidate.rs` rather than duplicating `focus_cue`'s logic in `scan.rs`,
  to avoid drift between the scan and the CLI.
- **Granularity**: cancellation still cannot interrupt a stage in progress;
  the acceptance criterion is "much shorter", not instant.
- **`extract_all` doc contract** changes from "files already running finish"
  to "files already running stop at the next stage and are not delivered";
  the CLI never cancels, so its benchmarks are unaffected.

## Progress

- (2026-09-26) Step 1 complete
