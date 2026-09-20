<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Hide a kept pick while XMP is the sidecar format

## Purpose

`Index::reset_sidecars` (`crates/app/src/index.rs`) keeps the `pick` of a dirty
row across a sidecar format switch on purpose (see
`docs/plans/_archived/20260920-format-switch-pick-race/learnings.md`), and
`folder_entries` hands that `pick: true` to the frontend with no format check.
After a Dop -> Xmp switch the strip therefore draws the green flag dot, and the
"picked" filter matches the file, even though the backend's `set_rating`
(`crates/app/src/commands.rs`, `pick && rating != Some(-1) && format == Dop`)
will never persist a pick under XMP. The frontend should treat a pick the way
the backend does: it exists only while `.dop` is selected. Once done, the
strip, the flag filter and undo agree with what the sidecar will hold.

The user chose the `applyRating` gate over a backend change or a `strip.ts`-only
render gate (see Trade-offs).

## Steps

- [ ] Step 1: Gate the pick on `sidecarFormat` in the frontend, with a tested pure helper
  - Done when:
    - With `sidecarFormat === "xmp"`, a row that arrives from `folder_entries`
      with `pick: true` renders no flag dot, is not matched by the "picked"
      flag filter, and `judge`/`undo` see it as unpicked (so a later judgement
      sends `pick: false`, matching what the backend would store anyway).
    - With `sidecarFormat === "dop"`, behaviour is unchanged: the dot, the
      filter and the pick action work as before.
    - The `case "pick"` early return keeps working and now shares the same
      rule as the render path rather than a separate literal comparison.
    - A vitest unit test covers the helper for both formats (and the
      rejected case if the helper also folds in `rating === -1`).
    - `mise run ci` passes.
  - Implementation approach (as far as it is known; omit if unknown):
    - Gate at the data boundary, not at the paint: in `crates/app/ui/src/main.ts`
      `applyRating`, compute the effective pick once (e.g.
      `const kept = pick && sidecarFormat === "dop"`) and use it for both the
      `picks` set and `strip.setRating`. Because `passes`, `judge`, `undo` and
      `refilter` all read `picks.has(path)`, this keeps the strip, the flag
      filter and undo consistent in one change. `crates/app/ui/src/strip.ts`
      then needs no change: it only ever receives a pick the format allows.
      (The todo names `strip.ts`, but gating there alone would leave the
      "picked" filter matching a hidden dot; only touch `strip.ts` if the
      implementation finds a path that bypasses `applyRating`.)
    - Put the rule in a small pure module so it is testable under the
      `environment: "node"` vitest setup (`strip.ts` and `main.ts` are not
      importable there). Follow the existing pattern of `filter.ts` /
      `sort.ts` / `undo.ts` + `*.test.ts`: e.g. `crates/app/ui/src/pick.ts`
      exporting one function such as
      `pickAllowed(format: string): boolean` or
      `effectivePick(pick: boolean, format: string): boolean`, used by both
      `applyRating` and `case "pick"`. Keep it to one function; do not add a
      `SidecarFormat` type or an enum unless `pnpm exec vp check` requires it.
    - Mirror the backend rule from `commands.rs` `set_rating` exactly
      (`format == Dop`); the reject exclusion is already handled by the
      `judge` callbacks and the backend, so do not duplicate more than the
      format check unless the test makes it clearer.
    - Note for `learnings.md`: `sidecarFormat` starts as `"xmp"` and is set by
      the `sidecar_format` invoke (main.ts ~line 1356) which fires at module
      load, before `sort_order` -> `reopenLastFolder` -> `folder_entries`
      can resolve, so in practice the format is known before the first
      `applyRating`. If the implementer confirms otherwise (e.g. by logging
      order once in `pnpm exec vp dev`), the cheap fix is to call
      `refreshEntries()` from the `sidecar_format` `.then` when a folder is
      open; do not add that speculatively.
    - Files: `crates/app/ui/src/main.ts` (`applyRating`, `case "pick"`),
      new `crates/app/ui/src/pick.ts` + `crates/app/ui/src/pick.test.ts`
      (names are a suggestion). Update the comment on `picks` in `main.ts`
      only if it becomes wrong.
    - Commit as `fix(app): hide a kept pick while XMP is the sidecar format`
      (or similar Conventional Commit).

- [ ] Step 2: Close the todo and record the residual
  - Done when:
    - The "App: a pick kept across a sidecar format switch still shows its
      flag dot in the wrong format" heading and its TODO list are removed
      from `todo.md`.
    - `learnings.md` in this plan folder states where the gate lives and why
      it is in `applyRating` rather than `strip.ts`, so the next reader of
      `reset_sidecars`'s doc comment can find the frontend counterpart.
    - If the implementer judged a guide entry worthwhile (a frontend rule of
      the form "a pick is only meaningful while `.dop` is selected; gate at
      `applyRating`"), it goes in `docs/agents/tauri-app.md` next to the
      other sidecar entries; otherwise say in `learnings.md` that none was
      added and why.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Step 1 is merged. Docs-only PR (`docs(app): ...`). Do not do
      the archive move or the auto memory update here; that is the wrap-up.

## Trade-offs and risks

- Gate in `applyRating` (chosen by the user) vs. in `strip.ts` (what the todo
  literally names): gating only the render leaves `picks` true, so the
  "picked" filter would still show a file with no dot and `undo` would
  restore an invisible pick. Gating in `applyRating` fixes all three with one
  line, and `strip.ts` stays untouched.
- Frontend gate vs. backend gate (`folder_entries` stripping `pick` under
  XMP): a backend change would make every client see the same truth and
  needs no frontend format state, but `reset_sidecars` keeps the pick
  precisely so a switch back to `.dop` can restore it and so `mark_written`
  matches; filtering it in `entries` per format would need the
  `AppSidecarFormat` state in the reader path and a Rust test. The todo asks
  for the frontend fix, and the frontend already tracks `sidecarFormat`, so
  this plan keeps it frontend-only, as the user chose.
- Dropping the pick on the next judgement: once gated, a judgement made under
  XMP sends `pick: false`, and the backend row loses the kept pick. This
  already happens today (the backend forces `pick = false` under XMP
  regardless of what the frontend sends), so it is not a new loss; it is
  noted so nobody expects the pick to reappear after switching back to
  `.dop` once the file was re-judged under XMP.
- Launch ordering: `sidecarFormat` defaults to `"xmp"` until the
  `sidecar_format` invoke resolves. If `folder_entries` ever resolved first
  in `.dop` mode, real picks would be hidden until the next refresh (the
  `sidecar-format` event already reopens the folder, so a switch is safe).
  Step 1 asks the implementer to confirm the order rather than add a
  speculative refresh.
- Overlap with the in-flight `docs/plans/20260920-trash-rejected` plan,
  which also edits `main.ts`; both changes are small and in different
  regions, but rebase carefully.

## Progress

- (none yet)
