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

# Real-folder scan measurements

## Purpose

Every folder-scan and second-open figure in `docs/performance.md` was measured
on 5000 symlinks to one ARW or on freshly `cp`-copied files, never on a real
folder on real hardware. The user has now measured a release build on Windows
11 (internal SSD, 22 extraction threads, Sony ARW) from the `Riffle.log`
timing lines. Recording those numbers replaces the optimistic extrapolation
(9.5s for 5000 files) with what actually happens (~82-97s extrapolated, far
over the 30s target), closes two `todo.md` measurement items, and files the
three problems the measurement surfaced so they can be worked on.

## Steps

- [x] Step 1: Record the Windows real-folder scan measurements in `docs/performance.md` and update `todo.md`
  - Done when:
    - `docs/performance.md` has a new subsection under "Folder scan
      throughput" (after the extrapolation paragraph and its "What this
      cannot measure" caveat, before "Sharpness scoring cost") that records
      the real-folder measurements with their conditions (2026-09-21,
      Windows 11 Home, internal SSD, release app, 22 extraction threads, Sony
      ARW, numbers taken from `Riffle.log` `scan prepare` / `scan extract` /
      `open list` / `open entries` lines):
      - cold first scan, 2677 files, Defender real-time protection on:
        prepare 1427ms + extract 42509ms = ~43.9s (~16ms/file, ~61 files/s),
        0 errors
      - cold first scan, 3401 files, folder excluded from Defender:
        prepare 639ms + extract 65605ms = ~66.2s (~19ms/file); Defender is
        not the cause
      - first scan with a warm page cache (index cleared, files read moments
        before), 2134 files: prepare 47ms + extract 2084ms (~1ms/file), so
        CPU is not the bottleneck and cold IO dominates
      - extrapolation to 5000 files: ~82-97s, far over the 30s target;
        effective throughput ~63MB/s for 1MiB bounded reads on an internal
        SSD, root cause unknown
    - the existing extrapolation paragraph in "Folder scan throughput" gets a
      one-sentence caveat pointing at the new subsection (the 9.5s / 21.7s
      extrapolation does not hold on real cold reads); the surrounding text is
      otherwise left alone
    - "Opening an indexed folder again" records the real second open
      (relaunch, 2677 files: `open list` 34ms + `open entries` 56ms +
      `scan prepare` 73ms = ~163ms, with a second `open entries` call of
      76ms per open) and the focus rescan of the unchanged 2677-file folder
      (`scan prepare` ~65ms, not noticeable), next to the existing
      symlink-based table, and the sentence "the real number on a real folder
      of 5000 distinct files has never been measured by anyone" is corrected
      to point at the new numbers
    - the observed anomalies are noted briefly in the doc where the numbers
      are (a `scan_id` superseded by the next with no `scan extract` line
      after a cache clear, two focus rescans fired at the same instant, and
      `open entries` called twice per open), each with a pointer to the
      corresponding `todo.md` item
    - `todo.md`: the section "App: real-folder scan and second-open numbers
      are still missing" (~line 21) and the section "App: unmeasured cost of
      a rescan on an unchanged large folder" (~line 176) are removed whole
      (heading, body and `#### TODO` block), matching how #291 closed an item
    - `todo.md`: three new sections are added under "Cross-cutting / other",
      each with a short body and a `#### TODO` checkbox, following the
      existing `### App: ...` / `#### TODO` shape:
      1. "App: cold first scan on an internal SSD is far slower than the
         extrapolation" — body states the ~16-19ms/file cold read on Windows,
         that Defender and CPU were ruled out, and that the cause is unknown;
         TODO: run `riffle-cli scan` on a cold real folder on Windows at
         thread counts 1 / 4 / 8 / 22 (cold each run) to separate IO
         concurrency from per-file cost, and compare the bounded 1MiB read
         against reading the whole file
      2. "App: a scan can be started twice after a cache clear / focus
         rescan" — body describes the observation (after a cache clear
         `scan_id` N was superseded by N+1 with no `scan extract` line;
         two focus rescans fired at the same instant), files:
         `crates/app/src/commands.rs` (`scan_folder`),
         `crates/app/src/watch.rs`, the settings-window clear-cache path;
         TODO: find why two scans start and make the second one not fire
         (or coalesce), verified by the log showing one `scan extract` per
         trigger
      3. "App: `open entries` is called twice per folder open" — body states
         the log shows two `open entries` lines (56ms and 76ms on 2677
         files) for one open; TODO: find the second caller (frontend
         `crates/app/ui/src/main.ts` / backend `crates/app/src/commands.rs`)
         and remove the redundant call, or document why both are needed
    - no local machine paths appear anywhere (say "an internal SSD" / "a real
      folder of N ARWs"); English only
    - `mise run ci` passes
  - Implementation approach:
    - Docs-only PR; commit as `docs(perf): record real-folder scan
      measurements on Windows` (Conventional Commits, English)
    - Keep edits surgical: add a subsection and short caveat sentences rather
      than rewriting the existing symlink tables; match the existing table
      and prose style in `docs/performance.md` (tables with conditions in the
      first column, per-file numbers in prose)
    - Use the per-run numbers as measured (files, prepare, extract, total,
      per-file, files/s) and mark file counts per run explicitly, since the
      three cold/warm runs used different folders (2677 / 3401 / 2134 files)
    - "Measuring on your own folder" already describes the log lines; do not
      duplicate the procedure, just reference it

## Trade-offs and risks

- Placement of the second-open / rescan numbers: adding a small table row set
  under the existing "Opening an indexed folder again" table vs. a separate
  subsection. The implementer may choose either, but should not delete the
  symlink-based rows, since the footnote and caveats refer to them.
- The `scan_id 6/7` duplicate-start anomaly and the simultaneous focus rescans
  may be one bug or two. They are filed as a single todo item to avoid
  guessing; the investigating step can split it.
- The "~63MB/s effective throughput" figure derives from a 1MiB bounded read
  per file at ~16ms/file; it is presented as derived, not measured.
- Both closed items sit under the single "Cross-cutting / other" heading,
  which stays non-empty, so no heading removal is needed.

## Progress

- (none yet)
