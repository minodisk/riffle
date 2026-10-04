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

# Drop the first-launch Lightroom language real-device check from todo.md

## Purpose

`todo.md` still carries the section "App: real-device check of the first-launch Lightroom UI-language question (todo-five-more-items Step 4)" with three manual GUI checks. The user has decided the real-device check is not needed: the Japanese Lightroom color-label presets were already verified end to end (an XMP written by Riffle, read into Japanese-UI Lightroom Classic 15.5.1, shows the default colors rather than white custom labels), and the dialog logic itself is covered by `firstrun.test.ts` and `mise run ci`. Removing the section keeps `todo.md` an accurate list of open work.

## Steps

- [x] Step 1: Remove the first-launch Lightroom UI-language real-device check section from `todo.md`
  - Done when: the `###` heading "App: real-device check of the first-launch Lightroom UI-language question (todo-five-more-items Step 4)", its `#### Background` paragraph, its `Files:` line and its `#### TODO` list (three items) are gone from `todo.md`; the neighbouring sections ("… Expand All / Collapse All … (todo-five-more-items Step 3)" before it and "… trash undo / redo after a folder rename (todo-five-more-items Step 5)" after it) are byte-for-byte unchanged and separated by exactly one blank line; no other file changes; `mise run ci` passes.
  - Implementation approach:
    - Docs-only change to `todo.md` (currently lines 1215-1228, i.e. the heading through the blank line before the Step 5 heading). Touch nothing else.
    - The heading text occurs nowhere else in the repository, so there are no links or cross-references to update.
    - Commit as `docs(todo): drop the first-launch Lightroom language real-device check`.

## Progress

- (2026-10-05) Step 1 complete
