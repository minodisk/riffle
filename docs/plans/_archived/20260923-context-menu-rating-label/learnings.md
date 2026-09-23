# Learnings

## Step 1

- `labels` in `main.ts` stores the capitalized label name (`"Red"`), while the
  action names are lower-case (`red`); the checked test compares against the
  capitalized name.
- The menu buttons use `role="menuitemradio"` since `aria-checked` is not a
  valid attribute on a plain `menuitem`.
- A rating of `0` (as well as `null`) counts as "No stars".
- Not verified manually on a small window (no GUI in this environment);
  `menuPosition` clamping and `overflow-y: auto` are unchanged.

## Step 2

- The `click` hit test only checked the horizontal gap; the extracted
  `comparePaneAt` in `crates/app/ui/src/compare.ts` also rejects the vertical
  gap, as the step requires a right-click in a gap to do nothing.
- `openContextMenu` is a hoisted function declaration, so the new `contextmenu`
  handler on `#canvas` can sit above it next to the `click` handler.
- While a comparison is still loading (`compareFrames` empty) no pane is hit,
  so a right-click only suppresses the native menu.
- The manual check the plan asks for (2 / 3 / 4 frames, meta pane switching,
  undo, multi-selection on the single-image view) could not be run: this
  environment has no GUI. It is covered by the `comparePaneAt` unit tests and
  by `judge` being untouched.
