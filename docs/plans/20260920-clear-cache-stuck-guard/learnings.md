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

- `publish_scan_state(&AppHandle, bool)` emits `scan-state`; the four call
  sites each compute `scanning()` under the `Scans` lock, drop it, then emit.
  `scan_folder` and `start_scan` emit a constant `true` (both have just made
  `scanning()` true), `Preparing::drop` and the scan task's `finish` emit the
  recomputed value.
- `start_scan` is a synchronous command holding the guard until it returns, so
  the store into `running` is now followed by an explicit `drop(state)` before
  the emit.
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
