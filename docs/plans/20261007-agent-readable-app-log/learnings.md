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

## Step 2

- The plan's check 2 (`start_scan has a pending scan`) would have fired in a
  normal session: `scan_folder` inserts no `pending` entry when neither pass
  has a file to do (`idle`, the usual focus rescan of an indexed folder) or
  when the index cache is unavailable, and `start_scan` then reaches the
  `let Some(pending) = ... else` branch with `scan_id == latest_id`. The
  check was narrowed to `start_scan once per scan`: in that branch it logs
  only when `running` already holds `scan_id` (a second `start_scan` while
  the first is running). An id equal to `latest_id` was always handed out by
  `scan_folder`, so the "id never handed out" case cannot reach the branch.
  A second call after the first scan finished is not caught (`finish` has
  cleared `running`), accepted as a gap.
- The frontend `faces-done after scan-done` check was kept: no false
  positive found. `scanId` is assigned in `startScan` before `start_scan` is
  invoked, and a scan's events are emitted only after `start_scan`, in order
  (`scan-done` then `faces-done`, from one thread, including
  `emit_empty_scan_events`), so a live `scanId` always sees its own
  `scan-done` first. `openDirectory`'s `scanId = null; scanDone = null`
  filters out the old scan's late events (ids only grow), a `resync` assigns
  a fresh id the same way, a superseded `startScan` (`seq !== currentScan`)
  never sets `scanId` nor calls `start_scan`, and `index-cleared` /
  `sidecar-format` go through `openDirectory`.
- `invariant!` is a `macro_rules!` re-exported with `pub(crate) use` from
  `diagnostics.rs`; the expansion calls `$crate::diagnostics::invariant_line`,
  so that function is `pub(crate)`.

## Step 3

- The plan asked to name the rotated file "as observed on disk", but
  tauri-plugin-log 2.9.2's `RotationStrategy::KeepOne` leaves none: its
  `rotate` calls `fs::remove_file` on `Riffle.log` and reopens it. Only
  `KeepAll` / `KeepSome` rename to `Riffle_<YYYY-MM-DD_HH-MM-SS>.log`. The
  Windows log folder holds only `Riffle.log`, which matches. The guide says
  the early lines of a long session are lost outright.
- The plugin's desktop default format is `[date][time][target][LEVEL]`
  (target before level), as the plan said; only `timezone_strategy()`
  swaps the order to level before target, and the app does not call it.
  The plugin also logs to stdout by default.
- The macOS / Linux paths come from tauri 2.11.6 `app_log_dir`; only the
  Windows path was checked on disk.

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
- Pending manual check (Step 2, Windows debug build): the step's checkbox was
  ticked on the automated criteria (unit test, `mise run ci`); the "no line in
  a normal session" criterion needs a hands-on run. Start the debug app,
  open a folder not yet indexed (cold), let it finish, switch the window
  away and back after 5 s or more (focus rescan), open another folder while
  a scan runs, then clear the cache in Settings. Afterwards
  `grep invariant: "%LOCALAPPDATA%\com.minodisk.riffle\logs\Riffle.log"`
  must print nothing. Basis: plan Step 2 Done when; files
  `crates/app/src/commands.rs`, `crates/app/src/diagnostics.rs`,
  `crates/app/ui/src/main.ts`.
