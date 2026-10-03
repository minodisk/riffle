# Learnings

## Step 1: Drop the `running` entry before emitting `faces-done`

- The emit itself cannot be exercised in the `commands.rs` tests: the scan
  task needs an `AppHandle`, which the module's tests do not build. The new
  test `a_scan_started_after_the_done_signal_finds_nothing_to_join` mirrors
  the task's tail (`finish`, then a channel send standing in for
  `faces-done`) and asserts that after the signal `scanning()` is false and
  `running.take()` is `None`. The ordering of the real emit is covered by the
  comment in `start_scan`'s closure and the manual log check in Step 3, not
  by a test of the emit.
- The reorder wraps the lock in its own block so the guard is dropped before
  the emit; the closure's earlier `state` binding is the outer one and is
  unaffected.

## Step 2: Name the trigger of a deferred rescan in the timing log

- `resyncPending` became `RescanTrigger | null` and keeps the _first_
  deferred trigger (`??=`), so a burst of triggers during a long scan logs one
  `rescan deferred:` line each but the drained run names what first asked
  for it. `resync(trigger, deferred)` takes a second flag so the drained run
  logs `deferred=true`; no `"drained"` member was added to the trigger union,
  since the drained run always carries the original trigger.
- The line format lives in `rescanLine(trigger, phase)` in `refresh.ts`
  (`phase`: `defer` / `start` / `drained`), tested in `refresh.test.ts`.
- A drained rescan that is held off again (a listing still in flight) logs
  another `rescan deferred:` line with its original trigger; expected.

## Step 3: Update the docs and verify one `open entries` per cold scan

- `docs/humans/performance.md` named the item "App: `open entries` is called
  twice per folder open", not the plan's title for it; the paragraph was
  rewritten either way and no longer points at `todo.md` for it.
  `docs/humans/performance.ja.md` was updated in the same change.
- The `docs/agents/tauri-app.md` item is tagged **Inferred** (the forced read
  was found by reading the code, never seen in a log) and its Source points
  at the archived plan path the wrap-up will move this folder to.
- The step's checkbox was ticked on the doc criteria; the manual Windows log
  check is deferred below.

## Deferred issues (todo candidates)

- **Pending manual check (Windows): one `open entries` per cold scan.**
  Basis: Step 3's "Done when" in
  `docs/plans/20261003-follow-up-rescan-open-entries/plan.md` (the user's
  check after merge; Steps 1 and 2 changed `crates/app/src/commands.rs`,
  `crates/app/ui/src/main.ts` and `crates/app/ui/src/refresh.ts`). Step 3's
  checkbox was ticked on the automated (doc) criteria only.
  - Steps: on Windows, run `mise run tauri:release:devtools`, turn on
    `Timing logs` in the settings modal, clear the cache, open a folder large
    enough that the scan runs well past five seconds, switch to another
    window and back during the scan, and wait for the scan to end. Open
    `Riffle.log` (`Help > Open Log Folder`).
  - Expected: one `open entries` for the open, a `rescan deferred:
    trigger=focus` line, then the `scan list` / `scan prepare` of the next
    `scan_id` with `todo=0` and no further `open entries`.
  - Record the result in this plan's `learnings.md` (or the todo). If a
    second `open entries` still appears, note its `changed` / `total` from
    the surrounding lines and reopen the `todo.md` item "App: `open entries`
    is called again after a scan's follow-up rescan" instead of closing it in
    the wrap-up.
