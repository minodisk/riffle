# Learnings: view-menu

## Step 1

- `VIEW_ITEMS` in `app_menu` is the single table the menu build, `refresh`,
  `on_event` and `commands::accelerators()` (through `app_menu::keyed_actions`)
  read, so a new View item cannot miss the accelerator refresh.
- `accelerators()` returned a fixed `[Option<String>; 4]`; it is now a `Vec`
  built from `keyed_actions()`, which is enough for the `!=` comparison in
  `update_keymap`.
- On Windows / Linux the `View` submenu is inserted after `Edit` by index,
  found by scanning `menu.items()` for a submenu titled `Edit` (appended when
  there is none).
- The macOS branch (prepending into the default `View` above Enter Full
  Screen, then a separator at `view_kinds.len()`) is compiled only on macOS
  and was not built here.
- No pure gate helper was extracted in `main.ts`: the `menu-action` listener
  reuses `modalOpen()` and `treeGate` inline, so there is no new Vitest test.
- Pending: the manual check on Windows (each View item by mouse; rebinding
  `toggleStrip` to e.g. `ctrl+alt+s` shows on the item and toggles exactly
  once; `Reset` blanks it) is left to the user after the PR is up. If it
  double-fires, pass `None` as the View items' accelerator (see the plan's
  Trade-offs).
