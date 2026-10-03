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

# dop-orientation-landscape-verified

## Purpose

The `todo.md` item "Core: an explicit `Orientation = 1` line for landscape
`.dop` files is unverified in PhotoLab" is now verified by hand. On 2026-10-04
a Riffle-made `.dop` (written by `riffle-core`'s `dop::write_rating`, with the
explicit `Orientation = 1,` line, for a landscape ILCE-7M4 uncompressed ARW
with EXIF Orientation 1) was opened in PhotoLab 10 at
`D:\Photos\tests\2026-10-03-dop-orientation-landscape`, and the image displayed
upright; the `.dop`'s `Rating = 3` was read, confirming PhotoLab used the
sidecar. The open item is therefore done and should leave `todo.md`.

## Steps

- [x] Step 1: Remove the verified `.dop` landscape-orientation item from `todo.md`
  - Done when: the section headed
    `### Core: an explicit \`Orientation = 1\` line for landscape \`.dop\` files is unverified in PhotoLab`
    (currently `todo.md` lines 607–620: the heading, its paragraph, its
    `#### TODO` list and the trailing blank line) is gone; the preceding
    section ("Release: the draft-then-publish release flow ...") and the
    following one ("App: SIGMA fp L strip thumbnails ...") are untouched and
    still separated by exactly one blank line; `git diff --stat` shows only
    `todo.md` (plus this plan folder); `mise run ci` passes.
  - Implementation approach:
    - Text-only change; no code is touched. Do not edit any other `todo.md`
      item.
    - Record the verification details in this folder's `learnings.md`,
      including the incidental observation: an extra item appeared in
      PhotoLab because the ARW was copied into a folder PhotoLab was already
      viewing before the `.dop` existed (PhotoLab registered the image with
      its own UUIDs, then the fresh `.dop` carried random ones). This is a
      test-procedure race, not a Riffle bug; the app avoids it through
      `crates/app/src/photolab.rs`'s UUID lookup (see
      `../../agents/photolab.md`).

## Progress

- (none yet)
