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

# User-customizable keyboard shortcuts

## Purpose

Every culling key is hardcoded in one `keydown` handler in
`crates/app/ui/src/main.ts` (the `pagingKeys` map and the if/else chain
below it). A user who wants, say, `j`/`k` the other way round, or a different
reject key, has no way to change it. This work moves the key-to-action map
behind the settings store (`settings.json`, next to `lastFolder` and
`sidecarFormat`), stores only the user's overrides under a `shortcuts` key,
and adds a `Keyboard Shortcuts` panel opened from the menu bar that rebinds,
rejects conflicts and resets. Without a `shortcuts` key nothing changes.

Current state the plan is based on:

- The handler at `main.ts` (`window.addEventListener("keydown", ...)`)
  ignores keys held with Cmd/Ctrl/Alt, lower-cases `event.key`, and
  dispatches: previous (`ArrowLeft`, `ArrowUp`, `w`, `a`, `h`, `k`), next
  (`ArrowRight`, `ArrowDown`, `s`, `d`, `j`, `l`), `o` open, `f` focus mark,
  `Space` 1:1 zoom, `1`-`5` stars, `x` reject, `p` pick (a no-op unless
  `sidecarFormat === "dop"`), `u` un-reject/un-pick, `0` clear.
- Settings live in `tauri-plugin-store`, opened by `settings(app)` in
  `crates/app/src/commands.rs` and read once at launch by `load_settings`
  (called from `setup` in `main.rs`). The frontend has no store capability;
  the backend owns the file. Keep it that way.
- Menus are built in `crates/app/src/main.rs` (`app_menu::build`,
  `build_menu`), and a menu item talks to the frontend by emitting an event
  (`open-in-photolab`) that `main.ts` listens for.
- There is no frontend test runner (`tsc --noEmit` only), so the merge and
  conflict logic lives in Rust where `cargo test` covers it.

Design decisions taken by this plan (approved by the user on 2026-09-18):

- The keymap model is **action -> list of keys** (previous/next have six
  aliases each today). Rebinding through the UI replaces the list with the
  single pressed key.
- Key names are the normalised form the matcher uses: `event.key`
  lower-cased, except `" "` which is stored and shown as `"space"`.
  Modifier combinations are not bindable (the handler keeps ignoring them).
- `p` is reserved: the `pick` row is not editable, and `p` is rejected for
  every other action (project policy).
- The settings UI is an **in-app panel** in the main webview, not a second
  window. Reasons: the frontend has no bundler and one `index.html`; a
  second `WebviewWindow` would need its own page, capability entry and
  cross-window sync of the resolved map, while the key listener that must
  pick up the change lives in the main window anyway. The panel also
  doubles as the "capture a key" mode: while it is open, culling keys are
  not dispatched.

## Steps

- [x] Step 1: Rust keymap: defaults, override merge, resolved map command
  - Done when:
    - `crates/app/src/shortcuts.rs` defines the actions (`previous`, `next`,
      `open`, `focus`, `zoom`, `rate1`..`rate5`, `reject`, `pick`, `unflag`,
      `clear`; the string names are what `settings.json` and the frontend
      use), the defaults listed in Purpose, and a `Keymap` with
      `Keymap::from_overrides(Option<&serde_json::Value>) -> Keymap` that
      logs a `log::warn!` for, and falls back to the default of, every entry
      it cannot use: a non-object `shortcuts` value, an unknown action name,
      a value that is not a non-empty array of non-empty strings, a `p`
      bound to anything but `pick`, and an override whose key is already
      bound to another action in the map being built.
    - `load_settings` also reads `shortcuts` and the resolved `Keymap` is
      managed as `AppKeymap(Mutex<Keymap>)` beside `AppSidecarFormat`.
    - A `shortcuts` command returns the resolved map as an ordered
      `Vec<Binding { action: &'static str, keys: Vec<String> }>` (the order
      the panel will show).
    - Unit tests in `shortcuts.rs`: no overrides equals the defaults; a valid
      override replaces that action's list and leaves the others; each
      invalid shape above is skipped with the default kept; a `p` override
      on `reject` is skipped; an override colliding with another action's
      default is skipped; two overrides colliding with each other keep the
      first in action order; the defaults themselves contain no duplicate
      key.
    - `mise run ci` passes. No frontend change; behaviour is identical.
  - Implementation approach:
    - Merge rule: start from the defaults, walk the actions in their fixed
      order, apply an override only if its keys collide with no key currently
      assigned to a different action. Defaults are conflict-free and each
      accepted override preserves that, so the result never has a key bound
      twice. Order dependence (a hand-written swap `{next:["o"],open:["j"]}`
      is dropped entirely) is accepted; the UI never produces it.
    - Mirror `SidecarFormat::from_setting`'s style: missing or unusable means
      default, never an error. Use `log::warn!` (the `tauri_plugin_log`
      plugin is initialised) rather than `eprintln!`.
    - Keep defaults in Rust only; the frontend gets them through the command.

- [ ] Step 2: Frontend dispatches through the resolved map
  - Done when:
    - At launch `main.ts` invokes `shortcuts` (as it does `sidecar_format`)
      and builds a `Map<key, action>` from the result; the `keydown` handler
      becomes: ignore modifier keys as today, normalise the key, look up the
      action, `switch` on it. The `pagingKeys` map and the literal key
      comparisons are gone; `pick`'s `sidecarFormat !== "dop"` guard and the
      sticky/idempotent `judge` closures are unchanged.
    - The key normalisation is one function (`keyName(event)`), reused by
      Step 4's capture.
    - With no `shortcuts` key every key behaves exactly as before; with e.g.
      `"shortcuts": {"reject": ["r"]}` written by hand into `settings.json`,
      `r` rejects and `x` does nothing after a relaunch (**manual**, GUI
      automation is unavailable; see `docs/agents/tauri-app.md`).
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - Until the invoke resolves the map is empty and keys do nothing; that
      window is a few milliseconds at launch and avoids duplicating the
      defaults in TypeScript.
    - Add `applyKeymap(bindings)` so Steps 3-4 can swap the map without a
      restart.

- [ ] Step 3: Rust rebind, reset and persistence commands
  - Done when:
    - `Keymap::rebind(action, key) -> Result<(), String>` replaces the
      action's keys with `[key]`, and fails with a message naming the cause:
      the action is `pick` (not editable), the key is `p` and the action is
      not `pick` (reserved), or the key is bound to another action (the
      message names it, e.g. `"j" is bound to next`). `reset(action)` and
      `reset_all()` restore defaults. `overrides() -> serde_json::Value`
      returns only the actions whose list differs from the default.
    - Commands `set_shortcut(action, key)`, `reset_shortcut(action)` and
      `reset_shortcuts()` apply to `AppKeymap`, persist `overrides()` under
      `shortcuts` (removing the key when there are none), and return the new
      `Vec<Binding>`. A save failure is logged and the in-memory change
      stands, as `remember_folder` does.
    - Unit tests: rebind then `overrides()` contains only that action;
      rebind back to the default removes it; conflict, reserved-`p` and
      `pick` rejections; `reset_all()` empties `overrides()`;
      `from_overrides(overrides())` round-trips.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged. Same file as Step 1; commands next to
      `sidecar_format` in `commands.rs`, registered in `generate_handler!`.
    - The store write is the same `store.set` / `store.save` call
      `remember_folder` makes synchronously; follow it unless review asks
      for `async`.

- [ ] Step 4: Menu item and the Keyboard Shortcuts panel
  - Done when:
    - A `Settings` submenu with a `Keyboard Shortcuts...` item (accelerator
      `CmdOrCtrl+,`) exists in all builds, built in `app_menu::build` like
      `Folder`, and its handler emits `open-shortcuts`.
    - `main.ts` listens for it and shows a panel (`<div id="shortcuts"
      hidden>` in `index.html`, styled in `style.css` like the update bar):
      one row per binding from `shortcuts` (label from a TypeScript
      `action -> label` table, keys shown with `space` as `Space`), a
      per-row `Reset`, and `Reset all` and `Close` buttons. The `pick` row
      shows `p` and is not clickable.
    - Clicking a row puts it in a `Press a key...` state; the next key
      without modifiers is sent to `set_shortcut`; `Escape` cancels; a
      rejected rebind shows the command's message in the panel and leaves
      the binding as it was. While the panel is open the culling `keydown`
      dispatch is skipped.
    - Every successful command result goes through `applyKeymap`, so the
      new key works as soon as the panel closes, without a restart.
    - **(manual)** the user confirms: open from the menu, rebind `reject`
      to `r`, see the conflict message on `r` for `clear`, see the reserved
      message on `p` for `unflag`, reset one row, reset all, close, and the
      keys behave accordingly; `settings.json` holds only the overrides.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 2 and 3 are merged.
    - Panel state (`open`, `capturing: action | null`) is module-level in
      `main.ts`, or a small `shortcuts.ts` imported with a `.js` suffix
      (`docs/agents/tauri-app.md`, "Write relative imports with `.js`").
    - Conflict feedback is a text line in the panel, matching the
      status-line style already used.

- [ ] Step 5: Documentation and the user's confirmations
  - Done when:
    - `README.md`'s key table is titled as the defaults, and a short
      paragraph documents `Settings > Keyboard Shortcuts...`, the
      `shortcuts` key (overrides only, action names and key names, invalid
      entries logged and ignored), that `p` is reserved for pick, and that
      modifier combinations are not bindable. The manual results of Steps 2
      and 4 are recorded in the "confirmed / awaiting the user's
      confirmation" split.
    - `CLAUDE.md` "Layout" mentions `crates/app/src/shortcuts.rs` and the
      `shortcuts` settings key.
    - `docs/agents/tauri-app.md` gains any pitfall Steps 1-4 hit.
    - `mise run ci` passes.

## Trade-offs and risks

### Settings window versus in-app panel (decided: panel)

A native second window is the conventional "Settings" shape, but here it
costs a second HTML page with no bundler, a second capability entry, and a
window-to-window sync of the map through events, for a panel with a dozen
rows. The panel reuses the existing listener and needs none of that. Cost: it
covers the preview while open.

### One key per action versus a list (decided: list; UI rebind replaces)

Previous/next carry six aliases each today, so the stored shape is a list.
The UI writes a single key, so rebinding `next` drops the other five aliases
until reset. Swapping `j`/`k` works with replace semantics (rebind
`previous` to `k` first, then `next` to `j`).

### Is `pick` rebindable at all (decided: no)

The policy says `p` is reserved for pick. The `pick` row is read-only. The
alternative — `pick` may gain another key while `p` stays unusable elsewhere
— is a one-line relaxation in `rebind` if wanted.

### Defaults only in Rust

Between launch and the `shortcuts` invoke resolving, keys do nothing. This is
the same pattern `sidecar_format` already uses and avoids a second copy of
the defaults that would drift.

### Order-dependent merge of hand-edited overrides

Two overrides that only make sense together (a swap) are both dropped with a
warning when written by hand. A two-pass merge would accept the swap but is
more code for an input the UI cannot produce.

### Menu placement

A top-level `Settings` submenu matches the existing `Folder`/`Sidecar`
pattern. macOS convention is the app menu's `Settings...` with `Cmd+,`;
inserting into `Menu::default`'s app submenu means locating it by walking
`menu.items()`.

### Manual verification

GUI automation is impossible on this machine (`docs/agents/tauri-app.md`), so
the panel, the menu item and the "override takes effect" check are
**(manual)** for the user; unit tests cover merging, conflict detection,
reserved `p` and persistence round-trips.

## Progress

- (2026-09-19) Step 1 complete
