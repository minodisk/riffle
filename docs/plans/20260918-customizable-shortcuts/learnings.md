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
