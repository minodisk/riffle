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

# Remove the folder-targeted items from the File menu

## Purpose

Every item in the native File menu is global (open, reload, settings, quit)
except two that act on one folder: `Move Rejected in This Folder to Trash…`
and `Sequence JPEG Timestamps…`. Users read the menu as a set of app-wide
actions and are confused by the two that are not. Both actions are already
offered where a folder is the obvious target, the folder tree's right-click
menu (`Move Rejected to Trash…`, `Move Rejected to Trash, Including
Subfolders…`, `Sequence JPEG Timestamps…`), so the File menu entries go, along
with the code only they used. While the tree pane is hidden (F7) the two
actions are unavailable; that is accepted by the user, and no replacement
entry point is added.

The concurrent plan `20260928-trash-rejected-from-tree` is not edited. This
change keeps to the hunks it does not touch. Line numbers below were taken
before that plan's later steps landed on `main`; locate code by name, not by
line.

## Steps

- [x] Step 1: Remove the two File menu items, their event plumbing, the orphaned code and icon, and update the docs
  - Done when:
    - `crates/app/src/main.rs` no longer defines `TRASH_REJECTED_ID` /
      `SEQUENCE_TIMESTAMPS_ID`, builds `trash_rejected` /
      `sequence_timestamps` (both `cfg` variants), lists them in
      `file.prepend_items`, or emits `trash-rejected` /
      `sequence-timestamps` in `on_event`. `File` shows `Open Folder…`,
      `Reload Folder`, separator on every platform.
    - `crates/app/ui/src/main.ts` no longer has the open-folder wrapper of
      the trash action (File-menu only), `sequenceTimestamps()` (the
      `pick_folder` path), or the two `event.listen(...)` calls; the tree
      path (`trashRejectedIn` or its successor, `sequenceTimestampsOf`,
      `previewSequence` and everything the tree's context-menu `switch`
      calls) is unchanged. Comments that name the File menu path describe
      the tree path instead.
    - `crates/app/icons/menu/trash.png` is deleted if nothing else uses it,
      and `"trash"` is dropped from the symbol list in
      `tools/macos/export-menu-icons.swift`.
    - Module/header comments no longer say `File > ...`:
      `crates/app/src/sequence.rs`, `crates/app/ui/src/sequence.ts`,
      `crates/app/ui/src/trash.ts`.
    - `SequenceFlow.picked` no longer takes `null`: the null branch and the
      test `a dismissed picker ends the flow` are removed (decided: Option A
      below). The `picking` phase keeps its name.
    - Docs: `docs/usage.md` (the tree section and the two feature entries,
      rewritten to lead with the tree's right-click items and to drop "the
      File menu item", "as with the File menu item", "pick the export folder
      here, or ... to skip the picker"), `README.md` and `README.ja.md`
      (kept in sync), `CLAUDE.md` (the `src/sequence.rs` description: drop
      "`File > `"), `docs/agents/tauri-app.md` (the menu item list; the
      "Menu icons" section's item count and list). Neither README mentions a
      `File >` entry for either action.
    - `grep -rn "trash-rejected\|sequence-timestamps\|TRASH_REJECTED_ID\|SEQUENCE_TIMESTAMPS_ID\|trash.png" crates tools docs/usage.md README*.md CLAUDE.md`
      returns nothing (archived plans and `todo.md` are historical and left
      alone).
    - `File > Open Folder…` still opens the picker (`pick_folder` stays).
    - `mise run ci` passes.
  - Implementation approach:
    - Rust: pure deletion. Keep `Image`, `IconMenuItem`, `MenuItem` imports
      still used by the other items. `refresh()` does not reference the two
      ids.
    - Frontend: `pick_folder` remains registered and used by Open Folder, so
      do not remove the command. `sequenceFlow.busy` / `isOpen` users are
      unaffected.
    - Do not touch the context-menu `switch`, `context.ts`, `folders.ts`,
      `tree.ts`, or `trash.ts` beyond its header comment, to stay clear of
      the other plan.
    - Docs edits overlapping the other plan's files are kept to the sentences
      that name the File menu so a rebase is trivial.
    - Commit type: `feat(app)` (a user-visible removal).

## Trade-offs and risks

- **`SequenceFlow.picked(null)` after the picker path is gone.** Option A
  (chosen): narrow to `picked(dir: string)` and drop the dead null branch
  and its test. Option B (not chosen): leave `SequenceFlow` untouched as a
  stable reference for the other plan; costs a dead branch plus its test.
- **No entry point while the tree pane is hidden (F7).** Accepted by the
  user.
- **Merge conflicts with `trash-rejected-from-tree`.** Same files
  (`main.ts`, `docs/usage.md`, `README*.md`), different code hunks; the
  prose for Move Rejected to Trash is where both edit the same paragraphs.
  Whichever lands second rebases.
- **`docs/agents/tauri-app.md` count.** The "Menu icons" section's item
  count drops by one; update it rather than leaving a stale figure.

## Progress

- (none yet)
