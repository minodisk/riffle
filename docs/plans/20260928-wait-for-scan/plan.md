<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Wait for the scan instead of refusing operations

## Purpose

`File > Move Rejected to Trash…`, `Rename…` (folder and file) and the settings
modal's `Clear Cache` all refuse with `a scan is running; wait for it to
finish` while a scan runs. On real hardware that message shows up for
operations the user did nothing to collide with: every time the main window
regains focus, `resync()` (the `tauri://focus` listener in
`crates/app/ui/src/main.ts`) starts a rescan, and even a rescan with nothing
to do keeps the frontend's `scanRunning` true across the `scan_folder` →
`start_scan` → `scan-done` → `faces-done` round trip. Coming back from a
terminal and opening the File menu is enough to hit the window; pressing
again a few seconds later works.

After this work an operation pressed during a scan is held, the status line
says what is waiting, and it runs by itself when the scan ends. Switching
folders while it waits drops it. The backend's `SCAN_RUNNING` refusals stay
as they are, as the last line of defense against a scan that starts in the
gap the frontend cannot see; they should no longer be reachable from normal
use.

Decisions taken here (see "Trade-offs and risks" for the alternatives):

- The waiting happens in the **frontend only**. The frontend already owns the
  `scanRunning` flag, the folder token and the status line, which is
  everything "wait, show, discard on folder switch" needs; the backend is
  not touched, which keeps this change out of the way of the concurrent
  trash-folders and tree-watch work in `crates/app/src/commands.rs`,
  `crates/app/src/folders.rs` and `crates/app/src/watch.rs`.
- A press during a **long scan waits too** (no cancel-then-run, no
  threshold). Every held operation is either confirmed by a native dialog
  before anything moves (trash, clear) or was typed into an inline edit
  (rename), so a late run is never a surprise; the status line names what is
  waiting, and a folder switch discards it.
- One pending slot: a second press while one operation waits replaces the
  first (the last press is what the user meant).

## Steps

- [x] Step 1: Hold Move Rejected to Trash and the two renames until the scan ends
  - Done when:
    - A new pure module `crates/app/ui/src/idle.ts` holds the deferral. Its
      shape is up to the implementation, but it must be testable without the
      DOM or Tauri, e.g. a class constructed with a `busy: () => boolean`
      that offers: `request(label, run)` (runs `run` at once when not busy,
      else holds `{ label, run }`, replacing any earlier one), `drain()`
      (runs and clears the held operation; called when the scan ends),
      `discard()` (drops it without running; called on a folder switch), a
      `waiting` getter returning the held label or `null` for the status
      line, and an `inFlight` flag (or equivalent) set by the caller from
      dispatch to settle, so `resync()` can queue behind the running
      operation.
    - `crates/app/ui/src/idle.test.ts` (vitest, like
      `crates/app/ui/src/refresh.test.ts`) covers: runs at once when idle;
      holds when busy and `waiting` reports the label; `drain()` runs the
      held operation exactly once and clears `waiting`; a second `request`
      while busy replaces the first (the first never runs); `discard()`
      drops it and a later `drain()` runs nothing; `drain()` with nothing
      held is a no-op.
    - `main.ts` owns one instance (`busy: () => scanRunning`) and routes
      through it:
      - `trashRejected()`: the `scanRunning` refusal (~line 546) is replaced
        by `request("Move Rejected to Trash", …)` around the existing
        `invoke("trash_rejected")`; the existing `dir` / `token` guards on
        the result stay. Because the held closure runs later, it must read
        `openDir`, `folderToken`, `allFiles` and `flags` (`rejectedPaths`)
        **at run time**, not at press time: the rejects may have changed
        while waiting, and "No rejected files in this folder" is decided
        then.
      - `renameFolder(path, name)` and `renameFile(path, name)`: the invoke
        goes through `request("Rename…", …)`; the inline edit itself starts
        immediately (the `scanRunning` checks in the context-menu
        `renameFolder` / `renameFile` branches, ~2022 and ~2280, are
        removed, and the `renameAllowed` callbacks passed to `strip.init`
        (~2108) and `folders.init` (~2294) drop their `scanRunning` term:
        `() => !viewOnly` and `() => true`). The existing `renameFile`
        `dir` / `token` guards stay and are read at run time.
      - The instance's `inFlight` is set from dispatch until the invoke
        settles (`.finally`), and `resync()` treats it like `scanRunning`
        (`resyncPending = true; return;`); the operation's settle path calls
        `drainResync()`. This closes the race where the confirm dialog's
        close refocuses the window, `resync()` starts `scan_folder`, and the
        backend's second guard (after the sidecar-writer drain) sees
        `preparing > 0`. Nothing is lost: `trashRejected` already calls
        `resync()` on success, and a rename reopens the folder.
      - `drain()` is called in the `faces-done` listener (~2773) right after
        `setScanRunning(false)` and **before** `drainResync()`, and in
        `startScan`'s `.catch` (~2370) next to its `setScanRunning(false)`.
      - `discard()` is called in `openDirectory` next to
        `setScanRunning(false); resyncPending = false;` (~2482), so a folder
        switch while waiting drops the operation.
      - `renderMeta()` appends a status line while `waiting !== null`, next
        to the `scanning` / `sequencing` lines (~491), e.g.
        `Move Rejected to Trash: waiting for the scan to finish`, so the
        wait is visible for as long as it lasts (it is not a transient
        `note`, which the next `setStatus` would erase).
    - The string `"a scan is running; wait for it to finish"` no longer
      appears in `main.ts`.
    - `CLAUDE.md`'s layout paragraph names `src/idle.ts`; `docs/usage.md`'s
      folder `Rename…` (~line 50), file `Rename…` (~89) and
      `Move Rejected to Trash…` (~209) paragraphs say the operation waits for
      a running scan and runs when it ends, and that switching folders
      meanwhile drops it; `docs/agents/tauri-app.md`'s `resync` bullet
      (~1160) mentions that an in-flight deferred operation defers `resync`
      the same way `scanRunning` does. `README.md` has no matching sentence,
      so it and `README.ja.md` are unchanged (check with a grep before
      concluding).
    - `mise run ci` passes.
  - Implementation approach (as far as it is known; omit if unknown):
    - Keep `main.ts` edits to the listed sites; the module holds the logic
      so the tests do not need `main.ts`.
    - Leave the backend (`commands.rs`, `rename.rs`) untouched, including
      the `SCAN_RUNNING` constant and its refusals; they remain the guard
      against the gap the frontend cannot see (the backend emits
      `faces-done` before it takes the `Scans` lock and calls `finish`, and
      `Preparing` can start between the frontend's check and the command's
      own).
    - Do not remove the `renameAllowed` parameter from `folders.init` /
      `strip.init`; the tree-watch worktree touches `folders.ts`, so pass a
      constant instead (see "Trade-offs and risks").
    - Hand-check on the user's Windows setup if available: open an ARW
      folder, reject a file, click away to a terminal and back, open
      `File > Move Rejected to Trash…` immediately; the status line should
      show the waiting text briefly and the confirm dialog should follow
      without the error.

- [x] Step 2: Hold Clear Cache the same way
  - Done when:
    - `crates/app/ui/src/settings.ts` no longer disables the `Clear Cache`
      button while a scan runs. A click while `scanRunning` holds the clear
      through a second instance of the Step 1 module (`busy: () =>
      scanRunning`, local to `initSettings`), and `setScanRunning(false)`
      drains it; the existing `clearInFlight` handling (button disabled
      while the invoke is out, size refresh afterwards) is unchanged.
    - `#clear-index-note` in `crates/app/ui/index.html` is repurposed as the
      waiting indicator: shown while the clear is held, with text along the
      lines of `The cache is cleared as soon as the running scan finishes.`;
      hidden otherwise (`updateClearButton` keys `hidden` on `waiting ===
      null` instead of `!scanRunning`).
    - `docs/usage.md`'s Clear Cache paragraph (~line 327) says a press
      during a scan waits and clears when the scan ends, instead of "the
      button is unavailable".
    - `mise run ci` passes.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Step 1 is merged (reuses `idle.ts`). No new tests are needed
      for the module; if the settings wiring gains any non-trivial pure
      decision (e.g. "note visible when"), extract and test it, otherwise
      the Step 1 tests cover the logic.
    - The `tauri://focus` listener already skips `resync()` while
      `settings.isOpen`, so a scan running when the modal opens is the only
      one Clear Cache can meet; no `inFlight` gating of `resync` is needed
      from `settings.ts`. Folder switches cannot happen while the modal is
      open, so no `discard()` call is needed there either; note this in a
      comment.
    - Keep the backend `clear_index` refusals as they are.

## Trade-offs and risks

- **Long scan: wait (chosen) vs cancel-then-run vs refuse as before.**
  Waiting is the simplest and is safe because every held operation is
  confirmed or typed before it runs; the cost is that a press during a
  5000-file first scan surfaces its confirm dialog minutes later, with only
  the status line as a reminder. Cancel-then-run would need the frontend to
  cancel the scan (there is no command for that today; `scan_folder` cancels
  the previous scan as a side effect), run, then restart, and it interacts
  with `scanId` / `resyncPending`; refusing above a threshold needs an
  arbitrary number and would still refuse the user's actual case if the
  threshold were wrong. If the caller wants a way out of a long wait besides
  switching folders, the cheapest is `Escape` calling `discard()`; not
  planned.
- **Frontend-only (chosen) vs a backend wait.** A `Condvar` next to the
  `Scans` mutex with a bounded `lock_idle` helper replacing the
  `if state.scanning() { return Err(SCAN_RUNNING) }` guards would make the
  backend itself wait out a scan that starts in the gap. It would touch
  every `scan-state` emit site in `commands.rs` (`Preparing::drop`,
  `finish`, `start_scan`, the `pending.clear()` path) and `rename.rs`, right
  where the trash-folders worktree is adding commands, and it cannot honor
  "discard on folder switch" (the backend does not know the frontend's
  folder). The frontend's `inFlight` gating of `resync()` closes the one
  realistic race (dialog close → focus → rescan). Two backend items remain
  as optional follow-ups the caller may promote to a step: moving
  `state.finish(scan_id)` under the lock *before* the `faces-done` emit
  (`commands.rs` ~1444–1467, a 3-line reorder that makes "frontend saw
  `faces-done`" imply "backend is idle"), and the `Condvar` wait above.
- **Discard on folder switch applies to a tree rename of a folder that is
  not the open one.** The requirement says to discard; the plan applies it
  uniformly through `openDirectory`. If the caller would rather let a rename
  of an unrelated tree folder survive a folder switch, `renameFolder` can
  skip the shared instance's `discard()` by using its own instance; not
  planned.
- **`renameAllowed` callbacks become constants** (`() => true` for the
  tree, `() => !viewOnly` for the strip) instead of removing the parameter
  from `folders.init` / `strip.init`, to stay out of `folders.ts` while the
  tree-watch worktree is in it. Removing the tree's parameter is a small
  cleanup for after that work lands.
- **Run-time reads.** A held `trashRejected` must compute `rejectedPaths`
  when it runs, not when it was pressed, or it can trash a file the user
  un-rejected while waiting. The Step 1 acceptance calls this out; a review
  should check it.
- **Concurrent work.** The trash-folders worktree adds a folder-menu
  `Move Rejected to Trash` for one or several folders; it will likely copy
  today's `scanRunning` refusal. Whichever lands second should route the new
  entry through `idle.ts` as well (a one-line change once both are merged).

## Progress

- (2026-09-28) Step 1 complete
