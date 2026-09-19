# Learnings

## Step 1

- `passes` in `filter.ts` iterates the `exif` map's entries instead of
  `main.ts`'s `exifGroups` list; the map is built from that list, so the
  order and coverage are unchanged.
- `anchorAfterFilter` returns `undefined` for an undefined anchor; `refilter`
  keeps its `?? 0` fallback, so the no-anchor case still lands on the first
  file.
