# Learnings

## Step 1

- gpui-component's `Tooltip` renders without `Root`. Its `Render` (0.9.1,
  `src/tooltip.rs`) is a plain view that reads the global `Theme` only;
  `Root` holds the overlay of gpui-component's own managed tooltips
  (`Tooltip::apply`, `ManagedTooltipExt`), which this app does not use. It is
  handed to GPUI's `.tooltip(..)` through `Tooltip::new(text).build(window, cx)`
  or, for the strip's wrapping text, `Tooltip::element` around a `div` with a
  `max_w`.
- The region `div`s do not swallow the strip's input: GPUI's
  `Window::hit_test` stops only at a `BlockMouse` hitbox, and a `div` without
  `occlude()` has a `Normal` one, so the strip `div`'s hitbox stays hovered
  and its `on_mouse_down` / `on_scroll` (which check `is_hovered`) fire.
- GPUI flips a tooltip above or left of the pointer when it would leave the
  window (`Window::prepaint_tooltip`), so the strip's tooltips stay inside
  without any placement code.
- `self.files` holds `PathBuf`s while `facts` and `judgments` are keyed by
  `String`; `strip_cells()` looks both up through `to_string_lossy()`.
- This worktree had no `node_modules`, so `mise run fmt` failed on `vp`
  until `pnpm install --frozen-lockfile` ran (environmental).

## Deferred issues (todo candidates)

- **Pending manual check** (Step 1; the step's checkbox was ticked on the
  automated criteria: the unit tests of `MarkState::icon` and
  `strip::tip_regions`, clippy and the native tests). On Windows (and, when
  at hand, macOS / Linux), with `mise run native:dev`:
  1. Open a folder with faces on a fresh index (`--index` pointing at a new
     file): the smiling `scan-face` in bright green appears at the image
     box's bottom-left on good frames and the frowning one in gray on low
     ones while pass 2 runs, with no other input; nothing on neutral frames
     or JPEGs. The icon is crisp at 100 % and 150 % display scale.
  2. Reject a good frame: its icon is dimmed with the cell.
  3. Hover a cell the scan failed on (e.g. a truncated RAW): after about
     500 ms the full error shows in a themed tooltip, wrapping at 360 px and
     inside the window. Hover a labeled cell's name band: the label's name
     shows. On a failed and labeled cell, the band shows the name and the
     rest the error. A plain cell shows nothing.
  4. Move from one cell's region to the next: the tooltip swaps. Scroll the
     strip under a still pointer: the tooltip of the cell that moved away
     goes. A cell cut by the window's left edge still shows its tooltip.
  5. Over a cell with a tooltip, a click, a ctrl / shift click, a
     right-click and the wheel still select, open the menu and scroll.
  6. Hover the tree's pending rename (rename a folder during a scan): the
     `PENDING_TITLE` tooltip shows as before, now in gpui-component's style.
  Files: `crates/native/src/main.rs` (`strip_tips`, `paint_face_icon`),
  `crates/native/src/strip.rs` (`tip_regions`),
  `crates/native/src/folderops.rs` (`tree_name`).
