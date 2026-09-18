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
