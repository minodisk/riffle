# Learnings

## Step 1

- `Exif`'s Maker note fields are safe to skip when `None` because the meta
  pane's `Metadata` (`commands.rs`) copies them into its own struct instead of
  serializing `Exif`, and nothing under `crates/app/ui/src`, `mcp.rs` or
  `companion.ts` reads them off an `Exif`.
- The roots refresh waits on `rootsLoaded` before re-invoking `folder_roots`,
  so a focus that lands before the launch-time `loadRoots` (which is deferred
  until the sort and keymap settings load) does not race it. It renders
  through `requestRender`, since the focus can come from a click whose mouse
  button is still held.
- Rewording one parenthetical in `docs/usage.md` reflows the rest of the
  paragraph by hand; there is no Markdown auto-wrap in `mise run fmt`.
