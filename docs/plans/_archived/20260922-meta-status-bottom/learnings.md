# Learnings

## Step 1

- The pane-level box (width, background, padding, font) moved to a new
  `#info` column-flex wrapper; `#meta` keeps its id and only gains
  `flex: 1; min-height: 0; overflow-y: auto`, so the `#meta .name` / `dl` /
  `dt` / `dd` rules did not need touching.
- The `#info` wrapper's padding now also surrounds the status block, so the
  status lines sit 0.75rem above the pane's bottom edge.
