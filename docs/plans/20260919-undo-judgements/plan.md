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

# Undo for judgements

## Purpose

A mistaken star, flag or colour label can only be fixed today by navigating
back to the file and re-keying it (`todo.md`, "App: no undo for judgements").
Once auto-advance lands the mistake is already off screen. `Edit > Undo`
(`CmdOrCtrl+Z`) reverts the most recent judgement, returns to that file, and
writes the restored state to the sidecar through the normal `set_rating`
path, so the index and the sidecar writer thread see it like any other
judgement.

Findings that shape the plan (2026-09-19):

- Tauri 2.11.5's `Menu::default` already places `PredefinedMenuItem::undo`
  (Cmd+Z) and `::redo` at positions 0 and 1 of the `Edit` submenu on every
  platform. A custom `Undo` with the same accelerator cannot sit next to it;
  the predefined pair has to be removed and replaced.
- The keymap only binds plain keys and `ctrl+alt+`; `keys.ts`'s
  `isUnboundModifier` returns early for Meta and lone Ctrl, so `CmdOrCtrl+Z`
  cannot be a rebindable keymap action. Undo is therefore a menu item whose
  click is forwarded to the frontend as an event, exactly like
  `File > Open in DxO PhotoLab` (`open-in-photolab` in
  `crates/app/src/main.rs::app_menu`).
- `judge` (`crates/app/ui/src/main.ts`) already computes the previous
  `(rating, pick, label)` of the file before applying the new one, and
  reverts on invoke failure. `openDirectory` clears the per-folder maps and
  the `sidecar-format` listener reopens the folder through it, so a history
  cleared in `openDirectory` is cleared on both a folder change and a format
  switch.
- `refilter(anchor)` returns early when the visible list is unchanged, before
  it touches `index`, so navigating to the undone file must be explicit.

Redo (`Shift+CmdOrCtrl+Z`) is out of scope: it needs a second stack and rules
for invalidating it on the next judgement, which is not trivial, and nothing
in the todo asks for it. The predefined Redo item is removed with Undo so the
Edit menu does not advertise a redo that does nothing.

## Steps

- [x] Step 1: `Edit > Undo` reverts the last judgement and returns to its file
  - Done when:
    - The `Edit` submenu shows a custom `Undo` item with the `CmdOrCtrl+Z`
      accelerator in place of the predefined Undo and Redo items, on every
      platform (`crates/app/src/main.rs::app_menu`). Clicking it or pressing
      the accelerator emits an `undo` event to the main window.
    - In the main window, an undo pops the most recent history entry, restores
      that file's previous rating, pick flag and colour label through the same
      local-apply + `set_rating` invoke path `judge` uses (so `touched`, the
      strip cell, the meta pane, the filter and the sidecar writer all see it),
      and makes that file current (strip and preview), including when it is
      not the file currently shown. Repeated presses walk further back, one
      judgement per press, across files.
    - The history is per open folder: `openDirectory` clears it (this also
      covers the sidecar-format switch, which reopens the folder), and its
      depth is bounded (e.g. 100 entries, oldest dropped). An undo itself
      pushes nothing (no redo). Undo with an empty history does nothing.
    - A judgement whose `set_rating` invoke fails (and is reverted by the
      existing catch) is removed from the history, so an undo does not "revert"
      a judgement that never happened.
    - When the undone file is hidden by the active filter after restoring its
      state, the filter and the current file are left alone and the status
      line says so (e.g. `Undid <name> (hidden by the filter)`). Decided by
      the user on 2026-09-19.
    - `README.md`: the Features list gets an Undo bullet (menu item,
      accelerator, what is restored, per-folder history, no redo) and the Keys
      table gets a `CmdOrCtrl+Z` row; `todo.md`'s "App: no undo for
      judgements" section is removed.
    - `docs/agents/tauri-app.md`'s "App items go into the default menu's own
      submenus" item notes that `Edit` ships a predefined Undo/Redo pair which
      is replaced, not added to.
    - `mise run ci` passes. Any pure history logic split out of `main.ts` has a
      Vitest test next to it (`crates/app/ui/src/*.test.ts`, as
      `exif.test.ts`); there is no Rust logic beyond menu wiring, so no Rust
      test is expected unless one becomes possible.
    - Manual confirmation by the user (GUI automation does not work on this
      Mac; list these in the PR): star file A, reject file B, label C, press
      Cmd+Z three times and watch C, B, A each return to their previous state
      with the strip following; check the sidecars carry the restored values
      after the writer's debounce; open another folder and confirm Cmd+Z does
      nothing; switch the sidecar format and confirm the same.
  - Implementation approach:
    - Rust (`crates/app/src/main.rs::app_menu`): add `UNDO_ID`, look up the
      `Edit` submenu with the existing `submenu(&menu, "Edit")`, remove the two
      predefined items (by position, or more robustly by matching the
      predefined Undo / Redo items) and `insert(&undo, 0)`. In `on_event`,
      `app.emit("undo", ())` (or `emit_to("main", ...)`) mirroring
      `PHOTOLAB_ID`. Do not add an `Undo`/`Redo` pair for a text field: neither
      window has editable content.
    - Frontend (`crates/app/ui/src/main.ts`): a history array of
      `{ path, rating, pick, label }` (the previous state) pushed in `judge`
      right where `previous` / `previousPick` / `previousLabel` are computed,
      after the idempotence early return. In the invoke `.catch`, drop that
      entry by identity (later judgements may have been pushed since). Listen
      to `undo` with `window.__TAURI__.event.listen` next to the
      `open-in-photolab` listener; the handler guards with `openDir !== null`
      and `allFiles` membership (the listener outlives every folder, see the
      guide's "one token per folder" item). Restore with `applyRating`,
      `touched.add`, `renderMeta`, `refilter(path)`, then set `index` from
      `fileIndex.get(path)` and call `show()` when it changed; invoke
      `set_rating` with the same `labelKnown` expression `judge` uses. Consider
      factoring the "apply and invoke" body of `judge` into a helper taking the
      target `path` and the new triple so undo and keypress share one code
      path rather than duplicating the invoke.
    - If the pure stack (push with cap, pop, clear, remove-by-entry) is worth a
      unit test, put it in a small module (e.g. `crates/app/ui/src/undo.ts`,
      imported with the `.js` extension) with a Vitest test; otherwise keep it
      inline. Either is acceptable; do not build more than that.
    - Keymap (`crates/app/src/shortcuts.rs`, `keys.ts`, the settings window)
      stays untouched: undo is not rebindable.
    - README wording: file it under Features as a menu action (it is not a
      culling key), and add the `CmdOrCtrl+Z` row to the Keys table with a
      note that it is the `Edit > Undo` accelerator and not rebindable.

## Trade-offs and risks

- **Undo target hidden by the active filter.** Decided: restore the state,
  leave the filter and the current file alone, and report it in the status
  line, so the user's filter setup is never discarded.
- **Menu item vs keymap action.** The keymap cannot bind Cmd, so a keymap
  action would need either a new modifier rule or a Ctrl+Alt default; both
  diverge from the platform convention for undo. The menu item also means the
  accelerator fires while the settings window is focused (the menu is
  app-wide); the main window still undoes in that case. Accepted.
- **Replacing the predefined Undo/Redo.** The native items only ever acted on
  editable content, which neither window has, so nothing is lost. Removing by
  position assumes Tauri keeps Undo/Redo at 0 and 1; matching the predefined
  items is more robust to a Tauri upgrade at the cost of a few lines. The
  implementer picks; either way the guide gets the note.
- **History semantics under sidecar-format switch.** Clearing on the reopen is
  the safe choice: a `.dop` pick cannot be restored under XMP. It also means a
  switch loses the history even for entries that would still be valid.
- **Redo** is explicitly out of scope; the Redo item is removed rather than
  left as a dead native item.
- **`set_rating` under XMP drops picks** (`pick && format == Dop`), so an undo
  under XMP that restores `pick: true` is silently clamped by the backend, as
  any keypress would be. Not a new behaviour.

## Progress

- 2026-09-19: Step 1 done (Edit > Undo, undo.ts history)
