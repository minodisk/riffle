# Learnings

## Step 1

- The prepended File items are bound to a local array typed
  `[&dyn tauri::menu::IsMenuItem<Wry>; 3]` (the same trait-object form the
  View menu's `view_kinds` uses), since the elements are of different concrete
  types and the separator has to outlive both calls. On non-macOS
  `insert_items` takes `own.len()`, so Settings and Check for Updates land
  right after File's own items whatever that list grows to.
- The rendered menu could not be checked by the agent; the step was ticked on
  the automated criteria (`mise run ci`).

## Deferred issues (todo candidates)

- Pending manual check (Windows): run the app and open the File menu. Expected
  order: Open Folder, Reload Folder, ─, Settings..., Check for Updates…, ─,
  Close Window, Quit, with a single separator between each group (no doubled
  separator). Basis: Step 1 of this plan, change in `crates/app/src/main.rs`
  (`app_menu::build`). Step 1's checkbox was ticked on the automated criteria
  only. Linux follows the same code path but was not built here.
