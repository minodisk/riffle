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

# Drop the trash step from the README example workflow

## Purpose

PR #636 added `### Example workflow` to `README.md` and `### ワークフローの例` to
`README.ja.md` with six steps. Step 2, clearing the rejects with
`Move Rejected to Trash…`, is an optional feature, not part of the core
cull -> develop -> sequence -> upload flow. Removing it leaves a five-step
workflow that reads as the essential path. The original change is recorded in
`../_archived/20261001-readme-example-workflow/plan.md`.

## Steps

- [x] Step 1: Remove the `Move Rejected to Trash…` step from both READMEs and renumber
  - Done when:
    - `README.md` `### Example workflow` has five steps numbered 1-5: cull, develop in Lightroom / DxO PhotoLab, `Sequence JPEG Timestamps…`, open `<folder>-sequenced/` view-only, upload to Google Photos. The old step 2 is gone.
    - `README.ja.md` `### ワークフローの例` has the same five steps in the same order; the old step 2 is gone.
    - Every remaining anchor link (`#key-features`, `#working-with-other-software`, `#主な機能`, `#他のソフトとの連携`) is untouched and still resolves.
    - `mise run ci` passes (its `lint` task runs lychee with `--include-fragments` over `*.md`, so broken anchors would fail).
  - Implementation approach:
    - Docs-only; both READMEs in the same PR.
    - Delete the step and renumber; change nothing else in the section (the user chose not to add a "filter out the rejects" phrase to the develop step).
    - Keep `README.md`'s existing hard-wrapped line style for the step text.

## Trade-offs and risks

- No link risk: the removed step's only link is `#key-features` / `#主な機能`, which other steps still use.

## Progress

- (none yet)
