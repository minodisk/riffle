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

# Disk I/O series in the performance overlay

## Purpose

The performance overlay (`d`) shows preview latency bars next to Riffle's and
the system's CPU. Latency spikes of ~4.5 s have been seen while both CPU lines
sit near 0, which points at disk I/O (the preview read waiting behind the
background scan passes reading RAW files) rather than compute, but the overlay
cannot show that. Adding Riffle's own disk read throughput and the system's
disk busy time to the same graph lets a spike be read against disk activity
at a glance, on the same 60 s window and 500 ms cadence as the CPU lines.

## Steps

- [x] Step 1: Sample Riffle's disk read throughput and the system's disk busy %, send them in `perf-sample`, and draw them in the overlay
  - Done when:
    - `perf-sample`'s payload carries, next to `process` and `system`, Riffle's
      read throughput in bytes per second over the sampling interval and the
      system's disk busy time as 0–100, the latter `null` on a platform that
      cannot provide it.
    - With the overlay on (Windows), the graph shows a Riffle disk read line
      labeled in MB/s on its own axis and a system disk busy line on the
      0–100 % axis, each with a legend entry with a swatch next to
      `Riffle CPU` / `System CPU` / `Latency`, updating every 500 ms.
    - The overlay's colors are, as the user chose: the two CPU lines blue, the
      two disk lines green, the latency bars red.
    - A `null` disk busy value draws nothing for that series and shows no
      error; the other series are unaffected.
    - Pure Rust delta / ratio computations and the `PerfModel` changes have
      unit tests; `mise run ci` passes.
    - `CLAUDE.md`'s descriptions of `src/perf.rs` and `src/perf.ts`, and the
      performance-graph paragraph of `docs/humans/usage.md` and
      `docs/humans/usage.ja.md`, name the new series. (`README.md` does not
      describe the overlay; leave it.)
    - Hand check on Windows recorded in `learnings.md`: browsing during a scan
      shows disk activity aligned with latency spikes (or that it does not,
      which is also a result).
  - Implementation approach (as far as it is known):
    - **Riffle read throughput, all platforms, no new dependency.** In
      `crates/app/src/perf.rs` `run()`, add `.with_disk_usage()` to the
      existing `ProcessRefreshKind::nothing().with_cpu()` and read
      `p.disk_usage().read_bytes` (sysinfo 0.39.6's delta since the previous
      refresh: `GetProcessIoCounters.ReadTransferCount` on Windows,
      `/proc/self/io read_bytes` on Linux, `proc_pid_rusage` on macOS).
      Divide by the measured elapsed time between refreshes (an `Instant`
      kept in the loop, not the nominal 500 ms) to get bytes/s. Document in
      the module doc that on Windows this counts every read the process
      issues (file, pipe, network), which is acceptable for this overlay.
    - **System disk busy %, Windows: PDH.** sysinfo's `system` feature already
      enables `windows/Win32_System_Performance` on `windows` 0.62.2 in
      `Cargo.lock`, so add
      `windows = { version = "0.62", features = ["Win32_System_Performance"] }`
      under a new `[target.'cfg(windows)'.dependencies]` table in
      `crates/app/Cargo.toml` (same shape as `crates/core`'s) and confirm
      `cargo tree -d` shows no second `windows` version. Open one query with
      `PdhOpenQueryW`, add `\PhysicalDisk(*)\% Idle Time` with
      `PdhAddEnglishCounterW` (English path, so it works on a Japanese
      Windows), call `PdhCollectQueryData` each period and read it with
      `PdhGetFormattedCounterArrayW(PDH_FMT_DOUBLE)`; busy = `100 - min(idle)`
      over the instances, `_Total` skipped (the busiest disk, as on Linux),
      clamped to 0–100. The first collection has no rate yet, which the
      existing "drop the first reading" rule in `Readings` already covers.
      Close the query (`PdhCloseQuery`) when the thread exits. Keep all
      `unsafe` in a small `#[cfg(windows)]` struct (`DiskBusy` or similar)
      with a `fn read(&mut self) -> Option<f32>`; any PDH failure returns
      `None` for that sample and logs once at `warn`.
    - **System disk busy %, Linux (if feasible within the step): `/sys/block`.**
      Field 10 of each whole disk's `/sys/block/<dev>/stat` (`io_ticks`),
      reporting the max over disks of `delta_io_ticks / elapsed_ms * 100`,
      clamped. If this grows beyond a screenful, return `None` on Linux and
      note it in `learnings.md`.
    - **macOS:** `None`. The UI omits the series.
    - **Payload.** Extend `Sample` with `read: f64` (bytes/s) and
      `disk: Option<f32>`. Test the bytes-over-elapsed throughput (zero
      elapsed does not divide by zero), the busy clamp, and `Readings` still
      dropping the first sample.
    - **Model (`crates/app/ui/src/perf.ts`).** Add `read` and
      `disk: number | null` to the sample type; `PerfView` gains `readMax`,
      computed like `latencyMax` (window max with a floor `READ_FLOOR_BPS`,
      tuned from the hand check). Tests in `perf.test.ts` mirror the
      latency-axis tests.
    - **Drawing (`crates/app/ui/src/main.ts`, `drawPerf` / `PERF_LINES` /
      legend).** Disk busy on the 0–100 % axis, read on `0–readMax`; a `null`
      disk point breaks the path. Legend entries `Riffle disk` and
      `System disk` with swatches, following the existing weight / dash
      convention (Riffle thick solid, system thin dashed). Show the read axis
      maximum as `NN MB/s` at the right of the header next to the latency
      `ms` label.
    - **Colors (user decision).** CPU lines blue, disk lines green, latency
      bars red (the latency bars are green today and change to red). The
      styling guide requires colors as `:root` tokens read through
      `token()`; add overlay tokens (e.g. `--chart-cpu`, `--chart-disk`,
      `--chart-latency`) to `style.css`, read them into `THEME`, and record
      them in `docs/agents/ui-styling.md` as the overlay's chart palette.
      Run the two color greps in `ui-styling.md` before the PR.
    - Widen the `listen<{ process: number; system: number }>` payload type in
      `togglePerf` to the new shape.
    - Docs: `CLAUDE.md` (`src/perf.rs`: CPU and disk sampler; `src/perf.ts`:
      the new series and the read axis maximum with its floor),
      `docs/humans/usage.md` and `usage.ja.md` (the `Performance graph`
      paragraph: the series and their colors, how to read the disk lines,
      that the busy line is absent on macOS). Add the hand-check result to
      `learnings.md`.

## Trade-offs and risks

- **`% Idle Time` inverted vs `% Disk Time`.** `% Disk Time` on `_Total`
  routinely exceeds 100 %; `100 - % Idle Time` clamped stays in 0–100. If the
  hand check shows it understating a saturated disk, the counter path is a
  one-line change.
- **Linux busy % via `/sys/block` (max over disks) vs `None`.** Either is
  acceptable for an overlay; `null` is allowed by the requirements.
- **Read-axis floor.** Too low makes idle noise fill the plot; too high
  flattens real reads. Take it from the hand check and record the measured
  range in `learnings.md`.
- **Windows process read counter counts non-disk I/O** (pipes, sockets,
  Tauri IPC). Documented; expect a small baseline.
- **Payload shape change.** `perf-sample` is internal (one listener in
  `main.ts`); no compatibility concern.

## Progress

- Step 1 done on the automated criteria (`mise run ci`). The Windows hand
  check, the last Done-when item, is still pending (see `learnings.md`).
  Round 1 review: the Windows busy line now reads the busiest disk through
  `\PhysicalDisk(*)` instead of the `_Total` average, and the legend hides
  only an empty `System disk`.
