<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Rename the View menu's pane items to Folders / Metadata

## Purpose

The `View` menu's three check items are `Left Pane`, `Right Pane` and
`Filmstrip`. The first two name a position, the third names the content. The
left pane is the folder tree and the right pane the metadata pane, so the
items become `Folders` / `Metadata` / `Filmstrip`: every label names what it
shows. Only user-visible text changes; the keymap actions (`toggleLeft`,
`toggleRight`), the menu ids (`toggle-left`, `toggle-right`) and the stored
`panels` keys (`left`, `right`) stay, since saved shortcut overrides and the
settings store key on them.

## Steps

- [x] Step 1: Rename the labels and bring the docs in line
  - Done when:
    - `VIEW_ITEMS` in `crates/app/src/main.rs` reads `Folders` / `Metadata` /
      `Filmstrip`; the ids and actions are unchanged
    - The shortcuts panel descriptions in `crates/app/ui/src/settings.ts`
      (`toggleLeft`, `toggleRight`) read `Show / hide the folder tree` /
      `Show / hide the metadata pane` (`toggleSides: "Show / hide both side
      panes"` stays)
    - No `View > Left Pane` / `View > Right Pane` / `Left Pane` / `Right Pane`
      remains outside `docs/plans/**` and `CHANGELOG.md` (`grep -rni
      "left pane\|right pane"` is the check; positional prose that survives
      is listed below)
    - `mise run ci` passes
    - Manual check by the user on Windows (`mise run tauri:dev`): `View`
      shows `Folders`, `Metadata`, `Filmstrip` with their accelerators and
      checks, and each still toggles its pane
  - Implementation approach:
    - Code: `crates/app/src/main.rs` `VIEW_ITEMS` (the two label strings only;
      `panel_checked` and its tests key on the actions and need no change);
      `crates/app/src/shortcuts.rs` doc comments naming `View > Left Pane` /
      `Right Pane` -> `View > Folders` / `View > Metadata`;
      `crates/app/ui/src/settings.ts` the two descriptions. No other code
      carries the labels (`mcp.rs` / `companion.ts` only say "pane" for the
      compare panes, which are unrelated).
    - `docs/usage.md`: the sentence naming the `View` menu's items
      (`Folders`, `Metadata` and `Filmstrip`); the shortcut table rows (`also
      \`View > Folders\`` / `also \`View > Metadata\``, and "show / hide the
      folder tree" / "the metadata pane" to match the settings panel). The
      positional prose "the left pane (the folder tree)" / "the right pane
      (the metadata)" and the layout sentences stay.
    - `docs/agents/tauri-app.md`: the View paragraph's item list -> `Folders`,
      `Metadata` and `Filmstrip`. Positional prose elsewhere stays.
    - `todo.md` "App: real-device checks for the View menu": the labels in
      the intro and check items -> `Folders` / `Metadata`. Keep the
      plan-archive reference as is.
    - `README.md` / `README.ja.md` use layout prose only and do not name the
      menu, so no change is expected; if `README.md` is touched anyway,
      update `README.ja.md` in the same PR.
    - `CHANGELOG.md` is generated history; do not edit.

## Trade-offs and risks

- Settings panel wording: `Show / hide the folder tree` / `the metadata pane`
  (names the content, reads as a sentence) was chosen over echoing the menu
  label verbatim (`Show / hide Folders`).
- Nothing stored keys on the labels, so there is no migration risk; a user's
  saved `toggleLeft` override keeps driving `View > Folders`.

## Progress

- (2026-09-30) Step 1 complete (Windows GUI confirmation pending the user)
