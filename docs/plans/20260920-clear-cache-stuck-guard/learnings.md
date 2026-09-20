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
