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

# Keep the rest of a stored shortcut override when one key conflicts

## Purpose

`Keymap::from_overrides` in `crates/app/src/shortcuts.rs` merges the stored
`shortcuts` setting over `DEFAULTS`. Today an override is applied whole or
not at all: if any one of its keys is already bound to another action, the
whole override is skipped (logged, parked in `inactive` so saving does not
drop it) and the action keeps its default.

The defaults have rotated twice (burst grouping took `arrowleft` /
`arrowright`, then the Lightroom layout moved `previous` / `next` back to
`arrowleft` / `arrowright` and gave `arrowup` / `arrowdown` to
`burstPrevious` / `burstNext`). `Keymap::add` keeps an action's existing
keys, so a user who added `w` / `a` / `h` / `k` to `previous` under the old
layout has a stored override `["arrowup", "w", "a", "h", "k"]`. Its stale
old default `arrowup` now collides with `burstPrevious`'s default, and the
user silently loses `w` / `a` / `h` / `k` too. This is the todo.md item
"App: an old shortcut override for `previous`/`next` can silently conflict
with new burst defaults".

After this work an override whose keys partly conflict still loads: the
conflicting keys are dropped (logged) and the remaining keys apply, so an old
override survives a default rotation.

Decision (user, 2026-09-29): option (B) below, persisting the filtered keys.

## Steps

- [x] Step 1: Drop only the conflicting keys of a stored override at load
  - Done when:
    - `Keymap::from_overrides(Some(&json!({"previous": ["arrowup", "w", "a", "h", "k"], "next": ["arrowdown", "s", "d", "l", "j"]})))`
      yields `previous = ["w", "a", "h", "k"]`, `next = ["s", "d", "l", "j"]`,
      `burstPrevious = ["arrowup"]`, `burstNext = ["arrowdown"]`, and
      `overrides()` returns the filtered lists, which round-trip
      (`from_overrides(Some(&keymap.overrides())) == keymap`), and a
      `log::warn!` names each dropped key and the action that holds it.
    - An override whose every key conflicts behaves as today: skipped,
      logged, kept verbatim in `inactive` (the existing tests
      `p_on_another_action_is_skipped_while_pick_holds_it`,
      `colliding_overrides_keep_the_first_in_action_order`,
      `a_collision_with_another_default_is_skipped`,
      `inactive_overrides_round_trip`,
      `a_conflicting_override_keeps_only_its_filtered_keys` still pass
      unchanged).
    - The retry loop that lets a later override release a key
      (`{"pick": ["q"], "reject": ["p"]}`, guide entry "Resolve shortcut
      overrides order-independently") still works: keys are only dropped
      after the loop has settled, never while another pending override
      could still free them (`a_pick_override_applies_and_releases_p` still
      passes).
    - The regression test above is added; `a_store_from_the_old_defaults_still_loads`
      is extended (or a sibling added) with the stale-default case so the
      next rotation is caught.
    - The module doc comment and the `from_overrides` doc comment describe
      the new rule; the guide entry in `docs/agents/tauri-app.md` ("Resolve
      shortcut overrides order-independently, not in a single pass") has its
      "When two overrides genuinely want the same key, the first in action
      order still wins" line reworded to the per-key rule (the first in
      action order keeps the key; the later override keeps its other keys).
    - The `###` section "App: an old shortcut override for `previous`/`next`
      can silently conflict with new burst defaults" is removed from
      `todo.md` (wrap-up's todo-curator would also do this; doing it in the
      PR keeps the item from lingering).
    - `mise run ci` passes.
  - Implementation approach:
    - Only `crates/app/src/shortcuts.rs` changes in code. Keep the
      `pending` / retry structure; change the final `for (i, keys) in pending`
      block: partition `keys` into those held by another action (`keymap.bindings.iter().any(|b| b.action != action && b.keys.contains(key))`)
      and the rest. If the rest is non-empty, warn per dropped key
      (`"dropping {key:?} from the shortcut for {action}: it is bound to {other}"`),
      set `keymap.bindings[i].keys` to the rest, and do not touch
      `inactive`. If the rest is empty, keep today's branch (warn, insert
      into `inactive`). Because applying a partial override never frees a
      key (it only gives the action fewer keys than requested), a second
      retry pass is not needed after the partial applications; verify this
      reasoning holds and note it in a comment. (Implementation: it does not
      hold, as a partial override replaces the action's default keys and may
      free one; one partial applies at a time and the retry loop reruns. See
      `learnings.md`.)
    - The filtered keys are what `overrides()` persists on the next save
      (they differ from the default), so the stale key leaves the store the
      first time the user edits any shortcut. That is intended: the
      dropped key is logged and the saved value is exactly what the panel
      shows. Do not add a second "partially inactive" store.
    - `Keymap::add` / `remove` / `reset` / `check_bindable` are unchanged:
      the UI still refuses to bind a key held elsewhere, so a conflict can
      only enter the store through a default rotation or a hand edit.
    - The frontend (`crates/app/ui/src/settings.ts`, `main.ts`) needs no
      change: it renders whatever `shortcuts` returns and shows backend
      errors in the status line; confirm nothing in `crates/app/ui/src`
      duplicates the conflict rule (a grep for `bound to` / `conflict`
      found none at planning time).
    - Commit as `fix(app): keep the rest of a shortcut override when one key conflicts`.

## Trade-offs and risks

Precedence rule options considered; the user chose (B):

- (A) The stored override wins; the other action's default loses that key.
  `previous` keeps `["arrowup", "w", "a", "h", "k"]`, `burstPrevious` ends
  up with no keys. Pros: the stored value is honored verbatim; this is what
  the todo item suggests. Cons: (1) an action with zero keys is a state
  nothing else produces (`remove` refuses the last key because `[]` does
  not load back, `parse_keys` rejects `[]`), so either `parse_keys` must
  accept `[]` as "unbound" (changing the `invalid_shapes_keep_the_defaults`
  contract) or the stripping must be derived state that `overrides()`
  skips (a `stripped` set on `Keymap`, cleared by `add` / `reset` on that
  action) and re-derived in `update_keymap` after every edit so that
  resetting `previous` in-session gives `burstPrevious` its key back;
  (2) in the only way the conflict arises (a rotated default left inside
  the override, since `add` refuses a key held elsewhere), the stale key
  was never a deliberate choice, so (A) takes the new burst default away
  to keep a key the user did not pick; (3) the settings panel would show an
  action with only "+" and "Reset", and Reset on it fails with
  "arrowup is bound to previous" until the user removes it from `previous`,
  which needs a hint in the panel to be discoverable.
- (B) Drop only the conflicting keys, keep the rest (chosen). Pros:
  smallest change, no new persisted shape, no unbound action, every
  existing test keeps passing, the stale old default disappears while the
  keys the user actually added survive, and the new default layout keeps
  working. Cons: a key the user did deliberately want on that action is
  lost from the override on the next save (logged, visible in the panel);
  as argued above this cannot happen through the UI, only through a hand
  edit of the store.
- (C) Apply (B) only to conflicts with another action's default and keep
  the whole-override skip for override-vs-override conflicts. Rejected:
  an override-vs-override conflict also cannot be made through the UI, and
  a uniform per-key rule is simpler to explain and to test. Under (B) the
  override-vs-override case `{"clear": ["r"], "reject": ["r"]}` behaves as
  today because `clear`'s override is left with no keys.
- Persisting the filtered keys vs. keeping the stored value verbatim in
  `inactive`: keeping it verbatim would make `previous` regain `arrowup` on
  a later launch if the user ever frees it, which is the existing
  `inactive` semantics, but it needs a third state ("applied in part")
  and the panel would show something other than what is stored. Persisting
  the filtered keys was chosen.
- The settings panel gives no notice of a dropped key beyond the log; a
  one-time status line message would need a new event and is out of scope.

## Progress

- Step 1: Implemented in `crates/app/src/shortcuts.rs`; fixed in local review
  round 1 to also recover a key freed by a later partial application. See
  `learnings.md`.
- (2026-09-29) Step 1 complete
