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

# Todo: face/eye-aware focus check

## Purpose

Record, in `todo.md`, the idea of a face/eye-aware focus check so it is not lost. The current sharpness score (`crates/core/src/sharpness.rs`, `score_preview`) measures a window around the Sony `FocusLocation` or, without one, the sharpest tile of the embedded preview (`tile_max`). A portrait focused on the background, the nose or the ear instead of the eye can therefore still score high, and the burst-group sharpness cue (`crates/app/ui/src/sharpness.ts`, `burst.ts`) inherits that blind spot. This plan only adds the todo item; no code changes.

## Steps

- [x] Step 1: Add the "App: face/eye-aware focus check for culling" item to `todo.md`
  - Done when: `todo.md` contains a new `### App: face/eye-aware focus check for culling` heading appended at the end of the `## Cross-cutting / other` section (the only section; append after the "GitHub: verify the issue report forms on GitHub" item), with a background paragraph, a `#### TODO` checklist and a trailing `Related:` line, all in English; `mise run ci` passes (the `.md` formatting is checked by `pnpm exec vp check` and links by `lychee --offline`, so put no links in the item, or only links that resolve offline).
  - Implementation approach:
    - Follow the existing item format exactly: `### <Area>: <title>`, a prose paragraph, `#### TODO`, `- [ ]` bullets (wrap continuation lines with six-space indentation as the recent items do), then `Related: \`path\`, ...`. Run `mise run fmt` before `mise run ci` so the formatter settles the wrapping.
    - Background paragraph content: the score in `crates/core/src/sharpness.rs` (`score_preview`, `trusted_focus`, `tile_max`) measures a `WINDOW` around the Sony `FocusLocation` or the sharpest tile, so a frame focused on the background, nose or ear rather than the eye still scores high; the stored `files.sharpness` column in `crates/app/src/index.rs` and the strip's relative cue (`crates/app/ui/src/sharpness.ts`) and burst group (`crates/app/ui/src/burst.ts`) inherit that. Idea: detect faces/eyes on the embedded preview at scan time with a lightweight detector (such as YuNet or BlazeFace via ONNX, e.g. the `ort` crate), store the region in the SQLite index, score sharpness on the eye/face region, and suggest the sharpest-eye frame within a burst group.
    - Checklist bullets (four):
      1. First check whether Sony ARW and Leica DNG MakerNotes record face/eye-AF detection positions (Sony's MakerNote parsing lives in `crates/core/src/arw.rs`, which already reads `FocusLocation` and `FocusMode`); if they do, no inference is needed for those bodies.
      2. Prototype a lightweight detector and measure per-image cost at scan time and the added bundle size (runtime plus model); note the numbers in `docs/performance.md` or the plan's learnings.
      3. Fall back to the current AF-point / tile scoring when no face is found (landscapes, animals), keeping `files.sharpness` semantics for such files.
      4. Optionally, eye-closed detection from landmarks.
    - `Related:` line: `crates/core/src/sharpness.rs`, `crates/core/src/arw.rs`, `crates/app/src/index.rs`, `crates/app/ui/src/sharpness.ts`, `crates/app/ui/src/burst.ts`.

## Trade-offs and risks

- Section placement: `todo.md` has a single `## Cross-cutting / other` section, so the item goes there rather than under a new `## App` heading.
- Wording of the detector suggestion: name YuNet/BlazeFace/`ort` as examples ("such as"), not decisions, so the todo does not pre-commit the eventual implementation to a specific runtime.

## Progress

- (none yet)
