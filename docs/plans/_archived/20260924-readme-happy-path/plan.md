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

# Trim the READMEs to the culling happy path

## Purpose

The READMEs currently list every feature and a full key table, which buries
the reason to install Riffle under details that belong in `docs/usage.md`.
After this change the READMEs describe only what is truly needed for culling
or makes choosing markedly easier (seven features), point at `docs/usage.md`
for everything else and for the key reference, and `docs/usage.md` becomes
the single complete reference: every feature, every default shortcut in
`crates/app/src/shortcuts.rs`, and the fixed keys the app handles outside the
keymap.

## Steps

- [x] Step 1: Trim both READMEs to "Key features", drop their "Keys" sections, and complete `docs/usage.md`'s Keys section
  - Done when:
    - `README.md` has `## Key features` and `README.ja.md` has `## 主な機能`, each with exactly these seven bullets: Filmstrip, 1:1 focus check, Focus mark, Sharpness cue, Bursts, Compare, Move Rejected to Trash. The section says that every feature is described in `docs/usage.md` (the Japanese link marked `（英語）`).
    - The Sharpness cue bullet says faces and eyes are found locally (no network); there is no separate "Offline face detection" bullet.
    - The Bursts bullet spells out `ArrowLeft` / `ArrowRight` (jump between bursts), `Alt+ArrowUp` / `Alt+ArrowDown` (step within one, stopping at its ends) and `Shift+x` (reject the rest).
    - Shooting info, Filter and sort, Undo / redo and Auto-advance no longer appear in either README.
    - Neither README has a `## Keys` / `## キー` section; a one-sentence pointer to `docs/usage.md#keys` replaces it.
    - `docs/usage.md` carries the YuNet link and MIT attribution (in its Sharpness cue bullet), and its `## Keys` section lists every default in `shortcuts.rs` plus the fixed keys: `Escape`, `Tab` / `Shift+Tab` in the first-launch dialog, `CmdOrCtrl+R`, `CmdOrCtrl+,`, and the Settings tab-strip keys.
    - `README.md` and `README.ja.md` say the same things section by section.
    - `mise run ci` passes (lychee with `--include-fragments` included).
  - Implementation approach:
    - Files: `README.md`, `README.ja.md`, `docs/usage.md`. Nothing else in the repo links to the README `#features` / `#keys` (or `#機能` / `#キー`) anchors, so no other file needs an anchor fix; keep the READMEs' existing `#compatibility` / `#what-the-camera-records` (ja: `#対応状況` / `#カメラが記録する情報と使える機能`) links as they are.
    - READMEs: rename the heading, keep the seven bullets' current wording (the Bursts bullet already lists the three shortcuts; keep them), fold "faces and eyes are found locally, with no network access" into the Sharpness cue bullet, delete the other bullets and the whole Keys table. Replace the trailing "Detailed behavior: docs/usage.md" line with one sentence such as `Every feature, and the full key reference, is described in [docs/usage.md](./docs/usage.md) ([Keys](./docs/usage.md#keys)).` at the end of "Key features"; the ja version adds `（英語）` after the link as elsewhere in `README.ja.md`. Do not also add it to "First steps". Order the bullets as listed above; the current README has Compare before Focus mark, so move the Compare bullet. Apply the identical edits to `README.ja.md` in the same PR (CLAUDE.md rule).
    - `docs/usage.md` Sharpness cue bullet: it is stale (it says the score is taken around the AF point, else the sharpest region, and never mentions the eye-AF frame or face detection). Rewrite it to match the README's scoring order (Sony eye-AF frame when a face was tracked, else between the eyes of a detected face, else around the AF point, else the sharpest region, as CLAUDE.md describes) and add: faces and eyes are found by the bundled [YuNet](https://github.com/opencv/opencv_zoo/tree/main/models/face_detection_yunet) model (MIT license, text in `crates/core/models/LICENSE`), run locally with no network access.
    - `docs/usage.md` Keys: the existing table already covers all 33 `DEFAULTS` entries of `shortcuts.rs` (verify by listing the action names against the table). Add a short "Fixed keys" table for the keys the app handles outside the keymap, which cannot be rebound: `Escape` closes the filter, sort or right-click menu and leaves Compare (`crates/app/ui/src/main.ts` keydown handler), and cancels key capture in Settings; `Tab` / `Shift+Tab` cycle the buttons of the first-launch developing-software dialog; `CmdOrCtrl+R` = `File > Reload Folder` and `CmdOrCtrl+,` = `Riffle > Settings...` (`File > Settings...` on Windows and Linux) from `crates/app/src/main.rs`; in the Settings window the tab strip takes `ArrowLeft` / `ArrowRight` / `Home` / `End` (`crates/app/ui/src/tabs.ts`). Keep the existing note that keys are changed in Settings.
    - Keep Lightroom listed before PhotoLab wherever both appear.
    - Commit as `docs: trim the READMEs to the culling happy path`.
    - Verify: `mise run ci` (lychee `--offline --include-fragments` must resolve `docs/usage.md#keys`).

## Trade-offs and risks

- Bullet order follows the culling flow (look, check focus, pick from the burst, compare, discard) rather than the current order; the user approved the plan as is.
- The key-reference pointer lives at the end of "Key features", where the Keys table used to be; "First steps" stays four lines.
- usage.md's Sharpness cue bullet is out of date independently of this task; it is rewritten here because it must host the YuNet attribution.
- The fixed-key rows for `Tab` in the first-launch dialog and the Settings tab strip are minor accessibility behaviors; they are included so the Keys section covers every key the app handles.

## Progress

- (2026-09-24) Step 1 complete
