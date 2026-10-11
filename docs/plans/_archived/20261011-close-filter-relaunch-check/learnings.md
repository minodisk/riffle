# Learnings

## Step 1

- The user ran the three persist-filter relaunch scenarios by hand on a desktop build and all passed: the filter came back with the menu checks and the lit button, `Reset` then relaunch was unfiltered, and a bogus `"filter"` value in `settings.json` launched unfiltered without error. The feature has no OS-specific path (tauri-plugin-store plus the frontend), so the macOS repeat was dropped rather than kept as a checkbox.
- The item was the last one in `todo.md`, so the deletion ran from the blank line before its heading through the end of the file, leaving the previous item's last line as the file's final line with a single trailing newline.
- The archived `docs/plans/_archived/20261010-persist-filter/learnings.md` was left untouched; it records what was filed at the time.
