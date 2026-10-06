<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Agent-readable app log

## Purpose

`Riffle.log` (written by `tauri_plugin_log`, configured in
`crates/app/src/main.rs`: Info in release, Debug in debug builds, 1 MB cap)
is the only trace an AI agent has of a running session, but today it misses
the failures that matter most: several errors still go to `eprintln!` (stderr,
which nobody captures in a bundled app), a Rust panic on a scan / faces /
sidecar-writer thread leaves no line, a sidecar write failure reaches only the
UI, and an uncaught JavaScript error or unhandled rejection stays in the
webview console. Nothing in the log says when an internal assumption broke
either. After this work every one of those lands in the log under a fixed,
greppable prefix (`panic:`, `uncaught-js:`, `invariant:`), and a guide under
`docs/agents/` tells an agent where the file is, what a line looks like, and
what to grep for.

## Steps

- [ ] Step 1: Stop losing errors: `eprintln!` to `log::`, a panic hook, and uncaught-JS forwarding
  - Done when:
    - No `eprintln!` remains in `crates/app/src` outside `#[cfg(test)]`
      modules (the two in `sidecar.rs`'s tests stay)
    - A panic on any thread (main, `spawn_blocking`, rayon worker, the sidecar
      writer thread) writes one `panic:` line to `Riffle.log` and still prints
      the default stderr message
    - An uncaught JS error and an unhandled promise rejection in the main
      window each write one `uncaught-js:` line, in release builds, with the
      Timing logs setting off
    - A sidecar write failure is logged as well as emitted to the frontend
    - The line formatters have unit tests (Rust and TS)
    - `mise run ci` passes
  - Implementation approach:
    - `eprintln!` in `crates/app/src/commands.rs` (lines ~514, 528, 548, 686,
      708, 756, 1668): replace with `log::warn!` keeping the same message text
      (every one is a degradation the app survives: settings fall back to
      defaults, a legacy file is left behind). `load_settings` runs from
      `.setup()`, after the log plugin is initialized, so the logger exists.
    - New module `crates/app/src/diagnostics.rs` (add `mod diagnostics;` in
      `main.rs`) holding:
      - `pub fn install_panic_hook()`: `std::panic::take_hook()` then
        `std::panic::set_hook` with a closure that calls
        `log::error!("{}", panic_line(...))` and then the previous hook.
        `panic_line(thread: Option<&str>, location: Option<&Location>,
        payload: &str) -> String` is a pure function producing one line of the
        form `panic: thread=<name or unnamed> location=<file>:<line>:<col>
        payload=<text>`. Payload text: `downcast_ref::<&str>()`, then
        `downcast_ref::<String>()`, else `"<non-string payload>"`. Newlines in
        the payload collapse to spaces so one panic is one line.
      - Call `install_panic_hook()` as the first statement of the `.setup()`
        closure in `main.rs` (plugin setup, including the log plugin, runs
        before the app's `setup`; a `log::error!` with no logger installed is
        a silent no-op, so installing earlier gains nothing). Document that a
        panic before `setup` only reaches stderr.
      - Backtrace: use `std::backtrace::Backtrace::capture()` (cheap and empty
        unless `RUST_BACKTRACE` is set) and append it only when its status is
        `Captured`, as additional lines after the `panic:` line in the same
        record.
      - `#[tauri::command] pub async fn log_frontend(kind: String, line:
        String)` next to the panic code (keep it `async fn` with owned args per
        `../../agents/tauri-app.md` "Synchronous commands run on the main
        thread"). It maps `kind` `"uncaught-js"` to
        `log::error!("uncaught-js: {line}")`; Step 2 adds `"invariant"` ->
        `log::warn!`. An unknown kind is logged at `warn` under its own text so
        nothing is silently dropped. Register it in `generate_handler!`. No
        capability change: `log_timing` already works under `core:default`.
    - Sidecar failures: in `main.rs`'s `Writer::spawn` closure (~line 711),
      add `log::warn!("sidecar write failed: path={} {message}", ...)` before
      the `sidecar-error` emit. The writer already retries with backoff, so
      the volume is bounded by `RETRY_LIMIT` per file.
    - Frontend: new pure module `crates/app/ui/src/uncaught.ts` (no DOM access
      at import time, so a node-env vitest can import it) with
      `uncaughtLine(kind: "error" | "unhandledrejection", message, source?,
      line?, col?, stack?) -> string` producing `kind=... message=...
      source=... line=... col=... stack=...` with newlines in `stack`
      collapsed to ` | `, and a small flood guard, e.g. `class UncaughtGate`
      that forwards the first 20 distinct lines per session and then one
      final `... suppressed` notice; a `requestAnimationFrame` or event-driven
      throw would otherwise append hundreds of lines per second and rotate
      the rest of the session out of the 1 MB file. Dedupe by the full line
      text so a repeated identical error counts once. Tests in
      `uncaught.test.ts` (follow `idle.test.ts`'s style).
    - `main.ts`: at the top (before any other initialization so early throws
      are caught), `window.addEventListener("error", ...)` (only
      `ErrorEvent`s: skip events without `message`, which are resource load
      errors) and `window.addEventListener("unhandledrejection", ...)`
      (`reason` may be an `Error` or anything: use `reason.stack ?? String(
      reason)`), each building the line and calling
      `window.__TAURI__.core.invoke("log_frontend", { kind: "uncaught-js",
      line })` fire-and-forget. Not gated by `debugLogging`. Keep the
      existing `console.error` behavior (do not `preventDefault`).
    - Rust tests for `panic_line` in `diagnostics.rs` (`#[cfg(test)] mod
      tests`), including the "non-string payload" and "no location" cases.
      Do not try to test the hook itself through a real panic in the test
      process.
    - Manual check before the PR: in a debug build, trigger a JS error from
      DevTools (`setTimeout(() => { throw new Error("x") })` and
      `Promise.reject(new Error("y"))`) and confirm the two `uncaught-js:`
      lines; a Rust panic can be forced temporarily on a scan worker in a
      scratch change and reverted.
    - Commit as `feat(app): log panics, uncaught JS errors and the remaining
      eprintln failures`.

- [ ] Step 2: Log invariant violations at the scan lifecycle's key points
  - Done when:
    - A helper exists that logs `invariant: <name>: <details>` at `warn` and
      has unit tests for the line format
    - The chosen checks are in place, log only (no control-flow change), and
      produce no line in a normal session (open a folder cold, rescan on
      focus, switch folders mid-scan, clear the cache); verified by running
      the app and grepping the log
    - `mise run ci` passes
  - Implementation approach:
    - In `crates/app/src/diagnostics.rs`: `pub fn invariant_line(name: &str,
      details: std::fmt::Arguments) -> String` (pure, tested) and a macro
      `invariant!(cond, "name", "details fmt", args...)` that, only when
      `cond` is false, calls `log::warn!("{}", invariant_line(...))`. The
      condition is evaluated once; everything else is behind the `if`, so the
      hot-path cost is one comparison.
    - Checks (backend, `crates/app/src/commands.rs`):
      1. `scan progress within total`: in the `scan-progress` and
         `faces-progress` callbacks of `start_scan` (~lines 1420 and 1455),
         `done <= total`, details `dir={dir} scan_id={scan_id} done={done}
         total={total}`. The callbacks fire at most ~10/s
         (`PROGRESS_INTERVAL`), so the cost is nil.
      2. `start_scan has a pending scan`: in `start_scan`, the `let
         Some(pending) = state.pending.remove(&scan_id) else { ... }` branch
         is reached only when `scan_id` is the latest id but `scan_folder`
         prepared nothing for it (a double `start_scan` or an id never handed
         out). Log `scan_id={scan_id} latest={latest_id}` there, keep the
         existing `emit_empty_scan_events` behavior.
    - Check (frontend, `crates/app/ui/src/main.ts`): `faces-done after
      scan-done`: in the `faces-done` listener, after the `scan_id !== scanId`
      filter, if `scanDone?.scanId !== payload.scan_id` the backend emitted
      `faces-done` for a scan whose `scan-done` the frontend never recorded
      (the backend always emits both, in order, including
      `emit_empty_scan_events`). Forward via `invoke("log_frontend", { kind:
      "invariant", line: "faces-done after scan-done: dir=... scan_id=..." })`
      and extend `log_frontend` to map `"invariant"` to `log::warn!`. Before
      keeping it, confirm against `main.ts` (~3070, ~3198, ~3489) that the
      folder-switch path (`scanId = null; scanDone = null`) cannot leave a
      stale `scanDone` for a live `scanId`; drop this check if a false
      positive is found and record why in `learnings.md`.
    - Deliberately not checked (record in the PR body): sidecar write results
      versus index state (`mark_written` legitimately leaves a row dirty when
      a newer judgment superseded the write, so a cheap check has false
      positives); trash undo/redo bookkeeping (`Runs` dropping a run past the
      newest 100 is by design and already surfaces as a defined user-facing
      error).
    - Commit as `feat(app): log invariant violations in the scan lifecycle`.

- [ ] Step 3: Document the log for agents and update the layout description
  - Done when:
    - A new guide `docs/agents/app-log.md` describes: the file location per
      OS (Windows `%LOCALAPPDATA%\com.minodisk.riffle\logs\Riffle.log`,
      macOS `~/Library/Logs/com.minodisk.riffle/Riffle.log`, Linux
      `~/.local/share/com.minodisk.riffle/logs/Riffle.log`, and `Help > Open
      Log Folder`), the line format
      (`[YYYY-MM-DD][HH:MM:SS][target][LEVEL] message`, UTC), levels per
      build, the 1 MB `KeepOne` rotation (name the rotated file as observed
      on disk), the existing timing lines in brief, and the exact shapes of
      the `panic:`, `uncaught-js:` and `invariant:` lines with a grep recipe
      (e.g. `grep -E '\[(ERROR|WARN)\]|panic:|uncaught-js:|invariant:'`) and
      how to read each
    - `CLAUDE.md`'s layout paragraph mentions `src/diagnostics.rs` (the panic
      hook, the `log_frontend` command, the invariant helper) and
      `ui/src/uncaught.ts`
    - `docs/agents/tauri-app.md`'s "Forward frontend timing lines through a
      command" item gets one sentence pointing at `log_frontend` as the
      second such command (keep the existing text)
    - `mise run ci` passes (lychee checks the new links)
  - Implementation approach:
    - Verify the paths and rotation naming against the running app on this
      machine (Windows) rather than from memory; take the macOS / Linux
      paths from `tauri-2.11.6/src/path/desktop.rs` `app_log_dir` and say so
    - No `README.md` / `docs/humans` change is needed, so no Japanese
      translation is touched
    - Commit as `docs(agents): describe the app log and its greppable lines`

## Trade-offs and risks

- Three PRs rather than one: the three concerns touch different review
  surfaces (error plumbing in Rust + TS, judgment calls about which invariants
  are safe, and documentation that depends on the final line shapes), so
  three PRs keep each review small and let Step 2's checks be reverted alone
  if one turns out to be noisy. (Chosen by the user.)
- Backtrace in the panic line: `Backtrace::capture()` is free unless
  `RUST_BACKTRACE` is set and gives the user a switch; `force_capture()` would
  give mostly addresses in a symbol-less release build.
- One command (`log_frontend(kind, line)`) rather than one per kind keeps the
  `generate_handler!` list short; unknown kinds are logged, not dropped.
- Flood guard threshold (20 distinct lines per session) is a guess; adjust if
  the manual check shows otherwise.
- Sidecar failure logging is a small addition beyond "replace eprintln",
  included because it is the one remaining error path that reaches only the
  UI.
- The frontend `faces-done after scan-done` invariant is the only check with
  any false-positive risk; Step 2 says to verify and drop it rather than keep
  a noisy check.
- `panic:` lines are multi-line when a backtrace is present; `grep panic:`
  still finds the head line, and the guide says so.

## Progress

- (none yet)
