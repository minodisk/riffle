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

# Sequence the timestamps of exported JPEGs

## Purpose

The culling workflow today needs three apps: Riffle to cull, a RAW developer
to export JPEGs, and [lapse](https://github.com/minodisk/lapse) to make the
`DateTimeOriginal` of the exported burst JPEGs unique at second granularity so
Google Photos (which ignores `SubSecTimeOriginal`) keeps them in shooting
order. This work brings that step into Riffle, so the workflow needs two apps:
`File > Sequence JPEG Timestamps…` picks the export folder, shows the
timestamps that would be written, and writes a complete copy of the folder's
JPEGs with the sequenced timestamps next to it.

It deliberately deviates from lapse in two ways, decided with the user:

- **Order by capture time, not filename.** A folder exported from two bodies
  (Sony `DSC…` and Leica `L…` names) interleaves by time, and Sony's
  `9999 → 0001` rollover breaks filename order. The sort key is
  (`DateTimeOriginal`, `SubSecTimeOriginal`, natural filename order).
- **Copy output, originals untouched.** The result goes to a sibling folder
  `<folder>-sequenced/`; the source is never written, so every run recomputes
  from the original times and is reproducible after files are added,
  removed or renamed. The output folder is rebuilt on every run so it always
  mirrors the source.

What stays as in lapse v0.4.0: `.jpg` / `.jpeg` (case-insensitive) directly
under the folder, no recursion; `new = max(original, previous new + 1 s)`,
first file keeps its time; the in-place, length-preserving patch of 0x9003 and,
when present, 0x9004 and 0x0132 (never creating tags, never touching another
byte); temp file + atomic rename per written file; per-file failure isolation;
"no JPEGs" is an error.

## Steps

- [x] Step 1: Port lapse's EXIF patcher into `riffle-core` as `sequence.rs`, with time ordering, copy output and cancel/progress hooks
  - Done when:
    - `crates/core/src/sequence.rs` (exported from `lib.rs`; module doc says
      "derived from minodisk/lapse v0.4.0, MIT" and keeps lapse's
      crate-selection notes explaining the in-house TIFF walker) holds:
      `ExifDateTime` (lapse's `exif_datetime.rs`), the APP1/TIFF walker and
      patcher (lapse's `exif_patch.rs`: `find_datetime_offsets`,
      `read_datetime_at`, `patch_datetimes`, `DATETIME_LEN`,
      `DateTimeOffsets`), a new reader for `SubSecTimeOriginal` (0x9291,
      Exif IFD, ASCII, digits; parsed as a fraction so `"5"` and `"12"` are
      0.5 > 0.12, i.e. left-aligned decimal digits, not an integer),
      `collect_jpegs` (natural order via `natord`, kept as the tie-break),
      `output_dir(folder) -> <parent>/<name>-sequenced`, `plan(dir, ...)`
      (read originals, sort, assign) and `run`.
    - Ordering: files are sorted by (`DateTimeOriginal`, `SubSecTimeOriginal`
      with a missing tag treated as 0, natural filename). Assignment walks
      that order: `new = max(original, previous new + 1 s)`, first keeps its
      original time. Files whose `DateTimeOriginal` cannot be read are
      excluded from the assignment and reported as failures (as in lapse).
    - Output: `run` creates `<folder>-sequenced/` if missing, deletes the
      `.jpg` / `.jpeg` files directly inside it (not recursive; any other
      file or subfolder is left alone), then writes every target JPEG there
      under its own file name, patched copies and unchanged copies alike,
      each through a `NamedTempFile` in the output folder and `persist`
      (rename), so no half-written file can ever be observed. The source
      files are never opened for writing. Copies do not carry the source
      mtime or permissions.
    - `run` follows `scan::extract_all`'s shape: it takes the directory,
      `dry_run: bool`, a `&AtomicBool` cancel flag and a progress callback
      (`done`, `total`, plus the file's outcome so the app can stream it),
      and returns a `Summary` with the results in the computed order, one
      `Result<FileOutcome { old, new, subsec, changed }, _>` per file, and
      `canceled: bool`. Reading is parallel (rayon), the assignment is
      sequential, writing is parallel; the cancel flag is checked before
      deleting the old output and before each file's write. A canceled run
      leaves the output folder with only the complete files written so far
      (documented on `run` and shown in the UI; the next run rebuilds it).
      In `dry_run` nothing on disk is touched, not even the output folder.
    - A picked folder already named `*-sequenced` is not refused; its output
      is `x-sequenced-sequenced`.
    - Dependencies: `natord = "1"`, `chrono = "0.4"`, `tempfile = "3"` in
      `riffle-core`; dev-dependencies `kamadak-exif = "0.6"` and `image`
      with the `jpeg` feature. No `clap`.
    - Tests (`crates/core/tests/sequence.rs` with the synthetic JPEG builder
      ported from lapse's `tests/common/mod.rs`, extended to take an optional
      `SubSecTimeOriginal` and to emit it inline in the value field when it
      fits in 4 bytes, plus a variant with it as an offset value):
      - ported from lapse and adapted to copy output + time order: lossless
        bytes-and-pixels on the copy (diffs confined to the 19-byte value
        areas; scan data and decoded pixels identical), other EXIF tags
        preserved, minute carry-over and 0x9004 / 0x0132 sync, copies still
        decode as JPEG, dry run touches nothing, no-JPEG error and no-EXIF
        per-file error, one failing file does not stop the others, scene
        gaps preserved and pushed only when caught up, idempotent (a second
        run yields byte-identical output), extension filter;
      - new: multi-body mix (`DSC00001..3` and `L1000001..3` interleaved by
        time) comes out in time order; SubSec tie-break inside one second;
        natural filename tie-break when time and SubSec are equal (and when
        SubSec is absent on both); a file without SubSec sorts before one
        with SubSec in the same second; source files byte-identical after a
        run; unchanged files are copied too; rerun after adding a file
        equals a fresh run on the same set; rerun removes an output file
        whose source was deleted; a `.txt` and a subfolder in the output
        folder survive a rebuild; cancel set before writing leaves no
        partial file (every file in the output folder is a complete JPEG
        and the temp file is gone); progress reaches `total`.
    - `cargo test -p riffle-core`, `cargo clippy --all-targets -- -D warnings`
      and `mise run ci` pass.
  - Implementation approach:
    - Keep lapse's code and comments; adapt error types to what
      `riffle-core`'s public functions return (check `scan.rs` / `xmp.rs`).
    - `ascii_value_abs` in lapse assumes the value is an offset (count ≥ 19).
      For 0x9291 add an inline path: when `count <= 4` the bytes are the
      value field itself. Riffle's `reader.rs` already parses ARW SubSec
      into `shot.subsec` as a string; reuse its interpretation of the
      digits if there is one, otherwise document the fraction rule here.
    - Model the cancel/progress plumbing on `for_each_path` in
      `crates/core/src/scan.rs` (`Ordering::Relaxed`).

- [x] Step 2: Backend commands, run state, events and the `File` menu item
  - Done when:
    - Step 1 is merged.
    - `crates/app/src/sequence.rs` (new; registered in `main.rs`'s
      `generate_handler!`) provides async commands:
      `sequence_preview(dir)` (dry run in `spawn_blocking`; returns
      `{ output_dir, output_exists, rows: [{ path, old, new, changed }],
      failed: [{ path, message }] }` in the computed order, or an error
      string such as the "no JPEGs" one), `sequence_run(dir)` (refused with
      an error string while another run is in progress; returns a run id,
      spawns the work, emits `sequence-progress` `{ run_id, dir, done,
      total }` and `sequence-done` `{ run_id, dir, output_dir, written,
      total, failed: [{ path, message }], canceled }`), and
      `sequence_cancel(run_id)`.
    - `File > Sequence JPEG Timestamps…` is added in `crates/app/src/main.rs`
      after `Move Rejected to Trash…` (icon on macOS if the neighbours have
      one under `icons/menu/`), emitting a `sequence-timestamps` event to
      the main window like `trash-rejected`.
    - Unit tests in the app crate: a second run is refused while one runs
      and the state clears when the task ends (mirror
      `a_finished_scan_leaves_no_scan_in_progress`); the preview payload for
      a synthetic folder is in time order with `changed` and `output_dir`
      set correctly.
    - `mise run ci` passes.
  - Implementation approach:
    - Follow `docs/agents/tauri-app.md`: blocking work in
      `tauri::async_runtime::spawn_blocking`; the run state in a
      `tauri::State` like `Scans` (`Mutex<Option<(run_id, Arc<AtomicBool>,
      JoinHandle)>>`, via `index::lock`). `sequence-done` must follow the last
      `sequence-progress` of the same run id; one worker thread gives that.
    - Nothing touches `index.rs` or `EXTRACTOR_VERSION`: JPEGs are not
      indexed. Do not refuse while a scan runs. If the source folder is the
      open RAW folder, the output is a sibling so the watcher's
      `folder-changed` does not fire for it; verify once and note it in the
      doc comment.

- [x] Step 3: Frontend: pick the folder, preview, run with progress, cancel and errors
  - Done when:
    - Step 2 is merged.
    - The menu event opens the native folder picker (`pick_folder`, no
      default folder), then calls `sequence_preview`. A dialog
      (`#sequence-dialog` in `crates/app/ui/index.html`, styled like
      `#format-dialog` / `#settings-dialog`) shows: the source folder; the
      output path and, when it exists, "will be rebuilt: its JPEG files are
      replaced"; one row per file in the computed order, `name  old -> new`
      (rows with `changed: false` dimmed); the line `N of M files get a new
      time`; the failures (`name: message`); buttons `Run` and `Cancel`.
      `Escape` closes it and keys do not reach the strip while it is open
      (reuse `modal.ts`'s handling as the settings modal does).
    - `Run` calls `sequence_run`; while it runs the meta-pane status line
      shows `sequencing done / total` the way `scanning done / total` does,
      the dialog's `Cancel` (and `Escape`) calls `sequence_cancel`, and
      `sequence-done` sets the status to `Wrote N of M files to
      <output_dir>` (`, K failed` appended; `canceled, N of M written` when
      canceled), adds each failure to the sticky `errors` list keyed by
      path, and closes the dialog. A preview error goes to the status line
      via `setStatus` like the other commands' errors.
    - The pure parts live in `crates/app/ui/src/sequence.ts` with
      `sequence.test.ts`: row formatting, the rebuild notice, summary and
      status text, and the dialog's state machine (picking → previewing →
      previewed → running → done), keeping `main.ts` to wiring.
    - `mise run ci` passes.
  - Implementation approach:
    - Follow `trashRejected` in `main.ts` for how a result updates the
      status line and `errors`, and `trash.ts` for the helpers' shape.
    - Ignore progress/done events whose `run_id` differs, as the scan
      listeners do with `scan_id`.

- [x] Step 4: Document the workflow
  - Done when:
    - `README.md` and `README.ja.md` (in sync) add a "Key features" entry:
      cull in Riffle, export JPEGs from the developer, then
      `File > Sequence JPEG Timestamps…` on the export folder; why (Google
      Photos and burst order), the ordering rule (capture time, sub-second,
      then filename; so two bodies in one folder sort correctly), the copy
      output to `<folder>-sequenced/` with the originals untouched, and the
      rebuild on rerun.
    - `docs/usage.md` gets a Features entry next to "Move Rejected to Trash…"
      with the full behavior: file selection, ordering, the assignment rule,
      the three tags, lossless patch, preview, rebuild of the output folder
      (only its JPEG files are removed), cancel semantics (complete files
      only remain), failures.
    - `CLAUDE.md`'s Layout paragraph mentions `crates/core/src/sequence.rs`,
      `crates/app/src/sequence.rs` and `crates/app/ui/src/sequence.ts`.
    - `mise run lint` (including lychee) passes.

## Trade-offs and risks

- **Port vs. depend on `lapse`.** Ported: lapse is not on crates.io, its
  `run` has neither the time ordering nor the copy output nor cancel/progress
  hooks, and a git dependency would be a second personal build dependency
  next to `muda`. Consequence: lapse's patcher now lives in two places.
- **Missing SubSec sorts first** (treated as 0) within the same second
  (decided with the user). Files from a body that writes no SubSec mixed
  with one that does may interleave unexpectedly inside a single second.
- **Rebuild deletes JPEGs in the output folder.** Only `.jpg` / `.jpeg`
  directly inside `<folder>-sequenced/` are removed; a user who put other
  JPEGs there loses them. The preview states this. Delete-then-write means a
  canceled run leaves a partial output folder (decided with the user); a
  temp-folder swap would double disk usage during a run.
- **Picking a `-sequenced` folder** derives `x-sequenced-sequenced`
  (decided with the user: not refused).
- **Copies do not carry the source mtime or permissions** (decided with the
  user). Google Photos orders by EXIF.
- **Memory:** each file is read whole, patched and written (as in lapse);
  with rayon that is `threads × file size`, fine for JPEGs.
- **Not covered:** subfolders, HEIC/other formats, creating missing tags —
  all as in lapse.

## Follow-ups

- Open and preview JPEG-only folders in the strip (e.g. to check the
  `-sequenced` output). Out of scope here; a separate plan, to be carried
  into `todo.md` at wrap-up.

## Progress

- (2026-09-26) Step 1 complete
- (2026-09-26) Step 2 complete
- (2026-09-26) Step 3 complete
- (2026-09-26) Step 4 complete
