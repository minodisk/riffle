# Learnings

## Step 1

- `window.remove_menu()` is only compiled on non-macOS, so the `window` binding
  would be unused on macOS; a `#[cfg(target_os = "macos")] let _ = window;`
  keeps the macOS build free of an unused-variable warning. Only the Linux
  branch is compiled by `mise run ci` here; the Windows behavior (no menu bar in
  Settings, main window keeps its menu) needs a manual check.
