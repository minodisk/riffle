# Learnings: strip-select-all

## Step 1

- Tauri 2.11's `Menu::default` Edit submenu is Undo, Redo, separator, Cut,
  Copy, Paste, Select All on every platform that has one, so the predefined
  Select All is the last item; `app_menu::build` removes it only when the
  last item is a `MenuItemKind::Predefined` and appends a custom `select-all`
  item. It is a plain `MenuItem` on every platform (no macOS icon), so
  `MenuItem` is now imported unconditionally in `main.rs`.
- The context menu's `MenuItem.checked` became `boolean | undefined`:
  undefined renders a plain `role="menuitem"` without `aria-checked`, so the
  checked-item highlight in `style.css` never applies to Select All.
- Local review round 1 found that removing the predefined `Select All` makes
  macOS text-input select-all depend on the menu accelerator, which follows
  the `selectAll` binding: rebinding it away from `meta+a` left `Cmd+A` doing
  nothing in a settings text input. Fixed by handling `meta+a` directly in
  `settings.ts`'s `keydown` (the `native` decision), independent of the
  current binding.
- Manual check still to do on the dev machine (the GUI cannot be driven from
  the agent session, as with undo / redo): press `Cmd/Ctrl+A` once in the
  strip (every file selected, the shown file stays), in the folder tree
  (nothing happens to the strip), and inside a settings text input (the text
  is selected); check `Edit > Select All` shows the accelerator and that
  rebinding `selectAll` updates it (including in place on Windows).
