# Learnings

## Step 1

- The reveal decision lives in `revealAfter` in `crates/app/ui/src/sequence.ts`
  and `finishSequence` in `main.ts` calls the existing `reveal_folder` command
  after the `SequenceFlow.done` gate, so early or foreign payloads never reveal.
  The error path mirrors the folder tree's right-click reveal (status line).
- The manual Windows check (Explorer opens with `<folder>-sequenced`
  selected; nothing opens on cancel or a folder without JPEGs) was not run by
  the implementation agent; it needs a hand check.

## Deferred issues (todo candidates)

- **Hand-check the reveal after a sequence run on Windows.** Change: none
  unless the check fails. Rationale: the automated tests cover only the
  `revealAfter` decision, not the opener call. Done when a human has confirmed
  that (1) a run that wrote files opens Explorer with `<folder>-sequenced`
  selected, (2) a cancelled run opens nothing, and (3) a folder with no JPEGs
  opens nothing, and any fix is merged.
