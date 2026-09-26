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

# Strip Select All

## Purpose

The strip can select a range (`Shift+click`, `Shift+←/→`) and toggle single
files, but there is no way to select every file at once, so a judgment for
"everything the filter shows" needs a long Shift+drag. This adds a
`selectAll` culling action (`Cmd+A` on macOS, `Ctrl+A` elsewhere), a `Select
All` item in the strip's right-click menu, and makes `Edit > Select All`
follow the action, the way `Undo` / `Redo` already do. With it, the user can
filter (e.g. `Focus candidates`) and then rate, pick or label everything the
filter lets through in one go.

## Steps

- [x] Step 1: Add the `selectAll` action, the menu item and the strip's Select All
  - Done when:
    - `Keymap::defaults()` has `("selectAll", &[SELECT_ALL_DEFAULT])` (`meta+a` on macOS, `ctrl+a` elsewhere); `meta+a` is gone from `MACOS_MENU` and `ctrl+a` from `OTHER_MENU`, so the key is rebindable and the freed combination is bindable elsewhere; `the_defaults_bind_no_key_twice`, `the_defaults_are_the_full_table`, `the_menu_defaults_follow_the_platform`, `the_menu_defaults_are_not_forbidden` and `accelerator_for_takes_the_first_convertible_key` cover it.
    - `Edit > Select All` is a custom item whose accelerator is `keymap.accelerator_for("selectAll")`, refreshed on rebinding like Undo / Redo, and whose click emits a `select-all` event.
    - With the strip focused (i.e. the folder tree does not have focus and the settings modal is closed), pressing the key or clicking the menu item or the strip's right-click `Select All` selects every file in `files` (the filtered, sorted list); the current file stays current and becomes the anchor. Judgments then apply to all of them (`targets`).
    - With focus in the folder tree, the key does nothing to the strip (swallowed by `treeGate`, since `selectAll` is not in `TREE_PASSTHROUGH`); a test in `treekeys.test.ts` asserts it is swallowed.
    - Inside a text input of the settings modal, `Cmd/Ctrl+A` still selects the input's text on every platform.
    - The strip's right-click menu shows a `Select All` item with the action's first key, above the existing three groups, and `context.test.ts` covers it (item, label, shortcut, and that an override shows).
    - `selection.test.ts` covers the new helper: all files selected, anchor = focused file, empty list gives an empty selection.
    - The shortcuts panel lists `Select all` (`shortcutLabels` in `settings.ts`) and add / remove / reset work like other actions.
    - `README.md`, `README.ja.md` (filmstrip bullet) and `docs/usage.md` (selection paragraph, Keys table, and the "menu-derived accelerators" exception sentence, which now also names Select All) mention it.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/app/src/shortcuts.rs`: add `const SELECT_ALL_DEFAULT: &str = if MACOS { "meta+a" } else { "ctrl+a" };` next to `UNDO_DEFAULT`; insert `("selectAll", &[SELECT_ALL_DEFAULT])` right after `extendNext` in `DEFAULTS` (it is a selection action; the panel order follows `DEFAULTS`); remove `"meta+a"` from `MACOS_MENU` and `"ctrl+a"` from `OTHER_MENU`; update the comment above `MACOS_MENU` to list `Select All` among the derived accelerators. Update the tests listed above; a store from before this change still loads (no test needed beyond the existing pattern, but `a_store_from_before_undo_redo_still_loads` shows the shape if one is wanted).
    - `crates/app/src/main.rs` `app_menu`: add `const SELECT_ALL_ID: &str = "select-all";`; in `build`, take `keymap.accelerator_for("selectAll")`; in the `Edit` block, after removing the predefined Undo / Redo, remove the trailing predefined `Select All` (last item of `edit.items()`, a `MenuItemKind::Predefined`) and append a custom item (`IconMenuItem` on macOS if an icon is wanted; a plain `MenuItem` is acceptable, see trade-offs) titled `Select All` with that accelerator; add `(SELECT_ALL_ID, "selectAll")` to the `refresh` list; in `on_event` emit `select-all`. Do not touch Cut / Copy / Paste.
    - `crates/app/src/commands.rs`: `accelerators()` becomes `[Option<String>; 4]` over `["open", "undo", "redo", "selectAll"]` so rebinding refreshes the menu.
    - `crates/app/ui/src/selection.ts`: add `export function all(files: readonly string[], focused: number): Selection` returning `{ selected: new Set(files), anchor: files[focused] }` (same shape `extend` produces; `single(undefined)` when `files` is empty).
    - `crates/app/ui/src/main.ts`: add `function selectAllFiles()` that sets `selection = all(files, index)`, calls `paintSelection()`, `renderMeta()` and `if (comparing) void loadCompare()` (mirror the no-move branch of `extendSelection`); add `case "selectAll": selectAllFiles(); break;` to `runAction`; add `window.__TAURI__.event.listen("select-all", ...)` next to the `undo` / `redo` listeners. The listener is the path the menu (and on macOS most likely the key equivalent) takes, so it must do the focus gating itself: if `document.activeElement` is an `HTMLInputElement` / `HTMLTextAreaElement`, call `.select()` on it (this replaces the native predefined item's behavior for the settings' text inputs); else if `settings.isOpen` or `folders.hasFocus()`, do nothing; else `selectAllFiles()`. The keydown path needs no new gating: the tree swallows it through `treeGate`, and the settings modal takes every key while open. Both paths firing on one press (unverified, as for undo) is harmless because selecting all is idempotent; note this in the code comment.
    - `crates/app/ui/src/context.ts`: add a first group holding `selectAll` / `Select All`. The existing `MenuItem` has `checked` and is rendered as `menuitemradio`; give the entry a way to render as a plain `menuitem` (e.g. an optional `radio: false` / `kind` field, or `checked: undefined`) rather than a never-checked radio. Keep `shortcut` from the binding's first key through `displayKey` like the others. In `openContextMenu`, set `role` / `aria-checked` accordingly.
    - `crates/app/ui/src/settings.ts`: `selectAll: "Select all"` in `shortcutLabels`, placed after `extendNext`.
    - `crates/app/ui/src/treekeys.test.ts`: add `"selectAll"` to the swallowed list.
    - Docs: `README.md` filmstrip bullet ("`Cmd+A` / `Ctrl+A` selects every file the filter shows"), `README.ja.md` same sentence in Japanese, `docs/usage.md` selection paragraph, Keys table row `CmdOrCtrl+A` ("select every file in the strip (also `Edit > Select All`, whose accelerator follows this key)"), and the sentence at lines ~279-283 listing the menu items that follow their action's keys.
    - Manual check to record in `learnings.md` (the GUI cannot be driven from the agent session, as with undo / redo): on the dev machine, press the key once in the strip, in the folder tree, and inside a settings text input; check the Edit menu shows the accelerator and that rebinding `selectAll` updates it.

## Trade-offs and risks

- **Replace the predefined `Select All` (chosen by the user) vs keep it and only add a JS keydown handler.** Keeping the predefined item leaves `Cmd+A` owned by the native menu on macOS, so the webview either never sees the keydown or WebKit selects the page's DOM text; it would also keep `meta+a` / `ctrl+a` unbindable and out of the shortcuts panel, unlike every other culling key. Replacing it follows the Undo / Redo pattern already in the code and makes the key rebindable. The cost is that native select-all in text inputs now depends on the frontend fallback (`.select()` on the focused input) instead of the OS; the only text inputs are in the settings modal, and the fallback is a one-liner.
- **Rebinding `selectAll` away from `meta+a` on macOS** would otherwise leave `Cmd+A` selecting nothing in a settings text input: the `select-all` event only fires through the menu's accelerator, which then follows the rebound key. `settings.ts`'s `keydown` covers this directly: on the `native` decision, `meta+a` on a focused `HTMLInputElement` / `HTMLTextAreaElement` runs `.select()` regardless of the current `selectAll` binding, so the done criterion holds for every binding, not just the default.
- **Whether the keydown also reaches the webview when the menu accelerator fires** is unverified on every platform (the same open item exists for `Cmd+O` / `Cmd+Z`). Select all is idempotent, so a double fire is invisible; no extra guard is planned.
- **Windows dark menu bar**: `refresh` patches items in place there rather than calling `set_menu`; the new item must be found through the same `get(id)` lookup, so the ID must be unique across submenus.
- **Menu icon on macOS**: Undo / Redo use `IconMenuItem` with PNGs under `crates/app/icons/menu/`. Adding an icon means adding a template PNG; a plain `MenuItem` avoids that. Left to the implementer; a missing icon is not a functional regression.
- **Single PR**: one feature whose halves are not independently useful, so it is planned as one step, docs included.
- A store that has `selectAll` unset simply gets the default; an old user override bound to `meta+a` was impossible (it was forbidden), so no migration.

## Progress

- (2026-09-26) Step 1 complete
