# Learnings

## Step 1

- `Keymap` held `format` only to resolve the per-format label defaults; once
  `LABEL_DEFAULTS` folded into `DEFAULTS`, `AppShortcutOverrides`
  (`crates/app/src/commands.rs`) lost its only reader, so the struct, its
  `app.manage` in `main.rs`, the write in `update_keymap` and the `Option<Value>`
  element of `load_settings`'s tuple all went with it. `cargo clippy
  -p riffle-app --all-targets` confirmed nothing else read it.
- Rewriting the tests, the keys that used to be free are not any more and vice
  versa: `j` / `6` are now unbound (so they no longer make a collision case),
  while `z`, `arrowup` and `arrowdown` are defaults (so they do). `previous`
  now has a single key, so `remove` cases have to `add` one first.
- The old-store regression test (`a_store_from_the_old_defaults_still_loads`)
  shows the point of keeping `inactive`: an old `{"zoom": ["space", "z"]}`
  still applies and round-trips.

## Deferred issues (todo candidates)

- `open` stays on `o`. The plan's "Trade-offs and risks" wants it in the File
  menu with a rebindable `Cmd+O` / `Ctrl+O` accelerator, which needs
  platform-dependent defaults, a key-name-to-accelerator conversion, a rule for
  which key becomes the accelerator and an exemption in `forbidden()`
  (`crates/app/src/shortcuts.rs`, `crates/app/src/app_menu.rs`). Deferred to its
  own PR.
