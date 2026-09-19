# Learnings

## Step 1

- Removing the pick special cases needed no new guard: the generic
  collision check in `check_bindable` / `from_overrides` already protects `p`
  while pick holds it, and the last-key rule keeps pick from becoming unbound.
- `overrides_round_trip` binds pick to `ctrl+alt+p` rather than `q`, since the
  test already puts `q` on `previous`.
- The manual `mise run tauri:dev` check was skipped: the implementation agent
  cannot drive a GUI.
- `from_overrides` applied overrides in action order with the conflict check
  against the keymap as built so far, and `reject` precedes `pick`, so
  `{"pick": ["q"], "reject": ["p"]}` left reject on its default (the plan
  assumed the generic check would suffice). Fixed by collecting the valid
  overrides first and retrying the conflicting ones until none applies, so a
  key released by a later override is picked up; the first-in-action-order
  rule for two overrides fighting over one key still holds.
