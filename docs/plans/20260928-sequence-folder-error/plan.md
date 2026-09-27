<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Show a Sequence folder-level error as an error

## Purpose

In `File > Sequence JPEG Timestamps…`, picking a folder with no JPEGs
reports `no target JPEG files found in: D:\Photos\...` in the meta pane's
status area with the same gray `.note` look as "Wrote 3 of 3 files to ...",
so on real hardware it is not recognizable as an error at a glance. The app
already has one error presentation for exactly this kind of thing: the
sticky `ErrorList` (`crates/app/ui/src/errors.ts`) that `renderMeta` draws
as `#meta-status .error` lines (orange, with a dismiss button), used by the
trash, sidecar-write, scan and Sequence per-file failures. This work routes
the Sequence folder-level error through that list on both paths it can take,
so it looks like every other error in the app, and stops the misleading
"Wrote 0 of 0 files ..., 1 failed" note that the `sequence-done` path adds
today.

The two paths (from `crates/app/src/sequence.rs` and
`crates/app/ui/src/main.ts`):

- Preview: `sequence_preview` propagates `riffle_core::sequence::plan`'s
  error (`crates/core/src/sequence.rs:457`) as a rejected invoke, and
  `failSequence` in `main.ts` shows it with `setStatus(String(err))`, i.e.
  as a gray note. This is the reported case; the dialog never opens.
- Run: a folder-level error after the preview succeeded arrives in
  `sequence-done` with `total: 0`, `written: 0`, `canceled: false` and one
  failure keyed by `dir`. `finishSequence` already adds it to `errors`, but
  also sets the note to `doneStatus(payload)`, which reads
  "Wrote 0 of 0 files to X, 1 failed".

Per-file failures (`failureText`) keep their current treatment. They are
already distinct as errors on both surfaces: the `--reject-color`
`#sequence-failed` list in the preview dialog, and orange sticky `.error`
lines keyed by file path after a run, while the gray note carries the
partial-success headline ("Wrote 2 of 3 files ..., 1 failed"), which is the
right summary for a run that mostly succeeded. Nothing was reported against
them, and changing them would touch consistent, working UI.

## Steps

- [x] Step 1: Route the Sequence folder-level error through the sticky error list on both paths, with the classification as a pure function
  - Done when:
    - `crates/app/ui/src/sequence.ts` exports a pure function that decides
      whether a `SequenceDone` payload is a folder-level error (the
      `total === 0 && !canceled && failed.length > 0` shape documented at the
      top of the file) and hands back the failure to report; e.g.
      `folderError(done: SequenceDone): SequenceFailure | null`. A normal
      completion, a canceled run, a run that wrote nothing without a failure
      (`written: 0, total: 3, failed: []`) and a run with per-file failures
      all classify as not a folder-level error.
    - `doneStatus` is not shown for a folder-level error: either it returns
      `null` for that case (and `finishSequence` skips `setStatus`), or
      `finishSequence` branches on the new function first. Pick whichever
      keeps `main.ts` smallest; the pure function is the tested contract
      either way. The existing `doneStatus` test case for the folder-level
      payload (`sequence.test.ts`, "the done status") is updated to the new
      behavior rather than left asserting "Wrote 0 of 0 files".
    - `finishSequence` in `main.ts` adds the folder-level failure to
      `errors` (keyed by the folder path, `failureText` as today) and sets no
      note for it; the per-file loop and `revealAfter` stay as they are.
    - `failSequence` in `main.ts` (the preview / `pick_folder` rejection) adds
      the error to `errors` instead of `setStatus`, keyed by
      `sequenceFlow.dir` when a folder was picked, so the message renders as
      an orange `.error` line with a dismiss button like every other sticky
      error. Read `sequenceFlow.dir` before `sequenceFlow.fail()`; when no
      folder was picked yet (a `pick_folder` failure, `dir === null`), fall
      back to a fixed key such as `"sequence"`. Call `renderMeta()` after
      `errors.add`, as the `sidecar-error` listener does. The `sequence_run`
      rejection handler in `main.ts` gets the same treatment so all
      three folder-level paths look alike.
    - No new CSS class, token or HTML: `#meta-status .error` and
      `ErrorList` are reused as is. `crates/app/ui/style.css` and
      `index.html` are untouched.
    - `crates/app/ui/src/sequence.test.ts` covers the classification: the
      folder-level payload (`total: 0, failed: [{ path: dir, ... }]`), a
      normal completion, a canceled run, a run with only per-file failures,
      and (per the decision above) that per-file failures are not classified
      as folder-level.
    - The `main.ts` comment above `sequenceFlow` ("Its errors join the sticky
      `errors` list, keyed by path") is still accurate, and the
      `SequenceDone` comment in `sequence.ts` names the function that
      recognizes the shape.
    - `mise run ci` passes.
    - Manual check on the same Windows folder (`D:\Photos\tests\2026-09-28-gui-check`
      or any JPEG-free folder): the message appears as an orange error line
      with a `×` in the meta pane, and no gray note appears for it.
  - Implementation approach (as far as it is known):
    - Frontend only; `crates/core/src/sequence.rs` and
      `crates/app/src/sequence.rs` do not change. The Rust error text stays
      as it is (it already names the folder), and `failureText` prefixes
      the folder's base name the same way as for a file, which matches the
      trash and sidecar error lines (`baseName(path): message`).
    - Match the existing test style in `sequence.test.ts` (the `done()`
      helper with overrides, `test.each` for the payload table).
    - Do not change `doneStatus`'s wording for the non-error cases or the
      per-file `#sequence-failed` rendering in `showSequencePreview`.
    - README.md / README.ja.md describe the feature but not the error
      display; no doc change is expected.

## Trade-offs and risks

- Sticky `.error` line vs a transient error note. The chosen sticky
  `ErrorList` is the app's only existing error presentation in the meta
  pane and is cleared on the next folder open, so the message stays until
  the user dismisses it or opens another folder. The alternative, a new
  transient `line("error", ...)` variant of `setStatus` without a dismiss
  button, would need a second status slot in `renderMeta` and a new
  convention; it was not taken because the constraint is to reuse what
  exists.
- Keying the sticky error by the folder path means a second attempt on the
  same folder replaces the earlier line in place (see the `ErrorList` note
  in `docs/agents/tauri-app.md`); that is the intended behavior for a retry.
- The `sequence.test.ts` case that pins "Wrote 0 of 0 files ... 1 failed"
  for the folder-level payload is replaced, not kept: that string is the
  behavior being removed.
- Per-file failures are deliberately unchanged (rationale in Purpose).

## Progress

- (none yet)
