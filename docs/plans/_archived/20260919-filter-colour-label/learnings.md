# Learnings

## Step 1

- `passes` in `filter.ts` iterates the `exif` map's entries instead of
  `main.ts`'s `exifGroups` list; the map is built from that list, so the
  order and coverage are unchanged.
- `anchorAfterFilter` returns `undefined` for an undefined anchor; `refilter`
  keeps its `?? 0` fallback, so the no-anchor case still lands on the first
  file.

## Step 2

- `FilterState` gained `labels: Set<string>`; the label key (`none` or the
  lowercased label) is computed inside `passes`, so foreign labels fail every
  colour and `none` without a dedicated "Other" item (user decision).
- The "No label" dot is an outlined circle (`border` + `background: none`) so
  it does not read as the grey `--label-other`.

## Step 3

- todo.md had no auto-advance section (auto-advance already landed and README's
  Auto-advance entry already notes the drop-out hand-off), so only the colour
  label section was removed.
- The drop-out tests drive `anchorAfterFilter` with a predicate built from
  `passes`, where only the judged file carries the new judgement.
