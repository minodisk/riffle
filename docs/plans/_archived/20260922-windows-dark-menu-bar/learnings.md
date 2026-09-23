# Learnings

## Step 1

- tauri 2.11.6 `Menu::get` only searches the top-level items; it does not
  recurse into submenus. The in-place patch looks each id up with
  `Submenu::get` on every top-level submenu instead.
- The non-macOS path cannot be exercised on Linux CI beyond compiling; the
  Windows dark-bar behavior is verified manually by the user after release.
