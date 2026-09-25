# Learnings

## Step 1

- The flat `{ heading, rows }` shape let `section()` and `group()` collapse
  into one `group(heading, rows)` helper; the renderer loses its inner loop
  and the label branch. No other caller read `MetaGroup.sections`.
- `main.ts` still mentions "EXIF sections" above `rebuildExifMenu()`; those
  are the filter menu's own sections, unrelated to the meta pane, so they were
  left alone.
