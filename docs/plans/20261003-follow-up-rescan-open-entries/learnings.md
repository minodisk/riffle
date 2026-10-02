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
