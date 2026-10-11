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

## Step 2: Latency points and the rolling-window model

- The "recording runs only while the overlay is on" gate lives in the model
  itself: `PerfModel` starts off, `start()` turns recording on, `stop()` turns
  it off and drops what was recorded, and `addCpu` / `addLatency` are no-ops
  while off. Step 3's toggle only calls `perf.start()` / `perf.stop()` (and
  the sampler command); `main.ts` checks `perf.on` before computing a point.
  A separate `let perfShown = false` in `main.ts` that nothing reassigns yet
  would have tripped the linter's prefer-const, and the model needs the
  "clear on off" behavior anyway.
- The selection-change time is a single `let switchAt` set in `show()`, not a
  `Map<seq, number>`: every `requestPreview()` for a `seq` comes from the
  `show()` that bumped it (directly or via the `.then` follow-up while that
  `seq` is still current), and the other `seq += 1` sites (an empty filter,
  a folder open) never request a preview without a following `show()`. A
  point is recorded only when `responseSeq === seq`, so `switchAt` is always
  that `seq`'s.
- Pruning drops points with `at < now - WINDOW_MS` from the front, relying on
  `performance.now()` being monotonic so the arrays stay sorted; `view(now)`
  prunes too, so Step 3's scrolling timer clears old points without an
  insertion. The latency axis maximum is `max(LATENCY_FLOOR_MS = 500, …)`,
  with no rounding to a "nice" value; Step 3 can round when it draws labels.
