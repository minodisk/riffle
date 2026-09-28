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

# Windows GUI check 2026-09-28: reflect the results in todo.md

## Purpose

On 2026-09-28 the user ran a batch of the open manual GUI checks on a real
Windows 11 machine (origin/main around e0b8d13, including PR #509 and #512,
`mise run tauri:dev`, test folders `D:\Photos\tests\2026-09-28-gui-check*`).
`todo.md` still lists all of them as open. This work closes what passed,
narrows the OS-split items to the OSes still unchecked, records the one
attempt that did not run (Clear Cache), and adds the Windows checks that the
recently merged and still in-flight work will need, plus two new issues seen
during the run. After it, `todo.md` again lists only unfinished work.

Decisions agreed with the user: closed TODO lines are **deleted** (repository
convention), not ticked; the measured `scan-progress` numbers go into the PR
body; the Windows checks of the merged work share one `###` item and those of
the in-flight work another.

## Steps

- [x] Step 1: Update `todo.md` with the 2026-09-28 Windows real-device results
  - Done when: every edit below is in `todo.md`; no closed check remains as an
    open `- [ ]`; each OS-split item states in its background that Windows
    passed on 2026-09-28 and lists only the remaining OSes in its TODO; the
    `scan-progress` and partial-read items are gone; no heading is duplicated
    (`rg -n '^### ' todo.md | sort | uniq -d` prints nothing); `mise run ci`
    passes; the PR is a Conventional Commit such as
    `docs(todo): record the 2026-09-28 Windows GUI check results`.
  - Implementation approach:
    - Convention (from `.claude/agents/todo-curator.md`, "Write the deletion
      proposals"): a completed TODO line is **deleted**, not turned into
      `- [x]`; when all of an item's TODOs are closed, delete the whole `###`
      item; for a partial close-out delete only the closed lines and edit the
      background so it does not mislead. The precedent for recording a
      Windows pass is the existing sentence "A later manual run on Windows
      (`mise run tauri:dev`, ...) passed all six checks" in the accelerators
      item. Keep the file's format: `### {area}: {summary}`, a background
      paragraph (or `#### Background`), `#### TODO`, `- [ ]` lines wrapped at
      the existing width with 6-space continuation indent. All text English.
    - Only `todo.md` changes. Do not touch other files.
    - **`### App: the real-device checks for the File menu accelerators are still open`**:
      - Delete the `- [ ] On Windows (`mise run tauri:dev`), verify: one
        `Ctrl+Z` press ...` line.
      - Delete the `- [ ] On the dev machine, verify Select All: ...` line.
      - Append to the background paragraph starting "Since
        `undo-redo-keymap`" a sentence recording the 2026-09-28 Windows run:
        `Ctrl+Z` / `Ctrl+Shift+Z` each fire exactly once, the Edit menu shows
        the accelerators, rebinding `undo` updates the menu in place and
        kills the old key with no `menu item undo not found` in the log, and
        both items work by mouse; Select All passed the same run (all five
        checks of the deleted line). Replace "The new keys have not been run
        by hand on any platform yet." with a statement that macOS is still
        unrun. Add the PowerToys note as a one-sentence aside: `Ctrl+A` first
        did nothing because of a global PowerToys Keyboard Manager remap (left
        `Ctrl+A` → `Home`), not the app; the tell-tale is a keydown sequence of
        `Unidentified` then `Home`.
      - Keep the macOS lines and the Linux `Some`/`None` line. Add Select All
        (`Cmd+A`, as checked on Windows) to the macOS Undo/Redo line or as a
        separate macOS line, so the macOS half of the deleted dev-machine
        check is not lost.
    - **`### App: the settings window's Copy buttons are unverified across webviews`**:
      edit the background to say the Windows (WebView2) run on 2026-09-28
      passed for the MCP tab's Copy button and the tree's `Copy Path` /
      `Copy Folder Name`; change the TODO lines to name only Linux
      (WebKitGTK) and macOS. Keep the clipboard-manager fallback sentence.
    - **`### App: burst grouping's manual checks are still open`**: the Sony
      and Leica band / badge TODO is half done. Rewrite it to a Leica-only
      line and record in the background that the Sony half passed on Windows
      on 2026-09-28 with an ILCE-7M5 folder (band, `position/count` badge and
      its tracking, gap fill, distinguishable from `.cell.current` and
      `.cell.failed`; `.failed` was produced with an ARW truncated to its
      first 2 KB). Do not change the existing `[x]` lines. PR #510
      (`position/count` on every member) is what this run checked, so no
      separate burst-badge TODO is needed.
    - **`### App: the manual GUI check of `scan-progress` `ready` is outstanding`**:
      delete the whole item (both checks passed; the numbers go in the PR
      body).
    - **`### App: a file picked up mid-copy may be scanned from a partial read`**:
      delete the whole item (100 ARWs copied into an open 25-file folder, 19
      rescans, 116 extractions with 16 mid-copy re-reads, 0 errors, nothing
      visibly wrong; no guard needed). The scroll jump-back and flicker seen
      during it belong to the strip-scroll TODO added below.
    - **`### App: hand-check the sequence-run output-folder reveal on Windows`**:
      delete the whole item; (1)–(3) passed, including a mid-read cancel on a
      500-JPEG folder (`canceled, 0 of 500 written`, Explorer did not open).
      PR #512 also passed; it has no todo item.
    - **`### App: the manual GUI checks for Move Rejected to Trash are still open`**:
      replace the Windows-and-Linux line with a Linux-only line; add to the
      background that the Windows run on 2026-09-28 passed (confirmation with
      count and button labels, Cancel changes nothing, confirming moves the
      RAW, `.xmp` and `.ARW.dop` to the Recycle Bin with the right original
      location, the strip moves to the next file, restoring all three shows
      the file still as a reject, and a message appears for 0 rejects). Keep
      the macOS line; if its mid-scan wording is stale after `wait-for-scan`
      (a mid-scan press is now held, see `crates/app/ui/src/idle.ts`), reword
      only that part.
    - **`### App: the Clear Cache button's manual GUI verification is still open`**:
      keep the TODO (1)–(8) as is; append to the background that it was
      attempted on Windows on 2026-09-28 with
      `mise run tauri:release:devtools` but not carried out, because the
      process was killed for low memory on the PC before the checks ran.
    - PR #509: no todo item exists; nothing to close.
    - **Add new items** under `## Cross-cutting / other`, near the existing
      tree / rename items. `rg` for each feature name first to avoid a
      duplicate heading:
      1. `### App: Windows real-device check of the merged folder-tree and scan-wait work`
         Background: merged since the 2026-09-28 Windows run's base:
         `tree-live-watch` (#517, #520, #523), `trash-rejected-from-tree`
         (#518, #521, #526, #529, #530), `wait-for-scan` (#519, #522, #525).
         None run by hand. TODOs: (a) tree watch: with a folder expanded,
         create / delete / rename a subfolder in Explorer and confirm the tree
         follows; confirm renaming a folder from the app still works on
         Windows with the watch on; (b) tree trash: right-click a folder, a
         multi-selection and one with subfolders → Move Rejected to Trash
         shows per-folder counts and the space freed, Cancel changes nothing,
         confirming moves the rejects of every listed folder; (c) scan-wait:
         press Move Rejected to Trash and a rename while a large folder scans,
         confirm the status line says what is waiting and each runs when the
         scan ends (cross-reference the Clear Cache item rather than
         duplicating it).
      2. `### App: Windows real-device check of the in-flight strip-scroll, resume-selection and undo-trash work`
         Background: not on `main` yet: strip-scroll (keep the strip's scroll
         position across a rescan without flicker; the jump-back and flicker
         were seen on Windows on 2026-09-28 while copying 100 ARWs into an
         open folder), resume-selection (landing via resume left the first file
         in the selection, showing `2 selected`), and undo-trash-rejected (Undo
         of a Move Rejected to Trash). TODOs, each phrased "once ... has
         merged": (a) repeat the 100-ARW copy and confirm the scroll position
         holds without flicker; (b) reopen a folder with a remembered file and
         confirm only that file is selected; (c) trash rejects, then Undo, and
         confirm the files come back with their judgment.
      3. `### App: a large folder gives no visible loading feedback beyond the status line`
         Background: opening a 500-JPEG folder, the only sign of progress is
         the small bottom-left status text (`JPEG folder: view only`,
         `scanning 223 / 500`), so it is unclear whether loading started.
         TODO: consider a progress bar in the strip or main view during the
         scan and/or a loading indicator in cells with no thumbnail yet;
         decide and implement.
      4. `### App: every main-window focus runs resync() and holds scanRunning until faces-done`
         Background: the `tauri://focus` listener in
         `crates/app/ui/src/main.ts` calls `resync()` on every focus; even
         with 0 files to process, `scanRunning` stays true until `faces-done`.
         `wait-for-scan` removes the refusals this caused, but the rescan
         itself is still unthrottled. TODO: throttle the focus rescan, and
         clear `scanRunning` immediately when the scan has 0 files to process.
    - Verify: `rg -n '^### ' todo.md | sort | uniq -d` is empty;
      `rg -n '2026-09-28' todo.md` shows every touched item; `mise run ci`.

## Progress

- (none yet)
