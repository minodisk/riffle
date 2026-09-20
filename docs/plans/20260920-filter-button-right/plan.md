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

# Right-align the filter button in the tools row

## Purpose

PR #238 turned the filter menu into a fly-out that opens beside the sidebar
(`#filter-menu { top: 8px; left: calc(100% + 4px) }`, anchored to `#side`).
The filter toggle, however, is the left-most item of `#tools`, so the button
and the menu it opens are about 140px apart and the menu does not read as
belonging to that button. After this work the filter toggle sits at the right
end of the tools row, next to the sidebar edge the fly-out opens from. The
sort toggle stays on the left and keeps its drop-down.

## Background (from investigation)

- `#tools` is a plain `display: flex` row holding `#filter` then `#sort`
  (`crates/app/ui/index.html`, `crates/app/ui/style.css`). Both wrappers are
  `flex: none; padding: 0.5rem 8px 0.25rem`, and `#sort` overrides
  `padding-left: 0`, giving `[8px][filter][8px][sort][8px]`.
- `#filter` has no `position`, so `#filter-menu` is positioned against
  `#side` (the containing block set up in PR #238) and is unaffected by where
  `#filter` sits inside the row. Nothing about the fly-out needs to change.
- `#sort` is `position: relative` and `#sort-menu` uses `left: 8px`. Because
  `#sort` currently has no left padding, the menu's left edge is 8px to the
  right of the sort button's left edge.
- No TypeScript reads `#tools` or relies on the order of its children, and
  the frontend tests run under vitest with `environment: node` (no DOM), so
  there is nothing to update or test on the JS side. Visual checks are manual
  (`docs/agents/tauri-app.md`: GUI automation does not work on this Mac).

## Steps

- [x] Step 1: Move the filter toggle to the right end of `#tools` with CSS only
  - Done when:
    - The filter toggle is the right-most item in the tools row, 8px from the
      sidebar's right edge, and the sort toggle is the left-most item, 8px
      from the left edge.
    - The sort drop-down still opens below the sort button, and the filter
      fly-out still opens at `top: 8px; left: calc(100% + 4px)` beside the
      sidebar (CSS for `#side` and `#filter-menu` is untouched).
    - `mise run ci` passes.
    - The PR description carries a manual checklist for the user: filter
      button at the right, sort button at the left, sort menu drops down
      under its button, filter fly-out opens beside the sidebar level with
      the top of the row, both menus still close as before, and the sidebar
      is still 160px with the filter toggle's `active` state visible.
  - Implementation approach:
    - `crates/app/ui/style.css` only. Do not reorder `index.html` (see
      trade-offs). Do not touch the `#side` block, its comment, or the
      `#filter-menu` rule.
    - Push `#filter` to the right with `margin-left: auto` on the `#filter`
      wrapper (not on `#filter-toggle`, since the wrapper carries the
      padding).
    - Swap the left-padding override: `#sort` currently has
      `padding-left: 0`; after the move `#sort` needs the 8px left padding
      (it is now at the left edge) and `#filter` takes `padding-left: 0`
      (its right padding already provides the 8px edge gap). The resulting
      row is `[8px][sort][...auto...][filter][8px]`.
    - Leave `#sort-menu { left: 8px }` as is (the user's decision). With
      `#sort` now padded 8px on the left, the menu's left edge lines up with
      the sort button's left edge, 8px left of where it sits today.
    - Verify with `mise run ci`; keep the property order consistent with the
      neighbouring rules.

## Trade-offs and risks

- **CSS-only (`margin-left: auto`) instead of reordering the DOM.** Decided by
  the user. The visual order becomes sort, filter while the DOM (and therefore
  the Tab order) stays filter, sort. Reordering `index.html` would match the
  Tab order but moves the whole ~120-line `#filter-menu` block, inflating the
  diff for a two-button row that is rarely keyboard-navigated (the app is
  driven by the keymap in `shortcuts.rs`, not by Tab).
- **Sort drop-down offset.** Keeping `left: 8px` shifts the sort menu 8px left
  relative to today (it now aligns with the button's left edge). Decided by
  the user as a small improvement, not a regression.

## Progress

- (none yet)
