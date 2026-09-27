# Learnings

## Step 1

- `doneStatus` now returns `null` for a folder-level error (it asks
  `folderError` first), so `finishSequence` only branches on `null`: the
  existing per-file loop already adds the folder-level failure to `errors`
  keyed by `dir`, and it calls `setStatus()` with no note for that case
  (`sequencing = null` still needs a redraw), clearing an older gray note.
- `failSequence` reads `sequenceFlow.dir` before `fail()` and formats the
  line with `failureText({ path: dir, message })`, the same shape as the
  run path's folder-level line. The `sequence_run` rejection now calls
  `failSequence` after closing the dialog, so all three folder-level paths
  go through it.
- `folderError` also returns null for `total: 0` with no failure (and for a
  canceled run before any file), so only the documented shape counts.
- The manual check on a JPEG-free Windows folder (orange `.error` line with
  a dismiss button, no gray note) is pending; the user runs it.

## Review feedback

- Round 1 (`docs/plans/review-history/fix/sequence-folder-error/review-20260928-0847.md`):
  the initial implementation called `renderMeta()` directly in `failSequence`
  and in the `status === null` branch of `finishSequence`, leaving an older
  gray `note` on screen next to the new orange error (e.g. run Sequence on
  folder A, then on a JPEG-free folder B: A's success note stayed visible
  above B's error). `renderMeta()` only redraws; it does not touch `note`,
  so the fix was to call `setStatus()` with no argument instead, which
  clears `note` and re-renders.
