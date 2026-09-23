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

# Shortcut key names for any modifier combination

## Purpose

`keyName` in `crates/app/ui/src/keys.ts` only knows two shapes: the bare
`event.key` (lower-cased, `" "` as `space`) and `ctrl+alt+<code>` for
Ctrl+Alt. Shift is folded away (Shift+J acts as `j`), and
`isUnboundModifier` drops every Meta, Ctrl-only and Alt-only press before the
keymap sees it, so none of those can be bound from the settings window. This
work generalizes the name to `ctrl+alt+shift+meta+<key>` (fixed order, only the
modifiers held, key named from `event.code` whenever a modifier is held), so
any combination can be bound, while keeping the stored `ctrl+alt+N` names
byte-identical, refusing a lone modifier, and refusing the combinations the
system and the app's own menu already own.

Current state the plan is based on:

- `keyName` / `isUnboundModifier` are used only by the main window's keydown
  handler (`crates/app/ui/src/main.ts`, `applyKeymap` and the listener right
  after it) and the settings window's capture (`crates/app/ui/src/settings.ts`,
  the `keydown` listener at the end of the file). The main handler dispatches
  only keys present in `keymap`, so a key that is never stored is passed
  through to the system at runtime with no extra code.
- The settings capture already skips a lone modifier by `event.key`
  (`Control`, `Alt`, `Shift`, `Meta`) and shows any backend error in
  `#status`; `Escape` cancels the capture.
- `crates/app/src/shortcuts.rs` validates keys in two places: `check_bindable`
  (behind `add`) returns an `Err(String)` the settings window displays, and
  `from_overrides` logs and keeps the default. `PICK_KEY` (`"p"`) is the
  existing model for a reserved key. Key names are opaque strings otherwise.
- The app's menu (`crates/app/src/main.rs`, `app_menu::build` on top of
  `tauri::menu::Menu::default`) owns these accelerators. macOS: `Cmd+,`
  (Settings...), `Cmd+Z` (the app's Undo; the predefined Undo/Redo pair is
  removed, so `Shift+Cmd+Z` belongs to nothing), `Cmd+Q`, `Cmd+H`,
  `Cmd+Alt+H`, `Cmd+W`, `Cmd+M`, `Ctrl+Cmd+F`, `Cmd+X`, `Cmd+C`, `Cmd+V`,
  `Cmd+A`. Windows/Linux: `Ctrl+,`, `Ctrl+Z`, `Ctrl+X/C/V/A`, `Ctrl+M`,
  `Alt+F4`.
- Vitest runs under `environment: "node"` (root `vite.config.ts`), so
  `KeyboardEvent` is not available to tests.

## Steps

- [x] Step 1: Generalize the key names, refuse lone modifiers and the system / menu combinations, update tests and docs
  - Done when:
    - `keyName` returns `ctrl+alt+shift+meta+<key>` with only the held
      modifiers, in that order; with no modifier held the name is unchanged
      (`event.key` lower-cased, `" "` -> `space`); with any modifier held the
      key part comes from `event.code` (`Digit1` -> `1`, `KeyJ` -> `j`,
      `Numpad*` stripped as today, `Minus` -> `-`, `Equal` -> `=`, `Space` ->
      `space`, and the other punctuation codes mapped to the character the
      unmodified `event.key` would give: `Comma` -> `,`, `Period` -> `.`,
      `Slash` -> `/`, `Semicolon` -> `;`, `Quote` -> `'`, `BracketLeft` ->
      `[`, `BracketRight` -> `]`, `Backslash` -> `\`, `Backquote` -> `` ` ``).
      `Ctrl+Alt+1` still yields exactly `ctrl+alt+1`, so every stored override
      and the `.dop` label defaults keep working without migration.
    - `shift+j` and `j` are distinct names; Shift+J no longer pages.
    - A press of Control / Alt / Shift / Meta alone is never named or bound,
      in both the main window and the settings capture.
    - `crates/app/src/shortcuts.rs` refuses a forbidden combination in `add`
      (an `Err` string in the style of the existing ones, e.g.
      `"meta+q" is reserved by the system` / `"meta+z" is a menu accelerator`)
      and skips it in `from_overrides` with a `log::warn!`, keeping the
      default. The list covers every accelerator in the app's menu on the
      running platform and a reasonable set of OS shortcuts (see the approach).
    - The settings window shows the refusal in `#status` exactly as it shows
      the other `add_shortcut_key` errors; no new UI element.
    - New `crates/app/ui/src/keys.test.ts` covers: plain keys (`j`, `space`,
      `arrowleft`, `escape`), `shift+j`, `ctrl+alt+1` (compatibility, with
      and without `event.key` being `¡`), `ctrl+alt+shift+1`, `meta+k`,
      `ctrl+alt+shift+meta+k` ordering, `Minus`/`Comma`-style punctuation
      under a modifier, and a lone modifier returning nothing.
    - Rust tests in `shortcuts.rs` cover: a forbidden combo refused by `add`
      with the expected message, skipped by `from_overrides`, `meta+k` /
      `shift+j` / `ctrl+j` / `alt+j` accepted, and `ctrl+alt+1` still
      accepted; the platform-dependent list is tested through a function that
      takes the platform as a parameter so both lists run on every CI OS.
    - `README.md`'s "Keys can be changed..." paragraph describes the new rule
      (any modifier combination; Shift distinguished; system and menu
      shortcuts refused) in user-facing terms; the label table's
      `Ctrl+Alt+N` entries are unchanged. `docs/agents/tauri-app.md`'s
      "Name a Ctrl+Alt key from `event.code`" item is retitled and reworded
      for "a modified key" (the lesson, naming from `event.code` and skipping a
      lone modifier by `event.key`, still holds). The `DEFAULTS` doc comment
      in `shortcuts.rs` states the new name grammar.
    - `mise run ci` passes.
  - Implementation approach:
    - `keys.ts`: type the parameter structurally, e.g.
      `Pick<KeyboardEvent, "key" | "code" | "ctrlKey" | "altKey" | "shiftKey" | "metaKey">`,
      so tests pass plain objects under the node environment. Return
      `string | null` for a lone modifier, detected by `event.key` in
      `Control` / `Alt` / `Shift` / `Meta`, not by the derived code (see the
      `ctrl+alt+altleft` lesson in `docs/agents/tauri-app.md`). Delete
      `isUnboundModifier`; both callers early-return on null instead. The
      settings capture's own `["Shift","Meta","Control","Alt"].includes(event.key)`
      check becomes redundant and is removed with it. Keep `escape` as the
      capture-cancel key by its plain name.
    - Keep the `Numpad` handling as it is today; do not widen it.
    - Single source of truth for the forbidden list is Rust, in
      `shortcuts.rs`, next to `PICK_KEY`: no TypeScript copy. Add a
      `fn forbidden(key: &str, macos: bool) -> Option<&'static str>` (reason
      text) called from `check_bindable` and `from_overrides` with
      `cfg!(target_os = "macos")`. Names in the list must be spelled exactly as
      `keyName` produces them (`meta+,` not `meta+comma`).
    - macOS list (menu): `meta+,`, `meta+z`, `meta+q`, `meta+h`, `alt+meta+h`,
      `meta+w`, `meta+m`, `ctrl+meta+f`, `meta+x`, `meta+c`, `meta+v`,
      `meta+a`. macOS list (system): `meta+tab`, `shift+meta+tab`,
      `meta+space`, `alt+meta+space`, `ctrl+space`, ``meta+` ``,
      ``shift+meta+` ``, `alt+meta+escape`, `ctrl+meta+q`, `shift+meta+q`,
      `shift+meta+3`, `shift+meta+4`, `shift+meta+5`, `alt+meta+d`,
      `ctrl+meta+space`, `ctrl+arrowleft`, `ctrl+arrowright`, `ctrl+arrowup`,
      `ctrl+arrowdown`. Leave `meta+arrow*` bindable. Record the final list in
      a comment naming where each entry comes from (menu vs OS).
    - Windows/Linux list: `ctrl+,`, `ctrl+z`, `ctrl+x`, `ctrl+c`, `ctrl+v`,
      `ctrl+a`, `ctrl+m`, `alt+f4`, `alt+tab`, `shift+alt+tab`, `ctrl+escape`,
      `ctrl+shift+escape`, and every name containing `meta+` (the Windows /
      Super key is owned by the shell).
    - Manual confirmation for the user (GUI automation does not work on this
      Mac): `Ctrl+Alt+1` under `.dop` still sets red; `Shift+J` no longer
      pages; binding `meta+k` from the settings window works in the main
      window; trying to bind `Cmd+W` while capturing shows the refusal in
      `#status` (or, if the OS closes the settings window first, record that
      in `learnings.md`).
    - Files: `crates/app/ui/src/keys.ts`, new `crates/app/ui/src/keys.test.ts`,
      `crates/app/ui/src/main.ts`, `crates/app/ui/src/settings.ts`,
      `crates/app/src/shortcuts.rs`, `README.md`, `docs/agents/tauri-app.md`.

## Trade-offs and risks

- One step, single PR: the naming change alone would ship a window in which
  `Cmd+Q` etc. are bindable.
- The forbidden list lives in Rust only; the frontend already displays the
  `add` error, and the main window never finds a refused key in the keymap.
- The macOS system-shortcut list can never be complete (users remap them in
  System Settings). `shift+meta+z` is deliberately not forbidden because the
  app removes the Redo item.
- Visible behavior change: Shift+letter no longer triggers the unshifted
  binding, and `ctrl+alt+shift+x` is now distinct from `ctrl+alt+x`.
- A forbidden combination pressed during capture may be consumed by the OS or
  the menu before the page sees it (`Cmd+W`, `Cmd+Q`); the `from_overrides`
  check protects against a hand-edited store.
- `AltGr` on Windows reports Ctrl+Alt; unchanged from today and out of scope.

## Progress

- (2026-09-20) Step 1 complete
