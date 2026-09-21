# Learnings

## Step 1

- `selection.ts` returns new `Selection` values instead of mutating, so
  callers replace the state held in `main.ts`. `extend` returns the new
  focused index alongside the selection so the clamping lives in one place.
- `prune` re-adds the focused file so the "focused is always selected"
  invariant holds after a filter change.

## Step 2

- A Shift+click moves the focus to the clicked file (and reloads the
  viewer), so the range from the anchor always contains the focused file and
  the "focused is always selected" invariant holds without a special case. A
  Cmd/Ctrl+click only changes the selection and keeps the viewer as it is.
- Pruning lives in `refilter` only: `resync` and `trashRejected` (through
  `resync`) and every filter / sort / judgement change go through it, and
  `strip.setFiles` clears the strip's copy, so `refilter` repaints it.
- `move`, `moveBurst` and the arrow keys do not collapse the selection yet
  (Step 4), so until then the focused file can move outside the selection.

## Step 3

- A command is `(focused) => (own) => State`: the outer call decides the
  value once from the focused file, the inner one applies it to each file's
  own state, so fields the command does not touch survive per file. The flag
  commands (`pick`, `unflag`) turn a reject into "no stars" per file, since a
  reject lives in the rating field; stars on the other files are kept.
- `judgements` always puts the focused file first, even when it is not in
  the selection (possible until Step 4 collapses on arrow keys), so the
  focused file is always judged.
- `judge` now returns the number of targets (0 when nothing changed), and
  auto-advance fires only when it is 1.
- The strip context menu routes its items through `runAction`, so they act
  on the selection too. A right-click on a cell outside the selection
  collapses the selection to that cell first (as file managers do); one
  inside keeps it and only moves the focus.
- No slowness measured; N `set_rating` invokes per batch as `rejectRest`.

## Step 4

- `extendPrevious` / `extendNext` sit after `burstFrameNext` rather than
  `burstNext` as the plan said: the `burstFrame*` actions landed after the
  plan was written, and this keeps the navigation actions together.
- `move`, `moveBurst` and `moveBurstFrame` collapse the selection to the new
  focused file only when the focus actually moves; a plain arrow at either end
  of the strip leaves a multi-selection as it is. This closes the Step 2 gap
  where the focused file could leave the selection.
- `shift+arrowup` / `shift+arrowdown` have no accelerator (`accelerator`
  returns `None` for shift-only keys), so they never reach a menu item; no
  menu binds them.

## Step 5

- The docs also cover the behaviours that differ from the plan: Shift+click
  moves the focus, a right-click outside the selection collapses it (inside
  keeps it), and a plain arrow at a strip end keeps a multi-selection.
- Judgement key rows in the usage Keys table still say "the current file";
  the Judgements paragraph states they apply to the selection instead of
  rewording every row.
