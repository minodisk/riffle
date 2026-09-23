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

# Ignore lone modifier keys during shortcut capture

## Purpose

While a shortcut row in the settings window is capturing, pressing a modifier
key on its own (Control / Shift / Option / Command) is meant to be ignored
until the real key arrives. In the shipped app the modifier is instead
registered as a key of its own: chips such as `control`, `shift`, `alt` and
`meta` appear in the row. Once fixed, a lone modifier press registers nothing
and combos keep registering as before.

## Steps

- [x] Step 1: Make `keyName` reject lone modifiers by `event.code` and by a case-insensitive `event.key`
  - Done when:
    - Pressing a modifier key alone while a row captures adds no chip.
    - Combos (`ctrl+p`, `shift+alt+3`, `meta+,` ...) and plain keys still
      produce the same names as before; all existing tests in
      `crates/app/ui/src/keys.test.ts` pass unchanged.
    - New tests in `keys.test.ts` cover lone modifiers under the failing event
      shape: `event.key` in a different case (`control`, `shift`, `alt`,
      `meta`) with the modifier flag **not** set, and the same with the flag
      set; the right-hand codes (`ControlRight`, `ShiftRight`, `AltRight`,
      `MetaRight`) and the legacy `OSLeft` / `OSRight`; and a case where
      `event.key` is unrecognizable but `event.code` is a modifier code.
    - `mise run ci` passes.
  - Implementation approach:
    - Only `crates/app/ui/src/keys.ts` and `crates/app/ui/src/keys.test.ts`
      change. The single capture path is
      `crates/app/ui/src/settings.ts:173` (window `keydown` → `keyName` →
      `add_shortcut_key`); `crates/app/ui/src/main.ts:1441` uses the same
      function for culling, and it benefits from the same fix (a lone
      modifier no longer reaches the keymap lookup), so no change there.
    - Why the current guard misses: `MODIFIER_KEYS.includes(event.key)` is an
      exact-case match on `event.key` only. The chips are lower-case and have
      no `ctrl+` prefix, which means the plain-key branch
      (`event.key.toLowerCase()`) ran, i.e. the modifier flag was not set on
      the modifier's own keydown and `event.key` was not exactly `"Control"`
      etc. Had the flag been set, the name would have been
      `ctrl+controlleft`; guard on `event.code` so that shape is rejected too.
    - Replace the guard with one that returns `null` when either holds:
      `event.code` is one of `ControlLeft`, `ControlRight`, `ShiftLeft`,
      `ShiftRight`, `AltLeft`, `AltRight`, `MetaLeft`, `MetaRight`, `OSLeft`,
      `OSRight`; or `event.key.toLowerCase()` is one of `control`, `shift`,
      `alt`, `meta`. Keep it as a small constant list plus one `if`, matching
      the existing style of the file; update the comment above `keyName`
      accordingly.
    - Extend the `press` helper's existing "a lone modifier has no name" test
      rather than adding a new helper.

## Trade-offs and risks

- Already-persisted bogus keys (`control`, `shift`, `alt`, `meta` chips in
  the user's `shortcuts` store) are not cleaned up by this step. After the fix
  they are harmless: `keyName` can never produce those names, so they never
  match, and the user can remove them from the row. Migrating the store to
  drop them is deliberately out of scope; if the caller wants it, it is a
  separate small change in `crates/app/src/shortcuts.rs`.
- The exact webview event shape that triggered the bug was not reproduced
  (no way to drive the WKWebView from the test suite). The fix covers every
  shape consistent with the symptom (any casing of `event.key`, flag set or
  not, any modifier `event.code`), so it does not depend on pinning it down.
  If the implementer can reproduce it in the running app, note the observed
  `key` / `code` / flags in `learnings.md`.
- `event.code` may be an empty string for synthesized events; the
  `event.key` check still applies then, so nothing regresses for the existing
  test fixtures.

## Progress

- (2026-09-20) Step 1 complete
