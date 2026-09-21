# Learnings

## Step 1

- `burstFrameStep` is a one-liner over the same `ids` array as `burstStep`
  (unburst files get unique negative ids), so run-based clamping falls out of
  comparing the neighbour's id with the current one.
- The by-hand check that the webview does not swallow `Alt+ArrowUp/Down`
  (Trade-offs) was not done in this step; it needs `mise run tauri:dev`.
