<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Performance graph overlay: preview latency against Riffle's and the system's CPU

## Purpose

The user saw slow preview switching while an external `riffle-cli.exe`
benchmark with 24 threads saturated the CPU. `Riffle.log` had nothing to say:
the only preview timing (`page … keypressToPixels=…` from `debugLog` in
`crates/app/ui/src/main.ts`, written through `log_timing`) appears only when
the dev-only `Timing logs` item is on, and even then it cannot tell whether
Riffle itself was busy or something else took the CPU.

This plan adds an overlay the user can toggle on the viewer that plots three
series on one rolling time axis (the last 60 s):

1. per-switch preview latency, one bar per switch, in ms from the keypress /
   selection change to the preview being drawn;
2. Riffle's own process CPU usage, normalized by the logical CPU count to
   0–100 %;
3. system-wide CPU usage, 0–100 %.

Reading rule: a latency spike with (2) high means Riffle is busy; a latency
spike with (2) normal and (3) pegged means an external process. (2) and (3)
are sampled on the Rust side with the `sysinfo` crate every 500 ms, only while
the overlay is visible, and streamed as a `perf-sample` event.

Out of scope: an FPS / frame-time graph, disk I/O, persisting the overlay
state across launches, a charting library.

## Decisions (user, 2026-10-11)

- The overlay is toggled by a keymap action, `performance`, available in every
  build (release included) and listed in the settings modal's Shortcuts tab.
  No `Debug` tab checkbox, no `View` menu item.
- Its default key is `d`.

## Steps

- [x] Step 1: Rust CPU sampler (`crates/app/src/perf.rs`) behind `set_perf_sampling`, emitting `perf-sample`
  - Done when: `set_perf_sampling(true)` starts one sampler thread that emits
    `perf-sample { process: f32, system: f32 }` (both 0–100) every ~500 ms,
    skipping the first (diff-less) reading; `set_perf_sampling(false)` stops
    it and no thread or `sysinfo` refresh runs while off; calling
    `set_perf_sampling(true)` twice does not start a second thread; the
    normalization is a pure function with Rust tests (raw 0 → 0, raw equal to
    `cpus × 100` → 100, clamped at 100, `cpus == 0` does not divide by zero);
    the frontend (Step 3) calls `set_perf_sampling(false)` at startup so a
    reloaded window never leaves a sampler running; `CLAUDE.md`'s Layout
    describes `src/perf.rs`; `mise run ci` passes.
  - Implementation approach:
    - Add `sysinfo` (latest 0.39.x; it is not in `Cargo.lock` yet) to
      `crates/app/Cargo.toml` only. Check the build time / binary size does
      not jump; if it does, pass `default-features = false, features = ["system"]`.
    - sysinfo semantics to respect (docs.rs, fetched 2026-10-10):
      `System::refresh_cpu_usage()` and `Process::cpu_usage()` are diff-based,
      so the first reading is inaccurate and must be dropped; calls must be at
      least `sysinfo::MINIMUM_CPU_UPDATE_INTERVAL` apart (take
      `max(MINIMUM_CPU_UPDATE_INTERVAL, 500 ms)` so a future sysinfo cannot
      break it); `Process::cpu_usage()` "might be bigger than 100 if run on a
      multi-core machine", divide by `System::cpus().len()`;
      `System::global_cpu_usage()` is already 0–100. Refresh only what is
      needed: `refresh_cpu_usage()` and
      `refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), false, ProcessRefreshKind::nothing().with_cpu())`
      with `pid = sysinfo::get_current_pid()`.
    - Lifecycle, modeled on `sequence.rs`'s `(Arc<AtomicBool>, JoinHandle)`
      pattern: a managed `AppPerf(Mutex<Option<(Arc<AtomicBool>, JoinHandle<()>)>>)`
      registered with `app.manage` in `main.rs`'s setup next to
      `TimingLogs`. `set_perf_sampling(enabled)` is a plain sync command (it
      only flips a flag and spawns / signals a `std::thread`; it must not join
      the thread on the main thread, see `docs/agents/tauri-app.md`). The
      thread checks the stop flag every loop and exits within one period.
    - The thread is a plain `std::thread::spawn` (no priority change; the
      work is two refreshes every 500 ms). Emit with `app.emit("perf-sample", …)`
      as `sequence.rs` does; a failed emit is logged at `warn` once, not per
      tick.
    - Keep the pure logic testable: the normalization
      (`fn normalized(raw: f32, cpus: usize) -> f32`) and the "drop the first
      reading" rule as functions the thread calls, with `#[cfg(test)]` tests
      in `perf.rs`.
    - Register the command in `generate_handler!` in `main.rs`; follow the
      doc-comment style of the neighboring `timing_logs` / `set_timing_logs`.
- [ ] Step 2: Latency points and the rolling-window model (`crates/app/ui/src/perf.ts`), with tests
  - Done when: `perf.ts` exports a pure model (`PerfModel` or similar) that
    holds the CPU samples and latency points of the last `WINDOW_MS = 60_000`,
    prunes older ones on each insertion / tick, and exposes what the drawing
    needs (the series clipped to the window, the latency axis maximum with a
    sensible floor, e.g. 500 ms, so a quiet graph does not stretch 20 ms bars
    to full height); `main.ts` records one latency point per *shown* switch
    (a superseded request, i.e. `responseSeq !== seq` in the worker's
    `message` handler, records nothing); the point's start is the keypress
    when the page turn came from a key, else the selection change; the
    recording runs only while the overlay is on; `pnpm exec vp test` covers
    the model (window pruning, floor of the axis maximum, insertion order)
    and the latency start rule; `CLAUDE.md`'s Layout describes `src/perf.ts`.
  - Implementation approach:
    - `vite.config.ts` runs vitest with `environment: "node"`, so `perf.ts`
      must hold no DOM / canvas access; the drawing (Step 3) takes a
      `CanvasRenderingContext2D` and the model's output as arguments, or
      lives in `main.ts`.
    - The existing timing in `main.ts`: `pageKeypressAt` is set at the four
      key-driven moves (`move`, `moveBurst`, `moveBurstFrame`,
      `extendSelection`) and consumed by `requestPreview()` into
      `pageTimings` (`startedAt`, `invokeMs`, `postedAt`, `keypressAt`); the
      worker's `message` handler computes `drawnAt` and logs the `page …`
      line only when `responseSeq === seq`. Reuse this: the latency point is
      taken right where that `debugLog` is, from the same `timing`.
    - The gap to close: a strip click or a folder open has
      `keypressAt === null`, and a key pressed while a request is in flight
      has its `pageKeypressAt` cleared by the early return in
      `requestPreview()`, so the follow-up request started from the `.then`
      also sees `null`. Measuring those from `startedAt` would hide exactly
      the queueing the overlay is for. Record the time of the last selection
      change per `seq` in `show()` (a `switchAt` next to the `seq` bump, or a
      `Map<seq, number>` cleared like `orientations`) and use
      `timing.keypressAt ?? switchAt` as the point's start. Decide in the
      step which representation is smaller; do not change the `page …` log
      line's content.
    - Keep `pageTimings` bookkeeping as is (it is cleared for every response,
      shown or not); only add the one point.
- [ ] Step 3: The overlay canvas, its toggle (`performance` keymap action, default `d`), the `perf-sample` listener and the sampler wiring
  - Done when: a `performance` action with the default key `d` toggles a
    `<canvas id="perf">` overlaid on `#viewer`; while on, the canvas redraws
    at least every 500 ms (each `perf-sample`, each latency point, and a timer
    for the scrolling axis) showing the three series on one 60 s axis with a
    legend / scale labels readable on the dark theme; toggling on invokes
    `set_perf_sampling(true)` and registers the `perf-sample` listener,
    toggling off invokes `set_perf_sampling(false)`, stops the timer and
    clears the model; the overlay does not take pointer events
    (`pointer-events: none`) and does not shift the preview; the action
    appears in the settings modal's Shortcuts tab with a label; the
    `shortcuts.rs` default-keys test and the `DEFAULTS` table include it; the
    overlay is off at launch; `mise run ci` passes.
  - Implementation approach:
    - Toggle path, modeled on `focus` / `zoom` / `grayscale` (actions with a
      key and no menu item, as the `shortcuts.rs` test comment documents):
      add `("performance", &["d"])` to `DEFAULTS` in
      `crates/app/src/shortcuts.rs` (and the expected list in its test), a
      label in `settings.ts`'s action label table (e.g.
      `performance: "Performance graph"`), a `case "performance":` in
      `main.ts`'s `runAction`.
    - Markup / style per `docs/agents/ui-styling.md`: `<canvas id="perf" hidden>`
      inside `#viewer` after `#empty`; an id rule in `style.css` for
      placement only (absolute, bottom of `#viewer`, fixed height ~ 120 px,
      full width, `pointer-events: none`), scoped `:not([hidden])` if it sets
      `display`. Colors come from tokens read once at startup through
      `token()` into `main.ts`'s `THEME` (never per frame, with keyword
      fallbacks): the panel background from `--card` with alpha, the axis /
      grid from `--border`, labels from `--muted-foreground`. The three
      series: system CPU as `--muted-foreground`, Riffle CPU as
      `--foreground`, latency bars in the focus mark's green already used on
      the canvas (`#6bdc8a` in `main.ts`), or `--destructive` for bars above a
      threshold; decide in the step and keep it to existing canvas colors.
    - DPR handling like the main canvas: size the backing store by
      `devicePixelRatio` on show and on `#viewer` resize (reuse the existing
      resize observer / `draw()` hook if one covers `#viewer`).
    - Sampler wiring: `window.__TAURI__.event.listen<{process:number; system:number}>("perf-sample", …)`
      pushes into the model with `performance.now()` as the sample's time
      so the CPU series and the latency points share one clock (do not use
      a backend timestamp). At startup invoke `set_perf_sampling(false)`
      (Step 1's reload rule).
    - Draw with plain 2D canvas calls: CPU series as two polylines on a
      0–100 % left scale, latency as vertical bars on a right scale (0 to
      the model's maximum), time running right-to-left over 60 s, a few
      labels (`100%`, the latency maximum in ms, `60 s`). No library.
    - Gate the action like `focus` / `zoom` in `runAction` (not in a text
      input, not while a modal is open).
- [ ] Step 4: Verify the acceptance scenario by hand and document the overlay
  - Done when: `results.md` in this plan folder records a hand check on the
    user's machine: (a) overlay on, paging through a folder adds latency bars
    aligned in time with the CPU series; (b) with `riffle-cli` (or another
    hog) running on all cores, the system series pegs while Riffle's stays
    low and the latency bars grow; (c) with the overlay off, Task Manager /
    the log show no sampler activity (no `perf-sample`, CPU of the idle app
    unchanged); `docs/humans/usage.md` and `docs/humans/usage.ja.md` gain the
    action in the shortcut table and a short paragraph with the reading rule
    (Riffle CPU high → Riffle busy; system pegged, Riffle low → external;
    a running scan counts as Riffle CPU; latency is measured from the key or
    click), in the same PR; `README.md` / `README.ja.md` only if they list
    shortcuts; `mise run ci` passes.
  - Implementation approach:
    - Put the hand-check folder under `D:\Photos\tests\<date>-<topic>\`
      per the user's auto memory, not a scratchpad.
    - Record measured latencies with and without the hog so the thresholds
      chosen in Step 2 / 3 (axis floor, bar color threshold) can be revisited
      with numbers.

## Trade-offs and risks

- **Latency start for non-key switches.** The existing `keypressToPixels`
  only exists for key-driven turns. Step 2 measures a click / in-flight
  follow-up from the selection change in `show()`, so the overlay's number
  can be a little larger than the `page …` log's `total=` and is not the
  same quantity as `keypressToPixels`.
- **Overlay-only sampling vs always-on with a ring buffer.** Always-on would
  show the seconds *before* the user toggled the overlay, but sampling must
  cost nothing while off; the plan follows that, so the graph starts empty.
- **`sysinfo` on the main thread.** The sampler lives on its own thread; the
  command only spawns / signals. Stopping does not join, so a stop followed by
  an immediate start may briefly have two threads; the old one exits within
  one period and the stop flag is per thread, so no sample from it reaches the
  new listener if the frontend ignores samples while off.
- **Process CPU on Windows.** sysinfo's per-process CPU counts the whole
  process (all threads, including the scan workers), so a running scan shows
  as Riffle CPU; the docs paragraph mentions it.
- **Series colors.** `ui-styling.md` forbids new hex colors on the canvas;
  the three series are told apart by the two grays plus the one green already
  on the canvas. If that is not readable enough in Step 4, the fallback is
  line weight / dashes, not a new color.

## Progress

- (2026-10-11) Step 1 complete
