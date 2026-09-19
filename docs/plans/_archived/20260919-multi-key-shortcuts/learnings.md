# Learnings

## Step 1

- `rebind`'s three refusal checks moved into `Keymap::check_bindable`; `remove`
  checks `pick` itself since the `p` and conflict checks do not apply to it.
- The keys cell is no longer clickable; the `+` button enters capture mode and
  is replaced by the `Press a key...` prompt while capturing. Existing chips
  stay visible during capture.
- New error strings: `"x" is already bound to reject`, `"j" is not bound to
  reject`, `"x" is the only key for reject; use Reset`.
