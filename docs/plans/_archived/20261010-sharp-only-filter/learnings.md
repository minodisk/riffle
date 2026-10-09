# Learnings

## Step 1

- `passes()` now looks up one key per frame, `tier ?? candidate ?? "unknown"`,
  the same fallback order as `markState`, so the four `AF eye` items partition
  the frames by construction. The existing `passes: focus candidates` tests
  pass no tier (`undefined`), so they fall through to the candidate state
  unchanged.
- The user docs had never listed the `Good` item (#754 added it to the menu
  only), so the usage / README bullets now list all four items.
- The README's icon sentence said the candidate icon is also on the filter
  menu's `Sharp` item; both `Good` and `Sharp only` carry the `scan-face` icon,
  so it now names both.
- Added a one-line note to `20261008-burst-keep-score/plan.md` Step 6, whose
  Done-when says the filter's `AF eye` text "stays", so that step keeps the
  rewritten bullet instead of reverting it.
