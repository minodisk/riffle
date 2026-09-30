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

# Close the orientation-neutral preview real-device check todo

## Purpose

The user ran the real app on Windows and confirmed every TODO of the `todo.md`
item `### App: real-device check of the orientation-neutral preview JPEGs
(portrait RAF, ARW and JPEG folders)`: the GFX 100 RAF
(`D:\Photos\samples\RAF\GFX_100_fujifilm_gfx_100_13.raf`, Orientation 8) shows
upright in the main preview, and the ARW and JPEG-only folders show no
regression. The item is done and should leave `todo.md`; the archived plan's
learnings still call the manual check pending and should say it is done.

## Steps

- [x] Step 1: Remove the completed item from `todo.md` and mark the manual check done in the archived learnings
  - Done when:
    - The heading `### App: real-device check of the orientation-neutral preview JPEGs (portrait RAF, ARW and JPEG folders)` (under `## Cross-cutting / other`) and its whole body (`#### Background` paragraph, `#### TODO` with its three checkboxes) are gone, up to but not including the next heading. One blank line separates the neighbouring items.
    - Nothing else in `todo.md` changes.
    - `docs/plans/_archived/20260930-orientation-neutral-preview/learnings.md` no longer presents the manual check as pending: under `## Deferred issues (todo candidates)`, the "Pending manual check (Step 1; ...)" bullet gets a short note that the check was done on Windows on 2026-09-30 (GFX 100 RAF upright in the main preview; ARW and JPEG-only folders no regression), with the guide entry left as is.
    - `docs/agents/tauri-app.md` is not edited: its entry `### The preview payload's JPEG is orientation-neutral by construction (Hit)` does not mention a pending check, and the todo's last TODO says it "stands as is" when the checks hold.
    - `mise run ci` passes.
  - Implementation approach:
    - Docs-only. Files: `todo.md`, `docs/plans/_archived/20260930-orientation-neutral-preview/learnings.md`, plus this plan's `plan.md` (step ticked, Progress entry) and `learnings.md`.
    - Keep the learnings edit minimal (append a sentence or a sub-bullet to the existing "Pending manual check" bullet rather than rewriting it) so the archived record stays a record.

## Trade-offs and risks

- Whether to touch the archived learnings at all: leaving archives frozen would leave a stale "pending" with no todo pointing at it, so the plan adds a minimal note.

## Progress

- (2026-09-30) Step 1 complete
