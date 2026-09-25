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

# Left-align the filmstrip tools and right-align the counter

## Purpose

The filmstrip's header bar (`#strip-bar`) shows the position counter at the
left and the sort / filter toggles at the right. The user wants the actions
(sort, filter) at the left edge and the page information (the counter) at the
right edge, as in the usual toolbar convention. After this work the bar reads
`[sort][filter] ........ [n / total · k selected]`.

## Background (from investigation)

- `#strip-bar` (`crates/app/ui/index.html`, lines 24-179) holds two children,
  in this source order:
  - `#position`: the **page information**. `main.ts` line 408 writes
    `${index + 1} / ${files.length}` into it, plus ` · N selected` when a
    multi-selection exists (line 410). It is empty when the folder has no
    files. This is the only page-information element in the bar.
  - `#tools`: the **actions**. It wraps `#filter` (the `#filter-toggle`
    button and its `#filter-menu` drop-up, including `#filter-exif`) and
    `#sort` (the `#sort-toggle` button and its `#sort-menu`), in that source
    order. There are no other action controls in the bar.
- The current visual order is made by CSS only (`crates/app/ui/style.css`):
  - `#tools { margin-left: auto }` (line 70) pushes the tools to the right.
  - `#filter { order: 1; margin-left: auto; padding-left: 0 }` (lines 80-84)
    puts the filter to the right of the sort inside `#tools`. This is a
    leftover of `docs/plans/_archived/20260920-filter-button-right`, when the
    filter menu was a fly-out beside the old sidebar. Visually the row is
    `[position] ... [sort][filter]`.
  - `#filter-menu, #sort-menu { bottom: calc(100% + 4px); right: 8px }`
    (lines 130-135) anchor each menu to its toggle's right edge, with a comment
    (lines 125-129) explaining they sit at the window's right edge.
- No TypeScript reads `#tools`' children order or measures the toggles.
  Frontend tests run under vitest `environment: "node"` (no DOM), so no test
  asserts on this DOM.
- README / README.ja.md and `docs/agents/tauri-app.md` do not describe the
  bar's order, so no docs update is needed.

## Decisions (agreed with the user)

- The left-side order is `[sort][filter]` (today's visual order is kept).
- Get that order from the markup, not from CSS `order`: swap `#sort` before
  `#filter` inside `#tools` in `index.html`, and drop `order: 1` from
  `#filter`.

## Steps

- [x] Step 1: Put `#tools` at the left edge and `#position` at the right edge
      of `#strip-bar`
  - Done when:
    - The sort and filter toggles are the left-most items of the bar, 8px from
      the window's left edge, in the order `[sort][filter]`, with that order
      coming from the markup (no CSS `order` on `#filter` / `#sort`).
    - The counter (`n / total`, with the ` · k selected` suffix) is the
      right-most item, with its existing `0 0.5rem` padding.
    - Each menu still opens upward from the bar and its left edge is aligned
      with its toggle's left edge (it must not be clipped by the window's left
      edge or open far from its button).
    - The CSS comment above the menu rule no longer claims the toggles sit at
      the window's right edge.
    - No behavior change: filter / sort logic, the counter's text and the
      `#filter-toggle.active` highlight are untouched.
    - `mise run ci` passes.
    - The PR description carries a manual checklist (GUI automation is not
      available, per `docs/agents/tauri-app.md`): bar order, menu placement
      of both menus, the counter with and without a multi-selection, and the
      bar when the folder is empty (the counter is empty, tools stay left).
  - Implementation approach:
    - `crates/app/ui/index.html`: inside `#tools`, move the `#sort` block
      before the `#filter` block. No other markup change.
    - `crates/app/ui/style.css`, surgical edits only:
      - `#tools`: drop `margin-left: auto`.
      - `#position`: add `margin-left: auto` (it is `flex: none` and comes
        after `#tools`, so the auto margin pushes it to the right edge).
      - `#filter`: drop `order: 1` and `margin-left: auto`. Rework the
        paddings so the row is `8px[sort]8px[filter]8px` (e.g. `#sort` keeps
        `padding: 0.25rem 8px`, `#filter` keeps `padding-left: 0`, which is
        already the case).
      - `#filter-menu, #sort-menu`: replace `right: 8px` with a `left` value
        that puts the menu's left edge on the toggle's left edge (the menus
        are positioned against `#filter` / `#sort`, which are
        `position: relative`, so the value equals that wrapper's left
        padding: `8px` for `#sort`, `0` for `#filter`). Update the comment
        accordingly.
    - Do not change `main.ts` or any test.
    - Run `pnpm exec vp fmt` / `mise run ci` before opening the PR.

## Trade-offs and risks

- **Menu anchoring.** If the menus keep `right: 8px` after the move, they
  would extend leftwards past the window edge (the wrappers are only as wide
  as their toggles, and the menus are `min-width: 180px`). Re-anchoring to
  `left` is therefore mandatory, not cosmetic.
- No docs risk: README / README.ja.md do not describe the bar's layout.

## Progress

- (2026-09-25) Step 1 complete
