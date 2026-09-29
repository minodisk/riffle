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

# Record the pending manual GUI checks for trash, undo and redo in todo.md

## Purpose

`trash-rejected-from-tree` (#518, #521, #526, #529; wrap-up #530) and
`undo-trash-rejected` (#540, #547, #549, #551; wrap-up #554) were implemented
from agent sessions that cannot drive the app, so their manual GUI checks
were written down only in the archived `learnings.md` files and in the PR
bodies. `todo.md` holds three related items, but they miss most of those
checks and some of their background text now contradicts the code (the File
menu's trash item was removed by `file-menu-folder-items`, #537; the single
`trash_rejected` command with a native dialog became
`trash_rejected_preview` / `trash_rejected_run` and the in-window
`#trash-dialog`; the undo work is on `main`). After this change every
pending check is a checkbox in `todo.md`, once, with its source named, so the
user can run them on a real device and tick them off.

Sources:

- `docs/plans/_archived/20260928-undo-trash-rejected/learnings.md`
  (Step 2 macOS check; Step 3 and Step 4 "Manual GUI checks on Windows").
- `docs/plans/_archived/20260928-trash-rejected-from-tree/learnings.md`
  and the bodies of #518, #521, #526, #529 (the tree items, the tree
  multi-selection, the confirmation dialog).

## Steps

- [x] Step 1: Rewrite the three related `todo.md` items so every pending check appears once, with the stale background fixed
  - Done when:
    - `todo.md` contains every check below as a checkbox, exactly once,
      each item naming its source (the archived `learnings.md` path in
      backticks and the PR numbers).
    - No background text in the touched items contradicts the code: no
      `File > Move Rejected to Trash…`, no `trash_rejected` /
      `trash_context` command names, no "native confirmation dialog", no
      "not on `main` yet".
    - `mise run ci` passes (lychee included).
    - No other `todo.md` item is touched.
  - Implementation approach:
    - Keep the existing item format: `### App: …` heading, a background
      paragraph (or `#### Background`), then `#### TODO` with `- [ ]`
      checkboxes. Refer to plan files in backticks, as the surrounding
      items do (the `docs/plans/_archived/...` paths are not links, so
      lychee does not resolve them).
    - **Item A, `### App: the manual GUI checks for Move Rejected to Trash are still open`**
      (todo.md line 109 at planning time). Fix the background: the action
      is now only the folder tree's right-click items
      (`Move Rejected to Trash…`, `…, Including Subfolders…`, and the
      `Move Rejected in N Folders…` pair for a multi-selection; the File
      menu item went with #537, and with it the template-icon sentence);
      the confirmation is the in-window `#trash-dialog`
      (`crates/app/ui/index.html`, `crates/app/ui/src/trash.ts`) driven by
      `trash_rejected_preview` / `trash_rejected_run`
      (`crates/app/src/commands.rs`); on macOS the mover is
      `trash::trash_file` (`crates/app/src/trash.rs`, #547), which calls
      `NSFileManager.trashItemAtURL` directly rather than the crate's
      `DeleteMethod::NsFileManager`, so the "Put Back" question
      (trash-rs#14) still stands. Keep the note that a Windows run on
      2026-09-28 passed for the earlier native-dialog flow, marked as
      predating the dialog. Rewrite the macOS checkbox to the tree items
      and dialog (count / singular, `Move to Trash` / `Cancel`, Cancel
      leaves the folder untouched, confirming moves the RAW plus `.xmp`
      and `.ARW.dop`, the strip shows the next passing file, the
      zero-reject case writes its message to `#status` with no dialog, a
      press during a scan is held by `whenIdle`, "Put Back" or drag-out
      restores the file with its judgment); drop the "no-folder case",
      which no longer exists (the item is always invoked on a folder).
      Keep the Linux checkbox. Add one checkbox from undo Step 2
      (learnings, `undo-trash-rejected` Step 2, #547): on macOS, trash the
      rejects of a folder, then `Edit > Undo`; the files come back with the
      reject flag, and a file with the same name placed at the original
      location beforehand is reported ("already exists at the original
      location") and not overwritten.
    - **Item B, `### App: Windows real-device check of the merged folder-tree, scan-wait and strip-scroll work`**
      (line 413). Replace the single "Tree trash on Windows" checkbox with
      the checks from #521, #526 and #529 (source:
      `docs/plans/_archived/20260928-trash-rejected-from-tree/learnings.md`
      Steps 2–4 and the PR bodies), each as its own checkbox so they can be
      ticked separately:
      1. Right-click items (#521): a non-root folder offers
         `Move Rejected to Trash…` and
         `Move Rejected to Trash, Including Subfolders…`; the home root
         and a volume root offer only the first; the recursive item trashes
         the rejects of the folder and every visible subfolder (dot and
         hidden folders skipped); when the open folder is a target, the
         strip refreshes; when it is not, only the status line and the error
         list change.
      2. Multi-selection (#526): `Ctrl+click` toggles a folder in and out
         of the selection (the open folder can be toggled off, and the
         selection can go empty), `Shift+click` selects the range from the
         anchor without selecting the rows' text, a plain click selects
         only the clicked folder, and opening a folder any other way (Enter,
         drop, `File > Open Folder…`, the reopen at launch) resets the
         selection to it; selected rows draw `.selected` distinct from
         `.current` / `.cursor`; the container has `aria-multiselectable`
         and each row `aria-selected` following the selection; collapsing
         a parent or a `tree-changed` re-list drops folders no longer
         drawn; right-clicking a selected folder acts on the whole
         selection, and the menu offers only
         `Move Rejected in N Folders to Trash…` and its
         `Including Subfolders` twin (the latter hidden when any selected
         folder is a root); right-clicking an unselected folder selects it
         alone first.
      3. Confirmation dialog (#529): `#trash-dialog` lists one row per
         folder with its reject count (the verbatim `\\?\` prefix
         stripped), hides zero-reject folders behind a summary line, shows
         the total count and the space freed, and opens with
         `Move to Trash` focused so Enter runs it; Escape and `Cancel`
         close it with nothing moved; a folder that cannot be read (for
         example one removed after the right-click, or a sidecar that does
         not parse) appears in the failure list with its error and is
         excluded from the run, and when every folder has zero rejects but
         one failed the dialog still opens with `Move to Trash` disabled;
         a folder with zero rejects and no failure writes
         `No rejected files in …` to the status line with no dialog; the
         dialog, the sequence dialog and the settings modal never open on
         top of each other (`Cmd/Ctrl+,` and the tree's
         `Sequence JPEG Timestamps…` do nothing while it is up); confirming
         shows `Moved N files to the Trash` and the failures, if any, in
         the error list.
      Keep the other three checkboxes (tree watch, scan-wait, strip
      scroll) as they are. The obsolete "relabeled File menu item" check
      from #518 is not added: the item was removed by #537, and a sentence
      in the background says so.
    - **Item C, `### App: Windows real-device check of the in-flight resume-selection and undo-trash work`**
      (line 442). Both features are merged now, and its resume checkbox is
      already covered by
      `### App: the manual GUI checks for the resume landing's selection are still open`
      (#553). Turn this item into the undo / redo item: rename the heading
      to
      `### App: the manual GUI checks for Undo and Redo of Move Rejected to Trash are still open`,
      rewrite the background (merged as #540, #547, #549, #551; a run is one
      undo unit, `Edit > Undo` / `Ctrl+Z` restores through
      `trash_rejected_undo`, `Edit > Redo` / `Ctrl+Shift+Z` re-trashes
      through `trash_rejected_redo` without the dialog; the backend side
      was verified against the real Recycle Bin with throwaway tests, the
      GUI side not at all; files `crates/app/src/trash.rs`,
      `crates/app/src/commands.rs`, `crates/app/ui/src/undo.ts`,
      `crates/app/ui/src/trash.ts`, `crates/app/ui/src/main.ts`; source
      `docs/plans/_archived/20260928-undo-trash-rejected/learnings.md`
      Steps 3 and 4), drop the resume checkbox with a one-line pointer to
      the #553 item, and replace the undo checkbox with, on Windows:
      1. Trash rejects in the open folder, `Ctrl+Z`: the files come back,
         the strip shows them again with the reject flag (read from the
         sidecars), the status line says `Restored N files from the Trash`,
         and the folder tree's count follows on its next re-list.
      2. The same from the tree for a folder that is not open: only the
         status line changes.
      3. A run over several selected folders undone by one `Ctrl+Z`.
      4. A conflict: copy a trashed RAW back by hand before the undo; the
         status line says `…, N failed` and the error list shows
         `could not restore from the Trash: already exists at the original
         location`, and the RAW's sidecars stay in the Trash, reported as
         `left in the Trash: its RAW could not be restored`.
      5. An emptied Recycle Bin: every file is reported
         `not in the Trash (emptied or restored by hand)` and the entry
         leaves the undo history.
      6. After a restore, the index picks the reject judgments back up on
         the `resync` (a following `Ctrl+Z` undoes the judgment batch
         beneath, not the trash run again).
      7. `Ctrl+Z` (files back), `Ctrl+Shift+Z` (moved to the Trash again
         with no dialog, the strip refreshed, status
         `Moved N files to the Trash`), `Ctrl+Z` again (back again).
      8. The same undo / redo / undo from the tree with no folder open.
      9. A judgment made after the undo clears the redo: `Ctrl+Shift+Z`
         does nothing.
      The macOS undo check from Step 2 goes in Item A, not here.
    - **Item D, `### App: the wait-for-scan manual checks for Move Rejected to Trash and Rename are still open`**
      (line 1047). Only fix the stale text: both mentions of
      `File > Move Rejected to Trash…` (background and the first checkbox)
      become the folder tree's `Move Rejected to Trash…` item, with a
      clause that the File menu item was removed by `file-menu-folder-items`
      (#537). Do not add checks here.
    - Do not touch `### App: renaming a folder after a Move Rejected to Trash run …`
      or the `treewatch` flaky-test item; they are code todos, not checks.
    - Verify before opening the PR: `rg -n 'File > Move Rejected|trash_context|trash_rejected\b|native confirmation|Not on .main. yet' todo.md`
      returns nothing, each check of the two learnings files maps to one
      checkbox, and `mise run ci` passes.

## Trade-offs and risks

- **Where the macOS undo check lives (decided: Item A).** Item A is the
  macOS / Linux trash item and Item C is the undo / redo item (Windows
  checks); putting the single macOS undo check in Item A preserves the
  platform split of the existing items.
- **Repurposing Item C vs. adding a new item (decided: rename).** The plan
  renames Item C and drops its resume checkbox (already recorded by #553).
  Keeping Item C's resume checkbox would leave a duplicate.
- **Granularity of the dialog / tree checks in Item B (decided: three
  checkboxes).** One checkbox per bullet would give about 20 boxes; the
  implementer may split further if a checkbox exceeds roughly ten lines.
- **The "relabeled File menu item" check (#518) is intentionally not
  recorded**: the item no longer exists; a sentence in Item B's background
  says so.
- **Item A's Windows-passed note** describes the old native-dialog flow.
  The plan keeps it, marked as predating #529, rather than deleting it,
  since it is a record of a real run.

## Progress

- (none yet)
