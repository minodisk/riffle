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

# Strip cell geometry: burst band gap and selection outline

## Purpose

Two visual defects in the strip, both CSS geometry in `crates/app/ui/style.css`:

1. The burst band of one burst touches the band of the next. At a burst's first
   and last cell the band's `::before` reaches 5px past the padding box (4px
   past the 1px border), and `--cell-gap` is 8px, so the last cell of one burst
   and the first of the next each claim half the gap and the rounded ends meet.
   Consecutive bursts read as one block.
2. The selected cell's white outline is hidden on the right. `.cell` is 144px
   wide with `box-sizing: border-box` and a 1px border, so its padding box is
   142px, while `.cell img` is a 144px box positioned at `left: 72px` with
   `translate(-50%)`, relative to that padding box. The image spans 0..144px in
   a 142px box and covers the right border by 2px.

After this change consecutive bursts show a clear break, members of one burst
stay joined, the selection outline is visible on all four sides, and the image
sits inside its cell.

## Steps

- [x] Step 1: Fix the burst band edge reach and the cell width in `style.css`
  - Done when:
    - Burst gap: `.cell.burst.burst-first::before` uses `top: -2px` and
      `.cell.burst.burst-last::before` uses `bottom: -2px`; the interior rule
      `.cell.burst::before { top: calc(-1px - var(--cell-gap)) }` is
      unchanged, so members of one burst still fill the gap between each
      other.
    - Outline: `.cell` is `width: 146px` (144px padding box + 1px border each
      side) and `left: 7px` so it stays centred in the 160px column; the
      thumbnail is fully inside the border, and the white `.cell.current`
      outline (and the grey `.cell.selected` one) is visible on all four sides.
    - `mise run ci` passes.
  - Implementation approach:
    - All changes are in `crates/app/ui/style.css`; do not touch `burst.ts`,
      `strip.ts` or the tests. `first` / `last` classification is already
      correct and covered by `burst.test.ts`, and `strip.ts` reads only
      `--cell-height`, never the cell width.
    - Burst gap: change the two edge offsets to `-2px`, keep the horizontal
      `-5px` overhang and the 6px corner radii, and update the comment above
      `.cell.burst` to say the band stops short of the neighbouring burst.
    - Outline: change `.cell { width: 144px }` to `146px`, `left: 8px` to
      `7px`, and add a short comment that the width is the 144px image box plus
      the border, since absolute children are placed against the padding box.
      Do not shrink the image to 142px: the 144px square footprint is a
      documented contract in both `style.css` and `strip.ts`.

## Trade-offs and risks

- Outline fix: widening the cell (chosen) keeps the 144px image box contract;
  shrinking the image to 142px would break the documented 144x96 / 96x144
  sizing.
- Not taken: widening `--cell-gap`. That changes every row height and the
  virtual-list maths in `strip.ts` for a cosmetic fix.

## Progress

- (none yet)
