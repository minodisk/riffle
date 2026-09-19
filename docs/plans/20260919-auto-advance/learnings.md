# Learnings

## Step 1

- `set_auto_advance` persists synchronously like `update_keymap` (store set +
  save on the calling thread); the write is a tiny JSON file, so
  `spawn_blocking` was not worth it.
- `load_settings` now returns a 4-tuple with the `autoAdvance` bool; parsing is
  in the pure `auto_advance_setting(Option<&Value>) -> bool`.
