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

# One `open entries` per cold scan: the follow-up rescan

## Purpose

`todo.md` ("App: `open entries` is called again after a scan's follow-up
rescan") records that a cold scan is followed by a second scan (`scan_id=2`
right after the first one's end) and that `folder_entries` (the `open
entries` log line) runs again for it.

The follow-up rescan is a deferred `resync()`. `resync()` in
`crates/app/ui/src/main.ts` sets `resyncPending` while `scanRunning` is
true, and the `faces-done` handler's `drainResync()` runs it once the scan
ends: `list_arw`, then `startScan` → `scan_folder` (the `scan list` /
`scan prepare` lines of `scan_id=2`). During a plain folder open the
triggers that can set it are the main window's `tauri://focus` (gated only
by `focusRescanDue`, five seconds after the scan _started_, so any switch
away and back during a long cold scan counts), a `folder-changed` watcher
event, or `File > Reload Folder`. The scan itself raises no watcher event:
the index is SQLite outside the folder, the app's sidecar and temp writes
are dropped by `watch::triggers`, and notify's Windows watcher does not
subscribe to last-access changes, so reading the RAWs is silent. The
2026-09-22 log (a 39-minute cold scan) was therefore a focus rescan. That
rescan is intended, the belt-and-braces re-listing after the window regained
focus, and stays.

The extra `open entries` is a separate matter. Both measurements in
`todo.md` predate `docs/plans/_archived/20260926-scan-done-refresh-skip`,
when every `scan-done` re-read the rows. Since then `refreshOnScanDone` and
`refreshOnFacesDone` (`crates/app/ui/src/refresh.ts`) skip the read when
nothing was written, so an idle follow-up rescan (`total: 0` on both events)
reads nothing, with one hole: `scan_folder` counts `changed` as
`removed + changed + usize::from(joined_previous)`, and `start_scan`'s task
emits `faces-done` _before_ `state.finish(scan_id)` drops the `running`
entry (`crates/app/src/commands.rs`, the closure in `start_scan`). A resync
drained off `faces-done` can reach `scan_folder` while the entry is still
there, be counted as having joined a scan, and force one `folder_entries`
read. The window is microseconds against an IPC round trip, so it is rarely
hit, but it is the only path left by which the follow-up rescan reads the
rows.

After this work: the backend drops the `running` entry before `faces-done`,
so a rescan drained off that event never counts a joined scan; the deferred
rescan names its trigger in the timing log so the next measurement shows why
`scan_id=2` ran; and the docs stop describing the double read. The `todo.md`
section "App: `open entries` is called again after a scan's follow-up
rescan" is closed out by the wrap-up (delete it; its finding is in this
plan and in `docs/humans/performance.md`).

## Steps

- [x] Step 1: Drop the `running` entry before emitting `faces-done`
  - Done when:
    - In `crates/app/src/commands.rs`, the scan task spawned in `start_scan`
      calls `state.finish(scan_id)` (releasing the lock) _before_ it emits
      `faces-done`, with a comment saying why: the frontend drains a
      deferred rescan off `faces-done`, and that rescan's `scan_folder`
      must not find this scan still in `running`, or `joined_previous`
      makes `changed` nonzero and forces a `folder_entries` read of an
      unchanged folder (see `refreshOnScanDone` in `refresh.ts`).
    - The `refreshOnScanDone` comment in `crates/app/ui/src/refresh.ts`
      (the paragraph starting "`changed` also counts 1 when `scan_folder`
      joined a previous scan") gains one sentence: a scan that has ended
      is never joined, because `start_scan`'s task drops its entry before
      `faces-done`.
    - A Rust test in `commands.rs`'s `tests` module (next to
      `a_finished_scan_leaves_no_scan_in_progress`, reusing its
      `spawn_scan` helper shape) asserts the ordering at the level the
      module can test: after the task's `finish`, `ScansState::scanning()`
      is false and `running.take()` is `None`, so a `scan_folder` that
      runs after the "done" signal sees nothing to join. If the emit itself
      cannot be exercised without an `AppHandle` (it cannot today), record
      in `learnings.md` that the ordering is covered by the comment and the
      manual check in Step 3, not by a test of the emit.
    - `cargo test -p riffle-app` (or the workspace test command CI uses)
      passes; `mise run ci` passes.
  - Implementation approach:
    - Only reorder: move the `let scans = app.state::<Scans>(); let mut
state = index::lock(&scans.0); state.finish(scan_id);` block above
      the `app.emit("faces-done", …)` call and drop the lock before the
      emit. Nothing else in the closure changes.
    - Check `clear_index` and the settings modal's "scan running" gate:
      `ScansState::scanning()` goes false a moment before the frontend's
      `scanRunning` does. The frontend already gates its Clear Cache button
      on its own flag, and `clear_index` re-checks `scanning()` itself, so
      the earlier false only lets a clear that was already waiting proceed
      slightly earlier. State this in the comment if it matters; do not add
      handling.
    - Do not touch `scan_folder`'s `joined_previous` counting or the
      frontend's `refreshOnScanDone`: the deliberate over-count on a folder
      switch mid-scan stays.

- [ ] Step 2: Name the trigger of a deferred rescan in the timing log
  - Done when:
    - `resync()` in `crates/app/ui/src/main.ts` takes a `trigger` argument
      (a small string union, e.g. `"focus" | "watch" | "reload" | "trash" |
"restore" | "rename" | "drained"`), and every caller passes its own
      (`tauri://focus` → `focus`, `folder-changed` → `watch`,
      `reload-folder` → `reload`, the trash / restore / rename callers
      theirs). `drainResync()` passes the trigger that was deferred, so
      the log of the drained run names the original cause, not
      "drained"; keep the deferred trigger in a variable next to
      `resyncPending` (or replace the boolean with `string | null`).
    - `resync()` writes one `debugLog` line when it defers
      (`rescan deferred: trigger=<t>`) and one when it starts the listing
      (`rescan: trigger=<t> deferred=<true|false>`), under the existing
      `Timing logs` gate (`debugLog` → `log_timing`), so a cold-scan log
      shows why `scan_id=2` ran.
    - A pure helper in `crates/app/ui/src/refresh.ts` builds the line
      (same shape as `refreshTimingLine`), with a `refresh.test.ts` case
      per form, so the strings are tested without `main.ts`.
    - `mise run ci` passes.
  - Implementation approach:
    - `main.ts` has no unit tests; keep the testable part (the line format)
      in `refresh.ts` as the existing timing helpers do.
    - Update the comments above the `tauri://focus`, `folder-changed` and
      `reload-folder` listeners only where the trigger name is introduced;
      do not reword them otherwise.
    - This step assumes Step 1 is merged only for the manual check in
      Step 3; it has no code dependency on it.

- [ ] Step 3: Update the docs and verify one `open entries` per cold scan
  - Done when:
    - `docs/humans/performance.md`: the paragraph under the 2677-file table
      that says "each open called `open entries` twice (56ms and 76ms)" and
      points at the `todo.md` item is rewritten to say the second call was
      the pre-2026-09-26 `scan-done` re-read (removed by the refresh skip)
      and that a rescan deferred during a scan (focus, watcher, reload) runs
      at the scan's end and reads the rows only when it changed something;
      point at the new `rescan:` log line for seeing which trigger it was.
      Leave the "two focus rescans once fired at the same instant" sentence
      alone (that is the parallel todo).
    - The "Measuring on your own folder" section mentions, in one sentence,
      that a `scan_id` following the cold scan's with `todo=0` is such a
      deferred rescan and is not part of the first-scan total.
    - `docs/agents/tauri-app.md`: a short item in the Hits list (next to the
      `tauri://focus` one) that `faces-done` must be emitted after the
      scan's `running` entry is dropped, with the `joined_previous` reason.
    - Manual check on Windows (`mise run tauri:release:devtools`, Timing
      logs on), done by the user after merge: clear the cache, open a
      folder large enough that the scan runs well past five seconds, switch
      to another window and back during the scan, wait for the end. Expect
      in `Riffle.log`: one `open entries` for the open, a `rescan
deferred: trigger=focus` line, then the `scan list` / `scan prepare`
      of the next `scan_id` with `todo=0` and no further `open entries`.
      Record the result in `learnings.md`. If a second `open entries` still
      appears, note its `changed` / `total` by reading the surrounding
      lines and reopen the item instead of closing it in the wrap-up.
    - `mise run lint` (lychee) passes on the edited docs.

## Trade-offs and risks

- **Whether Step 1 is needed at all.** The `faces-done`-before-`finish`
  window is microseconds of Rust against the frontend's `list_arw` round
  trip, so the forced read is very unlikely in practice, and the current
  code may already yield one `open entries` per cold scan. The step is
  kept because it is a five-line reorder that makes the invariant hold
  deterministically and is the only remaining code path for the reported
  symptom.
- **`scanning()` false slightly before `faces-done`.** After Step 1 a
  `clear_index` (or the eviction's `VACUUM`) that was waiting on the Scans
  lock can run between `finish` and the frontend handling `faces-done`.
  The frontend's own `scanRunning` gate still disables Clear Cache until
  `faces-done`, so the only way in is a request already in flight, which
  `clear_index` already refuses while `scanning()` is true and accepts
  otherwise; nothing new becomes possible. Call this out in the comment.
- **Keeping the follow-up rescan.** Removing the rescan instead (e.g.
  dropping a focus trigger that arrived during a scan) would save the
  `scan list` / `scan prepare` cost (~65 ms on 2677 files) but lose the
  re-listing that catches files added while the window was away and the
  watcher missed them (a network volume, a failed watch). Not taken; the
  cost is already deemed not noticeable in `performance.md`.
- **Overlap with the parallel "scan can be started twice" session.** That
  item lives in the same `resync` / `drainResync` / `tauri://focus`
  machinery. Step 2 changes `resync`'s signature and the pending flag;
  if that session edits the same functions, rebase one on the other and
  keep the trigger argument. Report the `faces-done`-before-`finish`
  ordering to that session too: a rescan that lands in that window is
  joined and canceled rather than refused, which could look like a
  "started twice" in the log.
- **No automated test of the end-to-end count.** `main.ts` is untested
  and the scan runs through Tauri IPC, so the "exactly one `open entries`"
  criterion is a manual log check (precedent:
  `docs/plans/_archived/20260922-focus-rescan-main-window`). The unit tests
  cover the ordering helper (Step 1) and the log line (Step 2).

## Progress

- (none yet)
