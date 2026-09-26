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

# Correct the SIGMA fp L strip thumbnail todo item

## Purpose

`todo.md` says (`### App: SIGMA fp L strip may decode the full-size JPEG per thumbnail`, around line 575) that the strip decodes the ~28 MB full-size JPEG per thumbnail. It does not, and never did since the item was written: the thumbnail cache (#16 / #18, 2026-09-18) predates the item (#376, 2026-09-24). At scan time `thumbnail_jpeg` in `crates/core/src/decode.rs` decodes the preview tier at 2/8 scale and re-encodes it, `crates/core/src/scan.rs` (`extract_unless`, ~line 57) stores it in the index, and the strip's `thumbnail` command in `crates/app/src/commands.rs` (~line 1520) reads `Index::thumbnail` (`crates/app/src/index.rs` ~line 947). The item's false premise would send the next person to fix the wrong thing. What actually remains is that the SIGMA fp L preview tier falls back to the 9520x6328 full-size JPEG (no strip JPEG at or above `PREVIEW_MIN_WIDTH` = 1600 in `crates/core/src/arw.rs`), so the 2/8 cached thumbnail is about 2380x1582, more than ten times the pixels of the ~404x270 thumbnail of other bodies, which makes the strip's per-cell decode and IPC heavier, and each scan decodes the ~60 MP JPEG (at 2/8 scale). The `### App: SIGMA fp L main view appears late on Linux` item refers back to the old premise and should say the same thing.

## Steps

- [x] Step 1: Rewrite the SIGMA fp L strip item in `todo.md` and align the Linux item's cross-reference
  - Done when:
    - The heading no longer claims a per-thumbnail full-size decode; it names the symptom (e.g. `### App: SIGMA fp L strip thumbnails are about ten times larger than other bodies'`).
    - The body states the verified facts: the preview tier falls back to the 9520x6328 full-size JPEG because no strip JPEG reaches `PREVIEW_MIN_WIDTH`; scan-time `thumbnail_jpeg` decodes that at 2/8 scale and the index caches the result; the strip reads the cache via the `thumbnail` command; the consequence is a ~2380x1582 cached thumbnail (vs ~404x270) that costs more to decode and ship per cell, plus one sentence on the ~60 MP JPEG decode (at 2/8) per scan. It names `crates/core/src/decode.rs` (`thumbnail_jpeg`), `crates/core/src/scan.rs`, `crates/app/src/index.rs`, `crates/app/src/commands.rs` (`thumbnail`) and `crates/core/src/arw.rs` (`PREVIEW_MIN_WIDTH`, tier selection).
    - The TODO list reads roughly: measure strip thumbnail load time on a SIGMA fp L folder (per-cell decode and IPC); candidate fix, not decided: cap the thumbnail size, e.g. pick a larger DCT scale-down (`d.scale(n)`) when the preview is large, or resize to the normal thumbnail width.
    - In `### App: SIGMA fp L main view appears late on Linux`, the cross-reference to the strip item uses the new heading, and the candidate-fix bullet is reworded so it no longer implies the strip item proposes a mid-size preview; a backend-cached mid-size JPEG would still serve the main view and would shrink the thumbnail as a side effect, so keep the cross-reference but say that.
    - Nothing else in `todo.md` changes; `mise run ci` passes.
  - Implementation approach:
    - Docs-only, English, surgical: touch only the two items. Keep the existing item structure (`### App: ...`, prose body, `#### TODO` with `- [ ]` bullets) and wrapping width.
    - Do not describe the fix as decided; list it as a candidate as the surrounding items do.
    - Verify the numbers before writing: 9520/4 = 2380, 6328/4 = 1582; 2380x1582 ≈ 3.77 MP vs 404x270 ≈ 0.11 MP (about 35x the pixels; "more than ten times" is safe).
    - `grep` for `strip may decode the full-size JPEG` across the repo before finishing; update any other reference.

## Trade-offs and risks

- The heading names the symptom rather than the cause, matching the item's user-facing angle.
- The per-scan 60 MP decode is a one-time scan cost, not a strip cost; it gets one sentence.

## Progress

- Step 1: done (todo.md SIGMA fp L strip item rewritten, Linux item cross-reference aligned)
