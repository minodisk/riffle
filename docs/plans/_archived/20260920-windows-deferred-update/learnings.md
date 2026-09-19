# Learnings

## Step 1: Defer the Windows install to quit

- `tauri_plugin_updater::Update` cannot be constructed outside the plugin, so
  the pending slot and `install_pending` are compile-checked only; the Windows
  install path (download at launch, `install` from `ExitRequested` after the
  flush) is not exercised end-to-end. The existing `todo.md` verification item
  tracks it.
- The startup log and dialog texts are split with `#[cfg(windows)]` /
  `#[cfg(not(windows))]` function pairs, so the non-Windows build never sees
  the `pending` write and `main.rs` stays `cfg`-free.
