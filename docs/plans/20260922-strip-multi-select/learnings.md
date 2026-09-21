# Learnings

## Step 1

- `selection.ts` returns new `Selection` values instead of mutating, so
  callers replace the state held in `main.ts`. `extend` returns the new
  focused index alongside the selection so the clamping lives in one place.
- `prune` re-adds the focused file so the "focused is always selected"
  invariant holds after a filter change.

## Step 2

- A Shift+click moves the focus to the clicked file (and reloads the
  viewer), so the range from the anchor always contains the focused file and
  the "focused is always selected" invariant holds without a special case. A
  Cmd/Ctrl+click only changes the selection and keeps the viewer as it is.
- Pruning lives in `refilter` only: `resync` and `trashRejected` (through
  `resync`) and every filter / sort / judgement change go through it, and
  `strip.setFiles` clears the strip's copy, so `refilter` repaints it.
- `move`, `moveBurst` and the arrow keys do not collapse the selection yet
  (Step 4), so until then the focused file can move outside the selection.
