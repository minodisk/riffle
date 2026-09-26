# Learnings

## Step 1

- Keyboard cursor and focus: a right-click does not move the tree's cursor and
  does not change focus. The `#folders` container is `tabindex="0"`, so a
  right-button `mousedown` would focus it (silently giving the tree the
  keyboard and turning the culling keys off); `folders.ts` now
  `preventDefault()`s a `mousedown` with `button === 2`, which keeps focus
  wherever it was. On macOS, Control+click is the other standard way to
  right-click (common on trackpads); WebKit reports it as a primary-button
  `mousedown` with `ctrlKey === true` and no `click`, so the guard also
  `preventDefault()`s a `button === 0` `mousedown` with `ctrlKey` when
  `navigator.platform` reports macOS (limited to macOS, since Ctrl+click is
  an ordinary click on Windows/Linux and should still focus the tree). The
  document-level `mousedown` that closes the menu still sees the event,
  since only the default is prevented.
- `Escape` ordering: `folders.keydown` consumes `Escape` to blur the tree, so
  with the tree holding the keyboard the old `Escape`-closes-the-menu branch
  (after the tree block) was never reached. The "any key closes the context
  menu, `Escape` is consumed doing so" check moved ahead of the tree block,
  replacing the two later branches. Side effect: the strip's menu now also
  closes on a key the tree consumes, and when both the filter / sort menu
  and the context menu were somehow open, `Escape` closes the context menu
  first (in practice the `mousedown` handlers keep only one open).
- `folderMenuGroups(label)` was kept in `context.ts` with a vitest test,
  rather than inlined in `main.ts`: it keeps the item shape next to
  `contextMenuGroups` and gives the acceptance criterion's vitest test
  something to cover. The menu DOM building was factored into
  `showMenu(groups, x, y, run)`; the strip menu passes `runAction`, the
  folder menu a closure invoking `reveal_folder`.
- The label comes from the sync `reveal_label` command, invoked once when
  `main.ts` loads and cached as a promise; the folder menu awaits it.
- Drive roots: not verified on a real device (no GUI run in this step).
  `tauri-plugin-opener` 2.5.5's Windows `reveal_items_in_dir` resolves the
  parent through `windows_shell_path::shell_parent_path` and returns
  `Error::NoParent` if there is none, which would be reported through the
  status line. No `open_path` fallback was added; any error surfaces via
  `setStatus`.
