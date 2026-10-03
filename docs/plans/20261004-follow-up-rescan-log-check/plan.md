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

# Close the cold-scan follow-up-rescan `open entries` manual check

## Purpose

The `todo.md` item "App: real-device check that a cold scan's follow-up
rescan reads no second `open entries`" (the last item of the file) asked for
a Windows log check of the `follow-up-rescan-open-entries` fix (`start_scan`
drops the scan's `running` entry before it emits `faces-done`, so a rescan
drained off that event never joins the ended scan and never forces a
`folder_entries` read). The check ran on Windows on 2026-10-04
(`mise run tauri:release:devtools`, Timing logs on, `Riffle.log`) and passed:

- Folder `D:\Photos\2026\2026-06-20` (1050 RAWs), cold for the current
  extractor version (`todo=1050`).
- 17:28:28 `open entries rows=437` (the open); 17:28:33
  `rescan deferred: trigger=focus`; 17:28:36 scan extract done, then
  `open entries rows=1050` (the scan wrote rows, so the read is the expected
  `refreshOnScanDone` one); 17:29:36 scan faces `canceled=false`; 17:29:36
  `rescan: trigger=focus deferred=true`; one `open entries rows=1050` for the
  `faces-done` refresh (the faces pass wrote 1050 rows), logged between
  `scan_id=26`'s `scan list` and its reconcile; `scan_id=26` has `todo=0` and
  no further `open entries` follows it.
- `D:\Photos\2026\2026-05-05`, focus with no scan running:
  `rescan: trigger=focus deferred=false`, `scan_id=24` `todo=0`, no
  `open entries`.

So the only `open entries` reads after the open are the ones a pass that
wrote rows legitimately triggers; the deferred focus rescan itself reads
nothing. The item's own TODO says what to do on a pass: drop the `(Inferred)`
tag from the "Emit `faces-done` only after the scan's `running` entry is
dropped" item in `docs/agents/tauri-app.md` and close the todo item.

## Steps

- [x] Step 1: Close the todo item and re-tag the `tauri-app.md` item
  - Done when:
    - `todo.md` no longer contains the section "### App: real-device check
      that a cold scan's follow-up rescan reads no second `open entries`"
      (its Background, Files and TODO paragraphs). No other todo item
      changes. The file still ends in a single newline.
    - In `docs/agents/tauri-app.md`, the heading of the "Emit `faces-done`
      only after the scan's `running` entry is dropped" item carries
      `(Measured)` instead of `(Inferred)` (the file's legend requires every
      item to be tagged Hit, Measured or Inferred; the user chose Measured).
    - The item gains one bullet recording the verification, in the prose
      style the file's other verified items use: verified on Windows
      (2026-10-04, `Riffle.log` with Timing logs on, a 1050-file folder cold
      for the extractor version): after `rescan deferred: trigger=focus`
      during the scan, the rescan drained off `faces-done` logged
      `rescan: trigger=focus deferred=true` with `todo=0` and read no
      `open entries` of its own; the only reads after the open were the
      `scan-done` and `faces-done` refreshes, which each followed a pass that
      wrote rows. A focus with no scan running logged
      `rescan: trigger=focus deferred=false`, `todo=0`, no `open entries`.
      Keep the existing bullets; the Source bullet adds this plan's
      post-archive path
      (`docs/plans/_archived/20261004-follow-up-rescan-log-check/plan.md`)
      as the source of the verification.
    - The log evidence above is recorded in this plan folder's
      `learnings.md`.
    - `mise run ci` passes.
  - Implementation approach:
    - Files: `todo.md`, `docs/agents/tauri-app.md`, and this plan folder.
      `docs/humans/performance.md` already describes the current behavior
      and needs no edit.
    - Match the file's prose style (wrapped at ~80 columns, backticks around
      log line names, no new sub-headings).

## Trade-offs and risks

- Replacement tag: `(Measured)` (chosen by the user) over `(Hit)`, since the
  check measured the fixed behavior and the forced read was never seen
  breaking anything.
- The log shows two `open entries` after the open (the `scan-done` and
  `faces-done` refreshes), not literally none. Both followed a pass that
  wrote rows, which is documented `refreshOnScanDone` /
  `refreshOnFacesDone` behavior; the deferred rescan's own `scan_id=26` read
  nothing. The verification bullet spells this out so it is not mistaken
  for the old symptom.

## Progress

- (none yet)
