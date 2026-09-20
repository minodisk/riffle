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

# Ignore stale `.js` build output next to the UI sources

## Purpose

Before the Vite+ migration the frontend was built with `tsc`, which emitted
`.js` files next to the `.ts` sources in `crates/app/ui/src/`. Those files were
left behind, untracked, in the developer's main checkout. Because the sources
import with the `.js` suffix (`import { keyName } from "./keys.js"`), the Vite
dev server resolved the real, stale `keys.js` instead of `keys.ts`. The stale
file predates #166 and #185, so the lone-modifier guard was never served:
pressing Ctrl alone during shortcut capture registered a `control` binding, and
the fix in #185 looked as if it had not worked. The stale files have been
deleted by hand; this plan makes sure a regenerated set can never be committed,
closes the path that regenerates them, and writes the trap down where the next
agent will read it.

## Steps

- [x] Step 1: Ignore `.js` under `crates/app/ui/src/`, stop `tsc` emitting, and document the trap
  - Done when:
    - `.gitignore` contains `crates/app/ui/src/**/*.js`.
    - `touch crates/app/ui/src/keys.js && git status --porcelain` shows nothing
      for that file (then remove it again), and
      `git check-ignore -v crates/app/ui/src/keys.js` names the new rule.
    - `git ls-files | grep '\.js$'` still lists
      `.claude/skills/merge-settings/scripts/merge.js`, the only tracked `.js`
      in the repository, unaffected.
    - `crates/app/ui/tsconfig.json` has `"noEmit": true`, so a stray
      `tsc -p crates/app/ui` cannot regenerate the files, and
      `pnpm exec vp check` still passes.
    - `docs/agents/tauri-app.md` gains a `(Hit)` entry in the
      "Frontend (`crates/app/ui`, Vite+)" section, directly after "Write
      relative imports with `.js`", saying that because imports carry the
      `.js` suffix, a real `.js` beside a `.ts` shadows the source in the dev
      server; that `.gitignore` now hides such files from `git status`; and
      that when the dev server disagrees with the source, check
      `ls crates/app/ui/src/*.js` (or `git clean -nX crates/app/ui/src`)
      before debugging the code. Its `- Source:` points at this folder's
      `learnings.md`.
    - The existing "Name a modified key from `event.code`, not `event.key`"
      entry gains one sentence (or a `- Source:` bullet) noting that the
      `control` chip seen after #185 was the stale `keys.js`, not a gap in the
      guard, so the next agent does not widen the guard again.
    - `learnings.md` in this folder tells the root-cause story: the
      `tsc` -> Vite+ migration left `exif.js`, `exif.test.js`, `keys.js`,
      `main.js`, `settings.js`, `strip.js`, `worker.js` untracked in
      `crates/app/ui/src/`; Vite served the stale `keys.js` for `./keys.js`
      imports; the bogus `control` binding and the apparently ineffective #185
      followed from that, not from a code defect. Record that the user
      confirmed the shortcut capture works once the stale files were removed.
    - `mise run ci` passes.
  - Implementation approach:
    - Keep the ignore pattern narrow: a repo-wide `*.js` would ignore the
      tracked `.claude/skills/merge-settings/scripts/merge.js`. Do not add
      `*.d.ts` (`crates/app/ui/src/tauri.d.ts` is tracked and hand-written) or
      `*.js.map` (`crates/app/ui/tsconfig.json` does not emit source maps).
    - No lint check that fails when a `.js` sits beside a `.ts`: it would not
      have caught this episode (the files were only on the developer's machine,
      and CI checks out a clean tree). See Trade-offs.
    - No code change to `crates/app/ui/src/keys.ts`: it is already correct.
      Leave the existing `todo.md` item about cleaning up bogus modifier-only
      entries already persisted in the user's store out of scope.
    - Match the guide's existing format: `### Title (Hit)`, a short body,
      `- Why:` and `- Source:` bullets. `mise run lint` runs lychee over the
      new relative link.
  - Files expected to change: `.gitignore`, `crates/app/ui/tsconfig.json`,
    `docs/agents/tauri-app.md`,
    `docs/plans/20260920-ignore-stale-ui-js/{plan.md,learnings.md}`.

## Trade-offs and risks

- `.gitignore` + `noEmit` (chosen) vs. a lint check in `mise run lint` that
  fails when a `.js` sits beside a `.ts` of the same basename. The check would
  not have caught this episode and adds a shell snippet to maintain; the two
  one-line changes meet both "cannot be committed" and "cannot be regenerated".
  Revisit if a stale file ever reaches a PR.
- Ignoring the files also hides them from `git status`, so a developer who
  still has a stale set gets no signal. The guide note is the mitigation; the
  alternative (leaving them visible as untracked noise) is what allowed them to
  be overlooked in the first place.
- The guide correction risks reading as blame on #185. Keep it factual: #185's
  guard is correct, it was simply never served.

## Progress

- (2026-09-20) Step 1 complete
