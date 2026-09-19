# Learnings

## Step 1

- `set_auto_advance` persists synchronously like `update_keymap` (store set +
  save on the calling thread); the write is a tiny JSON file, so
  `spawn_blocking` was not worth it.
- `load_settings` now returns a 4-tuple with the `autoAdvance` bool; parsing is
  in the pure `auto_advance_setting(Option<&Value>) -> bool`.

## Step 2

- The toggle mirrors the sidecar radios (clears `#status`, shows an invoke
  failure there) rather than the fire-and-forget `debugTiming` wiring, since
  the plan asks for failures to be visible.

## Step 3

- `judge` now returns whether it changed anything; the keydown handler records
  the current path before the switch and calls `move(1)` only when the judged
  file is still current, so a `refilter` move is not doubled. `move` already
  clamps at the last file.
- Undo (#136) merged in parallel; `judge` pushes history before `commit`, so
  the return value slotted in without touching undo. The plan's "no undo yet"
  trade-off was reworded.
- The todo bullet on files dropping out of the filter was removed: the
  decision is that the `refilter` move stands and auto-advance does not add a
  second skip.
