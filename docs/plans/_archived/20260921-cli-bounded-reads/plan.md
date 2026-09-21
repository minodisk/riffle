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

# CLI bounded reads

## Purpose

`info`, `focusbox` and `bench` in `crates/cli/src/main.rs` still read the whole
file with `std::fs::read` and parse it with `arw::parse`, while `crop`, the
`scan` path and every read in the app go through `riffle_core::reader`, which
reads a bounded 1 MiB prefix, reads an embedded JPEG by range when it lies past
that prefix, and falls back to the whole file when the prefix is too short to
parse. The todo.md item asked whether the three subcommands inherently need the
whole file. They do not: `info` needs only metadata (`reader::read_metadata`),
`focusbox` needs metadata plus the IFD0 preview (`reader::read_preview`), and
`bench` needs the preview and the JpgFromRaw (`reader::read_preview` plus
`reader::read_full`, whose ranged read is its normal path for the JpgFromRaw).
Moving them makes the CLI exercise the same read path as the app, and closes
the todo item.

`bench` times only the decodes (`decode_rgb`, `decode_focus_crop`); the file
read sits outside every timed window, so the reported numbers are unaffected.
The read cost stays excluded from the measurement, as it is today.

## Steps

- [x] Step 1: Move `info`, `focusbox` and `bench` to `riffle_core::reader` and drop the todo.md item
  - Done when:
    - `info` uses `reader::read_metadata(path)?` instead of `std::fs::read` + `arw::parse`.
    - `focusbox` uses `let (a, jpeg) = reader::read_preview(path)?;` and decodes `&jpeg` instead of `a.slice(&buf, e)`.
    - `bench` uses `reader::read_preview` for the preview timing and, only when `a.full.is_some()`, `reader::read_full` for the full-decode and crop timings, so a file without a JpgFromRaw is still skipped rather than erroring. The `Instant::now()` windows still wrap only the decode calls.
    - `std::fs::read` no longer appears in `crates/cli/src/main.rs`; the `arw` import is removed if it becomes unused.
    - The item "App: `riffle-cli info`/`focusbox`/`bench` read whole files instead of the bounded prefix" is removed from `todo.md`.
    - Output is verified unchanged: before the change, run `cargo run -p riffle-cli -- info <file>`, `focusbox <file> before.png` and `bench <files...>` on at least one ARW and one DNG the implementer has locally, and save the stdout and the PNG; after the change, run the same commands and confirm `info` stdout is byte-identical, `focusbox` stdout is identical apart from the decode-time line and the PNG is byte-identical (`cmp before.png after.png`), and `bench` prints the same three stat lines with numbers in the same range. Record the commands and results in `learnings.md`. If no sample file is available on the implementing machine, say so in `learnings.md` and rely on `mise run ci` plus the reader unit tests.
    - `mise run ci` passes.
  - Implementation approach:
    - Follow the shape `crop` used in commit `c2eafea` (`let (a, jpeg) = reader::read_full(path)?;`). No new helpers, no changes to `crates/core`.
    - `focusbox` currently produces `no preview in {path:?}` when the preview is missing; `read_preview` produces `no embedded preview`. Accept the reworded error; do not add code to preserve the old wording.
    - In `bench`, keep the `if let Some(e) = a.full` gate semantics by checking `a.full.is_some()` on the `Arw` returned by `read_preview` before calling `read_full`. The `Arw` returned by `read_full` is the same parse of the same prefix, so either can supply `a.shot.focus`.
    - Keep the whole change surgical: do not touch `crop`, `scan_dir`, `stats`, `draw_rect` or the usage string.

## Trade-offs and risks

- `bench` will open and parse the prefix twice per file (`read_preview` then `read_full`) where it used to read once. This is untimed setup, and it reads far less data than the whole file, so it is a net reduction; the alternative of adding a `reader` function that returns both JPEGs in one pass was rejected as a speculative abstraction for a single caller.
- The `focusbox` error message for a file without a preview changes wording. Only the error path is affected; success output is unchanged.
- The todo.md item is deleted rather than rewritten, because the code change resolves it and there is no residual "known but not worth changing" state to record.
- Verification of unchanged output depends on a local sample ARW/DNG; the repository holds no ARW/DNG fixture (`crates/core/src/fixtures` has only `dop`). The step spells out the fallback when none is available.

## Progress

- Step 1: `info`, `focusbox` and `bench` moved to `riffle_core::reader`; the todo.md item was
  removed. See `learnings.md` for the output-verification results and the round 1 review fix to
  `bench`'s preview/full gating.
- (2026-09-21) Step 1 complete
