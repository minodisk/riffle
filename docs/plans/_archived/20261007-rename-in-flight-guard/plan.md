<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Rename in-flight guard

## Purpose

Closes `todo.md`'s `### App: an inline rename can be re-started on a row or
cell whose rename is already in flight`, found in round 2 of the local review
of pending-rename-display
(`docs/plans/_archived/20261006-pending-rename-display/plan.md`; the problem
predates that feature).

Today, once a rename has left `IdleGate` (the scan ended, `rename_folder` /
`rename_file` is running) the tree row or strip cell still shows its pending
name and the inline editor can be started on it again. Confirming a different
name starts a second rename of the old path, which almost always fails since
the path is about to move. If a scan happens to be running by then, the
second rename is held and `markPending` draws it, but the first rename's
settle calls `clearPending(path)`, which matches by path and so clears the
second rename's pending display early.

After this work: the inline editor refuses to start, with a status note, on a
row or cell whose rename invoke is in flight (the folder tree already refuses
to *open* such a folder with `A folder is being renamed; try again in a
moment.`), and a pending mark is cleared only by the rename that set it. The
agreed re-edit behavior of a rename that is still *held* behind a scan
(edit the waiting name, replace, cancel, keep) is unchanged.

## Steps

- [x] Step 1: Refuse an inline edit on a row / cell whose rename is in flight, and key the pending mark to its own rename
  - Done when:
    - Folder tree: `folders.startRename(path)` returns without starting the
      edit, and reports `A folder is being renamed; try again in a moment.`
      through the existing `reportError`, when `renamesInFlight.blocks(path,
      ignoreCase)` holds (the renamed folder and everything under it, as
      `openFolder` already does). Both entry points (the context menu's
      `Rename…` and the slow second click) are covered because both go
      through `startRename`.
    - Strip: `strip.ts` gains `renameStarted(path)` / `renameSettled(path)`
      mirroring the tree's, and `main.ts`'s `renameFile` calls them around
      the `rename_file` invoke (before the invoke; in both `.then` branches),
      the way `renameFolder` calls `folders.renameStarted` /
      `renameSettled`. `strip.startRename(index)` returns without starting
      the edit, and reports a note (e.g. `This file is being renamed; try
      again in a moment.`), when `files[index]`'s rename is in flight. Both
      entry points (the strip's context menu `Rename…` in `main.ts` and the
      slow second click in `strip.ts`) are covered because both go through
      `startRename`.
    - Pending marks are identity-keyed: `markPending(path, name)` returns the
      mark it set and `clearPending` takes that mark, clearing only when it
      is still the one shown. `holdRename` in `main.ts` owns the mark and
      hands its clear to the run closure, so `renameFolder` / `renameFile`
      can no longer clear another rename's mark; the cancel callback clears
      only its own mark too.
    - Vitest (`rename.test.ts`, DOM-free) covers both cases: (1) the
      in-flight check used by the strip blocks the exact file path and not a
      sibling (if `RenamesInFlight` is reused for files, one file-path case
      added to its existing `describe`); (2) the identity-keyed pending slot:
      mark A, mark B on the same path, clear with A leaves B shown
      (`displayName` still gives B's name), clear with B clears, clearing
      twice is a no-op.
    - `docs/agents/tauri-app.md`'s `IdleGate` / `holdRename` note (around
      line 1428) gains a sentence on the two guards: an inline edit is
      refused while the path's rename invoke is in flight, and a pending
      mark is cleared only by the rename that set it. `docs/humans/usage.md`
      / `usage.ja.md` are unchanged (the existing open-refusal note is not
      documented there either; see Trade-offs).
    - `pnpm exec vp test` and `mise run ci` pass.
    - The todo.md item itself is closed at wrap-up (todo-curator), which
      also records the remaining GUI manual check as a new item: on a real
      device, confirm a rename while a scan runs, let the scan end, and
      while `rename_folder` / `rename_file` is still running (a large folder
      or a slow drive) try `Rename…` and the slow click on the same row /
      cell: the editor does not open and the status line shows the note;
      once the rename settles, editing works again. Also re-check that a
      rename still held behind the scan can still be re-edited (replace,
      cancel, keep) as before, in both views.
  - Implementation approach (as far as it is known):
    - `folders.ts` `startRename`: add the `renamesInFlight.blocks(...)` check
      after `cancelSlowClick()` and before the `editing` handling; reuse the
      exact message `openFolder` uses (consider a shared constant).
    - `strip.ts`: a `RenamesInFlight` instance (or a `Set<string>`) holding
      the file paths whose invoke is out; `startRename` checks it. For the
      note, add an `onError: (message: string) => void` parameter to
      `strip.init` (as `folders.init` has) and pass `setStatus` from
      `main.ts` (one call site, main.ts ~2751). The `canRename` (view-only)
      predicate stays as is.
    - `rename.ts`: a small DOM-free pending slot, e.g. `class PendingRename
      { current: Pending | null; mark(path, name): Pending; clear(mark:
      Pending): boolean }`, used by both views in place of their module
      `pending` variable; `displayName(pending.current, ...)` and
      `editOutcome(pending.current, ...)` keep working. Export it and test
      it in `rename.test.ts`.
    - `main.ts` `holdRename`: after `whenIdle` returns held, `const mark =
      view.markPending(path, name)`; the cancel callback and the run wrapper
      clear through `view.clearPending(mark)`. Change `run` to receive the
      clear, e.g. `run: (clearPending: () => void) => void`, and have
      `renameFolder` / `renameFile` call that instead of
      `folders.clearPending(path)` / `strip.clearPending(path)`. Verify the
      ordering: when not held, `run` executes synchronously inside
      `whenIdle` before any mark exists, so its clear must be a no-op then
      (e.g. a `let mark: Pending | null = null` captured by the closure).
      Keep `renaming` / `heldRename` and `cancelPendingRename` as they are.
    - Do not touch `idle.ts`; its one-slot semantics are not the issue.
    - Out of scope: `docs/plans/20261007-closed-eyes-detection/` is in
      progress in another session.

## Trade-offs and risks

- Scope of "in flight" (decided with the user, 2026-10-07): the edit is
  refused only once the invoke is out. The archived plan's agreed spec
  (items 3 and 4), `docs/humans/usage.md` and `docs/agents/tauri-app.md` all
  specify that a *held* pending row / cell can be re-edited (replace / cancel
  / keep), and that stays.
- The strip refuses with a status note (decided with the user), which needs a
  new `onError` parameter on `strip.init`, consistent with the tree's
  `openFolder` note.
- Fix 1 alone makes fix 2 unreachable from the UI (the only way a second
  rename of the same path can be issued is the now-refused re-edit). The
  identity-keyed mark is kept anyway because the acceptance criteria ask for
  it explicitly and it is the piece Vitest can cover without DOM. Cost: the
  `markPending` / `clearPending` signatures of both views change and
  `holdRename`'s `run` gains a parameter.
- The tree's `blocks` also refuses editing a subfolder of a folder whose
  rename is in flight (its path is about to move too). This matches
  `openFolder` and is intended; it is a brief window.
- `usage.md` / `usage.ja.md` are left unchanged: the refusal is a transient
  guard for a sub-second window and the existing folder-open refusal is not
  documented there either.

## Progress

- (2026-10-07) Step 1 complete
