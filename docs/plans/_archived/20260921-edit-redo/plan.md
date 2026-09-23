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

# Edit > Redo

## Purpose

`Edit > Undo` (`CmdOrCtrl+Z`) walks judgments back, but an undo pressed once
too often has no way forward: the user has to re-judge the file by hand.
`Edit > Redo` (`CmdOrCtrl+Shift+Z`) re-applies the most recently undone
judgment, so Undo → Redo → Undo round-trips, and a new judgment after an
undo forgets the redo branch as every editor does.

## Steps

- [x] Step 1: Add `Edit > Redo` (menu item, `redo` event, redo stack, README)
  - Done when:
    - `Edit > Redo` (`CmdOrCtrl+Shift+Z`) re-applies the most recently undone
      judgment and reports `Redid {name}` / `Redid {name} (hidden by the
      filter)`, mirroring Undo's filter behavior (current file moves to the
      redone file unless the filter hides it).
    - Undo → Redo → Undo returns the file to the same state.
    - A new judgment (`judge()`) after an undo clears the redo stack.
    - Files gone from the folder are dropped from both stacks (the trash
      handler's `removeWhere`); `openDirectory` clears both.
    - Undo/Redo remain menu accelerators only (nothing added to
      `shortcuts.rs` or the keymap).
    - README's Undo paragraph and key table mention Redo.
    - `mise run ci` passes; `cargo check` / `cargo clippy` for `crates/app`
      pass on macOS.
  - Implementation approach:
    - `crates/app/src/main.rs`: add `const REDO_ID: &str = "redo";` next to
      `UNDO_ID`. The Edit block already strips both predefined Undo and Redo
      (`for _ in 0..2`), so only the insertion is new: build a `redo` item with
      `Some("CmdOrCtrl+Shift+Z")` and `edit.insert(&redo, 1)` after the Undo
      insert. Give it the same `#[cfg(target_os = "macos")]` `IconMenuItem` /
      `#[cfg(not(...))]` `MenuItem` split as Undo (see
      `docs/agents/tauri-app.md`, "Adding a macOS menu item needs the same cfg
      split as its siblings"). In `on_event`, add
      `if event.id() == REDO_ID { let _ = app.emit("redo", ()); }` after the
      undo branch.
    - macOS icon: add `"arrow.uturn.forward"` to `symbols` in
      `tools/macos/export-menu-icons.swift` and regenerate with
      `swift tools/macos/export-menu-icons.swift`, committing only the new
      `crates/app/icons/menu/arrow.uturn.forward.png` (the existing PNGs must
      stay byte-identical; restore them if the run rewrites them).
    - `crates/app/ui/src/main.ts`:
      - Next to `history`, add `const redoable = new History<Judgment>(100);`
        with a comment explaining it holds the pre-undo state of each undone
        judgment for `Edit > Redo`, and that a new judgment forgets it.
      - `judge()`: after `history.push(entry)`, `redoable.clear()`.
      - `undo()`: the `current` object it already builds is exactly the
        pre-undo state; `redoable.push(current)` before/at the `commit`. Keep
        the existing shape of `undo()` otherwise.
      - `redo()`: mirror `undo()` with the stacks swapped — pop `redoable`,
        bail if `openDir === null` or the path is not in `allFiles`, build the
        file's current state, push it onto `history`, `commit` to the popped
        entry's state with the same `anchor` callback, then the same
        index/status logic with `Redid`. Do not route redo through `judge()`
        (it clears `redoable` and only judges the current file).
      - Consider extracting the shared body (pop from one stack, push current
        onto the other, commit, move, status verb) into one helper that
        `undo` and `redo` call with `(from, to, verb)`; only do so if it stays
        readable and does not change `undo`'s observable behavior.
      - Trash handler (~line 437): apply the same `removeWhere` predicate to
        `redoable`. `openDirectory` (~line 1245): `redoable.clear()` next to
        `history.clear()`.
      - Add `void window.__TAURI__.event.listen("redo", redo);` beside the
        undo listener.
    - A failed sidecar write leaves both stacks untouched: `undo()` passes no
      `onFail` to `commit` today, and `redo()` mirrors that. Do not add stack
      cleanup on failure to either.
    - `crates/app/ui/src/undo.ts` / `undo.test.ts`: no new stack behavior is
      expected. If the implementation does add a method to `History`, cover
      it in `undo.test.ts` in the existing style.
    - README (`/README.md` around the Undo bullet and the `CmdOrCtrl+Z` row):
      add Redo in the same voice (menu accelerator, not rebindable, cleared
      by a new judgment and by opening another folder).

## Trade-offs and risks

- **macOS icon for Redo.** Decided: bundle `arrow.uturn.forward` through the
  export script so Redo matches its sibling Undo visually. Only the new PNG
  should appear in the diff.
- **Failed commit during undo/redo.** Decided: mirror `undo()`'s lack of
  `onFail` in `redo()`, keeping the scope surgical. A stricter variant (remove
  the entry just pushed onto the opposite stack, as `judge()` does) is left
  out on purpose.
- **Sidecar-format switch.** README says the history is cleared when the
  sidecar format changes; the only `history.clear()` is in `openDirectory`,
  so the switch presumably reopens the folder. Clearing `redoable` in the same
  place preserves that; verify during implementation that no other path
  clears `history` (grep found none).
- **Redo of a file whose state changed by other means.** Redo re-applies the
  stored post-judgment state regardless of what a refresh from
  `folder_entries` has since shown; same limitation Undo already has, not
  addressed here.

## Progress

- (2026-09-21) Step 1 complete
