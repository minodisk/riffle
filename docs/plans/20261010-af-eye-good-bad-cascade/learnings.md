# Learnings

## Step 1: Merge `OK` into `Bad`

- The single helper is `afEyeState(focus)` in `crates/app/ui/src/focus.ts`
  (`unknown` without a focus, the focus's `candidate` when it is not
  `candidate`, else `photoTier(focus) ?? "not_candidate"`). `focusMark` reads
  it, `stripState` delegates to it, and `main.ts`'s `passes` hands its result
  to `filter.ts`'s `passes()`.
- `filter.ts`'s `passes()` now takes one `afEye?: AfEye` argument (where the
  `candidate` argument was) and lost the trailing `tier` argument, so the
  filter no longer re-derives the state from the candidate and the tier.
  `AfEye` is `MarkState`, whose values equal the `data-candidate` keys of the
  menu items (`good` / `not_candidate` / `unknown`), so the icon loop in
  `main.ts` iterates the states directly. `filter.test.ts` builds its "a
  focus candidate that is not good passes `Bad`" and partition cases through
  `afEyeState`, so the mapping is pinned end to end.
- `SCAN_EYE_SVG` had no other user and was removed, with `scan-eye` from the
  Lucide notice at the top of `icons.ts`.
- The backend still serializes `candidate` / `not_candidate` / `unknown`;
  `photoTier` still requires `candidate === "candidate"`.
- `CLAUDE.md` was not edited (plan rule); the `src/focus.ts` sentence stays a
  follow-up (the plan's "CLAUDE.md follow-ups" and the `todo.md` item's second
  checkbox).
