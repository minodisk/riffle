# Learnings

## Step 1

- `doneStatus` now returns `null` for a folder-level error (it asks
  `folderError` first), so `finishSequence` only branches on `null`: the
  existing per-file loop already adds the folder-level failure to `errors`
  keyed by `dir`, and a `renderMeta()` replaces `setStatus` for that case
  (`sequencing = null` still needs a redraw).
- `failSequence` reads `sequenceFlow.dir` before `fail()` and formats the
  line with `failureText({ path: dir, message })`, the same shape as the
  run path's folder-level line. The `sequence_run` rejection now calls
  `failSequence` after closing the dialog, so all three folder-level paths
  go through it.
- `folderError` also returns null for `total: 0` with no failure (and for a
  canceled run before any file), so only the documented shape counts.
- The manual check on a JPEG-free Windows folder (orange `.error` line with
  a dismiss button, no gray note) is pending; the user runs it.
