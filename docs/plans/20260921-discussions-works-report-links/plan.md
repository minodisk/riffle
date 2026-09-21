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

# Discussions works-report thread links

## Purpose

`README.md` and `.github/ISSUE_TEMPLATE/config.yml` still point the OS / camera / software "works" reports at the generic Discussions index with TODO markers, because the threads did not exist when they were written. The threads now exist (OS #286, camera #287, software #288), so readers and the issue template chooser should land on the right thread directly.

## Steps

- [x] Step 1: Point the works-report links at the real Discussions threads
  - Done when:
    - `README.md` links `OS works report thread` to https://github.com/minodisk/riffle/discussions/286, `camera works report thread` to https://github.com/minodisk/riffle/discussions/287, and `software works report thread` to https://github.com/minodisk/riffle/discussions/288, and the three `<!-- TODO: replace with the ... works-report thread -->` markers are gone (surrounding sentences re-wrapped so the prose still reads naturally)
    - `.github/ISSUE_TEMPLATE/config.yml` `contact_links` use the same three URLs in the same order (OS, camera, software) and the three `# TODO: replace with ...` comments are gone
    - `grep -rn "riffle/discussions\b\|works-report thread" README.md .github/ISSUE_TEMPLATE/config.yml` shows only the three `/discussions/28x` URLs and no TODO markers
    - The `### Docs: Discussions "works" report threads do not exist yet` section is removed from `todo.md` (heading through its single TODO item)
    - `mise run ci` passes
  - Implementation approach:
    - Pure text edit; no code changes. Keep the link texts and `name` / `about` fields as they are; only the URLs and the markers change.
    - `.github/**` is touched, so the PR needs the user's explicit merge approval.
    - Single-PR mode: the wrap-up (archive move, auto memory cleanup) goes in the same PR.

## Trade-offs and risks

- The todo.md section is deleted in this PR to keep it self-contained.

## Progress

- (none yet)
