# Learnings: split-head-pose-rows

## Step 1

- `group()` already drops a row whose value is `null`, so the three pose rows
  need no branching of their own: one `pose` picked from the stored focus cue
  or the judgment, and `pose === null ? null : degrees(...)` per row keeps
  the old absence rule.
- With `poseValue` gone, the far-profile and `-0` rounding cases are tested
  through `metaGroups` (slicing off the `Eyes` row) instead of exporting
  `degrees()`.
- Besides the plan's lines 829–839, `todo.md` line 39 (the unchecked Leica
  DNG check) also named the `Head pose` row; it was reworded too so the check
  stays accurate. The checked historical item around line 625 was left as is.
- In `usage.md` the paragraph's subject became plural (`Yaw`, `Pitch` and
  `Roll` are ...), so the following "It comes from" / "is left out" /
  "Read it" became "They come from" / "are left out" / "Read them".
- The first `mise run fmt` exited with a Node.js error from the frontend
  formatter; an immediate re-run passed with no change to the code, so it
  looked transient. `mise run ci` then passed on the first run.
