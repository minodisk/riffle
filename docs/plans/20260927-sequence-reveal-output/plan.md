<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Reveal the sequenced output folder after a Sequence JPEG Timestamps run

## Purpose

After `File > Sequence JPEG Timestamps…` finishes, the user has to find
`<folder>-sequenced/` by hand to upload or inspect the copies; the only hint
is the status line. Once a run has written files, Riffle reveals the output
folder in the OS file manager (Explorer, Finder, or the Linux equivalent)
through the folder tree's existing `reveal_folder` command, so the copies are
in front of the user the moment the run ends. A cancelled run, a folder-level
error, or a run that wrote nothing reveals nothing.

## Steps

- [x] Step 1: Reveal the output folder when a sequence run ends with files written
  - Done when:
    - After a `sequence-done` that belongs to the current flow with
      `canceled: false` and `written > 0`, the output folder
      (`payload.output_dir`) is revealed in the OS file manager; the dialog
      closes and the status line reads as it does today. Per-file failures
      do not stop the reveal as long as at least one file was written.
    - Nothing is revealed when `canceled` is true (even if some files were
      written), when `written` is 0 (including the folder-level error, which
      arrives with `total: 0` and one failure keyed by `dir`), or when the
      payload belongs to another run (`SequenceFlow.done` returned false).
    - A failed reveal (the command rejects) shows its message on the status
      line, as the right-click reveal does, and does not otherwise change the
      done handling.
    - `crates/app/ui/src/sequence.test.ts` covers the decision for the
      success, per-file-failure, cancelled, zero-written and folder-error
      payloads; `mise run ci` passes.
    - `README.md`, `README.ja.md` and `docs/usage.md` say the output folder
      is revealed in the file manager after a run that wrote files.
  - Implementation approach:
    - No Rust change. `crates/app/src/sequence.rs` already emits
      `output_dir`, `written`, `total`, `failed` and `canceled` in
      `sequence-done`; `crates/app/src/folders.rs` already has
      `reveal_folder(path)` (opener `reveal_item_in_dir`) and `REVEAL_LABEL`.
      Do not add a new command or a new opener call.
    - `crates/app/ui/src/sequence.ts`: add one pure helper next to
      `doneStatus`, e.g. `revealAfter(done: SequenceDone): string | null`,
      returning `done.output_dir` when `!done.canceled && done.written > 0`,
      else `null`. Keep the decision in this module so it is unit-testable
      like the other status helpers; `SequenceFlow` itself needs no new
      state or method.
    - `crates/app/ui/src/sequence.test.ts`: extend the existing `done()`
      fixture-based tests with a `revealAfter` case list (success ->
      `/x/export-sequenced`; written 2 with one failure -> revealed;
      `canceled: true, written: 1` -> null; `written: 0, total: 0` with the
      folder-level failure -> null).
    - `crates/app/ui/src/main.ts`, `finishSequence` (around line 696): after
      `setStatus(doneStatus(payload))`, do
      `const out = revealAfter(payload); if (out !== null) { window.__TAURI__.core.invoke("reveal_folder", { path: out }).catch((err: unknown) => setStatus(String(err))); }`.
      This sits after the `sequenceFlow.done(payload)` gate, so early or
      foreign payloads never reveal. Add `revealAfter` to the import list
      from `./sequence`. The error path must not overwrite the done status
      unless the reveal actually fails.
    - Docs, same PR: `README.md` (lines 117-119, the sentence about the
      complete copy in `<folder>-sequenced/`) and `README.ja.md` (line 55)
      add that the folder is shown in the file manager when the run ends;
      `docs/usage.md` "Progress and cancel" (lines 221-226) adds that after a
      run that wrote files the output folder is revealed in the OS file
      manager (selected in its parent, the same as the tree's reveal item),
      and not after a cancel or when nothing was written. `CLAUDE.md` does not
      need a layout change (the `sequence.ts` description already covers
      "the run's end").
    - Verify by hand on Windows: a normal run opens an Explorer window with
      `<folder>-sequenced` selected in its parent; Escape during a run
      (cancel) opens nothing; a folder without JPEGs opens nothing and shows
      the error on the status line as before.

## Trade-offs and risks

- **Automatic reveal vs. a button in the done state.** The user asked for the
  reveal to happen after the run, and the dialog closes on `sequence-done`
  today (there is no done state to put a button in), so the plan reveals
  automatically. The user approved the automatic reveal.
- **Select in parent vs. open the folder itself.** `reveal_folder` uses
  `reveal_item_in_dir`, which opens the parent folder with
  `<folder>-sequenced` selected (that is what the tree's right-click item
  does). Opening the folder's contents directly would need a new command
  (opener `open_path`); that is a follow-up if wanted.
- **Cancel with partial output.** A cancelled run can leave complete copies
  in the output folder. The user approved not revealing on cancel.
- **Focus.** On Windows the Explorer window comes to the front, so the user
  returns to Riffle by hand. That is inherent to the automatic reveal.

## Progress

- (none yet)
