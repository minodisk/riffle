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

# Undo / Redo: move the selection with the focus

## Purpose

`Edit > Undo` / `Redo` of a single file's judgment makes that file current
again, but the strip's selection is left where it was: judge A, move to B,
Ctrl+Z shows A while B's cell keeps `.selected` (confirmed on Windows,
`class="cell selected"` on B in DevTools). The cause is in
`crates/app/ui/src/main.ts` `step()`: its single-file branch does
`index = at; show();` without `selection = single(...)` /
`paintSelection()`, unlike every other focus move (`move`, `moveBurst`,
`moveBurstFrame`, the context-menu click). `refilter()` returns early when
the list is unchanged, so `prune` never runs and `selection` stays `{B}`.

Beyond the stale highlight, `selection` is what the judgment keys act on
(`targets(selection, files, index)`), so a stale selection also misdirects
the next key: when the list does change (a judgment filter or rating sort),
`refilter`'s `prune` keeps B and adds A, giving `{B, A}`, and the next
judgment key hits B as well; Shift+Arrow right after the undo also ranges
from B's anchor. After this work, a single-file undo / redo that moves the
focus leaves the selection equal to the focused file, as the arrow keys do.

## Steps

- [x] Step 1: Reset the selection when a single-file undo / redo moves the focus
  - Done when:
    - A pure helper in `crates/app/ui/src/selection.ts` decides the
      selection after a single-file undo / redo, from the current selection,
      the visible `files`, the file that was shown before the step
      (`shownPath`) and the restored file's `path`: when `path` is in `files`
      and differs from `shownPath`, it returns `single(path)`; otherwise it
      returns the selection unchanged (the file is hidden by the filter, or
      the focus did not move). Name and doc comment follow the module's
      style (short verb-ish names such as `click`, `extend`, `prune`;
      a comment stating the rule).
    - `step()` in `crates/app/ui/src/main.ts` uses it on its single-file
      branch after `commit(...)`, assigning `selection` and calling
      `paintSelection()`, and still does `index = at; show();` when
      `at !== index`. The batch branch (`batch.length > 1`) is unchanged.
    - `crates/app/ui/src/selection.test.ts` gets a `describe` for the helper
      with at least: focus moved onto a visible file -> exactly
      `single(path)` (the regression: start from `single("/b")`, path `/a`,
      shown `/b`); path equal to the shown file with a multi-selection ->
      selection unchanged; path not in `files` -> selection unchanged.
    - The comment above `step()` is updated to say the single file also
      becomes the selection when the focus moves.
    - `mise run ci` passes.
  - Implementation approach:
    - Gate on `path !== shownPath` (the pre-commit `files[index]` that
      `step()` already captures), not on `at !== index`: when the list
      changed, `commit` -> `refilter(anchor = path)` has already moved
      `index` onto `path` and `prune` has already produced `{B, A}`, so
      `at === index` there and a check on it would miss that case. When the
      list did not change, `index` is stale and `at !== index` still drives
      `show()` as today.
    - Call `paintSelection()` after assigning, as the other focus moves do.
      `refilter` also paints when the list changed; painting twice is
      harmless.
    - No new module: `selection.ts` is the home of pure selection decisions
      and its test file already covers `single`, `prune`, `extend`; keep
      `main.ts`'s change to a few lines.
    - Manual check on the desktop app: judge A, move to B, Ctrl+Z -> only A
      has `.selected`; then Ctrl+Y -> the focus stays on A and the selection
      is unchanged; repeat with a judgment filter on so the list changes, and
      confirm the next judgment key affects only A.

## Trade-offs and risks

- **Batch undo / redo (`batch.length > 1`) leaves the selection alone
  (decided).** The batch branch keeps the current file where it is
  (`commit(changes, undefined, () => shownPath)`) and `refilter`'s `prune`
  already drops any hidden member and keeps the focused file selected. The
  alternative, selecting the batch's files (`selectionOf`), would turn
  Ctrl+Z into a selection-changing action, would have to add the focused
  file when it is outside the batch (the invariant is that `files[index]` is
  always a member), would need the filter-hidden members dropped, and would
  make the redo asymmetric.
- **Collapse only when the focus moved (decided), versus always after a
  single-file undo / redo.** A single-file history entry can exist while a
  multi-selection is active (a judgment over `{A, B, C}` where only A's
  state actually changed, since `judgments()` skips unchanged files; or
  comparing mode). "Always" would collapse `{A, B, C}` to `{A}` on Ctrl+Z
  even though the focus stayed on A; "only when moved" keeps the user's
  selection there and still fixes both reported cases (unchanged list and
  changed list) because the gate is `path !== shownPath`.
- **Testability.** The helper exists because `main.ts` cannot be
  unit-tested, and the helper is the exact decision that was missing.

## Progress

- (none yet)
