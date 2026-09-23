# Learnings: burst-aware sharpness cue

## Step 1: Compute the sharpness cue per path over all files in capture order, by burst

- Shape chosen: `relativeSharpness(paths, lookup, radius)` in
  `crates/app/ui/src/sharpness.ts`, where `lookup` returns `SharpnessFacts`
  (`SortFacts` plus `score` and `burst`). Extending `SortFacts` lets the same
  lookup feed `orderFiles("capture", ...)` inside, as `groupBursts` does, so
  the tests can pass paths in any order with plain capture-time strings and
  the order-independence case is tested in the function itself. It returns a
  `Map<string, RelativeSharpness | null>` keyed by path; `applySharpness()`
  maps `files` through it.
- Burst members do not need the capture order (only a max per burst id), so
  the ordered pass only builds the compacted singles array and the per-burst
  max; a second pass over the input paths fills in the burst members.
- Refresh points verified: scores and `bursts` change only in
  `refreshEntries`, which already calls `applySharpness()`. `trashRejected`
  deletes scores and `resync()` replaces `allFiles` without recomputing
  `bursts`; when the displayed list does not change, `refilter` short-circuits
  and the cue is not recomputed until `scan-done` calls `refreshEntries()`.
  That window is brief and the next refresh corrects it, so no extra call was
  added.
