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

# Auto-advance after a judgement

## Purpose

Culling a folder takes two keypresses per file: a judgement, then a paging
key. This work adds an **Auto-advance** setting to `Riffle > Settings...`
that, when on, moves the selection to the next file after a star rating, a
reject or a pick is applied, halving the keypresses. The setting is persisted
in the settings store (`autoAdvance` key, next to `sidecarFormat` and
`shortcuts`) so it survives a restart. With the setting off, behaviour is
unchanged. This closes the `App: auto-advance after a judgement` item in
`todo.md`.

Default: **off** (decided by the user; see Trade-offs).

## Steps

- [x] Step 1: Add the `autoAdvance` setting to the backend
  - Done when:
    - `load_settings` in `crates/app/src/commands.rs` reads `autoAdvance` from
      the settings store (missing or non-boolean falls back to the default,
      `false`) and the value is held in a managed state.
    - Two commands exist and are registered in `generate_handler!` in
      `crates/app/src/main.rs`: `auto_advance() -> bool` and
      `set_auto_advance(enabled: bool)`, which updates the state, persists
      `autoAdvance` into `settings.json`, and emits an `auto-advance` event
      with the boolean payload.
    - A Rust unit test covers the store-value-to-bool parsing helper (missing
      -> default, `true`/`false`, a non-boolean -> default).
    - `mise run ci` passes.
  - Implementation approach:
    - Mirror the `TimingLogs(AtomicBool)` / `timing_logs` / `set_timing_logs`
      pattern in `crates/app/src/main.rs`, but the state belongs in
      `commands.rs` beside `AppSidecarFormat` since it is a persisted setting
      (e.g. `pub struct AppAutoAdvance(pub AtomicBool)`), managed in `setup`
      after `load_settings`.
    - Extend `load_settings`'s return value (or add a small sibling reader)
      rather than opening the store twice; keep the "a store that cannot be
      read falls back to the defaults" behaviour.
    - Persist the way `update_keymap` does: `settings(app)` then
      `store.set("autoAdvance", enabled)` + `store.save()`; log a save
      failure with `log::warn!` and let the in-memory change stand. If the
      implementer prefers to keep file IO off the main thread, follow
      `set_sidecar_format` (async + `spawn_blocking`) instead and say which in
      `learnings.md`.
    - Keep the parsing in a small pure function (`Option<&Value> -> bool`) so
      it can be unit-tested without an `AppHandle`.
    - No capability changes expected.

- [x] Step 2: Add the Auto-advance toggle to the settings window
  - Done when:
    - `crates/app/ui/settings.html` has a new section (heading e.g.
      `Culling`) with a checkbox labelled `Auto-advance after a star, reject
      or pick`, placed before `Keyboard Shortcuts`.
    - `crates/app/ui/src/settings.ts` reads the initial state via
      `invoke("auto_advance")`, calls `set_auto_advance` on change, shows an
      invoke failure in `#status` like the sidecar radios do, and follows the
      `auto-advance` event so a reloaded window stays in sync.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - Copy the `debugTiming` checkbox wiring; no new CSS should be needed.
    - No UI framework; plain DOM as in the rest of `settings.ts`.

- [ ] Step 3: Advance in the main window, with tests and docs
  - Done when:
    - With the toggle on, pressing `1`-`5`, reject or pick on a file whose
      judgement changes moves the selection to the next file in the current
      (filtered) strip; on the last file the selection stays put; with the
      toggle off nothing changes.
    - Toggling in the settings window takes effect in the main window without
      a restart (via the `auto-advance` event), and the main window reads the
      initial value with `invoke("auto_advance")` at launch.
    - A pure helper decides whether an action advances, covered by a Vitest
      test under `crates/app/ui/src/`.
    - `README.md` describes the setting in the keys / settings section (that
      it is in `Riffle > Settings...`, which keys trigger it, that it is off by
      default, and that a file that drops out of the filter is not skipped
      twice).
    - The `App: auto-advance after a judgement` item is removed from
      `todo.md`, and the "consider this together with auto-advance" bullet
      under `App: no colour label group in the filter menu` is reworded or
      trimmed to reflect the decision recorded below.
    - `mise run ci` passes. GUI behaviour is confirmed manually by the user
      (list what to check in the PR: on/off, last file, filtered strip, reject
      with the `rejected` filter excluded); report it as "not verified" if
      not done.
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged.
    - State in `main.ts`: `let autoAdvance = false;`, set from
      `invoke<boolean>("auto_advance")` and the `auto-advance` listener,
      like `debugLogging` / `timing_logs`.
    - New module `crates/app/ui/src/advance.ts` exporting
      `advancesAfter(action: string): boolean` (true for `rate1`-`rate5`,
      `reject`, `pick`).
    - Have `judge` return whether it changed anything, and advance only when
      it did, so key auto-repeat stays harmless.
    - Do not double-skip: advance only when `files[index] === path` after
      `judge` returns; otherwise the `refilter` move is the advance.
    - Advance with the existing `move(1)`. Do not touch `strip.ts`.
    - `pick` under XMP returns before `judge` and must keep doing so.

## Trade-offs and risks

- **Default on vs off.** Off, chosen by the user: behaviour stays identical
  for existing users, and there is no undo yet.
- **Which actions advance.** Stars, reject, pick. Colour labels are toggles
  and `unflag` / `clear` / `clearlabel` are corrections, so they do not
  advance.
- **Re-pressing the current value.** `judge` returns early when the judgement
  is unchanged, so `3` on a 3-star file does not advance; a held key does not
  page through the folder.
- **Filter interaction.** A judgement that drops the file out of the active
  filter already moves the cursor via `refilter`; the plan advances only when
  the judged file is still current.
- **Sync vs async persistence in `set_auto_advance`.** Left to Step 1, noted
  in `learnings.md`.

## Progress

- (2026-09-19) Step 1 complete
- (2026-09-19) Step 2 complete
