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

# Momentary grayscale preview

## Purpose

Judging composition is easier without colour pulling the eye. While the
`grayscale` key is held, the viewed image is rendered in grayscale; releasing
it restores colour, so a quick press-and-release is a glance at the
composition and nothing can be left switched on by accident. It is
display-only: nothing is written to files, sidecars or the folder index, and
there is no persisted state.

## Steps

- [x] Step 1: Add a momentary `grayscale` action (default `g`) that applies a
      CSS grayscale filter to the viewer canvas while its key is held
  - Done when:
    - Holding `g` in the main window shows the viewed image in grayscale;
      releasing it restores colour. Auto-repeat while held does not flicker.
      With a rebound modified key (e.g. `ctrl+g`) the filter ends on the
      release of either the key or the modifier, whichever comes first.
      Losing window focus while the key is held ends the filter.
    - Paging, `z` and `f` behave as before, whether or not the key is held.
    - The action appears as "Grayscale" in `Riffle > Settings...` >
      shortcuts, after "1:1 zoom", and can be rebound / reset like the others.
    - `the_defaults_are_the_full_table` in `crates/app/src/shortcuts.rs`
      lists `("grayscale", "g")`; `the_defaults_bind_no_key_twice` still
      passes (the same table serves both sidecar formats).
    - README `## Keys` and `docs/usage.md` (Keys table and the viewing
      bullets next to "Focus mark" / "1:1 focus check") document the key as
      "hold".
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/app/src/shortcuts.rs`: insert `("grayscale", &["g"])` into
      `DEFAULTS` right after `("zoom", &["z"])`, plus the matching row in the
      `expected` list of `the_defaults_are_the_full_table`. Nothing else in
      Rust: the panel, overrides and accelerators derive from the table, and
      `g` is in no menu or forbidden list.
    - `crates/app/ui/src/settings.ts`: add `grayscale: "Grayscale"` to
      `shortcutLabels` after `zoom`.
    - `crates/app/ui/style.css`: `#canvas.grayscale { filter: grayscale(1); }`
      beside the `#canvas` rule (~line 484). Viewer canvas only; the strip is
      untouched.
    - `crates/app/ui/src/main.ts`:
      - Module state beside `showFocus` (~line 238): `let grayscaleHeld:
        string | null = null;` holding the `event.code` of the physical key
        that started the hold (`null` when off). A small
        `setGrayscale(code: string | null)` sets the variable and
        `canvas.classList.toggle("grayscale", code !== null)`.
      - keydown (line 1990 handler): resolve `action` as today. Handle
        `grayscale` *before* the `runAction` call rather than inside the
        `switch`, since it is the only action keyed to the physical key:
        `if (action === "grayscale") { setGrayscale(event.code);
        event.preventDefault(); return; }`. Re-setting on every auto-repeat
        keydown is idempotent (no flicker) and also re-arms the hold if a
        stray keyup ended it early. Do not add an `event.repeat` guard; the
        other actions do not use one either.
      - keyup (new `window.addEventListener("keyup", ...)`): if
        `grayscaleHeld !== null` and (`event.code === grayscaleHeld` or
        `event.code` is a modifier code), call `setGrayscale(null)`. Match on
        `event.code`, not `keyName(event)`: with a modified binding the name
        on release differs from the name on press depending on which key
        goes up first (see Trade-offs). To test the modifier check, export a
        small `isModifierCode(code)` (or `endsHold(heldCode, code)`) from
        `keys.ts`, reusing the existing `MODIFIER_CODES` list, and add a
        vitest case in `keys.test.ts`.
      - `window.addEventListener("blur", () => setGrayscale(null))` so the
        hold cannot stick when the keyup is delivered to another window.
      - Do not reset in the empty-folder branch (~line 623); the class is
        harmless on an empty canvas and the keyup/blur handlers own it.
    - Docs: README.md Keys table (`| \`g\` (hold) | grayscale preview |` after
      the `z` row); docs/usage.md Keys table and a bullet beside "**1:1 focus
      check**" (~line 27) saying it is momentary, display-only and never saved.
    - Verify by hand in `mise run tauri:dev` (no GUI automation): hold/release
      `g`; hold `g` through auto-repeat; rebind to `ctrl+g` in the settings
      window and release Ctrl first, then `g` first; hold `g` and Cmd+Tab /
      open Settings with `Cmd+,`, then come back and confirm colour is back.

## Trade-offs and risks

- Release detection by physical key, not by name: `keyName` on keyup is not
  the name that started the hold once a modifier goes up first (`ControlLeft`
  keyup names `null`; the later `g` keyup names `"g"`). Matching
  `event.code` against the code recorded at keydown, plus ending on any
  modifier keyup, covers both release orders. Cost: releasing an unrelated
  modifier (e.g. tapping Shift) while holding plain `g` ends the filter for
  an instant; the next auto-repeat keydown re-arms it. Judged acceptable over
  tracking the full modifier state.
- Plain-key hold and auto-repeat: while `g` is held the browser fires
  repeated keydowns; the handler re-applies the class each time (no-op) and
  `preventDefault` stops any default. Nothing else in the app reacts to `g`.
- Focus loss: a DOM `blur` on `window` ends the hold. If it turns out WebView2
  or WKWebView does not fire it when the native window loses focus, fall
  back to `getCurrentWindow().listen("tauri://blur", ...)` (per-window, per
  docs/agents/tauri-app.md, not the global `event.listen`). Verify by hand on
  the platforms available.
- The focus-mark crosshair (`#3f3`) is drawn on the same canvas and turns
  grey while held. Accepted; keeping it green needs a second canvas.
- Filter scope is the viewer canvas only (strip thumbnails keep colour for
  labels and flags).
- Default `g`; `m` is the free alternative. As a plain key it gets no menu
  accelerator, like `focus` and `zoom`.
- The settings panel treats the action like any other (press-to-bind); the
  label "Grayscale" plus the docs are taken as enough.

## Progress

- (2026-09-22) Step 1 complete
