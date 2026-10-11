# Learnings

## Step 1: Rust CPU sampler

- `sysinfo` 0.39.6 (rust-version 1.95, the toolchain is 1.97) is pulled in
  with `default-features = false, features = ["system"]` from the start
  rather than measuring the default build first: the CPU and process APIs
  only need `system`, and the defaults add `component` (WMI on Windows),
  `disk`, `network` and `user`, none of which the sampler uses.
- The "drop the first reading" rule is a tiny `Readings` struct whose
  `sample()` returns `None` the first time; with `normalized()` it is the
  only pure logic, and both are tested in `perf.rs`.
- `set_perf_sampling(true)` checks `JoinHandle::is_finished()` on the stored
  thread, so a thread that died (a panic) does not block a restart, while a
  live one is never duplicated. `set_perf_sampling(false)` with nothing
  running is a no-op, so Step 3's startup call is safe.
- The thread sleeps a full period after each refresh and checks the stop
  flag at the top of the loop, so it exits within one period of a stop. The
  stop flag is per thread, so an immediate stop + start can briefly have the
  old thread finishing its sleep (as the plan's trade-offs note); it emits
  nothing after waking because it checks the flag before the next refresh.
