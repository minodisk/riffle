# Learnings

## Step 1

- The sequence item sits in its own group, so the folder menu draws an `hr`
  between it and the reveal item, matching the strip menu's convention of
  separating unrelated actions.
- The gate lives in one `startSequence()` in `main.ts` that both entry points
  call (`formatDialog.hidden && !settings.isOpen && sequenceFlow.start()`).
  The tree path then calls `sequenceFlow.picked(path)`, which with a non-null
  dir reaches `previewing`; no new `SequenceFlow` method, so
  `sequence.test.ts` needed no new case.
- The preview request and the shared failure handling were split out as
  `previewSequence(dir)` and `failSequence(err)` so the two paths share them.
- The HTML menu is not unit-tested; the manual checks in the plan (gate while
  settings are open, no-op while a sequencing is open, folder-level error on
  a folder without JPEGs) are left to the reviewer on a real build.
