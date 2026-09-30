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

# Close the HDR PQ CR3 preview real-device check

## Purpose

The hdr-pq-cr3-hevc-preview wrap-up (#615) left a `todo.md` section asking for
a real-device check of the decoded HDR PQ (HEIF) CR3 previews. The user ran
that check on 2026-09-30 in the app on `D:\photos\samples\CR3`: `R8.CR3` and
`R5m2.CR3` show correctly colored thumbnails and previews, the meta pane's
EXIF rows and the sharpness value, and `z` shows a crop of the 1620x1080
preview with no error (the image sits off-center with black margins because
the AF point is centered on an image barely larger than the viewport; the
user accepts this as-is, no change wanted). The section's sidecar sub-check
is format-independent (sidecars are keyed by file name and already exercised
on JPEG CR3s), so the whole section is done and should go, leaving `todo.md`
with only open items.

## Steps

- [x] Step 1: Remove the "App: real-device check of the HDR PQ (HEIF) CR3 preview decode" section from `todo.md` and note the completed check in the archived learnings
  - Done when: the `### App: real-device check of the HDR PQ (HEIF) CR3 preview decode` heading, its paragraph, its `#### TODO` heading and its single checkbox are gone from `todo.md`, the surrounding sections are untouched and still separated by one blank line, `rg` finds no remaining reference to the heading in the repo outside the archived learnings note, and `mise run ci` passes.
  - Implementation approach:
    - `todo.md`: delete the section through its checkbox and the blank line that follows, so exactly one blank line separates the previous section from `### App: keep the EXIF of a file whose extraction failed`. Do not touch any other section.
    - `docs/plans/_archived/20260930-hdr-pq-cr3-hevc-preview/learnings.md`: append one short bullet directly after the existing "Pending manual check (the user's; ...)" bullet under `## Deferred issues (todo candidates)`: done 2026-09-30 by the user on Windows (`R8.CR3` and `R5m2.CR3` show correctly colored thumbnails and previews, the EXIF rows and the sharpness value; `z` shows a crop of the 1620x1080 preview with no error; the zoomed image sits off-center with black margins because the AF point is centered on an image barely larger than the viewport, accepted as-is; the sidecar sub-check was not repeated because sidecars are keyed by file name and already exercised on JPEG CR3s; the `todo.md` section was removed). The user approved editing the archived file. Do not rewrite the pending bullet above it.
    - Nothing else references the heading, so no other file changes.

## Trade-offs and risks

- Editing an archived plan's `learnings.md` is a small new precedent (no archived learnings file was edited after its archive move before). The user chose it so the archived learnings do not end on a "pending" bullet forever.
- The off-center zoom is recorded as accepted behavior, not as a new `todo.md` item, per the user's decision.

## Progress

- (none yet)
