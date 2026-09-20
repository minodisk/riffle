# Learnings

## Step 1: filter menu fly-out

- CSS-only change in `crates/app/ui/style.css`. `#side` became
  `position: relative` (with a comment forbidding `z-index` / `overflow`
  there), `position: relative` moved off the shared `#filter, #sort` rule
  onto `#sort` alone, and the shared `#filter-menu, #sort-menu` rule kept
  only the look while `top` / `left` / `max-height` moved into per-menu
  rules.
- The filter menu is anchored with `left: calc(100% + 4px)` of `#side` and
  `top: 8px` (the tools row's `0.5rem` padding); `#side` spans the full
  window height, so `max-height: calc(100vh - 16px)` is the right bound.
- No DOM test covers this (vitest runs with `environment: node`), so the
  visual result is on the manual checklist in the plan.

## Deferred issues (todo candidates)

- Clicking a thumbnail while the filter menu is open still closes the menu
  (the outside-`mousedown` handler in `crates/app/ui/src/main.ts`, ~line
  1475). Now that the strip stays visible behind the fly-out, keeping the
  menu open on strip clicks may be wanted; it is a behaviour change and was
  left out of this step (noted in the plan's trade-offs).
