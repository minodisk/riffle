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

# Multiple keys per shortcut action

## Purpose

The keymap already stores `keys: Vec<String>` per action and several defaults
bind more than one key (`previous`, `next`), but the settings window can only
*replace* an action's keys with one captured key (`Keymap::rebind`,
`crates/app/src/shortcuts.rs`). A user who wants `x` *and* `r` for reject, or
who wants to drop `h` from `previous` while keeping the arrows, cannot. This
work lets the settings window add one key to an action and remove one
individual key, through two new backend commands that persist overrides the
same way as today.

## Decisions (agreed with the user)

- Removing the last key of an action is **refused** (use Reset).
- `rebind` / `set_shortcut` are **deleted**; add + remove supersede them.
- Adding a key the action already holds is an **error**.

## Steps

- [x] Step 1: Add and remove individual keys per action (backend + settings UI + README)
  - Done when:
    - `Keymap` in `crates/app/src/shortcuts.rs` has `add(&mut self, action, key) -> Result<(), String>` and `remove(&mut self, action, key) -> Result<(), String>`:
      - `add` appends `key` to the action's list; errors when the action already has that key; refuses with the same messages as today when `action == "pick"` (`pick is not editable`), `key == "p"` (`"p" is reserved for pick`) or the key is bound to another action (`"j" is bound to next`).
      - `remove` deletes `key` from the action's list; refuses `pick`; errors when the key is not bound to that action; refuses to remove the last key (e.g. `"x" is the only key for reject; use Reset`).
    - Two Tauri commands in `crates/app/src/commands.rs`, `add_shortcut_key(action, key)` and `remove_shortcut_key(action, key)`, both going through the existing `update_keymap`, and registered in `tauri::generate_handler!` in `crates/app/src/main.rs`. `rebind`, `set_shortcut` and its registration are removed; the `rebind_*` tests are ported to `add`.
    - Rust unit tests in `shortcuts.rs` `mod tests` cover: add keeps existing keys and appears in `overrides()`; duplicate add is an error; add of a key bound elsewhere / `p` / on `pick` is refused and leaves the keymap unchanged; remove drops one key and keeps the rest; remove of the last key is refused; remove of a key not on the action is an error; adding then removing returns to the default and `overrides()` becomes `{}`; a round trip `Keymap::from_overrides(Some(&keymap.overrides()), format)` equals the keymap after adds/removes (including under `Dop`); `reset`/`reset_all` still restore the defaults after adds/removes.
    - `crates/app/ui/src/settings.ts` renders each key of a non-`pick` row as a chip with a remove control (a small `×` button, `aria-label="Remove <key>"`) and an add control (`+` button) that puts the row into the existing capture mode (`Press a key...`, `Escape` cancels); the captured key is sent to `add_shortcut_key`. Errors keep showing in `#status` via `updateShortcuts`. `Reset` per row and `Reset all` are unchanged.
    - `crates/app/ui/settings.css` gets the minimal styling for chips / buttons (no framework).
    - README's "Keys can be changed from `Riffle > Settings...`" paragraph describes add/remove instead of "The new key replaces all of that action's keys", and mentions that the last key cannot be removed (use Reset).
    - `mise run ci` passes.
  - Implementation approach:
    - Backend: factor the three refusal checks out of `rebind` into a private helper (e.g. `fn check_bindable(&self, action, key) -> Result<usize, String>` returning the action index) and use it from `add`. Keep error strings byte-identical.
    - Persistence: `update_keymap` already saves `keymap.overrides()` and deletes the `shortcuts` key when empty; no change. Because `parse_keys` rejects an empty list on load, refusing to remove the last key keeps overrides round-trippable.
    - Known todo ("override skipped under the current format is dropped on the next rebind"): add/remove go through the same `update_keymap`, so the behavior is unchanged. Do not touch `from_overrides` for this.
    - Frontend: keep the single `capturing: string | null` state and the `window` `keydown` handler; change the invoked command to `add_shortcut_key`. The keys cell is no longer clickable as a whole. Format keys for display as today (`space` → `Space`). `main.ts`'s `applyKeymap` already flattens `keys`.
    - Frontend tests: optional (settings.ts is DOM-bound and untested today).

## Trade-offs and risks

- Refusing to remove the last key means an action whose default has keys cannot be fully unbound. Allowing it would need `parse_keys` to accept `[]` and a loader test change.
- The known todo (skipped-under-format override dropped on save) stays as is.

## Progress

- (2026-09-19) Step 1 complete
