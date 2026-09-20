# Learnings

## Step 1: Clear `running` when the scan task ends

- `ScansState::scanning()` is now the single definition of "a scan is in
  progress" (`preparing > 0 || running.is_some() || !pending.is_empty()`), used
  by both of `clear_index`'s checks. `spawn_eviction`'s `running.is_some()` was
  left alone on purpose (it runs once at setup).
- `running` became `Option<(u64, Arc<AtomicBool>, JoinHandle<()>)>`; the scan
  task calls `ScansState::finish(scan_id)` under the `Scans` lock after
  emitting `scan-done`, and `finish` only clears the entry when the stored id
  still matches. Step 2 can hang its `scan-state` emit off `finish`'s call site
  (after the lock is dropped).
- Confirmed the new tests fail on the pre-fix behaviour, by temporarily
  rewriting `finish`'s body twice and re-running the tests:
  - `finish` as a no-op (what `main` does today: nothing ever clears `running`)
    → `a_finished_scan_leaves_no_scan_in_progress` fails
    (`assertion failed: !index::lock(&scans.0).scanning()`).
  - `finish` clearing `running` unconditionally (the id check dropped)
    → `a_superseded_scan_does_not_clear_the_newer_scan` fails
    (`assertion failed: state.scanning()`).
- Test (a) waits for the entry to disappear on a spawned blocking task with a
  2 s deadline instead of joining the handle, because joining would need to
  take the entry out of `running` and would make the assertion vacuous.
- `riffle-app` is a bin crate: `cargo test -p riffle-app <name>` takes only one
  filter argument, so the two tests had to be run separately.

## Step 2: Backend signals for the settings window

- All four `scan-state` emit sites (`scan_folder`, `Preparing::drop`,
  `start_scan`, and the scan task's `finish`) compute `scanning()` under the
  `Scans` lock and emit while still holding it, so the mutex itself
  serialises every emit in the order the state actually changed. There is no
  `publish_scan_state` helper; it was dropped once all sites converged on
  emitting under the lock.
- `start_scan` and the scan task it spawns race for the same lock: on a fast
  or empty scan, the spawned task can reach `finish`/`scanning`/emit before
  `start_scan` reaches its own trailing emit, and since the two `app.emit`
  calls run on separate threads with the lock already dropped, nothing
  orders them — a stale `true` can land after the task's correct `false` and
  leave the settings window stuck showing "scanning" forever. `start_scan`
  and the task's closure now emit `scan-state` directly (not through
  `publish_scan_state`) while still holding the `Scans` lock, so the mutex
  itself serialises the two `app.emit` calls: the task cannot take the lock
  to emit until `start_scan` has stored `running` and emitted `true` under
  it, and vice versa if the task gets there first. `start_scan` never drops
  its guard mid-function, so `running` is always stored while the task is
  still blocked on the same lock, before either one can emit.
- `index::lock(&app.state::<Scans>().0)` inside the scan task no longer
  compiles once the guard is held across more than one statement (E0716: the
  `State` temporary is freed at the end of the statement), so the task binds
  `let scans = app.state::<Scans>();` first.
- Capability check (the constraint said to verify rather than assume):
  `crates/app/capabilities/default.json` grants `core:default`, which
  `crates/app/gen/schemas/desktop-schema.json` documents as including
  `core:event:default`, which includes `allow-listen`. No capability change is
  needed for the settings window to listen to `scan-state` / `index-clearing`.
- `scan_running` is `async` even though it only takes a mutex briefly, to match
  `index_size`/`clear_index` and keep it off the main thread.

## Step 3: Settings window state

- No new frontend module: the `(scanRunning, clearInFlight)` decision is two
  assignments (`clearIndex.disabled`, `clearIndexNote.hidden`) in
  `updateClearButton()`, which is not worth a vitest of its own, so
  `settings.ts` keeps it inline. `tabs.ts` stayed the only extraction.
- The initial `scan_running` invoke is applied only while `scanStateSeen` is
  false, so an early `scan-state` event is never overwritten by the stale
  answer (the frontend mirror of Step 2's ordering bug).
- `clear_index` emits `index-clearing` *before* the blocking task, which
  re-checks `scanning()` and can still fail. Without a fix the `#index-size`
  line would stay `Clearing the index cache…` forever on that path, so the
  `.catch` refetches `index_size` after writing the error to `#status`. The
  error itself stays in `#status`; nothing new clears it.
- `#clear-index-note` is styled grey (`#aaa`) in `settings.css`, deliberately
  not the red `#status` colour, because a running scan is a normal state.
- Verified: `pnpm exec vp check`, `vp test`, `mise run fmt`, `mise run ci`.
  Not verified: the GUI behaviour (disabled button, note, in-flight line, the
  native dialog), which needs the human run listed in Step 4.
- `mise run ci` failed once on
  `index::tests::the_reader_does_not_wait_on_an_open_write_transaction`
  (`crates/app/src/index.rs:1839`, `assert!(elapsed.0 < 100ms)`, measured
  161ms) and passed on the immediate re-run. The test is wall-clock
  sensitive and this step touches no Rust, so it is a flake, not a
  regression.

## Deferred issues (todo candidates)

- `index::tests::the_reader_does_not_wait_on_an_open_write_transaction` is
  flaky: it asserts a wall-clock budget of 100 ms for a read taken while a
  write transaction is open, and failed once at 161 ms on an otherwise idle
  machine during Step 3's local `mise run ci`. Basis: the Step 3 CI run
  above. File: `crates/app/src/index.rs:1839`.
