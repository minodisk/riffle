# Learnings: view-menu-panes

## Step 1

- `refresh`'s in-place accelerator patch (Windows / Linux) looked items up
  with `as_menuitem()`, which returns `None` for a `CheckMenuItem`; it now
  matches `MenuItemKind::MenuItem` and `MenuItemKind::Check`. Any future
  keyed check item needs the same.
- The panels the checks follow live in `AppPanels` (`commands.rs`), seeded
  in `setup` from `stored_panels` before the first `app_menu::refresh`, so
  the launch checks come from the store and the frontend's restore (which
  does not call `set_panels`) agrees with them. `set_panels` normalizes the
  value, stores it in `AppPanels`, applies the checks, then saves.
- The panels are read into a separate `stored_panels` helper rather than
  widening `load_settings`'s tuple to six elements; the `panels` command
  reuses it.
- A menu click re-applies the stored panels before emitting `menu-action`,
  undoing muda's native toggle-on-click. This relies on muda toggling the
  check before it delivers the event; confirmed only by reading, part of
  the manual checks below.
- The macOS `cfg` branches compile only in CI's macOS job; the new code has
  no macOS-only lines (`CheckMenuItem` has no icon variant, so no twin).
- The Bash tool's heredoc with a large quoted Python body failed with
  "unexpected EOF while looking for matching `'`"; writing the script body
  with a different terminator (`PYEOF`) worked.

## Deferred issues (todo candidates)

- **Pending manual check (Windows)**: the View menu after
  `view-menu-panes` Step 1 (`crates/app/src/main.rs` `app_menu`,
  `crates/app/src/commands.rs` `set_panels` / `AppPanels`,
  `crates/app/src/shortcuts.rs` defaults). Run `mise run tauri:dev` on
  Windows and verify: (1) at launch `View` shows `Left Pane`, `Right Pane`,
  `Filmstrip` with `Ctrl+Alt+ArrowLeft` / `ArrowRight` / `ArrowDown` and
  checks matching the panes the last session left; (2) each of the three
  keys toggles its pane exactly once (no double fire) and the check
  follows; (3) `Tab`, `f`, `z`, `v` still work and are not in `View`;
  (4) clicking `View > Filmstrip` twice hides then shows it with the check
  right each time; (5) with Settings open, clicking `View > Left Pane`
  changes nothing and leaves the check as it was; (6) rebinding
  `toggleStrip` to `ctrl+alt+s` shows it on the item, `Ctrl+Alt+S` toggles
  once, `Reset` restores `Ctrl+Alt+ArrowDown`; (7) with the folder tree
  focused, `Ctrl+Alt+ArrowLeft` still hides the tree; (8) `F6` / `F7` /
  `F8` do nothing. Also watch for an Intel graphics hotkey taking
  `Ctrl+Alt+Arrow` (screen rotation); if it does, document the driver
  setting in `docs/usage.md`. The step's checkbox was ticked on the
  automated criteria (`mise run ci`); these were not verified.
- **Pending manual check (macOS)**: the same checks (1)-(6) with
  `Alt+Cmd+Arrow`, plus the checks surviving a rebinding (the menu is
  rebuilt) and the separator above `Enter Full Screen` still being there.
  Already carried in `todo.md` under "App: real-device checks for the View
  menu"; not verified here (no Mac).
