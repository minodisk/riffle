# Learnings

## Step 1

- Confirmed in code before writing: `thumbnail_jpeg` in
  `crates/core/src/decode.rs` calls `d.scale(2)` (2/8), `PREVIEW_MIN_WIDTH`
  in `crates/core/src/arw.rs` is 1600, and the strip's `thumbnail` command
  (`crates/app/src/commands.rs`) reads `Index::thumbnail`
  (`crates/app/src/index.rs`). 9520x6328 at 2/8 is 2380x1582 (~3.8 MP) vs
  ~0.11 MP for a 404x270 thumbnail.
- The old heading string is also quoted in archived plans
  (`docs/plans/_archived/20260924-camera-support-todo/plan.md`,
  `docs/plans/_archived/20260924-linux-sigma-preview-latency-todo/`); those
  are historical records and were left as-is. Only `todo.md` referenced it
  live.
