# Learnings

## Step 1

- `burstFrameStep` is a one-liner over the same `ids` array as `burstStep`
  (unburst files get unique negative ids), so run-based clamping falls out of
  comparing the neighbour's id with the current one.
- The by-hand check that the webview does not swallow `Alt+ArrowUp/Down`
  (Trade-offs) was not done in this step; it needs `mise run tauri:dev`.

## Step 2

- The plan put the count badge top-left, but `span.flag` (the pick / reject
  dot) already sits there. The badge is placed top-left at `left: 14px`, just
  right of the dot; it stays above the sharpness bar (which starts at 24px).
- The band is a `::before` with `z-index: -1` inside `.cell.burst`, which gets
  `isolation: isolate`, so it paints under the cell's own background: inside
  the cell box `.cell.current` / `.cell.failed` backgrounds cover it, and the
  5px it reaches outside the box reads as a frame.
- `highlight` now calls `paintBurst` for every live cell, so the `n/m` badge
  follows `current` without tracking the previous index.
- Pending: the by-hand check in `mise run tauri:dev` on a folder with bursts
  (band visible, badge on the first cell, `n/m` follows the selection, gap
  filled, band distinguishable from `.cell.current` and `.cell.failed`) was not
  done by the implementation agent.
