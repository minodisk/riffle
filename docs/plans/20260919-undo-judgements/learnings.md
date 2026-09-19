# Learnings

## Step 1

- `refilter(anchor)` re-anchors on the undone file when it is still visible,
  but anchoring a file the filter now hides moves the current file to its
  neighbour. So `judge`'s apply-and-invoke body became `commit(...)` with an
  optional anchor thunk evaluated after the state is applied: undo anchors on
  the current file when `passes(path)` is false, leaving the view alone.
- The predefined Undo/Redo are removed from `Edit` by position 0, but only
  while the item there is `MenuItemKind::Predefined`, so a Tauri reordering
  cannot remove a non-predefined item.

## Deferred issues (todo candidates)

- None.
