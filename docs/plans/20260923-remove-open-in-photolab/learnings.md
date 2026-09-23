# Learnings: remove Open in DxO PhotoLab

## Step 1

- Why the feature was retired: on the user's Windows machine PhotoLab 10
  ignored a folder argument, and a file argument only opened that one file
  in an "external application selection" project. A folder hand-off is not
  achievable on Windows, so a macOS-only item was dropped everywhere rather
  than kept.
- The "Done when" grep (`git grep -i photolab` over `shortcuts.rs` and
  `ui/src` returning nothing) is stricter than the rest of the plan allows:
  the new regression test `a_store_with_the_retired_photolab_action_still_loads`
  in `crates/app/src/shortcuts.rs` has to name `photolab`, and `filter.ts`'s
  "after PhotoLab's" comment is kept on purpose (it is about PhotoLab's own
  filter menu). Those are the only remaining matches.
- `NativeIcon` was used only by this item, so `docs/agents/tauri-app.md`'s
  "Menu icons: native where one exists, a bundled SF Symbol otherwise"
  section was retitled to "bundled SF Symbols rather than `NativeIcon`"; only
  archived plans cite the old title.
- Beyond the plan's list, three sentences that counted "the two File menu
  items/accelerators" were also reworded (`docs/usage.md`'s rebinding
  paragraph under the key table, `docs/agents/tauri-app.md`'s Undo / Redo
  paragraph), and the "App items go into the default menu's own submenus"
  section now names the three File items the app actually prepends.
- `todo.md`'s historical paragraph about the Windows run of the File menu
  accelerator checks (which mentions `Shift+Ctrl+O` firing once) was left
  as is: it records a past run, not current behavior.
