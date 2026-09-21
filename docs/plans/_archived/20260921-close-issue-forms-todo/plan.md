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

# Close the "verify the issue report forms" todo

## Purpose

`todo.md` carries a "GitHub: verify the issue report forms on GitHub" item
left by the issue-forms PR (#282): the three forms under
`.github/ISSUE_TEMPLATE/` were only YAML-parsed before merge, and nothing in
`mise run ci` validates that directory. On 2026-09-21 the user opened
`https://github.com/minodisk/riffle/issues/new/choose` and confirmed that the
three forms (OS / camera / software) and the three Discussions contact links
show up, and that each form's fields render. The item is done; this work
removes it so `todo.md` only lists open items.

## Steps

- [x] Step 1: Remove the "GitHub: verify the issue report forms on GitHub" section from `todo.md`
  - Done when: the section (the `### GitHub: verify the issue report forms on GitHub` heading, its paragraph, the `#### TODO` subheading and its single checkbox item) is gone from `todo.md`; no other line of `todo.md` changes; `mise run ci` passes.
  - Implementation approach:
    - Delete the section and its trailing blank line. Confirm with `git diff` that only that deletion appears.
    - Do not touch `.github/ISSUE_TEMPLATE/` (form ordering and content stay as they are).
    - Single-PR mode: the plan's step tick goes in the same PR.

## Progress

- (2026-09-21) Step 1 complete
