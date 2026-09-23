<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Keep the strip visible while the filter menu is open

## Purpose

The filter menu opens from the filter button at the top of the 160px sidebar
and drops straight down over the thumbnail strip: with the flag, star, color
label and EXIF groups it fills up to 70vh, so only the last two or three
thumbnails stay visible. The point of the menu is to narrow the strip while
watching what remains, and that result is hidden behind the menu itself.

The live update already exists: every toggle runs `filterChanged()` ->
`refilter()` -> `strip.setFiles(...)` synchronously
(`crates/app/ui/src/main.ts`, ~line 1445), and the `N / M` counter under the
strip (`#position`) already reflects the match count. Only the placement is
wrong. After this work the menu opens as a fly-out beside the sidebar, over
the left edge of the viewer, so the whole strip stays visible and updates in
front of the user while they toggle filters. Behavior, state and the DOM are
unchanged.

## Background (from investigation)

- `crates/app/ui/style.css`: `#filter` and `#sort` are `position: relative`
  (line 54); `#filter-menu, #sort-menu` share one rule (line 90) with
  `position: absolute; z-index: 10; top: calc(100% - 0.25rem + 2px);
  left: 8px; min-width: 180px; max-height: 70vh; overflow-y: auto`. The
  containing block is the toggle's own wrapper, hence the drop-down.
- `#viewer` is `position: relative` with no `z-index`, and `#side` has no
  `position`, so an element anchored to `#side` with `z-index: 10` paints
  above the canvas without any stacking-context work.
- `main.ts` closes the menu on a `mousedown` whose target is not inside
  `#filter` (DOM `closest("#filter")`, ~line 1475) and on Escape (~line
  1595); both are DOM-based, so moving the menu visually does not affect
  them. The culling keymap stays live while the menu is open.
- The earlier viewer-empty-state plan decided the layout must not hard-code
  the `#side` / `#meta` widths; anchoring with `left: 100%` on `#side`
  respects that.
- Frontend tests run under vitest with `environment: node` (no DOM), and
  GUI automation does not work on this Mac (`docs/agents/tauri-app.md`), so
  the visual result is confirmed manually by the user from a checklist.

## Steps

- [x] Step 1: Open the filter menu as a fly-out beside the sidebar
  - Done when: with a folder open, clicking the filter button opens the menu
    to the right of the sidebar (over the viewer's left edge), the strip
    underneath is fully visible and visibly narrows/widens as items are
    toggled, every existing group (Reset, flags, stars, color labels, the
    EXIF groups) is reachable by scrolling the menu, outside-click and
    Escape still close it, the sort menu is unaffected, `mise run ci` passes,
    and the PR lists the manual checks for the user to confirm.
  - Implementation approach:
    - CSS only, in `crates/app/ui/style.css`; no change to `index.html` or
      `main.ts`. Do not touch the menu's item styling, only its box
      placement.
    - Make `#side` the containing block (`position: relative`) and drop
      `position: relative` from `#filter` (keep it on `#sort`: the sort menu
      stays a drop-down, per the user's decision). Split the shared
      `#filter-menu, #sort-menu` rule so the placement properties (`top`,
      `left`, `max-height`) are set per menu while the shared look
      (background, border, shadow, padding) stays in one rule.
    - Anchor with `left: 100%` of `#side` (plus a small gap, for example
      `4px`), never a pixel width of the sidebar. Align `top` with the tools
      row (the toggle's `0.5rem` padding) so the menu reads as belonging to
      the button.
    - Since the menu no longer costs strip space, let it use the window
      height: replace `max-height: 70vh` with a bound relative to the top
      offset (for example `max-height: calc(100vh - <top> - 8px)`), keeping
      `overflow-y: auto` so short windows still scroll.
    - Do not add a `z-index` to `#side`: doing so creates a stacking context
      and the menu would then paint below `#viewer`. Leave a comment in
      `style.css` saying so, and check the menu is above the canvas and is
      not clipped by `#strip`'s `overflow`.
    - README (`README.md`, "Filter menu" bullet) needs no change unless the
      wording is judged to describe placement; keep it as is otherwise.
    - Manual verification list for the PR (GUI automation is unavailable):
      1. Open a folder with 20+ files; click the filter button: the menu
         opens right of the strip, the strip stays fully visible.
      2. Toggle `Picked`, a star item, a color label and an EXIF item: the
         strip narrows immediately and the `N / M` counter updates while the
         menu stays open; untoggle and it widens.
      3. Scroll the menu to the last EXIF group (Focal length) in a short
         window (for example 700px tall): reachable, menu scrolls, strip
         still visible.
      4. Click a thumbnail while the menu is open: the menu closes and the
         thumbnail is selected (unchanged behavior).
      5. Escape closes the menu; the filter button stays lit while a filter
         is active.
      6. The menu paints above the viewer image and above the empty-state
         message; nothing is clipped.
      7. The sort menu still opens downwards and selects as before.

## Trade-offs and risks

- **The sort menu stays a drop-down.** Decided by the user: it has three
  items and covers at most one cell, so it does not cause the problem, and
  leaving it is the smaller diff. The cost is that the two adjacent toggles
  open their menus in different directions.
- **The fly-out covers the viewer's left ~200px while open.** For a portrait
  image centered in a wide viewer this is empty margin; for a landscape image
  it hides the left edge of the current photo while the menu is open. Accepted
  by the user for a culling tool (the viewer is the large, spare area; the
  strip is the one the filter is about), and the menu closes on any outside
  click. Alternative not taken: pushing the strip down with an inline panel;
  the menu can exceed 800px, so the strip would disappear or both would be
  crammed into 160px.
- **Clicking a thumbnail closes the menu.** Unchanged from today (outside
  `mousedown` closes). With the strip now visible, a user may want to click a
  thumbnail to inspect it while keeping the menu open. Keeping the menu open
  on strip clicks would be a behavior change and is out of scope here; note
  it as a follow-up if the user asks for it.
- **A live match count inside the menu** was considered and not planned: the
  `N / M` counter under the strip already updates on every toggle and is
  visible once the menu is no longer over the strip.
- **Stacking**: the fix relies on `#side` not forming a stacking context. A
  future `z-index` or `overflow` on `#side` would send the menu behind the
  viewer or clip it; the implementation comment in `style.css` says so.

## Progress

- (2026-09-20) Step 1 complete (GUI confirmation pending the user)
