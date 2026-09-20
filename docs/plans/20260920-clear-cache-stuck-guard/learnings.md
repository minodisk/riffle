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
