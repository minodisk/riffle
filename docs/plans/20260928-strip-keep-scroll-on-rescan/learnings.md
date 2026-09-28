# Learnings

## Step 1

- `createCell`'s listeners now read `cell.index` through a closure over the
  `cell` const declared further down the function. The listeners only run
  after `createCell` returns, so the temporal dead zone is never hit, and
  this kept the function's shape (no reordering of element creation).
- The cell under a resumed inline rename is never carried: `setFiles`
  removes its input while the `name` span is still detached, so a carried
  cell would have no `name` in its element and the resume block's
  `cell.name.replaceWith(input)` would be a no-op. It is released and
  recreated as before.
- A carried cell is repainted with the just-cleared per-index maps (rating,
  sharpness, burst, candidate) so it matches a fresh cell until `refilter`
  re-applies the values per index right after.
- Carrying happens whether or not `keepScroll` is set; without it the offset
  goes to 0 and `render()` releases the carried cells outside the new range.
- The Windows manual verification in the Done-when list (the 25 + 100 ARW
  repro, external delete of the focused file, resume landing, `View > Strip`
  toggle) is pending on the user after the PR is up; the step checkbox was
  marked on the automated criteria.
