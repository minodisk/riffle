# Learnings

## Step 1

- `selection.ts` returns new `Selection` values instead of mutating, so
  callers replace the state held in `main.ts`. `extend` returns the new
  focused index alongside the selection so the clamping lives in one place.
- `prune` re-adds the focused file so the "focused is always selected"
  invariant holds after a filter change.
