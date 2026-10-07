# Learnings

## Step 1

- The plan listed seven `eprintln!` lines in `commands.rs`; all were replaced
  with `log::warn!` keeping the text. The two in `sidecar.rs` sit inside its
  `#[cfg(test)]` module and stay.
- `uncaughtLine` omits the fields it was not given (an unhandled rejection has
  no source / line / col) rather than writing empty `source=` fields. For a
  rejection whose reason is an `Error`, `message` is `reason.message` and the
  stack goes in `stack=`; any other reason is `String(reason)`.
- The `error` / `unhandledrejection` listeners sit right after `main.ts`'s
  imports: ES module imports are evaluated first anyway, so this is the
  earliest point in the module body.

- A `panic:` line also appears for panics the core catches on purpose and
  turns into an `Err` (the mozjpeg decode / thumbnail / sharpness paths in
  `scan.rs`, `decode.rs`, `sharpness.rs`), because the hook runs before
  `catch_unwind`. Such a line is a crash only if no recovery follows, for
  example a scan failure entry for that file. The `docs/agents/` guide added
  in a later step must say this. Basis: review round 1, item 1.

## Deferred issues (todo candidates)

- Pending manual check (Step 1, Windows debug build): the step's checkbox was
  ticked on the automated criteria (unit tests, `mise run ci`). Run
  `mise run dev`-style debug app, open DevTools in the main window and run
  `setTimeout(() => { throw new Error("x") })` and
  `Promise.reject(new Error("y"))`; expect two lines in `Riffle.log`
  (`%LOCALAPPDATA%\com.minodisk.riffle\logs\Riffle.log`) starting with
  `uncaught-js: kind=error message=Uncaught Error: x` and
  `uncaught-js: kind=unhandledrejection message=y`. Then force a panic on a
  scan worker in a scratch change (e.g. `panic!("test")` in `run_scan`), open
  a folder, and expect one `panic: thread=... location=... payload=test`
  line plus the default stderr message; revert the scratch change. Basis:
  plan Step 1 "Manual check before the PR"; files
  `crates/app/src/diagnostics.rs`, `crates/app/ui/src/main.ts`.
