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

## Step 3: The overlay canvas, its toggle and the sampler wiring

- The plan named `#6bdc8a` as "the focus mark's green already used on the
  canvas", but no such hex exists in `main.ts`; the focus mark's green is
  `FOCUS_MARK_COLORS.good` (`#3f3`) in `focus.ts`, which the latency bars
  use. All bars are that one green; no `--destructive` threshold color, so
  Step 4 can pick a threshold from measured numbers if one is wanted.
- The legend is color-coded text in one header row (`CPU 100%  60 s`,
  `Riffle CPU`, `System CPU`, `Latency`) with the latency axis maximum in ms
  at the right; the plot has grid lines at 0 / 50 / 100 %. The panel is
  `--card` at 85 % alpha (`globalAlpha`, so no new color string).
- The `perf-sample` listener is registered on toggle-on and unlistened on
  toggle-off (the unlisten promise is awaited, so an off before `listen`
  resolves still unregisters it); `PerfModel.addCpu` ignores a sample that
  slips in after the stop anyway.
- `performance` had to be added to `MENU_LESS` in `main.rs`'s menu test, not
  only to `DEFAULTS` and the default-keys test: that test asserts every
  default action is either a menu item or listed as menu-less.
- `d` is swallowed while the folder tree has focus (`TREE_PASSTHROUGH` does
  not list it), like `focus` / `zoom`.
- CI attempt 1 failed on `a_stale_default_in_an_old_store_drops_only_that_key`:
  its stored override put `d` on `next` (a WASD-style store), and a default
  key now holds `d`, so `from_overrides` dropped it from `next`. The test's
  `d` was arbitrary data, so it now uses the free `e`. The same thing
  happens to a real user: an override that put `d` on another action loses
  `d` on upgrade (deferred below).

## Deferred issues (todo candidates)

- A user whose stored `shortcuts` override bound `d` to another action loses
  that key silently on upgrade, because `Keymap::from_overrides` gives a key
  already held by a default (`performance`'s new `d`) precedence over a
  partly applicable override. Found in Step 3 of
  performance-graph-overlay when `a_stale_default_in_an_old_store_drops_only_that_key`
  failed. Consider whether a newly added default should yield to an
  existing override (or log the drop at `warn`). Files:
  `crates/app/src/shortcuts.rs` (`DEFAULTS`, `from_overrides`).
