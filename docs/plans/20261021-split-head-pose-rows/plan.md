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

# Head pose as three meta pane rows

## Purpose

The meta pane's Analysis group shows the judged face's head pose as one
`Head pose` row, `yaw 12°, pitch -5°, roll 4°`. Each angle has its own
meaning and sign convention, and a one-line triple is hard to scan next to
the other single-value rows (`AF eye in focus`, `Eyes`). Showing yaw, pitch
and roll as three rows (`Yaw`, `Pitch`, `Roll`) puts each angle in the
label / value layout every other row uses, so a value can be read and
compared at a glance.

## Steps

- [x] Step 1: Show yaw, pitch and roll as separate Analysis rows
  - Done when:
    - `metaGroups` in `crates/app/ui/src/meta.ts` emits `Yaw`, `Pitch` and
      `Roll` rows, in that order, right after `Eyes`, each valued by the
      existing `degrees()` formatting (`12°`, `-5°`, `0°` for a rounded
      `-0`), instead of the one `Head pose` row.
    - The pose comes from the same place as before: the pass-2 stored
      `focus.pose` when `focus.eyes_ear` is stored, else the `eyes_of`
      judgment's `eyes.pose`; and the three rows are all absent exactly when
      the old row was (no pose from that source).
    - `crates/app/ui/src/meta.test.ts` covers the three rows in every case
      that asserted `Head pose` (the judged pose, the stored pose, the
      stored-over-judged preference, the far-profile yaw past 90 and the
      `-0` rounding, the absence cases).
    - The docs that name the row say `Yaw`, `Pitch` and `Roll` rows instead,
      with each translation pair updated in the same PR: `CLAUDE.md` (the
      `src/meta.ts` entry), `README.md` + `README.ja.md`,
      `docs/humans/usage.md` + `docs/humans/usage.ja.md` (the meta pane
      bullet, the `Eyes` / `Head pose` description and the MCP `get_view`
      row's "signed as the meta pane's `Head pose`"),
      `docs/humans/performance.md` + `docs/humans/performance.ja.md`.
    - `mise run ci` passes.
  - Implementation approach:
    - In `metaGroups` pick the pose once
      (`const pose = stored ? (focus?.pose ?? null) : (eyes?.pose ?? null)`)
      and add three `[label, pose === null ? null : degrees(pose.yaw)]`-style
      entries to the `ANALYSIS_HEADING` rows; `group()` already drops rows
      whose value is `null`, which keeps the absence rule unchanged with no
      new branching.
    - `poseValue` is used only by `meta.ts` and its test. Remove it (and its
      import in the test) rather than keep an unused export; if a helper is
      wanted for the per-angle text, keep `degrees()` as is or export it for
      the test instead. Keep `degrees()`'s comment about the far-profile yaw
      and the `-0` rounding.
    - Labels: `Yaw`, `Pitch`, `Roll`, matching the existing capitalized
      single-word row labels (`Eyes`, `Reason`, `Confidence`). The user chose
      these over `Head yaw` / `Head pitch` / `Head roll`.
    - Do not touch `crates/core/src/rank.rs`: its `("Head pose", 3..6)` is
      the ranking's `Reason` label (one reason summing the three pose
      columns), not the meta row, and the `usage.md` line listing the
      reasons (`Sharpness`, `AF eye focus`, `Eyes open`, `Head pose`, `Good
      photo`) stays as is.
    - Do not touch the `eyes_of` / `get_view` payloads or `companion.ts`;
      the MCP `pose` object keeps its `yaw` / `pitch` / `roll` fields and
      only the doc's cross-reference to the meta pane's row names changes.
    - Commit as `feat(app): show yaw, pitch and roll as separate meta rows`.

## Trade-offs and risks

- Ranking reason vs. meta row: the `Reason` row can still read `Head pose`
  (from `rank.rs`'s reason label) while the pose rows are now `Yaw` /
  `Pitch` / `Roll`. This is a deliberate non-change (the reason is one
  summed group of the three angles); renaming the reason would touch the
  ranking prior's column mapping and is out of scope.
- `todo.md` lines 829–839 describe an unchecked real-device check that
  expects `Head pose  yaw N°, pitch N°, roll N°`. Reword it to the three
  rows in the same PR so the check stays accurate.

## Progress

- (none yet)
