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

# Clear Cache: unstick the scan guard and show the button's state

## Purpose

The settings window's `Clear Cache` button (added in
`docs/plans/_archived/20260920-clear-index-cache/`) never reaches its
confirmation dialog: after the first folder open, `clear_index` refuses every
press with `a scan is running; wait for it to finish`, even though no scan is
running.

Root cause (verified by reading `crates/app/src/commands.rs`):
`ScansState.running` is set by `start_scan` and is only ever taken by the
*next* `scan_folder`. Nothing clears it when the scan's `spawn_blocking` task
finishes, so `running.is_some()` stays true from the first folder open until
the app quits. `clear_index` treats `running.is_some()` as "a scan is in
progress", which it never was; the original consumers (`scan_folder`, which
joins the handle, and `spawn_eviction`, which runs once before any scan) never
cared. `preparing` (RAII `Preparing`, dropped on every return of
`scan_folder`) and `pending` (removed on every branch of `start_scan`, cleared
by the next `scan_folder`) are drained correctly and are not the bug.

This shipped green because no test covers the guard's state after a scan
ends; the archived learnings record that the manual GUI check was never run.

Once this is done: with no scan running the button shows the dialog and the
clear works; while a scan genuinely runs the button is disabled up front with
the reason shown, and re-enables by itself when the scan ends; a clear that
takes seconds shows that it is happening; a refusal or error stays visible.

## Decisions (made by the user; not open)

- A running scan is still refused — never canceled, never queued.
- The button is disabled up front while a scan runs, with the reason shown,
  rather than looking pressable and then rejecting the press.
- **In-flight feedback replaces the size line**: `Index cache: 6.3 MB` becomes
  `Clearing the index cache…` and returns as the new figure when the clear
  finishes. Rejected: changing the button's label to `Clearing…`, because the
  label states what the control does and overloading it with state muddles
  that.

## Steps

- [x] Step 1: Clear `running` when the scan task ends, and pin the guard's state with tests
  - Done when:
    - `ScansState` gains one method, e.g. `fn scanning(&self) -> bool`, that is
      the single definition of "a scan is genuinely in progress", and both
      checks in `clear_index` use it instead of the inline expression.
    - The scan task spawned in `start_scan` clears `running` when it finishes,
      *only if* the entry still belongs to it (a later `scan_folder` may have
      already taken the entry and stored a newer one), and `scanning()` is
      false afterwards.
    - Unit tests in `commands.rs`'s `tests` module cover: (a) after a scan
      task completes, `scanning()` is false; (b) after a scan is canceled the
      superseding way (`scan_folder` takes the entry, sets the cancel flag,
      joins), the old task's own clean-up does not clear the newer scan's
      entry, and `scanning()` reflects the newer scan; (c) `preparing > 0`
      alone makes `scanning()` true. These tests must fail on `main` today.
    - `cargo test -p riffle-app` and `mise run ci` pass.
  - Implementation approach:
    - Store the `scan_id` alongside the handle in `running` (e.g.
      `Option<(u64, Arc<AtomicBool>, JoinHandle<()>)>`) so the task's
      clean-up can compare ids. Add a small method such as
      `ScansState::finish(&mut self, scan_id: u64)` that does
      `if running's id == scan_id { running = None }`; the task calls it
      under the `Scans` lock after emitting `scan-done`. Locking `Scans` from
      the task is safe: `scan_folder` never holds the lock across
      `previous_handle.await`, and `start_scan` holds it through the store
      into `running`, so a task that finishes instantly (empty `todo`)
      blocks until the entry exists and then clears it. Do not take the
      writer lock inside `finish`; lock order `Scans` then writer stays as
      documented in `docs/agents/tauri-app.md`.
    - `scanning()` = `preparing > 0 || running.is_some() || !pending.is_empty()`
      (see Trade-offs for the `pending` term; keep it).
    - Tests can build a `ScansState` by hand and get a real `JoinHandle`
      from `tauri::async_runtime::spawn(async {})` (works without an app;
      block on it with `tauri::async_runtime::block_on`). `Preparing` needs
      an `AppHandle`, so test the `preparing` term by setting the field
      directly. `riffle-app` is a bin crate: run tests with
      `cargo test -p riffle-app <name>`, not `--lib` (guide note).
    - Alternative considered and not taken: leave `running` as is and make
      `scanning()` check `handle.inner().is_finished()` (tokio's
      `JoinHandle::is_finished` via tauri's `inner()`, no new dependency).
      It fixes the guard with fewer lines but leaves no hook for Step 2's
      "scan ended" event, which needs a transition point anyway.
    - Leave `spawn_eviction`'s `running.is_some()` check alone; it runs once
      at setup before any scan can have been started, so it is not affected,
      and touching it is outside this fix.

- [x] Step 2: Backend signals for the settings window: `scan_running` command, `scan-state` event, `index-clearing` event
  - Done when:
    - A new async command `scan_running(app) -> bool` returns
      `ScansState::scanning()`, registered in `main.rs`'s handler list next
      to `index_size`/`clear_index`.
    - The backend emits `scan-state` (payload: `bool`) whenever the value of
      `scanning()` may have changed: when `scan_folder` increments
      `preparing`, when `Preparing` drops, when `start_scan` moves an entry
      from `pending` to `running`, and when the task's `finish` clears
      `running` (Step 1). Emitting a redundant `true` is fine; the frontend
      just assigns it.
    - `clear_index` emits `index-clearing` (no payload) right after the user
      confirms the dialog and before the `spawn_blocking` clear starts, so
      the frontend can tell "dialog up" from "clearing".
    - `cargo test` and `mise run ci` pass.
  - Implementation approach:
    - Follows the existing settings pattern of a query command plus a
      broadcast event: `sidecar_format` + `sidecar-format`, `auto_advance` +
      `auto-advance`, `timing_logs` + `debug` (see `settings.ts` lines
      116-135 and `main.rs` `set_sidecar_format`). `app.emit` broadcasts to
      every window, so the main window simply ignores `scan-state` and
      `index-clearing`.
    - Why not reuse `scan-progress`/`scan-done`: they only cover the extract
      phase. The prepare phase (listing, `reconcile`,
      `reconcile_sidecars_of`, seconds on a large folder) has no event, and
      a scan with an empty `todo` emits only `scan-done`. A new event keyed
      to `scanning()` is exact by construction, and the initial query covers
      a settings window opened mid-scan.
    - Compute the bool under the `Scans` lock and emit while still holding
      it, at every site (`scan_folder`, `Preparing::drop`, `start_scan`, and
      the scan task's `finish`): the mutex then serializes every `scan-state`
      emit in the order the state actually changed. This is required because
      `start_scan` and the scan task it spawns race for the same lock: on a
      fast or empty scan they can both reach an emit, and if either one were
      issued after dropping the lock, nothing would order the two `app.emit`
      calls, so a stale `true` could land after the task's correct `false`
      and leave the button stuck disabled. Emitting under the lock at every
      site prevents that race consistently rather than only at the two sites
      where it is easiest to observe.
    - Assumes Step 1 is merged (needs `scanning()` and `finish`).

- [x] Step 3: Settings window: disable the button while a scan runs with the reason shown, show in-flight feedback, keep errors visible
  - Done when:
    - `crates/app/ui/settings.html`'s Cache panel gains a note element next
      to the button (e.g. `<p id="clear-index-note" hidden>`) whose text says
      the cache cannot be cleared while a scan is running and that the button
      becomes available when it finishes. Word it in plain English; do not
      reuse the red `#status` styling for it, since it is not an error.
    - `settings.ts` calls `scan_running` on load and listens to `scan-state`;
      `clearIndex.disabled` is true while a scan runs or a clear is in
      flight, and the note is visible exactly while a scan runs. When the
      scan ends the button re-enables without reopening the window.
    - While a clear is in flight (after the dialog is confirmed, signaled by
      `index-clearing`), the `#index-size` line reads
      `Clearing the index cache…`; when the invoke resolves the normal
      `Index cache: <size>` line returns with the new figure. During the
      dialog itself nothing changes except the button being disabled (as
      today).
    - A refusal or error from `clear_index` is written to `#status` and stays
      there until the next user action; nothing in the new code clears it on
      an event.
    - `pnpm exec vp check`, `vp fmt`, `vp test` and `mise run ci` pass.
  - Implementation approach:
    - In-flight feedback: `showIndexSize` stays as is and a sibling helper
      writes `Clearing the index cache…` into `#index-size` on
      `index-clearing`; the existing `.then` path then refetches `index_size`
      and overwrites it. This needs no new element and no new styling, and
      matches the only progress display the app has (the main window's
      `scanning N / M` note is likewise a text line in the meta pane). Do not
      also change the button label — the user chose one, not both.
    - Ordering race between the initial `scan_running` invoke and an early
      `scan-state` event: apply the invoke result only if no event has been
      received yet (one boolean), so a state change that arrives first is
      not overwritten by a stale answer.
    - The `finally` that re-enables the button must respect the scan state:
      `clearIndex.disabled = scanRunning || inFlight`, computed by one small
      `updateClearButton()` used by the click handler, the event listener and
      the initial query.
    - No new frontend module unless a pure helper emerges that is worth a
      vitest (e.g. the disabled/note decision from `(scanRunning, inFlight)`);
      the existing `tabs.ts`/`tabs.test.ts` pair is the precedent for
      extracting a testable pure function from `settings.ts`.
    - `#status` investigation result: no listener or re-render in
      `settings.ts` clears it other than user-initiated handlers, so no
      wiper fix is needed; the change is that the disabled note makes the
      state visible independently of `#status`. Do not add event-driven
      `status.textContent = ""`.
    - Assumes Step 2 is merged.

- [x] Step 4: Documentation and the human verification list
  - Done when:
    - `docs/agents/tauri-app.md` gets a `(Hit)` entry recording that
      `Scans.running` was never cleared at scan end, that `is_some()` is not
      "a scan is running", that `ScansState::scanning()` is now the one
      definition of "in progress" and any new guard must use it, and that
      the guard's state after a scan ends is covered by tests. Update the
      existing "Folder-index eviction" paragraph where it says
      `Scans.running` is checked, so it no longer implies `is_some()` means
      running.
    - `README.md`'s Clear Cache bullet mentions that the button is
      unavailable while a scan is running, if it does not already.
    - `learnings.md` in this plan folder records what the human verification
      found, including whether the "first press shows nothing" symptom
      reproduces after the fix.
    - `mise run ci` passes (lychee checks the docs links).
  - Implementation approach:
    - Human verification (GUI automation does not work on this Mac; the
      agent cannot drive the native dialog). Run
      `mise run tauri:release:devtools` and check, in order:
      1. Open a folder, wait until the meta pane no longer shows
         `scanning N / M`. Open Settings > Cache: the button is enabled and
         the note is hidden.
      2. Press `Clear Cache` once: the confirmation dialog appears
         immediately. Press `Cancel`: the size figure is unchanged, `#status`
         stays empty, the button is enabled again.
      3. Press `Clear Cache`, press `Clear`: the size line reads
         `Clearing the index cache…` until it drops to a few tens of KB; the
         main window reopens the folder and rescans.
      4. While that rescan is running (or open a large folder with the
         settings window already open on the Cache tab): the button is
         disabled and the note is visible, without any press. When the scan
         ends, the button re-enables and the note hides with the window left
         open.
      5. Open a large folder, then open Settings while the meta pane still
         shows nothing (prepare phase, before the first `scanning` line): the
         button is already disabled.
      6. Force an error path if reachable (e.g. make the dialog appear and
         confirm while a scan starts from a drag-drop in the main window):
         the red `#status` line shows the refusal and stays until the next
         press.
      7. Note whether the very first press after opening the settings window
         ever does nothing (the original symptom 1). If it does, record the
         window focus state at that moment in `learnings.md`.

## Trade-offs and risks

- **Does `pending` belong in the guard?** `pending` holds a scan that
  `scan_folder` has fully prepared and that the frontend's `start_scan` call
  is about to start; the window is one IPC round trip. It is drained
  reliably (every branch of `start_scan` removes it; the next `scan_folder`
  clears it), so it is not the cause of the bug. Keeping it is conservative:
  a clear that slipped in between would be followed by `start_scan` writing
  the prepared `todo` rows into the freshly emptied index, and then by the
  `index-cleared` reopen superseding that scan; consistent, but wasted work
  and a confusing log. Dropping it makes the guard `preparing > 0 ||
  running.is_some()` and matches `spawn_eviction`. Chosen: keep it,
  because the documented intent ("refuse while a folder is mid-open") reads
  naturally as covering the prepared-not-yet-started moment, and the cost is
  nil. The correct condition is therefore: **a `scan_folder` past minting
  its id and not yet returned (`preparing > 0`), or a prepared scan waiting
  for `start_scan` (`pending` non-empty), or a spawned scan task that has
  not yet run to completion (`running`, now cleared by the task itself).**
- **Clearing `running` in the task vs `is_finished()` in the guard.** The
  `is_finished()` route (`handle.inner().is_finished()`, tokio via tauri's
  `inner()`, no new dependency) is the smaller diff and needs no id in
  `running`, but gives Step 2 no transition point to emit the "scan ended"
  event; polling would be the only alternative. The plan takes the task-side
  clear.
- **A new `scan-state` event vs reusing `scan-progress`/`scan-done`.**
  Reuse means no backend change for the signal, but the settings window
  would be blind during the prepare phase and after a cancel with no
  successor (which cannot happen today, but the coupling is fragile). The
  new event mirrors the existing query-plus-event settings pattern; cost is
  four one-line emits.
- **`index-clearing` event vs splitting `clear_index` into a dialog command
  and a clear command.** The split would let the frontend own the whole
  sequence and keep the under-lock re-check in the second command, but it
  is two invokes and a redesign of the command surface for a bug fix. One
  emit is the surgical choice.
- **The "first press showed nothing" symptom is not explained by the code.**
  No wiper of `#status` exists in `settings.ts`. Candidate causes are
  outside the repo's control (macOS delivering the first click on a non-key
  window as activation only; a press before the module attached its
  listener). The plan makes the disabled state visible without a press,
  which removes the case where a silent refusal is the only feedback, and
  asks the human run to record whether it recurs. If it does recur after
  the fix, that is a separate issue (window activation), not this guard.
- **Migration risk.** Changing the shape of `running` touches `scan_folder`'s
  destructuring and `start_scan`'s store; both are under the same lock, so
  the invariants in the `Scans` doc comment are unchanged. Locking `Scans`
  from the scan task is new; it must never be done while holding the writer
  lock (the task holds the writer lock only inside `run_scan`'s `flush`,
  which has returned by then).

## Progress

- (2026-09-20) Step 1 complete
- (2026-09-20) Step 2 complete
- (2026-09-20) Step 3 complete
- (2026-09-20) Step 4 complete
