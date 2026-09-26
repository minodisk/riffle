# Learnings

## Step 1

- The reveal decision lives in `revealAfter` in `crates/app/ui/src/sequence.ts`
  and `finishSequence` in `main.ts` calls the existing `reveal_folder` command
  after the `SequenceFlow.done` gate, so early or foreign payloads never reveal.
  The error path mirrors the folder tree's right-click reveal (status line).
- The manual Windows check (Explorer opens with `<folder>-sequenced`
  selected; nothing opens on cancel or a folder without JPEGs) was not run by
  the implementation agent; it needs a hand check.
