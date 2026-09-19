# Learnings

## Step 1: `riffle_core::sharpness`

- Scan cost, measured with `riffle-cli scan` (release build) before and after
  the change on the same folder: 1000 symlinks to one α7 V ARW
  (`~/Downloads/_DSC6978.ARW`, the only ARW in `~/Downloads`; 1616x1080
  preview, focus point recorded), warm page cache, Apple silicon, 12 cores.
  Runs alternated before/after back to back.
  - 1 thread: before `per file on a worker: mean 7.5ms p95 7.9ms` /
    `mean 7.2ms p95 7.6ms`; after `mean 10.4ms p95 10.9ms` /
    `mean 10.7ms p95 11.2ms`. About +3.2ms per file (~+45%).
  - 8 threads: before `mean 8.6ms p95 12.1ms`, `8.7 / 12.3`, `9.1 / 13.3`;
    after `mean 12.7ms p95 17.1ms`, `12.3 / 15.7`, `17.1 / 22.0` (noisier).
  - Same order of magnitude, so decision 1 stands. The full-scale grayscale
    decode was kept; `Decompress::scale(4)` was not needed.
  - The symlink folder repeats one inode, so this is CPU cost with no IO
    variety (see `docs/agents/tauri-app.md` on symlink folders).
- mozjpeg panics instead of returning `Err` on bytes that are not a JPEG, so
  `score_preview` wraps its own decode in `catch_unwind` and returns `Err`;
  `scan::extract` just calls `.ok()` on it.
- A preview whose thumbnail decodes but cannot be scored is hard to build
  (anything that breaks the grayscale decode breaks the RGB one too). A
  preview smaller than 3x3 has no interior pixel, so `score_preview` rejects
  it with `Err`; the `extract` test uses a 2x2 JPEG for the
  "thumbnail present, `sharpness: None`" case.
- `laplacian_variance` only uses pixels whose four neighbours are inside the
  window, so content just outside the window never leaks into the score.
- `crates/app/src/index.rs`'s test `entry()` builds an `Entry` literally and
  needed `sharpness: None`; `write_batch` ignores the field until Step 2.
- clippy's `manual_is_multiple_of` rejects `% 2 == 0` in test helpers too; use `.is_multiple_of(2)`.

## Step 2

- `SCHEMA_VERSION` on main was still 6, so v7 was used as planned.
- Accepting v6 in `prepare` meant the `label_known` `ALTER TABLE` (previously
  guarded by `version != SCHEMA_VERSION`) needed a `version < 6` guard, since a
  v6 `ratings` table already has the column. The two existing migration tests
  asserting `user_version == 6` were bumped to 7.

## Step 3

- The cue is a 3px bar up the left edge of the image box (`top: 24px`,
  96px tall, `transform: scaleY(ratio)` from the bottom), grey, and in the
  pick colour for the sharpest of its run. It sits below the top-left pick /
  reject dot and never touches the top-right stars.
- The meta pane's `Sharpness` row shows the raw score with one decimal
  (`toFixed(1)`).
- `refilter` returns early when the visible list did not change, so
  `refreshEntries` calls `applySharpness()` itself before `refilter()`; when
  `refilter` does rebuild, `setFiles` clears the strip's store and `refilter`
  re-applies it.
- Unverified (for the user): the manual on-screen checks (the burst's
  sharpest frame in the `best` colour, a missed frame's shorter bar, the cue
  recomputing on filter, an older-index folder rescanned once and then showing
  cues). GUI automation does not work on this machine.
