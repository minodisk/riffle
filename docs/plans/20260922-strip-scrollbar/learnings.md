# Learnings

## Step 1

- Took approach A. `#side` uses `flex: none; width: min-content` rather than
  plain `flex: none`: with max-content sizing, a long `#position` line
  (`N / M · a / b in burst`) could widen the column. The filter and sort
  menus are absolutely positioned, so they do not feed the intrinsic width.
- Chromium counting the `scrollbar-gutter: stable` gutter in intrinsic width
  could not be verified here (no GUI); it is left to the manual check in
  `todo.md`. If it fails on Windows, fall back to approach B (fixed ~171px).

## Deferred issues (todo candidates)

- None.
