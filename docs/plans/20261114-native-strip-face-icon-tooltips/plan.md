<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Native strip: the face focus icon and the cell tooltips

## Purpose

The Tauri strip's cells show two things the native (GPUI) strip's cells lack:

- the face focus icon at the image box's bottom-left (`strip.ts`'s
  `paintTier`, `focus.ts`'s `FOCUS_MARK_ICONS` / `FOCUS_MARK_COLORS`): a
  smiling `scan-face` in bright green on a good frame, the frowning variant
  in gray on a low one, nothing on a neutral one;
- the hover texts: the scan's full error on a failed cell (`cell.el.title`,
  `paintFailure`) and the label's name over a labeled cell's file-name band
  (`cell.name.title`, `paintRating`).

Once this is done the native strip tells a good frame from a low one at a
glance as the Tauri strip does, an error cut off inside the 144 x 96 box can
be read in full, and a label's name (not only its color) can be read.

Out of scope: the recommended (sparkles) icon (the ranking is the GPUI port's
Step 9), the pending-rename clock icon and the inline file rename, the cell's
pointer cursor, and adding tooltips the native app does not have yet
anywhere else (the tree's rows, the `Sort` / `Filter` buttons).

Also in scope (asked by the user): the app's one existing tooltip,
`folderops.rs`'s hand-drawn `Tip` on the tree's pending rename icon, moves
to gpui-component's `Tooltip` too, so every tooltip in the native app is
gpui-component's.

### What the investigation found (2026-11-14, `main` at `b8fad0f8`)

- `crates/native/src/main.rs`: `StripCell` (~4729) is what `paint_strip`
  (~4805) paints; `strip_cells()` (~3247) builds it on every render from
  `self.strip`, `self.thumbs` and `self.files`. `passes` (~2293) already
  derives the state as
  `mark::af_eye_state(self.facts.get(path).and_then(|f| f.focus.as_ref()))`.
- `self.facts` is updated by `filter::apply_face_ready` on every
  `Event::FacesProgress` (~2475) and by `set_entries` / `read_entries`; both
  paths end in `cx.notify()`. A state computed in `strip_cells()` therefore
  follows pass 2 with no extra bookkeeping.
- `mark.rs` has `MarkState::{Good, Neutral, Low}` and `MarkState::color()`
  (`0x33ff33` / `0xffffff` / `0x999999`, equal to `FOCUS_MARK_COLORS`). The
  filter menu's lead (`main.rs` ~4642) already maps the state to
  `icons::SCAN_FACE_FROWN` / `icons::SCAN_FACE` inline, through
  `svg().path(..)`.
- `style.css`: `.cell span.marks` is `bottom: 26px; left: 2px` with
  `filter: drop-shadow(0 0 2px #000)`, `.cell span.marks svg` is 12 x 12 px.
  It mirrors `.cell span.count` (`bottom: 26px; right: 2px`), which the
  native `paint_burst_badge` places 2 px inside the image box's bottom-right
  corner. So the native icon is 12 x 12 px, 2 px inside the image box's
  bottom-left corner. The flag dot is at the top-left, the stars at the
  top-right, the burst badge at the bottom-right: no overlap.
- The rejected cell's dimming is a `REJECTED_DIM` quad painted last over the
  whole cell in `paint_strip`, so anything painted before it is dimmed with
  the cell, as `.cell.rejected`'s opacity does.
- `Window::paint_svg(bounds, path, data: Option<&[u8]>, transformation,
  color, cx)` exists in `gpui-pre-0.3.12` (`src/window.rs` ~5233); it renders
  the path through `cx.svg_renderer` (the application's `AssetSource`,
  `icons::Icons`) into a monochrome sprite drawn in `color`.
- The label of a file is `judgment::State::label: Option<String>`, which
  `Strip::marks` reduces to a `Tint`; the Tauri title is that string as is.
  The failure text is `self.thumbs.failure(index)`, already put into
  `StripCell::failure`.
- Tooltips: the strip `canvas` takes no input (`docs/agents/native-app.md`,
  "A `canvas` gets no input; its wrapping `div` does"). GPUI's
  `.tooltip(..)` on a stateful `div` (`gpui-pre-0.3.12`,
  `src/elements/div.rs`, `handle_tooltip_mouse_move`) schedules the tooltip
  while that element's hitbox is hovered, keeps it while the pointer stays in
  the hitbox and drops it when the pointer leaves or the hitbox's bounds no
  longer contain the pointer at prepaint. One tooltip on the whole strip
  `div` would thus not change from cell to cell. Hitboxes that are not
  occluding are all "hovered" when stacked, so two stacked tooltip elements
  would both ask for a tooltip.
- The app already has one tooltip: `folderops.rs`'s `Tip(&'static str)` view
  (~1384) on the tree's pending name. gpui-component 0.9.1 has a tooltip
  view, `gpui_component::tooltip::Tooltip::new(text).build(window, cx)`,
  styled from the `Theme`.

## Steps

- [x] Step 1: Draw the face focus icon on the native strip's cells and show the failure and label-name tooltips
  - Done when:
    - A cell whose file's `mark::af_eye_state` is Good draws `scan-face` in
      `MarkState::Good.color()`, a Low one `scan-face-frown` in
      `MarkState::Low.color()`, a Neutral one (and a file without a focus,
      as a JPEG) nothing; the icon is 12 x 12 px, 2 px inside the image
      box's bottom-left corner, and a rejected cell's dimming covers it.
    - The icons fill in as pass 2 runs (opening a folder on a fresh index
      shows them appear without any other input) and are right after a
      re-listing, a sort and a filter change.
    - Hovering a failed cell shows the scan's full error text; hovering a
      labeled cell's file-name band shows the label's name; on a cell that
      is both failed and labeled the name band shows the label's name and
      the rest of the cell the error; a cell that is neither (and a failed
      cell with an empty message) shows no tooltip.
    - The tooltips are gpui-component's `Tooltip`, and so is the tree's
      pending rename icon's (`folderops.rs`), whose hand-drawn `Tip` view is
      removed; its text and hover behavior are unchanged.
    - A click, a ctrl / shift click, a right-click and the wheel over a cell
      with a tooltip still do what they did (select, open the menu, scroll).
    - The pure rules are unit-tested in their module's tests: which icon for
      which state, and which tooltip regions and texts a cell has for each
      combination of failure and label.
    - `CLAUDE.md`'s description of `strip.rs` (and of `mark.rs` if the icon
      rule lands there) names the new responsibilities, and
      `docs/agents/native-app.md` records what was verified about tooltips
      over a canvas and `paint_svg`.
    - `mise run ci:local` passes locally and the PR's GitHub checks pass.
  - Implementation approach (as far as it is known; omit if unknown):
    - Files: `crates/native/src/main.rs` (`StripCell`, `strip_cells`,
      `paint_strip`, the strip's `div` in `render`), `crates/native/src/strip.rs`
      (the pure tooltip regions), `crates/native/src/mark.rs` (the state's
      icon), `crates/native/src/folderops.rs` (`Tip` replaced), `CLAUDE.md`, `docs/agents/native-app.md`. Nothing in
      `crates/app`, `crates/backend` or `crates/core` changes.
    - Icon rule: a port of `FOCUS_MARK_ICONS` next to `MarkState::color()`
      in `mark.rs` (e.g. `MarkState::icon() -> Option<&'static str>`: Good
      `icons::SCAN_FACE`, Low `icons::SCAN_FACE_FROWN`, Neutral `None`),
      unit-tested there. Do not rewrite the filter menu's inline mapping
      unless using the new function there is a one-line change that keeps
      its behavior (its Neutral case does not occur).
    - State: give `StripCell` the `MarkState`, computed in `strip_cells()`
      from `self.facts` exactly as `passes` does. Do not cache it in
      `Strip`: `strip_cells()` runs on every render and every `facts` update
      already notifies, so there is nothing to invalidate.
    - Painting: in `paint_strip`, inside the cell's content mask and before
      the `REJECTED_DIM` quad, `window.paint_svg` with the icon's path,
      `data: None`, `TransformationMatrix::unit()` and the state's color,
      over a 12 x 12 px bounds at `image_box.left() + 2`,
      `image_box.bottom() - 2 - 12`. For the Tauri `drop-shadow` halo, paint
      black copies first at `overlay::HALO`'s offsets, as
      `paint_burst_badge` does for its text.
    - Verify while implementing: that `paint_svg` with `data: None` resolves
      the path through `icons::Icons` (the icon appears), that the 12 px
      icon is crisp at 100 % and 150 % scale, and that `paint_svg`'s `Err`
      is handled without a panic (log it).
    - Tooltips (decided by the user): lay transparent, absolutely positioned
      stateful `div`s over the canvas inside the strip's `div` (made
      `relative`), one per tooltip region of each on-screen cell, each with
      GPUI's `.tooltip(..)` building gpui-component's
      `Tooltip::new(text).build(window, cx)`. Keep the regions of one cell
      disjoint so no two tooltip hitboxes stack: the label's region is the
      name band (`CELL_WIDTH` x `NAME_HEIGHT` at the image box's bottom, as
      `paint_strip` draws it); the failure's region is the whole cell
      (`CELL_WIDTH` x `CELL_HEIGHT`), or the cell above the name band when
      the cell is also labeled.
    - The pure part goes in `strip.rs`: a function from a cell's failure
      text and label name to its regions (a rectangle relative to the cell's
      origin plus the text), an empty text giving no region as an empty
      `title` does. `strip_cells()` hands it the failure it already reads
      and the label from `self.judgments`; the label name is the stored
      string as is, not the `Tint`.
    - Give each region `div` an id that follows the file (e.g. the cell's
      index plus which region), since GPUI keys the tooltip's state by
      element id.
    - Verify while implementing: gpui-component's `Tooltip` renders without
      `Root` mounted (the adoption plan mounts it in its Step 5); if it does
      not, stop and record it in `learnings.md` and report rather than
      falling back silently. The strip `div`'s `on_mouse_down` (left and
      right) and `on_scroll` still fire under a region `div` (the region
      must not occlude); moving from one cell's region to the next swaps the
      tooltip; scrolling the strip under a still pointer drops the tooltip
      of a cell that moved away; a cell cut by the window's left edge
      (negative `x`) is fine; a long error wraps (give the tooltip a maximum
      width) and stays inside the window.
    - Manual check for the PR: a folder with faces on a fresh index (icons
      appear during pass 2), a rejected good frame (icon dimmed), a file the
      scan fails on (e.g. a truncated RAW), and a labeled file.

## Trade-offs and risks

- **Tooltip mechanism (decided: region `div`s).** Transparent region `div`s
  with GPUI's `.tooltip(..)`: the delay, the placement, hiding on leave /
  press and the layering are GPUI's. Its behavior is read from the GPUI
  source, not yet measured: the step verifies that the regions do not
  swallow the strip's clicks and wheel. The alternative, an own hover hit
  test, timer and popover, was not chosen.
- **Tooltip view (decided: gpui-component's `Tooltip`).** It follows the
  adoption plan's direction and the `Theme`; the tree's existing `Tip`
  is switched in the same step so every tooltip looks the same. Whether it
  needs `Root` was not verified.
- **Hover after keyboard input.** GPUI treats no hitbox as hovered while
  the last input was the keyboard, so after paging with the arrow keys a
  tooltip appears only once the pointer moves. The Tauri `title` behaves
  much the same; nothing is planned against it.
- **The in-flight gpui-component adoption plan**
  (`../20261114-gpui-component-adoption/plan.md`, Steps 4 to 8 open) also
  edits `main.rs`'s `render` and popovers. This step touches the strip's
  `div` and `paint_strip` only, which that plan leaves out of scope ("the
  canvas-drawn parts"), so conflicts should be limited to nearby lines.
- **Halo cost.** Eight black copies plus the icon is nine sprites per marked
  cell on screen (about a dozen cells), all from the sprite atlas. If the
  strip's paint time grows measurably, fall back to one offset copy as the
  stars do.

## Progress
