# Learnings

## Step 1: Remove the Open folder button

- The removal was purely subtractive: `#side` was already a flex column and
  `#folders` already had `flex: 1; min-height: 0`, so deleting the `#open`
  rules left the tree filling the pane with no other CSS change.
- `#folders` has `padding-bottom` but no top padding; the button's margin used
  to separate the tree from the pane's top edge. The tree now starts flush at
  the top. The layout was not checked in `vp dev` from the subagent (no
  browser available), so a visual check in the app is worth doing before
  merge.
- Remaining `"open"` references (`empty.ts` keymap lookup, the `open` label in
  `settings.ts`) are the keymap action, not the button, and stay.
- On Windows, `mise run lint` fails at lychee on an untouched file,
  `docs/plans/review-history/thumbnail-cache-filmstrip-step-8/review-20260918-0253.md`
  (`#running-the-app` fragment): lychee sees the path with backslashes, so
  `--exclude-path docs/plans/review-history` does not match. The frontend
  deps were also missing in this fresh worktree (`mise exec -- pnpm install`
  fixed `vp` not found for `mise run fmt`). Ran the remaining lint steps with
  `--exclude-path review-history` (0 errors) and `mise run test` (exit 0).

## Deferred issues (todo candidates)

- `mise run lint`'s lychee `--exclude-path docs/plans/review-history` does
  not match on Windows (the path is seen with backslashes), so local CI on
  Windows fails on the review-history files. Found while running local CI for
  Step 1. Related: `mise.toml` (the `lint` task).
