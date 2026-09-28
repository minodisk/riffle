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

# Record the wait-for-scan manual verification in todo.md

## Purpose

`wait-for-scan` (plan `../_archived/20260928-wait-for-scan/plan.md`, PRs #519
and #522) made `File > Move Rejected to Trash…`, the folder / file `Rename…`
and the settings modal's `Clear Cache` wait for a running scan (the
`IdleGate` in `crates/app/ui/src/idle.ts`, wrapped by `whenIdle` /
`settleIdle` in `crates/app/ui/src/main.ts`) instead of refusing with
`a scan is running; wait for it to finish`. None of it has been run by hand on
real hardware. The Clear Cache half already has its checks in todo.md (the
`### App: the Clear Cache button's manual GUI verification is still open`
item, whose checks (4) and (5) were rewritten for it); the trash and rename
halves have no item. This plan adds one todo item so the open checks are not
lost, in the file's item format, without duplicating the Clear Cache item.

## Steps

- [x] Step 1: Add the wait-for-scan manual checks as one item in `todo.md`
  - Done when:
    - `todo.md` has a new `### App: the wait-for-scan manual checks for Move
      Rejected to Trash and Rename are still open` item (title wording is
      free, but platform-neutral: no "Windows" in the heading) under
      `## Cross-cutting / other`, in the file's current format:
      `#### Background` paragraph(s) with a `Basis:` and `Files:` line, then
      `#### TODO` with `- [ ]` checkboxes (see the two `wait-for-scan` items
      already there, `### App: a pending rename waits silently with no
      visible pending state` and `### App: `folders.init`'s `renameAllowed`
      parameter is now always `() => true``, for the exact shape).
    - The Background says what the feature does, that the frontend `IdleGate`
      holds one operation (a second press replaces it), that the backend
      `SCAN_RUNNING` refusals (`crates/app/src/commands.rs` for trash,
      `crates/app/src/rename.rs` for renames) remain as the last line of
      defense, that nothing has been run by hand (the user's
      Windows setup is where they will first be run), and that the Clear
      Cache checks live in `### App: the Clear Cache button's manual GUI
      verification is still open` (referenced by heading, not repeated).
      `Files:` lists `crates/app/ui/src/idle.ts`, `crates/app/ui/src/main.ts`
      (`whenIdle`, `settleIdle`, `trashRejectedIn`, `renameFolder`,
      `renameFile`, the `faces-done` `idle.drain()` and the `openDirectory`
      `idle.discard()`), `crates/app/ui/src/strip.ts` (`setFiles` carrying
      the live inline edit across a rebuild).
    - The TODO has these checkboxes, worded against the current code (verify
      each reference before writing; the status text is built in
      `renderMeta` as `` `${idle.waiting}: waiting for the scan to finish` ``
      with labels `Move Rejected to Trash` and `Rename…`):
      1. In an ARW folder, reject a file, switch to a terminal and straight
         back (the `tauri://focus` `resync()` starts a rescan), press
         `File > Move Rejected to Trash…`: the status line briefly shows
         `Move Rejected to Trash: waiting for the scan to finish` and the
         in-window trash dialog (`#trash-dialog`) follows with no error.
      2. During a long first scan (thousands of files) press Move Rejected
         to Trash: the status line stays visible for the whole scan and the
         dialog appears when the scan ends.
      3. Un-reject a file while the trash waits: it is absent from the
         dialog's counts and is not trashed (the held closure calls
         `trash_rejected_preview` at run time, not press time).
      4. Switch folders while an operation waits: it is dropped
         (`idle.discard()` in `openDirectory`) and the status line clears.
      5. Rename a folder (tree inline edit) and a file (strip inline edit)
         during a scan: the edit starts at once, the status line shows
         `Rename…: waiting for the scan to finish`, the rename runs when the
         scan ends, and, when the renamed folder is the open one (or
         contains it), the folder reopens at the new path; a file rename
         keeps the cell's marks under the new name; an inline edit still
         being typed survives the strip being rebuilt by the scan's
         `setFiles` without losing typed text or confirming early.
      6. Pressing `Move to Trash` in the trash dialog (`trash_rejected_run`
         through `settleIdle`) and confirming a rename never make the
         backend refuse with `a scan is running`; the native-confirm
         refocus case (closing a dialog refocuses `main` and `resync()`
         starts a scan) is Clear Cache's and is covered by that item's
         check (3).
    - In the existing `### App: the manual GUI checks for Move Rejected to
      Trash are still open` item, the now-false clause "the zero-reject,
      no-folder and mid-scan cases each write their message to `#status`"
      becomes "the zero-reject and no-folder cases each write their message
      to `#status`" (the mid-scan case now waits; the new item owns it).
      Nothing else in that item changes.
    - `mise run ci` passes (markdown formatting and lychee).
  - Implementation approach (as far as it is known; omit if unknown):
    - Docs-only; no code changes. Place the item next to the other two
      `wait-for-scan` items (after `### App: `folders.init`'s
      `renameAllowed` parameter is now always `() => true``, before the
      next item) so the three read together.
    - Do not copy Clear Cache checks; one sentence pointing at the existing
      item's heading is enough.
    - Note in the Background that the trash confirmation is now an in-window
      HTML dialog (`trash-rejected-from-tree`, #529), so the archived plan's
      "confirm dialog close refocuses the window" race no longer applies to
      trash, only to Clear Cache's native confirm.

## Trade-offs and risks

- **New item (chosen) vs extending `### App: the manual GUI checks for Move
  Rejected to Trash are still open`.** That item is macOS-centric, about the
  `trash` crate's "Put Back" behavior and the confirmation's wording, and
  covers only trash; the wait-for-scan checks span trash and both renames, so
  a new item that references both existing items avoids burying rename
  checks under a trash heading.
- **Check (6) drifted from the request's wording.** The request said "the
  confirm dialog closing refocuses the window without the backend refusing";
  that race exists only for a native dialog, and the trash confirmation is
  now in-window, so the check is reworded with the refocus case pointed at
  Clear Cache's item.

## Progress

- 2026-09-28: Step 1 landed: added the `### App: the wait-for-scan manual
  checks for Move Rejected to Trash and Rename are still open` item to
  `todo.md`, updated the now-false clause in the existing trash item
  (commit `27c620a`, `docs(todo): record the wait-for-scan manual checks`).
  PR: not yet opened.
