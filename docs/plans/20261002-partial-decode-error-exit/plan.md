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

# Make the partial JPEG decode return `Err` on a malformed JPEG

## Purpose

`crates/core/src/partial.rs` (`decode_region`, behind `decode_focus_crop`
and `decode_crop`) calls libjpeg through `mozjpeg-sys` with the error manager
`jpeg_std_error` fills in and never replaces its `error_exit`. libjpeg's
default `error_exit` calls `exit(1)`, so a malformed full-size JPEG ends the
whole process: `riffle-cli check` over `D:\Photos\samples` died mid-run on
`NEF\NIKON_D70_Nikon.nef` and `DNG\CGO3P_YUN00007.dng` (stderr: `Empty input
file` / `Not a JPEG file`) without printing its summary, and the app's
`focus_crop` command (the 1:1 view) runs the same function, so opening such a
file at 1:1 would kill the app. `check_file` in `crates/cli/src/main.rs`
works around it by running the guarded `decode_rgb` on the bytes first, a full
decode per file that makes `check` take minutes instead of seconds.

Once the partial decode reports the fatal error as `Err`, the 1:1 view shows
an error for that file and stays up, and `check` can drop the pre-check.

## Steps

- [x] Step 1: Install a panicking `error_exit` in `partial.rs`, guard it with `catch_unwind`, drop the CLI pre-check, update the docs and close the todo
  - Done when:
    - `decode_region` in `crates/core/src/partial.rs` replaces
      `err.error_exit` after `jpeg_std_error(&mut err)` with an
      `extern "C-unwind"` function that unwinds, and both public entry
      points (`decode_focus_crop`, `decode_crop`) return `Err` instead of
      exiting on a malformed JPEG.
    - A unit test in `partial.rs`'s `tests` module asserts `Err` for an
      empty input (`b""`, libjpeg's `Empty input file`), a non-JPEG input
      (`b"not a jpeg"`, `Not a JPEG file`) and a truncated synthetic JPEG
      (the first half of `jpeg(320, 240)`), and the existing tests still
      pass.
    - The `decode_rgb(&jpeg)?` pre-check and its comment in `check_file`'s
      `full` stage (`crates/cli/src/main.rs`, around line 634) are removed;
      `riffle-cli check D:\Photos\samples` runs to its summary, lists the
      two repro files under the `full` stage, and the run time is recorded
      in `learnings.md` next to the previous ~2.5 minutes.
    - `focus_crop` in the app (or, as the stand-in, `riffle-cli check` /
      the `focus` CLI path at `crates/cli/src/main.rs` line ~678) returns
      an error on the two repro files and the process stays alive.
    - The `docs/agents/tauri-app.md` entry "`partial.rs`'s libjpeg calls
      `exit(1)` on a corrupt JPEG; `catch_unwind` cannot stop it (Hit)"
      (line ~258) is rewritten to say the guard is in place: `error_exit`
      now unwinds and `decode_focus_crop` / `decode_crop` return `Err`;
      callers no longer need the `decode_rgb` pre-check; keep the note that
      libjpeg warnings still go to stderr; keep the source line and add
      this plan's `learnings.md` as a second source.
    - The todo.md heading "Core: a malformed full-size JPEG makes the 1:1
      view end the whole app" is closed out the way the repository closes
      finished items (check its boxes / move it as the other done items in
      `todo.md` are handled).
    - `mise run ci` passes.
  - Implementation approach:
    - Model the handler on the `mozjpeg` crate's `src/errormgr.rs`
      (`~/.cargo/registry/src/*/mozjpeg-0.10.13/src/errormgr.rs`):
      `jpeg_std_error(&mut err)`, then `err.error_exit =
      Some(unwind_error_exit)`, where `unwind_error_exit` is an
      `extern "C-unwind" fn(&mut sys::jpeg_common_struct)` that formats the
      message through `err.format_message` into an 80-byte buffer and calls
      `std::panic::resume_unwind(Box::new(msg))` (no panic hook, so no
      stray stack trace on stderr). `mozjpeg-sys` 2.2.3 declares the
      `error_exit` field as `Option<unsafe extern "C-unwind" fn(...)>`
      (its `src/lib.rs` line 607), so no transmute is needed for the
      callback itself; the `format_message` field's buffer parameter is
      declared `&[u8; 80]` and the mozjpeg crate transmutes it to
      `&mut [u8; 80]` — copy that or write a simpler fallback
      (`msg_code` only) if the transmute feels like too much.
    - Unwinding across the C frames needs libjpeg built with unwind
      tables: `mozjpeg-sys`'s `unwinding` feature. It is in the crate's
      default features and `crates/core/Cargo.toml` depends on
      `mozjpeg-sys = "2.2"` with default features, so it is already on;
      the `mozjpeg` crate (also a dependency) enables it explicitly. State
      `features = ["unwinding"]` explicitly on the `mozjpeg-sys` dependency
      in `crates/core/Cargo.toml` so a later `default-features = false`
      cannot silently drop it (no `Cargo.lock` change expected). No
      workspace profile sets `panic = "abort"` (verified), so unwinding
      works in both debug and release.
    - Wrap the body with the same guard `crates/core/src/decode.rs` uses:
      `std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ...))
      .map_err(|e| anyhow!(...))?`. Pull the message out of the payload
      (`e.downcast::<String>()`) so the `Err` carries libjpeg's text
      (e.g. `libjpeg fatal error: Not a JPEG file: starts with 0x6e 0x6f`),
      which `check` then prints. Put the guard in `decode_region` (one
      place covers both public functions), e.g. rename the current body to
      `decode_region_unguarded` as `decode.rs` does.
    - Cleanup on the error path: `ScopeGuard` already calls
      `jpeg_destroy_decompress` in `Drop`, and the unwind runs it before
      `err` (declared before `cinfo`, so dropped after it) leaves scope;
      `jpeg_destroy_decompress` is the documented thing to call after
      `error_exit` returns control, so no extra work is needed beyond
      keeping the guard in place. Check that the region closure (which
      captures `point` by `&mut` in `decode_focus_crop`) still compiles
      inside the `AssertUnwindSafe` closure.
    - Leave `emit_message` alone (warnings keep going to stderr, as the
      docs entry notes); silencing them is not part of this task.
    - Test input: the existing `jpeg(w, h)` helper gives a valid JPEG to
      truncate; `b""` and `b"not a jpeg"` cover the two libjpeg messages
      seen on the repro files. Mirror `decode.rs`'s
      `a_malformed_jpeg_is_an_error` style test.
    - Verify on the real files: `cargo run -p riffle-cli --release -- check
      D:\Photos\samples` (expect the summary and the two files listed under
      `full`), and `cargo run -p riffle-cli --release -- focus
      D:\Photos\samples\NEF\NIKON_D70_Nikon.nef ...` (or whatever the CLI
      subcommand at line ~678 is named) to see the `Err` text instead of
      exit code 1. Running the desktop app on the two files is the
      acceptance check if the user can do it; the CLI path calls the same
      `decode_focus_crop`.
    - Files: `crates/core/src/partial.rs`, `crates/core/Cargo.toml`,
      `crates/cli/src/main.rs`, `docs/agents/tauri-app.md`, `todo.md`,
      this plan's `plan.md` / `learnings.md`.

## Trade-offs and risks

- `resume_unwind` vs `panic!`: `resume_unwind` skips the panic hook, so
  nothing is printed to stderr for an expected bad file (the `mozjpeg`
  crate's choice). `panic!` would print "thread panicked at ..." for every
  malformed JPEG. The plan takes `resume_unwind`; if the caller prefers the
  visible trace for debugging, switch to `panic!` with the same message.
- Where to put `catch_unwind`: in `decode_region` (chosen, one guard for
  both public functions) or at each public function. Guarding inside the
  `unsafe` block around the libjpeg calls only would be tighter but makes
  the `ScopeGuard` / `err` lifetimes harder to read.
- Unwinding through libjpeg's C frames relies on `mozjpeg-sys`'s
  `unwinding` feature, which is a build-flag change on the C side
  (`-fexceptions` / unwind tables). It is already on today because the
  `mozjpeg` crate requests it and the default features include it, so the
  binary's behavior does not change; making it explicit in
  `crates/core/Cargo.toml` is belt-and-braces only. If the implementation
  finds the process still exits on the repro files, the first thing to
  check is `cargo tree -e features -p mozjpeg-sys`.
- The app's `focus_crop` call site needs no code change: it already maps
  `decode_focus_crop`'s `Err` to a `String` for the frontend. Whether the
  frontend shows that error well for the 1:1 view is not in scope; note in
  `learnings.md` what the UI does if it is observed.
- Single PR: the core fix, the CLI pre-check removal, the docs and the todo
  close-out are all one step as requested. Splitting the CLI change out
  would leave an intermediate state where the pre-check is redundant but
  harmless, so the single step is the natural unit.

## Progress

- (none yet)
