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

# View menu: pane toggles only, with check marks and modifier keys

## Purpose

The `View` menu from [view-menu](../_archived/20260928-view-menu/plan.md)
mixes pane toggles with main-view keys, shows no state, and shows no
accelerator because its default keys (`F6`, `F7`, `F8`, `Tab`, `f`, `z`,
`v`) are modifier-less and `shortcuts::accelerator` refuses to register a
modifier-less key (a menu accelerator is app-global, so it would fire while
typing in a text field). The user's decision: `View` keeps only `Left Pane`,
`Right Pane` and `Filmstrip`, as check items whose check follows the panes
actually shown; the pane toggles move to modifier defaults (`Alt+Cmd+Arrow`
on macOS, `Ctrl+Alt+Arrow` elsewhere) so the accelerator column is filled;
`F6` / `F7` / `F8` are dropped as defaults; `toggleSides` (`Tab`), `focus`
(`f`), `zoom` (`z`) and `compare` (`v`) stay as keys but leave the menu.
`docs/agents/tauri-app.md` then states what a new menu item needs.

The dev machine is Windows: the macOS `cfg` branches compile only in CI's
macOS job, and every GUI check is a manual confirmation by the user.

The planner proposed two steps (code + user docs, then the guide rule +
`todo.md`); the user approved folding them into one step so the work ships
as a single PR.

## Steps

- [x] Step 1: Trim `View` to the three pane check items, move the pane toggles to modifier defaults, sync the checks from `set_panels`, and update the docs, the guide rule and `todo.md`
  - Done when:
    - `View` lists exactly `Left Pane`, `Right Pane`, `Filmstrip` (no
      separator among them; on macOS still above the default
      `Enter Full Screen` with the existing separator between). Each is a
      `tauri::menu::CheckMenuItem`; its check equals `panels.left` /
      `panels.right` / `panels.strip` at launch (from the stored `panels`)
      and after every change by key, menu click, or the frontend's restore.
    - `Both Side Panes`, `Focus Mark`, `1:1 Zoom`, `Compare` are gone from
      the menu; `toggleSides`, `focus`, `zoom`, `compare` are in `MENU_LESS`
      (reason: main-view / strip keys, not menu items) and `Tab`, `f`, `z`,
      `v` still work; `menu_covers_every_action` passes.
    - Defaults: `toggleLeft` = macOS `alt+meta+arrowleft` / else
      `ctrl+alt+arrowleft`; `toggleRight` = `…arrowright`; `toggleStrip` =
      `…arrowdown`; `F6` / `F7` / `F8` bound to nothing. The three items show
      their accelerator and follow a rebinding as `Open Folder…` does. A
      stored override that still binds `f7` keeps working (`from_overrides`
      replaces the keys as before).
    - A menu click never leaves a check out of sync: muda's native
      auto-toggle on click is reverted from the stored panels before
      `menu-action` is emitted, so a click the frontend gates (modal open,
      format dialog) leaves the check as it was, and an effective click is
      corrected by the frontend's `set_panels` a moment later.
    - Rust unit tests: the stored-panels-to-check mapping per `VIEW_ITEMS`
      action (`Some(bool)` for the three, `None` otherwise, missing / non-bool
      entries = shown, as `panels_setting` treats them); every `VIEW_ITEMS`
      action has a mapping; `the_menu_defaults_are_not_forbidden` covers the
      three new defaults on both platforms; the panel-toggle defaults test
      asserts the new defaults convert to `Ctrl+Alt+ArrowLeft` /
      `Alt+Cmd+ArrowLeft` etc.; `the_defaults_are_the_full_table` updated.
    - User docs: `docs/usage.md` (the tree-gate sentence near line 40, the
      Panels bullet, the Focus mark / 1:1 / Compare bullets lose
      `View > …`, the Keys table rows for `f` / `z` / `v` / the panel keys,
      the paragraph "A View item shows its action's key…" near line 500, the
      "While the folder tree has the keyboard" defaults list near line 508),
      `README.md` 81-83 and `README.ja.md` 47 (the bullet naming `F6` / `F7`
      / `F8` and the removed items), kept in sync.
    - `docs/agents/tauri-app.md`: "App items go into the default menu's own
      submenus" (View paragraph) and "Rebuild the menu on macOS, patch
      accelerators in place elsewhere" (add: `refresh` re-applies the
      checks) updated; plus a short rule next to "Menu icons: bundled SF
      Symbols rather than `NativeIcon`" / "Adding a macOS menu item needs the
      same `cfg` split": a new item is an `IconMenuItem` with an SF Symbol PNG
      exported by `tools/macos/export-menu-icons.swift` (needs macOS) plus a
      `MenuItem` twin elsewhere; if it mirrors a keymap action it goes into
      `keyed_items()` and its default key carries a modifier, because a menu
      accelerator is app-global and `shortcuts::accelerator` shows nothing
      for a modifier-less key (the pane toggles are the precedent); a
      `CheckMenuItem` cannot carry an icon (no icon-bearing check item in
      muda / Tauri), and a check item's state is applied by `apply_panels`
      and re-applied in `refresh`.
    - `todo.md`: the "View menu items have no macOS SF Symbol icons" entry is
      removed (check items cannot take icons), and the "real-device checks
      for the View menu" entry is rewritten to the current items and the
      macOS checks still open.
    - `mise run ci` passes.
    - Manual (Windows), exactly: (1) launch: `View` shows the three items
      with `Ctrl+Alt+ArrowLeft/Right/Down` and checks matching the panes the
      last session left; (2) press each of the three once: the pane toggles
      exactly once (no double fire) and the check follows; (3) `Tab`, `f`,
      `z`, `v` still work and are not in `View`; (4) click `View > Filmstrip`
      twice: hides, shows, check right each time; (5) open Settings, click
      `View > Left Pane`: nothing changes, check unchanged; (6) rebind
      `toggleStrip` to `ctrl+alt+s`: the item shows it, `Ctrl+Alt+S` toggles
      once, `Reset` restores `Ctrl+Alt+ArrowDown`; (7) with the folder tree
      focused, `Ctrl+Alt+ArrowLeft` still hides the tree (tree passthrough);
      (8) `F6` / `F7` / `F8` now do nothing. Also watch for an Intel graphics
      hotkey taking `Ctrl+Alt+Arrow` (screen rotation) — if it does, note it
      in `learnings.md` and `docs/usage.md` as a driver setting to turn off;
      not a blocker. On macOS (when the user next runs there): (1)-(6) with
      `Alt+Cmd+Arrow`, plus the checks survive a rebinding (menu rebuild) and
      the separator above `Enter Full Screen` is still there.
  - Implementation approach:
    - `crates/app/src/shortcuts.rs`: add `TOGGLE_LEFT_DEFAULT` /
      `TOGGLE_RIGHT_DEFAULT` / `TOGGLE_STRIP_DEFAULT` as `const … = if MACOS
      {…} else {…}` next to `OPEN_DEFAULT`, use them in `DEFAULTS`; no
      change to `accelerator()` or `forbidden()`. Do not add `f6`-`f8` to
      any forbidden list (a user may bind them).
    - `crates/app/src/main.rs` `app_menu`: `VIEW_ITEMS` shrinks to three
      rows; drop `VIEW_PANELS` and the in-menu separator (keep the macOS
      `view.insert(separator, view_kinds.len())`). Build each row with
      `CheckMenuItem::with_id(handle, id, label, true, checked, key)` (no
      icon variant exists, so no `cfg` twin). Add `MENU_LESS` entries with
      the reason comment. Add `pub fn apply_panels(app, panels: &Value)`
      that walks `menu.items() → as_submenu()?.get(id)` (as `refresh` does)
      and calls `as_check_menuitem()?.set_checked(panel_checked(panels,
      action))`, where `panel_checked(&Value, action) -> Option<bool>` is
      the tested pure mapping. `refresh` calls `apply_panels` at its end on
      every platform. `on_event`: for a `VIEW_ITEMS` hit, `apply_panels`
      first, then emit `menu-action`.
    - `crates/app/src/commands.rs`: manage `AppPanels(Mutex<Value>)` seeded
      in `setup` from the store (`panels_setting(store.get("panels"))`;
      extend `load_settings`'s tuple or read it there) *before*
      `app_menu::refresh`. `set_panels` stores the normalized value in
      `AppPanels` and calls `app_menu::apply_panels` in addition to saving
      (it stays a sync command: small file + main-thread menu work, per the
      guide). The `panels` command can keep reading the store.
    - Frontend: no change needed; `changePanels` already invokes
      `set_panels` on every change and the restore goes through
      `applyPanels` (which does not invoke `set_panels` — the launch check
      comes from the seed instead, so both agree). No `f6`/`f7`/`f8` literal
      in `ui/src`; `settings.ts` descriptions stay.
    - Registered accelerator + keydown: follow the existing Open / Undo
      pattern (registered accelerator fires the menu event, which the
      `menu-action` listener runs under the keydown gates; verified once on
      Windows for `Ctrl+O` / `Ctrl+Z` in `todo.md`). No frontend dedupe.

## Trade-offs and risks

- **Dropping `F6` / `F7` / `F8` changes muscle memory** for existing users;
  the settings modal still lets them bind the F-keys back per action, and a
  stored override survives. Documented in `docs/usage.md`.
- **`Ctrl+Alt+Arrow` on Windows** is bound by some Intel graphics drivers to
  screen rotation; the app would never see the key. Manual check only; if
  hit, document the driver toggle rather than changing the default (the
  user chose these keys).
- **Registered accelerator vs. keydown** was verified once-firing on Windows
  for the File / Edit items but never on macOS; the pane toggles inherit
  that open macOS check.
- **Check items forgo icons for good**: the SF Symbol todo for the View
  items is closed rather than deferred.
- **Menu-click gating while the folder tree has focus**: the pane toggles are
  tree passthrough (`treekeys.ts`), so a click runs; only the modal / format
  dialog gates leave the native check to be reverted by `apply_panels`.

## Progress

- (none yet)
