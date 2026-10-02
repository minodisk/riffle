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

# Close the double scan start on a focus rescan

## Purpose

The todo.md section "App: a scan can be started twice after a cache clear /
focus rescan" records two Windows observations: after a cache clear, `scan_id`
N was superseded by N+1 with no `scan extract` line for N, and `scan_id=3` and
`4` started in the same second (04:39:40, v0.2.0, 2026-09-22). Both are one
mechanism: a `tauri://focus` rescan (`resync()` -> `scan_folder`) racing a
programmatic `openDirectory()` -> `startScan()` -> `scan_folder` for the same
user action. Two ids are minted within a second, the later supersedes the
earlier under the `Scans` lock (`crates/app/src/commands.rs`, `scan_folder`'s
`latest_id` check), and the earlier never reaches `scan extract`.

The two recorded variants are already fixed on `main`, but the todo was never
re-verified and closed:

- The settings _window's_ focus reaching the main window's global
  `event.listen("tauri://focus")`: fixed by #297 (2026-09-22 02:03 JST,
  `getCurrentWindow().listen`), see
  `../_archived/20260922-focus-rescan-main-window/plan.md`.
- The 04:39:40 repro, recorded in #349 (14:59 JST, after #297, on v0.2.0):
  `clear_index` parents its native confirm dialog to the **main** window
  (`crates/app/src/commands.rs`, `dialog.parent(&window)` with the `main`
  webview). When the dialog closed, the main window's own focus fired v0.2.0's
  then-unguarded `getCurrentWindow().listen("tauri://focus", resync)` ->
  `scan_folder` (N), while `index-cleared` -> `openDirectory` -> `startScan`
  minted N+1; `openDirectory` resets `setScanRunning(false)` before its
  `startScan`, so the scan-running deferral did not catch it. Fixed by #412
  (2026-09-25): the listener skips while `settings.isOpen`
  (`crates/app/ui/src/main.ts`, the `tauri://focus` listener), and #608 added
  the `focusRescanDue` 5 s throttle.

One variant of the same mechanism is still live: the **folder picker**.
`openFolder()` mints `folderToken` _before_ `pick_folder`; the dialog's close
refocuses the main window; when the last scan started `FOCUS_RESCAN_INTERVAL`
or more ago, the listener calls `resync()` on the still-open _previous_
folder. `resync` captures `dir = openDir` (the old folder) and
`token = folderToken` (already the new token), so its post-`list_arw` guard
(`dir !== openDir || token !== folderToken`) lets it through whenever its
`list_arw` resolves before `openDirectory`'s `Promise.all([list_arw,
last_viewed])` does: `startScan(oldDir)` mints N, then `openDirectory` ->
`startScan(newDir)` mints N+1 in the same second, and N is superseded before
`scan extract`. #608's throttle only covers the other ordering (the focus
arriving after `startScan` stamped `lastScanAt`). Closing this leaves one
`scan_folder` per user action on every path that opens a dialog.

The related todo "App: `open entries` is called again after a scan's
follow-up rescan" is the same family (the picker's focus arriving _after_
`startScan`, deferred into `resyncPending` and drained at scan-done), but
#608 already skips that ordering (`focusRescanDue` is false inside 5 s of
`lastScanAt`), so this plan does not touch it.

## Steps

- [x] Step 1: Defer a rescan while the folder picker is open, and close the todo
  - Done when:
    - A `tauri://focus` (or `folder-changed`) that arrives between
      `pick_folder` opening and the picked folder's `openDirectory` settling
      no longer calls `scan_folder` for the previous folder: it is deferred
      the way a rescan during a scan is, and a successful open discards it
      (`openDirectory` already sets `resyncPending = false`) while a canceled
      picker drains it (the ordinary focus rescan still happens).
    - `scan_folder`'s `latest_id != scan_id` early return in
      `crates/app/src/commands.rs` logs
      `scan superseded: dir={dir} scan_id={scan_id} by={latest}` at info level,
      so a superseded id is named in the log rather than inferred from a
      missing `scan extract` line.
    - `mise run ci` passes (format, lint, type check, vitest, Rust tests).
    - Manual check on Windows (`mise run tauri:release:devtools`, timing logs
      on), by the user after merge: with folder A open and its scan older
      than 5 s, File > Open Folder and pick folder B. Expect exactly one
      `scan list` / `scan prepare` / `scan extract` set, for B, and no
      `scan superseded` line. Repeat with Settings > Clear Cache (one set for
      the reopen) and a plain alt-tab back to the app (one set).
    - `todo.md`: delete the section "App: a scan can be started twice after a
      cache clear / focus rescan" (its two recorded causes are fixed by #297
      and #412/#608; the picker variant by this step). Leave the "`open
entries` is called again" section as is.
    - `docs/agents/tauri-app.md`: extend the existing Hit "A global
      `event.listen("tauri://focus")` fires for every window" with the
      picker variant: a native dialog's close refocuses the main window, so
      every dialog-opening path (`clear_index`'s confirm, `pick_folder`) needs
      the focus rescan held off until its follow-up open has run, and how
      this step does it.
  - Implementation approach (as far as it is known):
    - Frontend change in `crates/app/ui/src/main.ts`; on the Rust side only the
      log line above. The `latest_id` supersede logic in `scan_folder` /
      `start_scan` is correct and stays as is.
    - Reuse the existing deferral: `resync()` already returns with
      `resyncPending = true` while `scanRunning || resyncInFlight ||
idle.inFlight`, and `settleIdle(promise)` marks an invoke as in flight
      and drains the pending rescan when it settles. Wrap `openFolder`'s
      whole chain (the `pick_folder` invoke through `openDirectory`) in
      `settleIdle(...)`: a picked folder's `openDirectory` discards the
      deferred rescan (`resyncPending = false`, `idle.discard()` leaves
      `inFlightCount` alone), and a canceled picker's `finally` drains it.
      Widen the comment on `settleIdle` (currently "a deferred operation's
      invoke") to say a folder picker counts too, and note it in the
      `tauri://focus` listener's comment next to the `settings.isOpen` skip.
    - No automated test reproduces the race itself: `main.ts` has no unit
      tests and the race is Tauri IPC ordering between two `invoke`s and a
      window event (the same conclusion as the archived focus-rescan plan).
      `IdleGate` is already covered by `crates/app/ui/src/idle.test.ts`. Do not
      add a helper only to have something to test.

## Trade-offs and risks

- `settleIdle` reuse vs a dedicated `pickerOpen` flag: reuse adds no state and
  also covers a `folder-changed` during the picker, at the cost of stretching
  `IdleGate`'s "deferred operation" wording (fixed in the comment). A flag
  would have to stay set until `openDirectory` has run, not just until
  `pick_folder` resolves.
- The deferred rescan on a canceled picker runs after the cancel, exactly as
  the focus rescan did before; no behavior change there. A deferred rescan
  on a _successful_ pick is dropped, which is right: the fresh open re-lists.
- The other `openDirectory` callers (folder tree click, drop, rename reopen,
  `index-cleared`, `sidecar-format`, `reopenLastFolder`) open no dialog, so
  no focus event races them; a watcher event landing inside their `list_arw`
  is theoretically the same race but has no observation behind it, so it is
  left alone (minimal change).

## Progress

- (none yet)
