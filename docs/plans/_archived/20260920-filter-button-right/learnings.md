# Learnings

## Step 1

- CSS-only move: `#filter { margin-left: auto; padding-left: 0 }` added right
  after the shared `#filter, #sort` rule, and `padding-left: 0` dropped from
  `#sort` so it keeps the shared 8px left padding. `#side`, `#filter-menu` and
  `#sort-menu { left: 8px }` untouched.
- Round-1 review (`docs/plans/review-history/fix/filter-button-right/review-20260920-2128.md`)
  found `margin-left: auto` alone was not enough: `#filter` is the first
  child of `#tools`, so the auto margin pushed the whole `[filter][sort]`
  group right instead of moving `#filter` past `#sort`, leaving sort as the
  right-most item. Fixed by adding `order: 1` to `#filter`, yielding
  `[8px][sort][auto][filter][8px]`.
- Visual confirmation is manual only (no GUI automation on this Mac), so the
  PR carries the manual checklist from the plan; the round-1 fix has not yet
  been visually re-confirmed.

## Deferred issues (todo candidates)

- (none)
