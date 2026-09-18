# Learnings

## Step 1

- `load_settings` now returns `(SidecarFormat, Keymap)` so the store is opened
  once at launch; `setup` in `main.rs` manages `AppKeymap` beside
  `AppSidecarFormat`.
- Unknown action names are warned about in a separate pass over the override
  object, since the merge itself walks the fixed action order and never visits
  them.
- A partially valid list (e.g. `["r", 1]`) rejects the whole override rather
  than keeping the valid keys.

## Step 2

- `keyName(event)` maps `" "` to `"space"` and lower-cases everything else,
  matching the Rust key names. `applyKeymap` rebuilds the whole
  `Map<key, action>` from a `Binding[]` so Steps 3-4 can pass command results
  straight in. `applyKeymap` and `keyName` have one caller each until Step 4.

## Step 3

- `reset(action)` returns `Result`: after rebinding `reject` to `r` and `clear`
  to `x`, restoring `reject`'s default `x` would bind `x` twice, so the reset is
  refused with the same `"x" is bound to clear` message `rebind` uses. The
  plan's signature did not say; this keeps the no-duplicate invariant.
- `reset_shortcuts` cannot fail, so it returns `Vec<Binding>` directly.
- `update_keymap` holds the `AppKeymap` lock across the store write so two
  quick rebinds cannot persist out of order.

## Step 4

- The panel logic lives in `main.ts` next to `applyKeymap`; the global
  `keydown` handler hands every key to `shortcutsKeydown` while the panel is
  open. Bare modifier keys (`shift`, `meta`, ...) are ignored during capture so
  pressing Shift does not bind `shift`. `Escape` cancels a capture, or closes
  the panel when nothing is capturing.
- `reset_shortcut` rejections go through the same status line as rejected
  rebinds (`updateShortcuts` handles every command).
- The `Settings` submenu is added in `app_menu::build` right after `Folder`,
  so `Sidecar` (appended later in `build_menu`) now sits after `Settings`.

## Step 5

- The manual GUI checks of Steps 2 and 4 are recorded in `README.md` as
  awaiting the user's confirmation, not confirmed. The only pitfall added to
  `docs/agents/tauri-app.md` is the submenu order (`Folder`, `Settings`,
  `Sidecar`) that follows from where each submenu is appended.
