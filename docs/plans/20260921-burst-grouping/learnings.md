# Learnings

## Step 1

- `groupBursts` returns `Map<path, {burst, position, size}>` rather than an
  array of arrays: `main.ts` needs a per-path lookup for the `#position`
  counter and for the per-displayed-index strip marks, and Step 2's
  navigation needs a burst id per displayed file. The strip value comes from
  a separate pure `burstMarks(files, members)` (`{first, last}` or `null`),
  so the "previous / next displayed cell in the same burst" logic is
  unit-tested too and the strip never sees paths.
- The bracket is a `::after` on `.cell` in the 8px gutter right of the cell,
  reaching up across `--cell-gap` to the member above. `.cell` had
  `overflow: hidden`, which would clip it; it was removed. Nothing inside a
  cell overflows (the image box is the 144px square, rotations stay inside
  it, spans ellipsise on their own), so no visible change otherwise.
- The bracket sits inside a `.rejected` cell and is dimmed with it
  (`opacity: 0.45`); acceptable, and it keeps the bracket continuous.
- The real-folder check (Sony ARWs and Leica DNGs under `~/Downloads`) was
  not done by the implementation agent: GUI automation is unavailable in
  this environment, so the bracket placement and the observed group sizes
  still need to be confirmed by hand.

## Step 2

- `burstStep(ids, current, direction)` takes one burst id per displayed file;
  `main.ts` gives a file missing from the grouping the unique id `-1 - index`
  so it acts as a singleton. Equal ids are only treated as one burst when
  adjacent, so members separated by a filter or the rating sort are walked
  as separate runs.
- The `a_store_from_the_old_defaults_still_loads` test stored
  `arrowleft` on `previous`; with `arrowleft` now the `burstPrevious`
  default that override conflicts and is dropped, so `arrowleft` was removed
  from the test's stored list.
- `arrowleft` / `arrowright` pass `forbidden()` on both platforms (only the
  `ctrl+` variants are reserved on macOS).

## Deferred issues (todo candidates)

- Verify Step 1's burst bracket and `· N / M in burst` counter by hand on a
  real Sony and Leica burst folder and note the group sizes (basis: Step 1
  "Implementation approach", last bullet; not possible without GUI access).
  Files: `crates/app/ui/src/burst.ts`, `crates/app/ui/style.css`.
- A stored `shortcuts` override that still binds `arrowleft` / `arrowright`
  to `previous` / `next` (older defaults) now conflicts with the new burst
  defaults and the whole override for that action is ignored at load (basis:
  Step 2, `a_store_from_the_old_defaults_still_loads` had to drop
  `arrowleft`). Consider letting an override win over a default of another
  action. Files: `crates/app/src/shortcuts.rs`.
