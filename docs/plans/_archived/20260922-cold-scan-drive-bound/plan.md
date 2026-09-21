<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Cold first scan root cause: the DRAM-less QLC SATA drive

## Purpose

PR #293 recorded that a cold first scan of a real Sony ARW folder on Windows
runs at ~16-19ms per file (~51-61 files/s), far over the 30-second / 5000-file
target, and left the root cause as unknown in `docs/performance.md` and as a
`todo.md` item. The thread-count sweep that item asked for has now been run
with `riffle-cli scan` on the same machine, and it shows the scan is bound by
the drive holding the photos, a DRAM-less QLC SATA SSD (Crucial BX500 4TB),
not by Riffle's code: throughput plateaus at ~50-67 files/s from 4 threads on
while per-file worker time roughly doubles with each doubling of threads.
Recording this closes the investigation, tells users what to expect and how
to work around it (keep the folder on an NVMe drive), and removes the
now-answered todo item. Because the 30-second / 5000-file target cannot be met
on such a drive regardless of Riffle's code, the user decided to drop the
target from the docs and code comments.

## Steps

- [x] Step 1: Record the drive-bound root cause in `docs/performance.md`, drop the 30-second target, and delete the answered `todo.md` item
  - Done when:
    - `docs/performance.md`, subsection "Real folders on Windows" (under
      "Folder scan throughput"):
      - the framing "from an internal SSD" is corrected to name the drive
        class: "a DRAM-less QLC SATA SSD (Crucial BX500 4TB)" (the drive
        model is fine; no local paths or folder names)
      - the sentence saying the root cause is unknown (with its pointer to
        the `todo.md` item "App: cold first scan on an internal SSD is far
        slower than the extrapolation") is replaced by the finding, and no
        reference to that todo section remains anywhere in the repo outside
        archived plans
      - a new thread-count table is added after the existing #293 table and
        paragraph, with its conditions stated inline: 2026-09-22, same
        Windows 11 machine (i7-13700), release `riffle-cli scan` cross-built
        for `x86_64-pc-windows-gnu`, each run on a different real folder of
        Sony ARWs never read since boot, on the same drive, 0 errors in all
        runs:

        | threads | files | total | files/s | per file on a worker (mean / p95) |
        |---------|-------|-------|---------|-----------------------------------|
        | 1 | 1337 | 41.08s | 33 | 30.7ms / 58.2ms |
        | 4 | 1415 | 27.34s | 52 | 77.0ms / 103.5ms |
        | 8 | 1520 | 29.47s | 52 | 154.9ms / 196.8ms |
        | 22 | 1545 | 22.90s | 67 | 324.2ms / 369.5ms |

      - the interpretation follows the table: throughput plateaus at ~50-67
        files/s from 4 threads on while per-file worker time roughly doubles
        with each doubling of threads, so the workers queue on one shared
        resource, the drive; such drives are slow at cold random reads (no
        DRAM for the mapping table, so extra NAND reads; QLC read latency,
        worse for data written long ago; SATA's single queue); on one thread
        ~30ms per file of which ~10ms is CPU (an estimate from the warm run)
        and ~20ms waiting on the drive; the cause is the drive, not Riffle's
        code, and more threads help little
      - it states that the boot NVMe (Crucial P5, TLC with DRAM) was not
        measured, so no NVMe-vs-SATA comparison exists
      - it gives the workaround: keep the folders being culled on an NVMe
        drive (internal, or an external USB NVMe enclosure with UASP; even on
        5Gbps USB it should be several times faster), stated as an
        expectation, not a measurement
      - the #293 numbers (the three-run table, "0 errors", the Defender and
        warm-run reasoning, the ~82-97s extrapolation to 5000 files, the
        derived ~63MB/s) are kept; the closing paragraph about the double
        scan (`todo.md`: "App: a scan can be started twice...") is kept
    - The 30-second / 5000-file target is removed (user decision):
      - `docs/performance.md`: every statement of the target is removed or
        reworded so no "30-second target" / "30s target" remains (on
        origin/main: the extrapolation paragraph in "Folder scan throughput"
        around line 77, the "Real folders on Windows" sentence around line
        107 — keep the ~82-97s figure, drop "far over the 30-second
        target" —, and the target column/value of the "Opening an indexed
        folder again" table around line 139: drop the target column if it
        only holds targets, keeping the measured values)
      - `crates/cli/src/main.rs` line ~166: the doc comment mentioning the
        "30s scan target" is reworded without the target (comment only; no
        code change)
      - `docs/agents/tauri-app.md` lines ~510 and ~961 are left alone: they
        are guide text / a recorded lesson about the target, not a statement
        of it; only reword ~510 if it reads as asserting a current target
    - `todo.md`: the whole section "App: cold first scan on an internal SSD
      is far slower than the extrapolation" (heading, body and `#### TODO`
      block) is removed; no other section is removed (the double scan start
      and double `open entries` items are app bugs independent of the drive
      and stay)
    - English only; no local paths or folder names anywhere
    - `mise run ci` passes (`lychee --offline --include-fragments` checks
      Markdown anchors, so any `docs/performance.md` heading referenced
      elsewhere must keep its text)
  - Implementation approach:
    - Base on `origin/main`.
    - Keep edits surgical inside the named places; do not restructure
      "Folder scan throughput". Reuse the "per file on a worker (mean / p95)"
      phrasing already used in "Sharpness scoring cost".
    - Follow the guide rule in `docs/agents/tauri-app.md` ("Write a
      performance number with its measurement conditions"): conditions inline
      with the new table; the NVMe workaround and the ~10ms-CPU / ~20ms-drive
      split labelled as estimates.
    - Optional: "Opening an indexed folder again" says "on an internal SSD"
      for the same machine's warm second-open numbers; change only the drive
      phrase if it reads inconsistent.
    - After editing, `git grep -n "cold first scan on an internal SSD" -- ':!docs/plans'`
      and `git grep -niE "30[- ]?s(econd)? (scan )?target" -- ':!docs/plans' ':!docs/agents'`
      return nothing.

## Trade-offs and risks

- The deleted todo item also asked to compare the bounded 1MiB read against
  reading the whole file (not done). The sweep already attributes the cost to
  the drive's cold random-read latency, so that comparison would not change
  the conclusion or the workaround.
- The NVMe expectation is unmeasured and must be worded as an expectation.

## Progress

- (2026-09-22) Step 1 complete
