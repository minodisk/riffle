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

# Move the burst count badge to the bottom-right of the cell

## Purpose

The burst count badge (`.cell span.count`, painted by `paintBurst` in
`crates/app/ui/src/strip.ts`: `position/size` on the current cell, the burst
size on the first cell of a run) sits at the top-left of the strip cell,
14px to the right of the pick / reject dot. That corner is already busy (flag
at top-left, stars at top-right, sharpness bar down the left edge), and the
count reads as part of the flag. Moving it to the bottom-right of the image
area, just above the file-name strip, gives it its own corner and keeps every
badge in a distinct place: flag top-left, stars top-right, sharpness left
edge, count bottom-right, name along the bottom.

## Steps

- [x] Step 1: Move `.cell span.count` to the bottom-right of the image area
  - Done when:
    - The count badge renders in the bottom-right corner of the 144px image
      box, above the `.cell span.name` strip, without overlapping the name,
      the stars (top-right), the flag (top-left) or the sharpness bar (left
      edge, `top: 24px; height: 96px`), on both a landscape and a portrait
      (rotated) thumbnail and on the current cell (which shows `n/m`).
    - The badge stays legible over a bright image, using the same backdrop
      as the other badges (`text-shadow: 0 0 2px #000`, already on the rule).
    - The CSS comment above the rule describes the new position; no other
      repository text still says the badge is top-left.
    - `mise run ci` passes.
  - Implementation approach:
    - Only `crates/app/ui/style.css` changes, in the `.cell span.count` rule
      (currently `top: 2px; bottom: auto; left: 14px; width: auto; ...`, with
      the comment `/* The burst count badge, top-left beside the pick or
      reject dot. */`). Replace the placement with a bottom-right anchor in
      the style of `.cell span.rating`: `left: auto; right: 2px; width:
      auto;` and a `bottom` value (dropping the `top` override). Keep
      `font-size`, `color` and `text-shadow` as they are.
    - Geometry: a cell is 144px wide and
      `calc(var(--cell-height) - var(--cell-gap))` = 168px tall; the image
      box is the top 144px square; the name span is the base `.cell span`
      rule at `bottom: 2px` with `font-size: 0.65rem` (about y 153-166). A
      `bottom` in the 22-26px range puts the badge's bottom edge at or just
      above the image box's bottom edge (y 144), clear of the name. Record
      the chosen value and why in `learnings.md`.
    - Do not touch `strip.ts`.
    - Commit as `style(app): move the burst count badge to the bottom-right
      of the cell`.

## Trade-offs and risks

- **Overlap with a portrait thumbnail.** A portrait (rotated) thumbnail fills
  the full 144px box and the badge paints over its bottom-right corner, the
  same as the stars over its top-right; the text-shadow is the accepted
  backdrop. Accepted.
- **`docs/agents/tauri-app.md:884` wording** calls the count badge part of a
  `::before`; not about position, left alone.

## Progress

- 2026-09-22: Step 1 done (count badge at bottom: 26px; right: 2px)
