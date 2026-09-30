# Learnings: view-pane-names

## Step 1

- The labels lived in one place in code (`VIEW_ITEMS` in
  `crates/app/src/main.rs`); ids, actions and the stored `panels` keys were
  left alone, so no test or migration changed.
- `grep -rni "left pane\|right pane"` still matches positional prose (code
  comments in `commands.rs`, `main.ts`, `panels.ts`, `folders.ts`,
  `style.css`, and the layout sentences in `docs/usage.md`,
  `docs/agents/tauri-app.md`, `README.md`); these name the position, not the
  menu item, and stay as the plan says. A case-sensitive
  `grep -rn "Left Pane\|Right Pane"` outside `docs/plans/**`, `CHANGELOG.md`
  and `tmp/` returns nothing.
- `todo.md`'s View menu real-device checks were updated to the new labels
  because the plan names that edit explicitly.

## Deferred issues (todo candidates)

- Pending manual check (Windows): run `mise run tauri:dev` and confirm that
  `View` shows `Folders`, `Metadata`, `Filmstrip` with their accelerators
  (`Ctrl+Alt+ArrowLeft` / `ArrowRight` / `ArrowDown`) and checks matching the
  shown panes, and that clicking each item still toggles its pane (folder
  tree, metadata pane, filmstrip). Basis: Step 1's Done-when in
  `docs/plans/20260930-view-pane-names/plan.md`; related file
  `crates/app/src/main.rs` (`VIEW_ITEMS`). The step's checkbox was ticked on
  the automated criteria only; this check is the user's.
