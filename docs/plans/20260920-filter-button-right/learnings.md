# Learnings

## Step 1

- CSS-only move: `#filter { margin-left: auto; padding-left: 0 }` added right
  after the shared `#filter, #sort` rule, and `padding-left: 0` dropped from
  `#sort` so it keeps the shared 8px left padding. `#side`, `#filter-menu` and
  `#sort-menu { left: 8px }` untouched.
- Visual confirmation is manual only (no GUI automation on this Mac), so the
  PR carries the manual checklist from the plan.

## Deferred issues (todo candidates)

- (none)
