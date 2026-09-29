# Learnings

## Step 1: Make the watch release synchronous on Windows

- The test retry the plan did not know about: #560 (merged after the plan
  was written) had wrapped the rename in
  `release_under_releases_the_folder_and_below_and_restore_puts_them_back`
  in a 50 x 20 ms retry loop. The step removes it again, back to a plain
  `std::fs::rename(...).unwrap()`, since the Done-when forbids retries around
  the rename and the barrier makes it unnecessary.
- Reproduction (Windows 11, 24 logical CPUs): the `riffle-app` test binary
  built with the retry removed but without the barrier, run as
  `<exe> treewatch:: -q --test-threads=1` 300 times while 24 bash busy loops
  (`while :; do :; done`) pinned every core: **5 of 300 runs failed**
  (iterations 94, 123, 253, 259, 287), every time in
  `release_under_releases_...` at the rename's `unwrap` (treewatch.rs
  line 355). The `apply` test's rename (line 322) never failed in these
  runs. 447 s in total.
- Fix: the same loop, same load, on the binary with `settle` in
  `State::remove` and `watch::release`: **0 of 300 failed** (485 s). The
  whole `riffle-app` suite with default parallelism, 5 runs: all passed
  (306 passed, 2 ignored each).
- Cost of the barrier: a throwaway test timed `apply(&[dir])` then
  `apply(&[])` (one `unwatch` + `settle`) 30 times, idle machine: median
  0.024 ms, max 0.099 ms. Note that notify 8.2's Windows `configure` does
  not call `wakeup_server` (unlike `watch` / `unwatch`), so if the server
  drained the `Unwatch` and went back to its 100 ms alertable wait before
  `Configure` was queued, the barrier can wait up to ~100 ms; it was never
  observed, since `unwatch`'s own wakeup normally lands after both actions
  are queued.
- Deadlock checks: `configure` on a watcher whose server thread has exited
  returns `Err` on the closed channel (ignored). The barrier runs under the
  `TreeWatch` / `Watch` lock; the only app code notify's server can run
  while closing a handle is the event handler (from a completion routine),
  which takes just the `keys` / `owner` locks, and neither is held across
  the barrier (`State::remove` drops `keys` before `unwatch`, `watch::release`
  clones `owner` and drops the guard first), so it cannot wait on the caller.
- `configure` with `Config::default()` is a no-op on every backend in 8.2
  (`configure_raw_mode` replies `Ok(false)`); on inotify it round-trips
  through the event loop, on fsevents it replies inline.
