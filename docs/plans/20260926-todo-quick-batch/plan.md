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

# Small todo batch: stale docs, lychee note, DSC gate, WSL roots

## Purpose

Close five small, independent items from `todo.md` in one PR, and drop the
`todo.md` sections the audit found already resolved. Afterwards
`docs/performance.md` names the real location of the `Timing logs` toggle,
agents writing plan/learnings Markdown know how lychee resolves relative links,
`sharpness.rs` no longer misreads older Sony `DSC-` bodies (whose `FocusMode`
is always 0) as manual focus, and the Linux folder tree on WSL no longer lists
`/mnt/wsl` / `/mnt/wslg` as roots.

## Steps

- [x] Step 1: Fix the stale doc text, add the lychee note, gate the manual-focus check on the model, filter WSL's internal `/mnt` mounts, and delete the resolved `todo.md` sections
  - Done when:
    - `docs/performance.md` "End to end, keypress to pixels" (line ~396) no longer says `Debug > Timing logs`; it says the toggle is `Timing logs` in the settings window, in the wording "Measuring on your own folder" (line ~359) already uses. `grep -n "Debug > Timing" docs/**/*.md` is empty.
    - The `<plan-guide>` block of the plan.md template in `.claude/agents/planner.md` carries one sentence: lychee (`mise run lint`) resolves relative links in `docs/plans/**` from the linking file's own directory, so a link must be relative to the plan folder (e.g. `../../usage.md`) and an example path that is not a real link target goes in backticks. `mise run lint` still passes.
    - `crates/core/src/sharpness.rs`'s `manual_focus` returns `false` for `Shot { focus_mode: Some(0), model: Some("DSC-RX100M3") }` and `true` for `Some(0)` with `DSC-RX100M7`, `ILCE-7M5` or `model: None`; a unit test next to `only_focus_mode_zero_is_manual_focus` asserts that. The module doc comment's item 2 ("a Sony frame shot in manual focus") mentions the gate.
    - `crates/app/src/folders.rs`'s Linux `volumes` keeps, on WSL only, just the `/mnt` children whose name is a single ASCII letter; `/media` and `/run/media` handling is unchanged. The decision is a pure function (e.g. `fn wsl_drive_mounts(dirs: Vec<PathBuf>) -> Vec<PathBuf>` or a `fn is_drive_mount(name: &str) -> bool`) with a unit test that feeds `/mnt/c`, `/mnt/d`, `/mnt/wsl`, `/mnt/wslg` and keeps only the first two. The test must not depend on running on WSL (a cfg-free pure function tested on every platform is preferred).
    - `docs/agents/tauri-app.md` line ~131-133 ("`/mnt` lists every child directory as a root, including WSL's own ... unfiltered by design; see the deferred issue") is reworded to state the new behavior (WSL: single-letter mounts only; other Linux: unfiltered).
    - `todo.md`: the section "Docs: CLAUDE.md's sharpness fallback order is stale since #396" is deleted in this PR (already resolved, evidence below). The sections this feature resolves are listed under "Sections for the wrap-up's todo curation" and are deleted by the curator.
    - `mise run ci` passes.
  - Implementation approach:
    - Item 1 (`docs/performance.md`): change only the `Debug > Timing logs` phrase at line ~396; do not touch the numbers.
    - Item 2 (CLAUDE.md fallback order): no edit. `CLAUDE.md` already reads "taken on the Sony eye-AF frame when the camera tracked a face, else around the AF point, else between the eyes of a detected face, else from the sharpest tile", which matches the module doc of `crates/core/src/sharpness.rs` lines 4-17. Only the todo section goes.
    - Item 3 (lychee note): add to the `<plan-guide>` in `.claude/agents/planner.md` (the template every plan.md copies). Do not create `docs/agents/docs-writing.md` for one rule. `mise.toml` line 73 is the lychee invocation (`--offline --include-fragments`, `docs/**/*.md`, `.claude/**/*.md`); the note's own example path must itself be in backticks or it fails lint.
    - Item 4 (DSC gate): `crates/app/src/exif.rs` lines 61-85 hold `DSC_EXCEPTIONS` (8 models) and `fn excluded_dsc(model: Option<&str>) -> bool` (`None` is not excluded; `DSC-` prefix minus exceptions). `Shot` (`crates/core/src/arw.rs` line 132) has `model: Option<String>` and `focus_mode: Option<u8>`. Core cannot depend on app, so move `DSC_EXCEPTIONS` and `excluded_dsc` into core as `pub` (in `sharpness.rs`, or next to `Shot` in `arw.rs`), make `manual_focus` return `false` when `excluded_dsc(shot.model.as_deref())`, and have `exif.rs` use the core helper so the exception list has one source; the existing `exif.rs` tests keep passing unchanged. `trusted_focus` and `eye_af_frame` go through `manual_focus`, so they inherit the gate without edits.
    - Item 5 (WSL roots): `volumes()` for Linux is `crates/app/src/folders.rs` lines 114-123, built on `child_dirs` (line 128). Detect WSL with the `WSL_DISTRO_NAME` env var; apply the single-letter filter to the `/mnt` children only when detected. Keep the filter a pure function over names so the test runs anywhere. Add the test to the existing `mod tests`.
    - Cross-check: `crates/app/src/commands.rs` line ~1907 and `docs/agents/tauri-app.md` line ~1382 mention WSLg only for the WebKitGTK pixel limit; they are unrelated and must not change.
    - `EXTRACTOR_VERSION`: the `manual_focus` change alters the sharpness strategy on excluded `DSC-` bodies with `FocusMode` 0. Decide whether to bump `EXTRACTOR_VERSION` in `crates/app/src/index.rs` so their cached `sharpness` is re-scored, and record the decision in `learnings.md`.

## Todo audit

### Already resolved in the current tree (delete in this PR)

- `### Docs: CLAUDE.md's sharpness fallback order is stale since #396` — the `CLAUDE.md` Layout paragraph already states the order in `crates/core/src/sharpness.rs` lines 4-17.

### Checked and still open (no action)

Every other open section still has code/doc evidence of being open, or is a manual GUI / real-device check with no recorded result.

### Sections for the wrap-up's todo curation (resolved by this feature)

- `### Docs: the "End to end, keypress to pixels" section of docs/performance.md has a stale menu path`
- ``### Docs: note that lychee resolves relative links in `docs/plans/**` from the linking file's own directory``
- ``### Core: `sharpness.rs`'s manual-focus check has no model gate``
- ``### App: `folder_roots` lists WSL's internal `/mnt/wsl` and `/mnt/wslg` mounts as roots``
- `### Docs: CLAUDE.md's sharpness fallback order is stale since #396` (deleted directly in Step 1)

## Trade-offs and risks

- The lychee note lives in the `<plan-guide>` of `.claude/agents/planner.md`, which reaches every plan.md; a new guide for one rule was rejected.
- The `DSC-` gate is shared from core rather than duplicated, so the exception list cannot drift.
- `WSL_DISTRO_NAME` is unset in rare setups; the tree then falls back to the current unfiltered behavior, the safe direction. Filtering `/mnt` on every Linux was rejected since plain Linux users mount arbitrary names there.

## Progress

- (none yet)
