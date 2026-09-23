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

# Default shortcuts cleanup

## Purpose

The default keymap binds six keys each to previous / next, uses `Space` for
the 1:1 zoom, and gives the color labels different defaults under XMP
(Lightroom's `6`-`9`, `-`) and `.dop` (PhotoLab's `Ctrl+Alt+1`-`7`, `0`).
Riffle's strip runs vertically, so only the up / down arrows are needed; `z`
follows Lightroom for zoom; and one PhotoLab-style label keymap under both
formats removes the format branch from the keymap, the format switch and both
windows' re-fetch of the shortcuts. Users who have overridden keys keep their
overrides.

## Steps

- [x] Step 1: Trim the default keys, unify the label defaults across formats, and drop the format dependency from the keymap
  - Done when:
    - `DEFAULTS` in `crates/app/src/shortcuts.rs` is `previous ["arrowup"]`,
      `next ["arrowdown"]`, `open ["o"]`, `focus ["f"]`, `zoom ["z"]`,
      `rate1`..`rate5 ["1"]`..`["5"]`, `reject ["x"]`, `pick ["p"]`,
      `unflag ["u"]`, `clear ["0"]`, then `red ["ctrl+alt+1"]`,
      `orange ["ctrl+alt+2"]`, `yellow ["ctrl+alt+3"]`, `green ["ctrl+alt+4"]`,
      `blue ["ctrl+alt+5"]`, `pink ["ctrl+alt+6"]`, `purple ["ctrl+alt+7"]`,
      `clearlabel ["ctrl+alt+0"]`, in that order (the shortcuts panel order is
      unchanged). `LABEL_DEFAULTS` and `default_keys(format)` are gone.
    - The keymap no longer takes or stores a `SidecarFormat`.
    - A stored `shortcuts` value written by the old version still loads: no
      panic, every parseable entry either applies or is kept as an inactive
      override exactly as today.
    - The defaults bind no key twice (existing test `the_defaults_bind_no_key_twice`).
    - `README.md` and the UI comments describe the new defaults; no text
      claims the label keys follow the format.
    - `mise run ci` passes.
  - Implementation approach:
    - **Keymap (`crates/app/src/shortcuts.rs`)**: fold the label rows into
      `DEFAULTS` with their `ctrl+alt+N` keys; iterate `DEFAULTS` directly.
      Remove the `format` field and parameter: `Keymap::defaults()`,
      `Keymap::from_overrides(overrides: Option<&Value>)`. Keep the
      `inactive` map and its handling unchanged: with no format branch it still
      preserves a stored override that collides with a default (or another
      override) so a save does not silently drop it, which is what keeps old
      stores intact (e.g. an old `{"zoom": ["space", "z"]}` still applies;
      an old override on another action that used `z`, `arrowup` or
      `arrowdown` now collides with a default and is kept inactive rather
      than deleted). Reword the module doc, the `Keymap` doc and the
      `inactive` doc ("another format can still apply them") and the
      `from_overrides` doc ("checked against `format`'s defaults") to drop
      the format wording. `use crate::sidecar::SidecarFormat` becomes unused
      in the non-test code; remove it (and the `XMP` / `DOP` test consts).
    - **Callers (`crates/app/src/commands.rs`, `crates/app/src/main.rs`)**:
      `load_settings` calls `Keymap::defaults()` / `Keymap::from_overrides(overrides.as_ref())`.
      `switch_sidecar_format` no longer rebuilds `AppKeymap` (lines 401-404),
      so `AppShortcutOverrides` has no reader: remove the struct, its
      `app.manage(...)` in `main.rs`, the write in `update_keymap`, and the
      `Option<Value>` element of `load_settings`'s return tuple (update its
      doc comment, which says the overrides are returned "so a format switch
      can rebuild the keymap"). Verify with `cargo build` / clippy that
      nothing else reads it.
    - **Frontend (`crates/app/ui/src/main.ts`, `crates/app/ui/src/settings.ts`)**:
      delete the `shortcuts` re-fetch inside the `sidecar-format` listeners
      (`main.ts:1194-1199`, `settings.ts:118-126`) and their "The label keys'
      defaults follow the format" comments; the `shortcuts-changed` event
      already covers edits. Keep `keys.ts`'s `" "` -> `"space"` naming and
      `settings.ts`'s `Space` display (space stays a bindable key; `keys.test.ts`
      unchanged). Reword the comments that name `Space` as the zoom key
      (`main.ts:206`, `755`, `845`, `1320`) to "the zoom key" / "`z`".
    - **Docs (`README.md`)**: rewrite the `### Keys` table rows for previous
      (`ArrowUp`), next (`ArrowDown`) and zoom (`z`); replace the two-column
      label table and its intro with a single-column table of the
      `Ctrl+Alt+N` keys (no longer "follow the format"); fix the feature
      bullet at line 49 (`Space` -> `z`); drop the sentences at lines 137-140
      about format-specific defaults and "the current format's defaults"
      (`Reset all` restores the defaults). The benchmark prose at lines
      344/378/382 describes the zoom keypress: change `Space` to "the zoom key"
      (or `z`) so no stale key name remains. Leave `CHANGELOG.md` (generated).
    - **Tests in `shortcuts.rs`** (exhaustive list of those that reference
      old defaults or the format; all other tests compile after dropping the
      `format` argument):
      - Delete or rewrite: `the_label_defaults_follow_the_format` -> a test
        asserting the full default table (action order and keys) including
        that `clear` is `0` and `clearlabel` is `ctrl+alt+0`;
        `a_label_override_applies_under_both_formats`,
        `a_collision_is_checked_against_the_current_formats_defaults`,
        `overrides_and_reset_use_the_current_formats_defaults` -> fold into
        format-free equivalents (a label override applies; `add`/`reset`/
        `remove`/`reset_all` on `red` against `ctrl+alt+1`).
      - `ctrl_alt_keys_are_bindable`: `reject: ["ctrl+alt+1"]` now collides
        with `red` under the only keymap; use a free combination such as
        `ctrl+alt+r` for the positive case and keep `ctrl+alt+1` as the
        skipped case.
      - `the_defaults_bind_no_key_twice`: drop the format loop.
      - `a_collision_with_another_default_is_skipped`,
        `add_rejects_a_key_bound_to_another_action`,
        `remove_rejects_a_key_not_on_the_action`: `j` is no longer bound;
        use `arrowdown` (bound to next) or `z`.
      - `remove_drops_one_key`, `reset_restores_the_defaults`,
        `overrides_round_trip`: `previous` has one key now (`h` is gone and
        removing the last key is refused); add a key first, or use an action
        with a temporarily added second key.
      - `overrides_round_trip`, `an_inactive_override_survives_unrelated_edits`:
        `add("zoom", "z")` now errors (already bound); use another key and
        replace `remove("zoom", "space")` accordingly.
      - `an_inactive_override_survives_unrelated_edits`,
        `editing_or_resetting_an_action_drops_its_inactive_override`,
        `inactive_overrides_round_trip`, `p_on_another_action_is_skipped_while_pick_holds_it`
        (fine as is): the stored `{"reject": ["6"]}` no longer collides (`6`
        is free), so pick a key that collides with a default, e.g.
        `{"reject": ["arrowup"]}`; the DOP re-load assertions go away.
        `{"clear": ["j"]}` now applies, so use a colliding key there too if
        the test needs it to stay inactive.
      - Add one regression test that an old-format store loads without
        panicking and keeps its entries: e.g.
        `{"zoom": ["space", "z"], "red": ["6"], "previous": ["arrowleft", "arrowup", "w", "a", "h", "k"]}`
        resolves with each override applied (none collide with the new
        defaults) and `overrides()` round-trips.
    - Run `mise run ci` (Rust fmt, clippy, tests; `pnpm exec vp check/test`).

## Trade-offs and risks

- **Dropping the `SidecarFormat` parameter** touches `shortcuts.rs`,
  `commands.rs` (`load_settings`, `switch_sidecar_format`, `update_keymap`,
  `AppShortcutOverrides`), `main.rs` (one `manage` line) and the two frontend
  listeners; all sites were found by grep and the compiler catches any miss, so
  the cascade is contained. Keeping the parameter would leave a dead branch,
  dead state (`AppShortcutOverrides`) and two dead re-fetches whose only
  purpose was the per-format label defaults, so the plan drops it.
- **`inactive` is kept.** Its doc justifies it by "another format", but it also
  prevents a colliding stored override from being dropped on the next save.
  With the new defaults, old user overrides that used `z`, `arrowup` or
  `arrowdown` on other actions will now collide and become inactive; keeping
  `inactive` preserves them in the store, which satisfies "do not break saved
  overrides".
- **Users who relied on the removed defaults** (`Space`, left/right arrows,
  `hjkl`/`wasd`) lose them unless they had explicitly overridden the action;
  an override that listed them keeps working. No migration, as agreed.
- **`open` stays on `o`.** Moving it into the File menu with a
  `Cmd+O` / `Ctrl+O` accelerator that stays rebindable is wanted, but it needs
  platform-dependent defaults, a key-name-to-accelerator conversion, a rule for
  which of an action's keys becomes the accelerator, and an exemption in
  `forbidden()` so the app's own accelerator is not refused. That is a separate
  change and is deferred to its own PR.

## Progress

- (2026-09-20) Step 1 complete: trimmed the default keys, unified the label defaults across
  sidecar formats, and dropped the format dependency from the keymap. Landed
  in `e4c70ed` (feat(app): trim the default keys and unify the label
  defaults).
