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

# Make the pick shortcut an ordinary, editable action

## Purpose

The settings window's Shortcuts tab lets every action gain, lose and reset
keys except `pick`, whose row is rendered read-only and whose `add` / `remove`
are refused in `Keymap` with `pick is not editable`. On top of that, `p` is
refused on every other action (`"p" is reserved for pick`) by a dedicated
`PICK_KEY` check in both `check_bindable` and `from_overrides`. Both were a
deliberate choice in the original shortcuts plan
(`docs/plans/_archived/20260918-customizable-shortcuts/plan.md`, "Is `pick`
rebindable at all (decided: no)"), but the user now wants pick to be no
different from any other action: `p` is simply pick's default key, protected
from other actions only by the generic "already bound to another action"
collision check, and released once pick no longer holds it.

The rest of the pipeline already handles a rebound pick: `Keymap::from_overrides`
accepts a `pick` override, `reset` / `reset_all` / `overrides()` are generic,
the Tauri commands in `crates/app/src/commands.rs` delegate to `Keymap`, and
the main window (`crates/app/ui/src/main.ts`) dispatches from the resolved
bindings with no hard-coded `p`. Removing the special cases makes pick
editable, persisted under the `shortcuts` settings key, and effective in the
main window.

## Steps

- [x] Step 1: Remove every `pick` / `p` special case from the keymap, the settings UI and the README
  - Done when:
    - In `crates/app/src/shortcuts.rs`:
      - The `PICK_KEY` constant is gone.
      - `check_bindable` no longer refuses `action == "pick"` nor `key == "p"`;
        `remove` no longer refuses `action == "pick"`. The error strings
        `pick is not editable` and `"p" is reserved for pick` no longer exist.
      - `from_overrides` no longer skips an override because it contains `p`;
        `p` on another action is skipped only when pick (or any action) still
        holds it, through the existing generic conflict check and its
        `"p" is bound to pick` warning.
      - The doc comments on `add`, `remove` and `from_overrides` no longer
        mention the reservation.
      - Tests: delete `p_on_another_action_is_skipped` and
        `only_plain_p_is_reserved` (or rewrite them to assert the generic
        behaviour); in `add_rejects_p_and_pick` and `remove_refuses_the_last_key`
        drop the assertions for the removed errors (keep the unknown-action and
        last-key assertions). Add tests that reproduce the bug and pass after
        the fix:
        - `keymap.add("pick", "q")` succeeds, keys become `["p", "q"]`, and
          `overrides()` is `{"pick": ["p", "q"]}`.
        - After adding `q`, `keymap.remove("pick", "p")` succeeds; then
          `keymap.add("reject", "p")` succeeds (`p` is free once pick releases
          it), and while pick still holds `p`, `keymap.add("reject", "p")`
          fails with `"p" is bound to pick`.
        - `Keymap::from_overrides(Some(&json!({"pick": ["q"]})), XMP)` resolves
          pick to `["q"]`; `json!({"pick": ["q"], "reject": ["p"]})` resolves
          pick to `["q"]` and reject to `["p"]`; `json!({"reject": ["p"]})`
          alone keeps the defaults because pick holds `p`.
        - `reset("pick")` restores `["p"]`, and removing pick's only key is
          refused with the generic `"p" is the only key for pick; use Reset`.
        - `overrides_round_trip` gains a pick add/remove so a pick override
          round trips through `from_overrides` under both formats.
    - In `crates/app/ui/src/settings.ts`, `renderShortcuts()` renders the
      `pick` row with the same chips, `×`, `+`, capture prompt and `Reset`
      button as the other rows: delete the `if (action === "pick")` branch so
      the loop body applies to every row.
    - `README.md`'s shortcut paragraph (the sentence "A key already used by
      another action is refused, `p` is reserved for pick, and a modifier
      pressed alone is ignored.") drops the `p` clause; no other README
      restructuring.
    - `docs/agents/tauri-app.md` does not state the reservation today (verify
      with grep); change it only if it does.
    - `mise run ci` passes.
  - Implementation approach:
    - Keep the diff surgical: only the pick / `p` special cases, their tests,
      the UI branch and the README sentence change. `forbidden`, the
      platform lists, the generic collision check, the last-key rule and the
      per-format label defaults stay as they are.
    - Manual check after the automated tests: run `mise run tauri:dev`, open
      Settings > Shortcuts, add a key to Pick and remove `p`, confirm the new
      key picks in the main window under the `.dop` format, that `p` now does
      nothing, that `p` can then be added to another action, and that the
      `shortcuts` entry in the settings store contains `"pick"`.

## Trade-offs and risks

- **`p` after pick releases it.** Once the user removes `p` from pick, `p`
  is unbound until added elsewhere; that is the ordinary behaviour of every
  other action and is what the user asked for.
- **Empty pick.** `remove` already refuses the last key of an action, so
  pick can never become unbound; no new guard is needed.
- **Existing stores.** Old `shortcuts` values never contain a `pick` key
  (the UI could not write one), so nothing migrates. A stored override that
  put `p` on another action was previously skipped by the reservation; it is
  now still skipped while pick holds `p` (generic conflict), and applied once
  a `pick` override releases `p`. This is a behaviour change only for
  hand-edited stores.
- **No UI test.** There is no `settings.test.ts`, so the reproducing test is
  a Rust unit test in `shortcuts.rs`; the UI change is verified manually.

## Progress

- (none yet)
