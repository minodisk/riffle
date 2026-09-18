# Learnings

## Step 1

- `ratings.xmp_size IS NOT NULL` reads as `bool` through rusqlite the same way
  `thumb IS NOT NULL` does; with the `LEFT JOIN`, a missing `ratings` row gives
  `NULL IS NOT NULL` = 0, so no `COALESCE` is needed.
- `rate()` and its failure path called `draw()` only to repaint the canvas
  badge; with the badge gone, `renderMeta()` (directly or through `setStatus`)
  is enough. `refreshEntries` keeps its `draw()` for the focus mark.
- `renderMeta()` runs on every `show()` via `setStatus`, so setting the strip
  pane's `N / M` there needed no new call sites.
- The transient notes (`note`, `scanning`, `1:1`) stay above the sidecar
  section, as planned. Not checked visually (no GUI automation here).
- The `.cell.rejected` opacity (0.45) also dims the strip's red `✕`, so the
  reject reads slightly darker in the strip than in the meta row even though
  both use `--reject-color`.

## Deferred issues (todo candidates)

- The sidecar header shows the predicted name (`FOO.xmp`), not an on-disk case
  variant such as `FOO.XMP`; exposing the real name needs a column or a
  `read_dir` (see plan Trade-offs). Files: `crates/app/ui/src/main.ts`
  (`sidecarName`), `crates/app/src/commands.rs` (`list_sidecars_in`).
- A `sidecar-error` does not revert the optimistic "has sidecar" flag (plan
  Trade-offs). File: `crates/app/ui/src/main.ts`.
