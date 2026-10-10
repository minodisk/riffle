# Learnings

## Step 1

- `README.ja.md`'s `AF eye` item list still read `Good` / `Sharp` / `Soft` /
  `Unknown` (it had missed the `Sharp only` rename of #756), so a plain
  `Sharp only` search did not find it; it now reads `Good` / `OK` / `Bad` /
  `Unknown` like `README.md`.
- The `not_candidate` test in `filter.test.ts` is now
  `"not_candidate (Bad) passes a bad frame only"`; its `soft` local variable
  was left as is (no assertion or body change).
