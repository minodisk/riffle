
## Step 1: inactive shortcut overrides

- `Keymap` now carries an `inactive` map (conflict-skipped stored entries).
  Because `Keymap` derives `PartialEq`, the existing tests asserting a
  conflict-skipped keymap equals the defaults were switched to compare
  `bindings()`; tests for other skip reasons still compare whole keymaps,
  which also checks those entries are not kept.
- `add`/`remove`/`reset(action)` drop the action's inactive entry only on
  success; a failed edit leaves the stored value untouched.

## Step 2

- `riffle-app` is a bin-only crate, so `cargo test --lib` fails; run
  `cargo test <name>` inside `crates/app`. The rewritten cancel test passed
  25/25 in a local loop.
- `PROGRESS_INTERVAL` had to become `pub` so the `start_scan` call site in
  `commands.rs` can pass it to `run_scan`.
