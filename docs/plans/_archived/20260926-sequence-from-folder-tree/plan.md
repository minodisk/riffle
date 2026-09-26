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

# Sequence JPEG Timestamps from the folder tree's right-click menu

## Purpose

`File > Sequence JPEG Timestamps…` always opens the OS folder picker, even
when the export folder is already visible in the folder tree. Adding a
`Sequence JPEG Timestamps…` item to the tree's right-click menu, next to the
per-platform reveal item, lets the user start the same preview on the
right-clicked folder without the picker. The File menu path is unchanged.

## Steps

- [x] Step 1: Add the `Sequence JPEG Timestamps…` item to the folder tree's right-click menu and route it into the existing sequence flow
  - Done when:
    - Right-clicking a folder in the tree shows `Sequence JPEG Timestamps…`
      below the reveal item; choosing it opens the Sequence dialog's preview
      for that folder with no folder picker, and Run / Cancel / progress
      behave as they do from the File menu.
    - `File > Sequence JPEG Timestamps…` still opens the picker and works as
      before.
    - The right-click path applies the same gates as the File menu path (no
      start while the format dialog or settings are open, or while a
      sequencing is already under way).
    - `context.test.ts` covers the new folder menu item; `sequence.test.ts`
      covers the preselected-folder path if `SequenceFlow` gains any new
      code (see approach).
    - `README.md`, `README.ja.md` and `docs/usage.md` mention the right-click
      entry point; `mise run ci` passes.
  - Implementation approach:
    - The tree menu is the HTML menu in `main.ts` (`showMenu`), not a native
      Tauri menu; no Rust change (`crates/app/src/folders.rs`,
      `crates/app/src/sequence.rs`, `main.rs`) is needed.
    - `crates/app/ui/src/context.ts`: extend `folderMenuGroups(revealLabel)`
      with a second item `{ action: "sequenceTimestamps", label: "Sequence JPEG Timestamps…", shortcut: "", checked: undefined }`.
      Use the same `…` (U+2026) as the File menu label. Update the
      `folderMenuGroups` test in `crates/app/ui/src/context.test.ts` and the
      comment above the function ("one item"). Whether it is a second item in
      the same group or its own group (drawn with an `hr`) is the
      implementer's call; see Trade-offs.
    - `crates/app/ui/src/main.ts`, `folders.init` callback (around line
      2018): switch on the `action` handed to `run` instead of ignoring it:
      `revealFolder` keeps the `reveal_folder` invoke, `sequenceTimestamps`
      calls the new preselected-folder entry point with `path`.
    - `crates/app/ui/src/main.ts`, `sequenceTimestamps()` (line 590): split
      out the part after the picker into a helper such as
      `previewSequence(dir: string)` that runs `sequence_preview`,
      `sequenceFlow.previewed()`, `showSequencePreview`, and the shared
      `.catch` (`sequenceFlow.fail()`, `setStatus`). The File menu path
      stays: gate, `sequenceFlow.start()`, `pick_folder`,
      `sequenceFlow.picked(dir)`, then the helper. The tree path: the same
      gate, `sequenceFlow.start()`, `sequenceFlow.picked(path)`, then the
      helper. `SequenceFlow` needs no new method for this: `start()` then
      `picked(dir)` with a non-null dir already reaches `previewing`, and
      `sequence.test.ts` already walks that transition. Keep the gate
      condition in one place (a small shared function or the same expression
      in both entry points) so the two paths cannot drift.
    - `crates/app/ui/src/sequence.ts`: only the header comment ("from picking
      the folder") needs to say the folder may also come from the tree's
      right-click menu; no behavior change unless the Trade-offs option is
      taken.
    - Docs: `README.md` (line 112: `File > Sequence JPEG Timestamps…` on the
      export folder) and `README.ja.md` (line 55) add "or right-click the
      folder in the folder tree" in the same PR; `docs/usage.md` lines 37-41
      no longer say the tree menu has "one item", and lines 178-183 mention
      the right-click entry that skips the picker. `CLAUDE.md` line 61
      ("its flow from the folder picker through the preview") may be left
      as-is or lightly reworded; the layout itself does not change.
    - Verify by hand (the HTML menu is not unit-tested): right-click a folder
      while the settings modal is open and confirm nothing starts; right-click
      while a sequencing dialog is already open and confirm the menu item is a
      no-op; a folder with no JPEGs shows the folder-level error on the status
      line as the File menu path does.

## Trade-offs and risks

- **One group vs. two groups in the folder menu.** Putting the sequence item
  in the same array as the reveal item draws no separator; a second inner
  array draws an `hr` between them, as the strip menu separates unrelated
  groups. Revealing and sequencing are unrelated actions, so a separator
  matches the strip menu's convention, but with only two items it may look
  heavy. Left to the implementer; the test pins whichever is chosen.
- **`SequenceFlow.start()` + `picked(dir)` vs. a new `startWith(dir)`.**
  Reusing the two existing calls needs no new state-machine code or tests and
  keeps `SequenceFlow` as it is. A dedicated `startWith(dir)` that goes
  straight to `previewing` would be a clearer API but is new code for a
  single caller; the plan prefers reuse. If the implementer finds the
  two-call form awkward in `main.ts`, adding `startWith` with a
  `sequence.test.ts` case is acceptable.
- **Gate placement.** If the gate (`!formatDialog.hidden || settings.isOpen`)
  is only kept in `sequenceTimestamps()`, the tree path could start a
  sequencing on top of the settings modal. Both entry points must share it.
- The `folders.init` callback currently receives the action and drops it; a
  `switch` there is the smallest change, but any future third item goes
  through the same switch. No abstraction is needed now.

## Progress

- (2026-09-26) Step 1 complete
