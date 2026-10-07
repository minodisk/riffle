# Learnings

## Step 1

- The near-boundary test uses `p = 0.505` (closed) → `50%` and `p = 0.495`
  (open) → `51%`. Checked in Node: `Math.round((1 - p) * 100)` gives 50 / 51
  there, while `100 - Math.round(p * 100)` would give 49 / 50, so the test
  pins the formula, not only the state.
- `docs/humans/usage.md` / `usage.ja.md` also named the row in the focus mark
  paragraph and called a downcast eye `Closed`; both now say `Eyes open` and
  "counts as closed", since the value no longer carries the word.
- Code comments in `crates/app/ui/src/facemesh.ts` and `main.ts` still say
  "the `Eyes` judgment"; they name the judgment, not the row, so they were
  left as is.
