<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Auto-advance on a repeated judgment

## Purpose

With `Auto-advance after a star, reject or pick` on, pressing the value the
current file already holds (`3` on a 3-star file, `x` on a rejected file,
`p` on a picked file) does nothing: no write, no undo entry, and no move to
the next file. The user wants the repeat press to move on like a new value
does, so a run of files that are already right can be confirmed with the
same key without reaching for the arrow key.

Why it stays put today: `runAction` in `crates/app/ui/src/main.ts` advances
only when `judge()` returns `1`, and `judge()` returns
`record(...) ? paths.length : 0`, where `record()` finds no change because
`judgments()` in `crates/app/ui/src/selection.ts` skips every file already
at its new state. The no-op is deliberate for the write (idempotent, no
undo entry) and should stay so; only the advance decision changes. Stars,
reject and pick are all sticky (none toggles off), so nothing about the
stored value changes. Color labels are toggles and never advance
(`crates/app/ui/src/advance.ts`); they are out of scope.

The old behavior is documented in `../../humans/usage.md` ("pressing the
value the file already has does not"), `../../humans/usage.ja.md` and
`../../agents/tauri-app.md` (the "judge returns the number of targets it
changed" note), and was decided in
`../_archived/20260919-auto-advance/plan.md` ("Re-pressing the current
value").

## Decisions (agreed with the user)

- Scope: stars, reject and pick all advance on a repeat press. Color labels
  stay as they are.
- A repeat press keeps the value (no toggle off) and writes nothing.
- Key auto-repeat: no `event.repeat` guard; a held key may also page over
  files that already hold the value.

## Steps

- [x] Step 1: Advance after a repeated star, reject or pick, with a pinned
      pure decision and the docs updated
  - Done when:
    - With auto-advance on and a single current file, pressing the star
      value, reject or pick the file already holds moves to the next file;
      the file's judgment is not rewritten (no `set_rating` invoke, no undo
      entry, status line untouched).
    - A multi-file selection, Compare, view-only folders, auto-advance off,
      and a file the judgment drops out of the filter behave as before (no
      advance / no double skip).
    - A unit test in `crates/app/ui/src/advance.test.ts` reproduces "same
      rating -> advance" through the new pure function and passes with
      `pnpm exec vp test`.
    - `docs/humans/usage.md` and `docs/humans/usage.ja.md` (the
      Auto-advance bullet) say the repeat press advances; the note in
      `docs/agents/tauri-app.md` about `judge`'s return value is corrected.
    - `mise run ci` passes.
  - Implementation approach:
    - Keep the write path idempotent: do not touch `judgments()` in
      `selection.ts` nor `record()`'s write logic; the "Idempotent" comment
      in `record()` stays true for the write and undo entry.
    - Separate "how many files the judgment addressed" from "did it change
      anything". Have `judge()` return `paths.length` whenever it got past
      the `viewOnly` guard (the number of targets, changed or not), and `0`
      when there are no files or the folder is view-only. `record()` keeps
      its boolean return, which the MCP companion's `view.judge` relies on.
      Check every caller of `judge()`'s return value: only `runAction`'s
      `judged` uses it (`rate1`-`rate5`, `reject`, `pick`).
    - The `viewOnly` check lives in `record()`, not `judge()`; either hoist
      it or have `record()` report it so a view-only folder still returns
      `0` (view-only keys "do nothing", per `usage.md`).
    - Put the advance decision in `crates/app/ui/src/advance.ts` as a pure
      function next to `advancesAfter`, e.g.
      `shouldAdvance({ action, targets, autoAdvance, comparing, stillCurrent })`
      returning `targets === 1 && autoAdvance && !comparing && advancesAfter(action) && stillCurrent`,
      and have `runAction` call it. The new test feeds `targets: 1` for an
      unchanged single file and expects `true`; also pin `targets: 0`
      (nothing targeted / view-only) and `targets: 2` (batch) as `false`.
      The existing `advancesAfter` tests stay.
    - Keep the `files[index] === current` guard: `commit`'s `refilter`
      already moves off a file the judgment filters out, and an unchanged
      file never refilters, so the repeat press always takes the `move(1)`
      branch.
    - Docs: `docs/humans/usage.md` Auto-advance bullet and the matching
      sentence in `docs/humans/usage.ja.md` in the same PR (the pair rule);
      `docs/agents/tauri-app.md` "A judgment's own move must not double up
      with `refilter`'s move" section, the trailing note on `judge`'s
      return value. README.md / README.ja.md do not describe this and stay
      untouched.
    - Hand-check with the app (auto-advance on): `3` twice on a file, `x`
      on a rejected file, `p` on a picked file, the last file, a 2-file
      selection, Compare, and a JPEG-only folder; report any not done as
      "not verified" in the PR.

## Trade-offs and risks

- `judge()` returning "targets addressed" instead of "targets changed" makes
  its meaning slightly looser; the comment and the `tauri-app.md` note have
  to say so.
- This reverses a documented, deliberate decision from the archived
  auto-advance plan on the user's request; the PR body says so and links the
  archived plan.

## Progress

- (none yet)
