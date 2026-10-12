# Learnings: perf-overlay-disk

## Step 1

- PDH works on this machine: an ad-hoc test (not committed) of `DiskBusy`
  read `[None, Some(30.0), Some(25.7), Some(25.4)]` over four 300 ms calls,
  so the first `PdhCollectQueryData` only primes the rate as expected. The
  first call skips `PdhGetFormattedCounterValue` entirely, so the "warn once"
  log is not spent on the expected first-sample `PDH_INVALID_DATA`.
- `Cargo.lock` already had two `windows` versions (0.61.3 through another
  dependency and 0.62.2 through sysinfo); adding
  `windows = "0.62"` to `crates/app` only adds an edge to 0.62.2, no third
  version.
- `riffle-app` has no lib target, so its unit tests run as
  `cargo test -p riffle-app perf::`, not `--lib`.
- Linux busy % was small enough to implement: the max over `/sys/block/*`
  entries that have a `device` link (whole physical disks; loop, ram and
  dm devices have none) of `delta io_ticks / elapsed`. It is compiled only on
  Linux and not exercised here beyond the pure `io_ticks` / `busy_from_ticks`
  unit tests.
- The TS sample type was renamed `CpuSample` -> `PerfSample` (and
  `addCpu` / `view().cpu` -> `addSample` / `view().samples`), since it now
  carries disk values too.
- Provisional `READ_FLOOR_BPS` = 10 MB/s (decimal). Rationale: a single
  preview switch reads the embedded JPEG (about 1-3 MB) within one 500 ms
  interval, i.e. roughly 2-6 MB/s, which stays clearly visible below the
  floor without filling the plot, while the idle baseline of Tauri IPC and
  small reads (Windows counts every read the process issues), expected in
  the tens to hundreds of KB/s, stays within a few percent of the axis. A
  scan reading RAW files from an SSD exceeds the floor and rescales the
  axis. Retune from the hand check below.
- The legend hides the `System disk` series when it has no reading in the window (the
  system disk line on macOS), so on Windows / Linux the `System disk` entry appears
  with the first sample rather than when the overlay opens.

## Deferred issues (todo candidates)

- **Pending manual check (Windows, the user's):** with the overlay on (`d`),
  open a large RAW folder that has not been scanned (or force a rescan) and
  browse with the arrow keys during the scan. Expected: the green lines
  (`Riffle disk` thick solid in MB/s, `System disk` thin dashed on 0-100 %)
  update every 500 ms, the CPU lines are blue, the latency bars red, and a
  latency spike lines up with high disk activity (or does not, which is also
  a result). Record the observed idle and scan read rates (MB/s) to retune
  `READ_FLOOR_BPS` in `crates/app/ui/src/perf.ts` (provisionally 10 MB/s).
  Basis: plan Step 1's last Done-when item. The step's checkbox was ticked
  on the automated criteria only.
