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

# `riffle-cli check`: verify every sample file opens as the app opens it

## Purpose

A dummy ExifTool test CR3 (`Canon_EOS_M50_CanonRaw.cr3`, whose `PRVW` box
declares a size past its uuid box) surfaced "CR3 preview box malformed" in
the app, and there was no CLI way to find such files ahead of time.
`riffle-cli scan <dir> [threads]` is a throughput benchmark: it lists only
the folder's direct children and reports the number of errors, never which
file or why. A `check` subcommand walks a sample tree such as
`D:\Photos\samples\<FORMAT>\...`, runs on every file the same core functions
the app's thumbnail, preview and 1:1 views call, prints each failure with
its stage and message, and exits non-zero when anything failed, so a new
parser or sample set can be verified in one command.

## Steps

- [x] Step 1: Add the `check` subcommand to `crates/cli/src/main.rs`
  - Done when:
    - `riffle-cli check <dir>... [threads]` parses its arguments as
      `candidates` does (one or more dirs, an optional trailing thread
      count; `bail!` with a usage line when no dir is given, and when
      `threads == 0`).
    - It walks every dir recursively and collects the files
      `riffle_core::scan::is_raw_file` or `riffle_core::scan::is_jpeg_file`
      accepts (the app lists both: RAW folders, and JPEG folders as
      view-only), sorted; other files (`.NRW`, `.xmp`, `.dop`, manifests)
      are skipped silently.
    - Per file, in parallel on a rayon pool of `threads` workers, it runs
      the stages the app runs and records the first error of each stage:
      - `scan`: `scan::extract(path)` (index pass 1: bounded read,
        metadata, thumbnail encode; returns `Result<Entry, String>`).
      - `preview`: `reader::read_preview(path)` then
        `decode::decode_rgb(&jpeg)` (the `preview` command's bytes, decoded
        as the WebView would).
      - `full`: `reader::read_full(path)` then
        `partial::decode_focus_crop(&jpeg, arw.shot.focus, CROP_SIZE, CROP_SIZE)`
        (the `focus_crop` command), skipped without error when the file's
        metadata reports no full-size image (`arw.full.is_none()`, as
        `bench` does).
    - Only failures are printed, one line per failed stage:
      `<path>  <stage>: <error>`, in path order (collect results, then
      print sorted, so the output is stable across runs).
    - A summary follows: one line per lower-cased extension with its ok
      and failed counts (e.g. `cr3: 108 ok, 1 failed`), then a total line
      with the file count, folder count, thread count and elapsed time in
      the style of `candidates`' final line.
    - The process exits non-zero when any file failed (a final
      `bail!("{n} file(s) failed")` is enough: `anyhow`'s `main` prints
      it and exits 1) and zero otherwise.
    - The usage string in `main`'s fallthrough arm gains
      `riffle-cli check <dir>... [threads]`; `scan` is untouched.
    - A `#[cfg(test)] mod tests` in `main.rs` unit-tests the pure
      per-extension tally helper (feed a few `(path, failed)` pairs with
      mixed-case extensions, assert the counts), or, if the helper is
      folded into the printing loop, the recursive collector on a
      throwaway directory under `std::env::temp_dir()` (no new dev
      dependency).
    - `cargo run -p riffle-cli --release -- check D:\Photos\samples` walks
      all subfolders, prints the failures and the summary, and the exit
      code matches; record the result (counts, any failures and whether
      they are genuine sample defects) in `learnings.md`.
    - `mise run ci` passes (`cargo clippy --all-targets -- -D warnings`
      covers the test module).
  - Implementation approach:
    - Only `crates/cli/src/main.rs` changes (no new dependencies: `anyhow`,
      `rayon`, `riffle-core` are already there; write the recursive walk
      with `std::fs::read_dir` rather than adding `walkdir`).
    - Reuse the existing imports (`decode_rgb`, `partial`, `reader`,
      `scan`, `CROP_SIZE`); mirror `candidates`' pool setup
      (`rayon::ThreadPoolBuilder` + `pool.install(|| paths.par_iter()...)`)
      and its default thread count.
    - A directory that cannot be listed during the walk should be reported
      as a failure line (`<dir>  list: <error>`) and counted, not abort
      the run, so one unreadable subfolder does not hide the rest; a top-
      level dir that does not exist may still `bail!`, as `candidates`
      does.
    - `scan::extract` already wraps the mozjpeg panic in `catch_unwind`
      and `decode_rgb` does too. `decode_focus_crop` turned out not to be
      pure Rust: it calls libjpeg through `mozjpeg-sys` with the default
      `error_exit`, which ends the process on a malformed JPEG (found on
      the sample run), so the `full` stage runs `decode_rgb` on the
      full-size JPEG first and only hands vetted bytes to the partial
      decode (see `learnings.md`).
    - Do not add an analysis stage: `scan::extract_analysis` only errors
      on an unreadable file, which the `scan` stage already catches, and
      it would add face detection on every file (minutes over 3700 files).
    - README.md / README.ja.md do not document the CLI subcommands, so
      leave them alone; `docs/performance.md` mentions `bench` / `scan` /
      `candidates` only as measurement tools and needs no update.

## Trade-offs and risks

- **Preview decode proxy.** The app's `preview` command sends the JPEG
  bytes to the WebView, which decodes them with the browser's decoder; the
  CLI cannot do that, so `decode_rgb` (mozjpeg) stands in. A file the
  browser rejects but mozjpeg accepts (or vice versa) will differ. This is
  the closest available check and matches `bench`.
- **`full` stage on files without a full-size JPEG.** Skipping when
  `arw.full.is_none()` mirrors `bench`; the alternative, always calling
  `read_full` as the app's `focus_crop` does, would report every
  preview-only file (some DNGs, HDR PQ CR3s) as failed even though the app
  simply shows no 1:1 view for them. If the caller wants those surfaced,
  print them as a separate `no full` note rather than a failure.
- **Faces stage.** `faces_of` (`faces::detect_around`) is another app
  path a corrupt preview could break, but it runs the YuNet model per file
  and would multiply the runtime; left out. Could be an opt-in flag later
  if a face-related error ever shows up in the app.
- **`.jpg` inclusion.** The app only lists JPEGs when a folder has no RAW;
  the check includes every JPEG regardless of its folder, so a JPEG the
  app would never show can be reported. This is intentional (it is a
  sample verification, not a listing) but means a `JPG` subfolder full of
  odd test images may produce noise.
- **Sample state.** `Canon_EOS_M50_CanonRaw.cr3` is not currently under
  `D:\photos\samples`, so the verification run proves the walk and the
  summary but not that specific failure; if the caller still has the file,
  dropping it into `CR3/` before the run demonstrates the failure line.
- **Exit via `bail!`.** Prints `Error: N file(s) failed` after the summary
  and exits 1; a bare `std::process::exit(1)` would be quieter. `bail!`
  matches how the rest of `main.rs` reports errors.

## Progress

- (none yet)
