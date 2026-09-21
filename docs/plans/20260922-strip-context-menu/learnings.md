# Learnings

## Step 1

- `keymap.get` returns `string | undefined`, so the listener guards `undefined` before calling `runAction(action: string)`; the old `default: return` path now returns `false`.

## Step 2

- `menuPosition` flips the menu to the other side of the pointer when it would
  overflow, then clamps at 0 so a menu larger than the space on both sides
  sticks to the top / left edge.

## Step 3

- The menu is shown (un-hidden) before `menuPosition` so `offsetWidth` /
  `offsetHeight` measure the rebuilt items; `#context-menu` overrides the
  shared `position: absolute` with `position: fixed` in its own rule.
- The menu is closed at the top of `openDirectory` and in `refilter`'s
  empty-`files` branch, so it never floats over a changed folder.
