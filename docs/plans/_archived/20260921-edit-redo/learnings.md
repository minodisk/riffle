# Learnings

## Step 1

- `undo()` and `redo()` are the same body with the two stacks swapped, so the
  shared part became `step(from, to, verb)`; `undo`/`redo` are one-line
  wrappers. `undo`'s observable behavior is unchanged apart from the new
  `to.push(current)`.
- `swift tools/macos/export-menu-icons.swift` rewrote only the new
  `arrow.uturn.forward.png`; the existing PNGs came out byte-identical (git
  reported no change), so nothing had to be restored.
- The Edit submenu already strips both predefined items, so inserting Redo at
  index 1 needed no extra removal.

## Deferred issues (todo candidates)

- (none)
