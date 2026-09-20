# Learnings

## Step 1

- `accelerator()` is a plain table in `crates/app/src/shortcuts.rs`; muda is
  not a dependency of `crates/app`, so nothing validates the string at compile
  time. Tauri parses it with `.parse().ok()`, so an unknown name would
  silently become "no accelerator" rather than an error.
- A key with only `shift` (or no modifier at all) converts to `None` on
  purpose, so a plain `o` override leaves `File > Open Folder…` without an
  accelerator.
- The menu is no longer passed to `Builder::menu`; `setup` calls
  `app_menu::refresh` right after `load_settings`, and `update_keymap` calls it
  again whenever the `(open, photolab)` accelerator pair changes. Rebuilding
  through `AppHandle::set_menu` is what clears a stale macOS key equivalent,
  which muda's `set_accelerator(None)` does not do.
- `MenuItem` is now imported unconditionally in the `app_menu` module: the new
  `Open Folder…` item is a plain `MenuItem` on every platform (no native icon,
  as decided in the plan's trade-offs).

### Unverified on a real device

`mise run ci` passes and the app builds, but the GUI could not be driven from
this session, so none of the "Real-device check on macOS" bullets are
confirmed. What the user needs to do, with a debug build of this branch on
macOS:

1. In the main window, press `Cmd+O` once: the folder picker must open exactly
   once (not twice, from the keydown handler and the menu accelerator both).
2. With a folder open, press `Shift+Cmd+O` once: PhotoLab must be handed the
   folder exactly once.
3. Check that the menu set from `setup` actually shows on the main window, and
   that `Riffle > Settings...` still opens.
4. In the settings window, rebind `open` (e.g. to `ctrl+alt+o`) and confirm
   `File > Open Folder…` shows the new accelerator, the new key works, `Cmd+O`
   no longer does, and `Open in DxO PhotoLab` is untouched. Then rebind `open`
   to a plain key only (`o`) and confirm the menu item shows no accelerator and
   `Cmd+O` is dead.
5. Confirm `set_menu` while the settings window is focused does not steal its
   focus or flicker the menu bar.
6. While a shortcuts row is capturing in the settings window, press `Cmd+O` and
   `Shift+Cmd+O`: neither should trigger the menu action.
7. Click both File menu items with the mouse.

Windows and Linux are unverified too; the `Some`/`None` accelerator paths are
only reasoned from muda 0.19.3's Windows accelerator-table and GTK
`remove_accelerator` implementations, as the plan's investigation notes.

## Deferred issues (todo candidates)

- **App: the real-device checks for the File menu accelerators are still open.**
  From this step's implementation: the GUI could not be driven from the agent
  session, so the double-fire, stale-key-equivalent, settings-capture and
  menu-set-from-`setup` checks listed above are unconfirmed on macOS, and
  Windows / Linux are unconfirmed entirely. Files:
  `crates/app/src/main.rs` (`app_menu`), `crates/app/src/shortcuts.rs`,
  `crates/app/src/commands.rs` (`update_keymap`), `crates/app/ui/src/main.ts`.
- **App: `Open Folder…` has no macOS menu icon.** From the plan's trade-offs:
  `NativeIcon::Folder` exists but some `NativeIcon`s proved to be legacy colour
  bitmaps in the macos-menu-icons work, so a plain `MenuItem` was used. A
  verified icon could be added later in `crates/app/src/main.rs` (`app_menu`).
