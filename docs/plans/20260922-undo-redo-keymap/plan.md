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

# Undo / redo as keymap actions

## Purpose

On Windows, `Ctrl+Z` / `Ctrl+Shift+Z` do nothing: they exist only as fixed
accelerators on `Edit > Undo` / `Edit > Redo` (`UNDO_ID` / `REDO_ID` in
`crates/app/src/main.rs`, emitting `undo` / `redo` to the frontend), and
WebView2 swallows the key press before the accelerator fires. Clicking the
menu items works (user-verified on Windows 11, v0.2.0). `Ctrl+O` works on
Windows because `open` is a keymap action: the frontend keydown handler runs
it and the menu accelerator merely mirrors the keymap.

This work makes `undo` and `redo` keymap actions in the same shape as `open`
and `photolab`: default keys `Cmd+Z` / `Shift+Cmd+Z` on macOS and `Ctrl+Z` /
`Ctrl+Shift+Z` elsewhere, run by the frontend keydown, rebindable from the
settings window, with the Edit menu accelerators following the keymap.
Afterwards one press undoes exactly once on every platform, and existing
stored `shortcuts` overrides still load.

It also records the 2026-09-22 Windows manual check results in `todo.md`.

## Steps

- [x] Step 1: Make `undo` and `redo` keymap actions and derive the Edit menu accelerators from the keymap
  - Done when:
    - `crates/app/src/shortcuts.rs` has `undo` and `redo` in `DEFAULTS` with
      platform defaults `meta+z` / `shift+meta+z` on macOS and `ctrl+z` /
      `ctrl+shift+z` elsewhere; `meta+z`, `shift+meta+z`, `ctrl+z` and
      `ctrl+shift+z` are no longer in `MACOS_MENU` / `OTHER_MENU`.
    - Pressing the undo / redo key in the main window runs `undo()` /
      `redo()` through the keydown handler's `runAction`, and the shortcuts
      panel lists "Undo" and "Redo" rows that can be rebound.
    - `Edit > Undo` / `Edit > Redo` show the keymap's accelerator
      (`Keymap::accelerator_for("undo"/"redo")`), updated by `refresh` on a
      rebind exactly as `Open Folder…` / `Open in DxO PhotoLab` are; clicking
      the items still emits `undo` / `redo`.
    - Rust tests cover the new defaults per platform, `accelerator_for` for
      the two actions, the defaults not being forbidden, and that a stored
      `shortcuts` value written before this change (no `undo` / `redo` keys,
      and overrides for other actions) loads with `undo` / `redo` on their
      defaults and the other overrides applied.
    - `docs/usage.md`, `docs/agents/tauri-app.md` and `README.md` no longer
      describe `CmdOrCtrl+Z` as a fixed, non-rebindable accelerator.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/app/src/shortcuts.rs`: add `UNDO_DEFAULT` / `REDO_DEFAULT`
      constants next to `OPEN_DEFAULT` / `PHOTOLAB_DEFAULT` (macOS names in
      the keymap's modifier order: `meta+z`, `shift+meta+z`). Insert the two
      actions into `DEFAULTS` in the position the shortcuts panel should show
      them (e.g. after `photolab`, before `focus`); the panel's order is this
      table's order. Remove the four Z combinations from `MACOS_MENU` /
      `OTHER_MENU` and extend the doc comment on `MACOS_MENU` (which today
      names only the two File accelerators as "deliberately absent") to cover
      Undo / Redo. Update the tests `the_defaults_are_the_full_table`,
      `the_menu_defaults_follow_the_platform`,
      `accelerator_for_takes_the_first_convertible_key` and
      `the_menu_defaults_are_not_forbidden`, and add the "old stored
      overrides still load" test using `Keymap::from_overrides` with a
      realistic pre-existing object (e.g. `{"pick": ["q"]}` or the burst
      overrides), asserting `undo` / `redo` keep their defaults.
    - `crates/app/src/main.rs` (`app_menu`): stop hard-coding
      `Some("CmdOrCtrl+Z")` / `Some("CmdOrCtrl+Shift+Z")` on the Undo / Redo
      items (both the macOS `IconMenuItem` and the other `MenuItem` variant;
      keep the `cfg` split). `build` needs the two extra accelerators; either
      widen its parameters or pass `&Keymap` and call `accelerator_for` inside
      — pick one, keep `refresh` and `setup` consistent. In `refresh`, extend
      the patched list to `[(OPEN_FOLDER_ID, "open"), (PHOTOLAB_ID, "photolab"), (UNDO_ID, "undo"), (REDO_ID, "redo")]`;
      the existing lookup iterates every top-level submenu with
      `Submenu::get`, so the Edit items are found without further changes
      (verify by reading, not assuming). Reword the "A fixed accelerator,
      like Settings and Undo" comment on `Reload Folder` (Settings and Reload
      remain fixed). `on_event` is unchanged. Do not add a runtime `set_menu`
      on Windows (it turns the dark menu bar white; see
      `docs/agents/tauri-app.md` "Rebuild the menu on macOS, patch
      accelerators in place elsewhere"). Check also whether `update_keymap`
      in `crates/app/src/commands.rs` decides when to call `refresh` by
      comparing only the `open` / `photolab` accelerators; if so, include
      `undo` / `redo` in that comparison.
    - `crates/app/ui/src/main.ts`: add `case "undo": undo(); break;` and
      `case "redo": redo(); break;` to `runAction` next to `open` /
      `photolab`. Keep the `undo` / `redo` event listeners for the menu
      items. The double-fire guard on macOS is the same one `open` relies on:
      the handler calls `event.preventDefault()` for a handled key, which
      WebKit treats as consuming the key equivalent, so the menu does not
      fire again. Match `open` exactly and do not add a second mechanism;
      whether it holds on a real Mac is the manual check Step 2 adds to
      `todo.md`. Windows and Linux never fire the accelerator for a key the
      webview handled, so no double fire there. Note `undo()` / `redo()` are
      declared before the keydown handler, so no hoisting concern.
    - `crates/app/ui/src/settings.ts`: add `undo: "Undo"` and `redo: "Redo"`
      to `shortcutLabels` (labels are looked up by action; the row order
      comes from the backend table).
    - Frontend tests: `main.ts` is not imported by any test, so the keydown
      dispatch itself has no unit test today; there is nothing to extend for
      the new `case`s. Add a test only if a natural seam exists; do not
      refactor `main.ts` for testability in this step. `keys.test.ts` needs
      no change (`ctrl+shift+z` and `shift+meta+z` follow the existing naming
      rules). `context.ts` (strip right-click menu) does not list undo and is
      out of scope.
    - Docs: in `docs/usage.md` change the two "not rebindable" rows of the
      key table to the rebindable wording, replace `Cmd+Z` in the "refused
      combinations" sentence with another fixed example (`Cmd+,`), and update
      "The two File menu accelerators are the exception" to cover the Edit
      menu's Undo / Redo as well; in `docs/agents/tauri-app.md` update the
      "App items go into the default menu's own submenus" and "Rebuild the
      menu on macOS, patch accelerators in place elsewhere" notes to say the
      Undo / Redo accelerators are keymap-derived and patched by `refresh`.
      `README.md` line 81 stays true as written.
    - Manual verification on Windows if available (`mise run tauri:dev`):
      `Ctrl+Z` undoes once, `Ctrl+Shift+Z` redoes once, the Edit menu shows
      the accelerators, rebinding `undo` in Settings updates the menu label
      and disables the old key. Record the result in `learnings.md`.

- [ ] Step 2: Record the 2026-09-22 Windows manual results in `todo.md`
  - Done when: `todo.md` reflects the user-verified results (Windows 11,
    v0.2.0, 2026-09-22) below, each still-open part stays a `- [ ]` item, and
    a macOS check for the new undo / redo keys is added.
  - Implementation approach:
    - Assumes Step 1 is merged (the macOS check refers to the keymap actions
      it adds). Keep the existing section headings and the
      `#### TODO` / checkbox style; mark finished items `- [x]` with the date
      like the existing `(exiftool 13.59, 2026-09-22)` entry, and split an
      item when only part of it is done.
    - "App: burst grouping's manual checks are still open": the
      `Alt+ArrowUp` / `Alt+ArrowDown` item is done on Windows (keys reach the
      app and step through bursts); macOS stays open. The `Shift+x`
      reject-rest and one-step undo item is done. The strip scrollbar item is
      done on Windows (thin and dark, thumbnail right edge and burst bracket
      not clipped); the macOS "looks unchanged (160px wide)" part stays open.
    - "App: the silent update path is unverified end-to-end": the background
      flow is verified on Windows (0.1.10 -> 0.2.0 downloaded, installed on
      quit, launched as 0.2.0). Still open: the `Check for Updates…` menu
      outcomes (up-to-date / installed / already-installed) and macOS /
      Linux AppImage.
    - "App: `open entries` is called twice per folder open": a plain folder
      open logs one `open entries` (seen at 03:15 and 04:00); the extra call
      appears after scan-done when a follow-up rescan runs (`scan_id=2`
      immediately after the cold scan finished at 04:39:37). Reword the
      description accordingly and point the TODO at the post-scan rescan
      path rather than a redundant frontend call.
    - "App: a scan can be started twice after a cache clear / focus rescan":
      add that it reproduced on 2026-09-22 with `scan_id=3` and `4` starting
      in the same second at 04:39:40.
    - "App: cold first scan on an internal SSD is far slower than the
      extrapolation": add the new data point, a cold first scan after an
      index schema change of 3045 Sony ARW in 25.5s (~8.4ms/file, ~42s
      extrapolated to 5000) against the earlier 16-19ms/file, noting it is
      unknown whether the OS cache was cold. Leave the thread-count TODO.
    - "App: the real-device checks for the File menu accelerators are still
      open": add a `- [ ]` item for macOS: one `Cmd+Z` press undoes exactly
      once and one `Shift+Cmd+Z` redoes exactly once (no double fire from
      keydown + the Edit menu key equivalent), rebinding `undo` / `redo`
      updates the Edit menu accelerator and kills the old key, and both Edit
      items work by mouse. Files: `crates/app/src/main.rs` (`app_menu`),
      `crates/app/src/shortcuts.rs`, `crates/app/ui/src/main.ts`. No
      Windows undo item exists in `todo.md`, so there is nothing to remove.
    - If the Windows manual run in Step 1 verified the new keys, say so in
      the same section (Windows passed) so the macOS item is the only open
      one.

## Trade-offs and risks

- **One PR or two.** Step 2 touches `todo.md` sections unrelated to undo
  (scans, updates, bursts), so it is kept as its own docs PR to keep the feat
  PR reviewable and revertable.
- **`build`'s signature.** Passing four `Option<&str>` keeps the current
  shape but grows awkward; passing `&Keymap` (or a small struct of
  accelerators) is cleaner. Either is fine; the step decides and keeps
  `setup` / `refresh` consistent.
- **Double fire on macOS is not verified by agents.** GUI automation is
  unavailable here; `open` relies on `preventDefault` consuming the key
  equivalent and that itself is still an open macOS check. The plan matches
  `open` rather than adding a frontend-side "skip the accelerator key"
  guard, so undo / redo inherit the same unverified assumption. If the macOS
  check later shows a double fire, the fix (the archived
  `menu-accelerators` plan's fallback: skip the accelerator key in keydown,
  or have the backend flag it in the `Binding` payload) applies to all four
  actions at once.
- **Removing the Z combinations from the menu lists changes what overrides
  are accepted.** A stored override binding `ctrl+z` to another action was
  refused before, so none can exist; after this change such a binding is
  refused instead by the "bound to undo" conflict rule, so behaviour for
  existing stores is unchanged.
- **Windows `refresh` patch-in-place for Edit items** is reasoned from the
  existing lookup code, not run. If `Submenu::get` on the Edit submenu does
  not find the item, the warning `menu item undo not found` appears in the
  log and the accelerator label stays stale; check the log during the
  Windows manual run.
- **Frontend keydown test.** There is no such test today (`main.ts` is not
  imported by tests), so the step adds none rather than refactoring
  `main.ts`.

## Progress

- (none yet)
