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

# Unify the key-display formatter in keys.ts

## Purpose

The `space` -> `Space` key-display formatter exists twice in the frontend:
`crates/app/ui/src/empty.ts` exports `displayKey`, and `renderShortcuts` in
`crates/app/ui/src/settings.ts` has an inline `display` closure with the same
body. Both files already import from `crates/app/ui/src/keys.ts`, which owns
the key naming (`keyName`, `Binding`), so the display formatter belongs there
too. After this work there is one `displayKey`, exported from `keys.ts`, and
any future change to how keys are shown (e.g. modifier formatting) is made in
one place. Pure refactor; no behaviour change.

## Steps

- [x] Step 1: Move `displayKey` into `keys.ts` and use it from `empty.ts` and `settings.ts`
  - Done when:
    - `displayKey(key: string): string` is defined and exported only in
      `crates/app/ui/src/keys.ts`, placed next to `keyName` since it is the
      inverse-direction (name -> label) counterpart.
    - `crates/app/ui/src/empty.ts` no longer defines `displayKey`; it imports
      it from `./keys.js` (the existing `import type { Binding }` becomes a
      value import, e.g. `import { type Binding, displayKey } from "./keys.js";`,
      matching the style used in `settings.ts`).
    - `crates/app/ui/src/settings.ts` no longer has the local
      `const display = ...` in `renderShortcuts`; the two call sites
      (`chip.textContent` and the `Remove ...` aria-label) call `displayKey`,
      imported from `./keys.js` alongside `keyName`.
    - `crates/app/ui/src/keys.test.ts` gains a small `describe("displayKey")`
      covering `"space"` -> `"Space"` and a plain key passing through
      unchanged. `empty.test.ts` needs no change (it never imported
      `displayKey`; its `openHint` "Or press Space." case keeps covering the
      integration).
    - `grep -rn "displayKey" crates/app/ui/src` shows the definition only in
      `keys.ts`.
    - `mise run ci` passes.
  - Implementation approach:
    - Keep the function body identical (`key === "space" ? "Space" : key`);
      do not extend it to handle modifiers or other keys.
    - Do not touch `main.ts` or other callers; nothing else imports
      `displayKey`.
    - Commit as `refactor(app): move displayKey into keys.ts`.

## Trade-offs and risks

- A minimal direct `displayKey` test is added to `keys.test.ts` because the
  function now lives there; the existing indirect coverage through `openHint`
  in `empty.test.ts` stays as it is.
- No other risk: the change is import-only for callers and the ESM `.js`
  import path convention is already followed by both files.

## Progress

- Step 1: `displayKey` moved into `keys.ts` (commit `d59c301`).
