# Learnings

## Step 1

- The facts moved to `docs/cameras.md` match `crates/core/src/sharpness.rs`
  at HEAD: `trusted_focus` drops the AF point only when Sony `FocusMode` is 0,
  and `eye_af_frame` also requires the AF point off the exact sensor center
  (bodies leave it there when tracking never locked). That last detail was
  left out of the page as too fine for user docs.
- `docs/usage.md` bullets got the pointer as a trailing "See ..." sentence
  (Sharpness cue, Bursts) or a parenthetical (Focus mark), so their wording
  stays untouched.

## Deferred issues (todo candidates)

- `docs/usage.md` still names cameras in the Focus mark, Sharpness cue and
  Bursts bullets (Sony, SIGMA BF, M11-P, Leica); the plan kept them on
  purpose (links only). If the README's camera-neutral wording should extend
  to the detailed doc, reword those bullets. Basis: plan Step 1 trade-off
  "`docs/usage.md` still names cameras". File: `docs/usage.md`.
