# Learnings

## Step 1: paging keys

- Letter keys are matched case-insensitively (`event.key.toLowerCase()`), so
  Shift+J pages like `j`. `o` goes through the same lower-cased key, so Shift+O
  now opens the folder picker too; that is consistent rather than a regression.
- `metaKey` / `ctrlKey` / `altKey` events return before any handling, so Cmd+W
  and Cmd+O reach the system. `shiftKey` is deliberately not excluded, since
  that is what makes the case-insensitive match useful.
- `event.key` is the produced character, not the physical key, so on a
  non-QWERTY layout WASD/HJKL land on different physical keys. Accepted for
  now; switching to `event.code` would break HJKL for anyone who chose a layout
  deliberately, and nobody has asked. Noted so it is not rediscovered.
- Not verified in the running app: osascript assistive access is denied on this
  machine, so the GUI could not be driven. Only `mise run ci` (which
  type-checks `main.ts`) was run; the user confirms the keys by hand.
- Collision check against keys claimed by later phases (`1`-`5`, `x`, `Space`,
  `o`): none.

## Deferred issues (todo candidates)

- Keyboard layout dependence of the WASD/HJKL bindings (from this step's
  implementation of the `keydown` handler in
  `crates/app/ui/src/main.ts`): the bindings follow `event.key`, so a Dvorak or
  AZERTY user gets scattered physical keys. Revisit only if a user asks; the
  fix would be a `event.code` fallback or a key-config layer, both out of scope
  now.
