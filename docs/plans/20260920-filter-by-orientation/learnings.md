# Learnings

## Step 1

- The change is frontend-only, as the plan expected: `IndexedFile.orientation`
  already reaches `entries` in `main.ts`, so `passes` only needed a fourth
  argument.
- The `filterChanged` / click-handler ternary chains now have four branches
  each (`flag` -> `label` -> `orientation` -> stars); the stars branch stays
  the fallback because it is the one without a string value.
- The two new items are text-only next to the dotted colour-label items. They
  line up with the star rows, so no CSS was added.
- **Left to the user:** the hand verification in `mise run tauri:dev` on a
  folder with both shapes (selecting `Portrait` / `Landscape` narrows the grid,
  `Reset` and unchecking restore it, the filter button lights up). A subagent
  cannot run the desktop app.

## Deferred issues (todo candidates)

- (none)
