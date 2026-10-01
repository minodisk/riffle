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

# Drop two todo.md items decided not needed

## Purpose

On 2026-10-01 the user decided two open items in `todo.md` are not worth
doing, so they should stop appearing in the backlog:

1. `### App: Open in Terminal from the folder tree's context menu`: the
   existing `Copy Path` folder-menu item already covers the use case (paste
   the path into a terminal), and the per-platform / Linux terminal choice is
   not worth the work.
2. `### Merge skill: jq reserved words as variable names in skill scripts`:
   judged not worth a guide; closed with no action, which is one of the two
   outcomes that TODO itself offered.

Removing them keeps `todo.md` an accurate list of open work.

## Steps

- [x] Step 1: Remove the two sections from `todo.md`
  - Done when:
    - The heading `### App: Open in Terminal from the folder tree's context
      menu`, its paragraph and its `#### TODO` list (one checkbox) are gone
      from `todo.md`.
    - The heading `### Merge skill: jq reserved words as variable names in
      skill scripts`, its paragraph and its `#### TODO` list (one checkbox)
      are gone from `todo.md`.
    - The neighbouring sections are intact and still separated by exactly one
      blank line: `### App: face/eye-aware focus check for culling` (ending
      with its `Related:` line) is followed directly by `### Agents: confirm
      the long-wait timeout fix on a real `/pr` or `/merge` run`, and `### App:
      the manual GUI checks for the resume landing's selection are still open`
      is followed directly by `### App: Refresh from the folder tree's context
      menu for a folder whose watch failed`.
    - Nothing else in `todo.md` or elsewhere in the repository changes
      (besides this plan folder).
    - `mise run ci` passes.
  - Implementation approach:
    - Docs-only. At planning time the sections sit at `todo.md` lines
      548-564 (jq section, up to the blank line before `### Agents: confirm
      the long-wait timeout fix ...`) and lines 945-959 (Open in Terminal
      section, up to the blank line before `### App: Refresh from the folder
      tree's context menu ...`). Re-check the line numbers before editing;
      other work may shift them.
    - Delete each section as a block: the `###` heading, the paragraph, the
      `#### TODO` heading and its checkbox, plus one of the two surrounding
      blank lines so a single blank line remains between the neighbours.
    - No other file needs an edit. The only other mentions of these items
      are historical and must be left alone:
      `docs/plans/_archived/20260927-folder-menu-copy/plan.md` (deferred the
      Open in Terminal item),
      `docs/plans/_archived/20260929-resume-selection-manual-checks/{plan,learnings}.md`
      and `docs/plans/review-history/docs/resume-selection-manual-checks/review-20260929-0830.md`
      (used the Open in Terminal heading as an insertion anchor), and
      `docs/plans/_archived/20260922-fix-jq-label-keyword/learnings.md` (the
      origin of the jq item). None is a Markdown link to a `todo.md` anchor,
      so lychee is unaffected.
    - Commit as `docs(todo): drop the Open in Terminal and jq keyword items`.

## Trade-offs and risks

- Recording the reasons for closing: the plan's Purpose above keeps the
  reasons (Copy Path covers the terminal case; the jq guide is not worth it),
  and the archived plan folder preserves them. Adding a "closed" note to
  `todo.md` or a guide would contradict the user's judgment that no guide is
  warranted, so it is not taken.
- The historical anchor references to the Open in Terminal heading in
  archived plans and the review history become descriptions of a heading
  that no longer exists. That is expected for historical records and they
  must not be edited.

## Progress

- (2026-10-01) Step 1 complete
