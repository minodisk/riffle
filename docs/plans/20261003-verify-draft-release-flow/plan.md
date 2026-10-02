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

# Drop the verified draft-then-publish release todo

## Purpose

The `todo.md` item "Release: the draft-then-publish release flow is unverified
on a real release" asked to run the "Verification on the next release"
checklist of `docs/plans/_archived/20260923-draft-release-publish/learnings.md`
on the first real release through the new flow. v1.3.0 (#644, Release workflow
run 36945908567) did that on 2026-10-03 and every item passed: release-please
created the tag and the draft release at 00:25:33Z before the builds ran
(00:25:43–00:47:05), the `publish` job published it at 00:47:11Z right after
the last build, there is exactly one v1.3.0 release, it is Latest with every
bundle and its `.sig` and a single `latest.json` covering linux-x86_64,
darwin-aarch64, darwin-x86_64 and windows-x86_64,
`releases/latest/download/latest.json` redirects (302) to
`releases/download/v1.3.0/latest.json`, and the next release PR #658 compares
`v1.3.0...v1.3.1` with only the commits since v1.3.0. The item is done, so it
leaves `todo.md`.

## Steps

- [x] Step 1: Remove the verified section from `todo.md`
  - Done when: the heading `### Release: the draft-then-publish release flow is unverified on a real release`, its paragraph and its `#### TODO` list are gone; the preceding and following sections are untouched and separated by exactly one blank line; no other file changes; `mise run ci` passes.
  - Implementation approach:
    - Locate the heading with a search (line numbers shift as `main` moves; it was lines 590–606 on 971a9b96) and delete from the heading through the blank line before the next `###` heading.
    - Do not edit the archived `learnings.md` checklist or anything else; scope is `todo.md` only.
    - Commit as `docs(todo): drop the verified draft-then-publish release item`.

## Trade-offs and risks

- The archived `learnings.md` checklist boxes stay unticked. Ticking them would record the verification in the archive but was explicitly excluded from scope; if wanted later it is a separate docs PR.

## Progress

- (none yet)
