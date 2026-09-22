# Learnings

## Step 1

- `app_menu::build` now takes `&Keymap` and derives all four keymap-backed
  accelerators itself; `refresh` was its only caller, so `setup` needed no
  change. `update_keymap`'s `accelerators` comparison in `commands.rs` did
  compare only `open` / `photolab`, so it now covers `undo` / `redo` too
  (otherwise rebinding undo would not refresh the menu).
- Existing tests used `meta+z` / `ctrl+z` as the example "menu accelerator"
  in `forbidden_follows_the_platform` and `add_refuses_a_forbidden_key`; they
  were switched to `meta+c` / `ctrl+c` / `meta+,` / `ctrl+,`.
- Manual GUI verification (Windows `mise run tauri:dev`: one press undoes /
  redoes once, Edit menu labels, rebinding) was not possible from this agent
  session; it remains a manual check.
