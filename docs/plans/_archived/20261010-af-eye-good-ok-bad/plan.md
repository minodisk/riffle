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

# AF eye filter items: Good / OK / Bad / Unknown

## Purpose

After `20261010-sharp-only-filter` (PR #756) the strip's filter menu `AF eye`
section reads `Good` / `Sharp only` / `Soft` / `Unknown`. The four items already
partition the frames (the good tier, in focus but not good, not in focus, not
judged yet; `passes()` in `crates/app/ui/src/filter.ts` checks
`state.candidates.has(tier ?? candidate ?? "unknown")`), but the labels do not
read as one scale: `Sharp only` and `Soft` are a different vocabulary from
`Good`. The user chose to rename them so the section reads `Good` / `OK` /
`Bad` / `Unknown`. Behavior, the `data-candidate` keys (`good` / `candidate` /
`not_candidate` / `unknown`), the `AfEye` type do not change. The rename is
a label and documentation change; the follow-up in Step 1 also gives each judged
state its own icon and color (bright green `Good`, bright green `scan-eye` `OK`,
gray `scan` `Bad`) on the strip, the filter menu and the focus mark.

## Steps

- [x] Step 1: Rename the `AF eye` filter items `Sharp only` -> `OK` and `Soft` -> `Bad` in the menu, the comments and the user docs, and give each judged state its own icon and color on the strip, the filter menu and the focus mark
  - Done when:
    - `crates/app/ui/index.html`: the `data-candidate="candidate"` item's text
      is `OK` and the `data-candidate="not_candidate"` item's text is `Bad`
      (both keep their `<i class="face"></i>` prefix; the `data-candidate`
      values are untouched).
    - `crates/app/ui/src/main.ts` (the comment above the AF eye face icon
      coloring) names `OK` instead of `Sharp only`.
    - `crates/app/ui/src/filter.ts` top comment, if it names item labels, uses
      the new ones.
    - `crates/app/ui/src/filter.test.ts`: the test named
      `"Sharp only does not pass a good frame"` is renamed to use `OK`, and the
      `not_candidate passes a soft frame only` test is reworded to `Bad`; no
      assertion changes.
    - `docs/humans/usage.md` Filter menu bullet and `docs/humans/usage.ja.md`:
      the four items are `Good` (bright green mark, the good tier), `OK` (bright
      green with the `scan-eye` icon, the AF eye in focus but not good), `Bad`
      (gray with the `scan` icon) and `Unknown`
      (white, including files the second pass has not reached), and the
      "check `Good` and `Sharp only` together" sentence becomes "`Good` and
      `OK` together".
    - `README.md` (`AF eye` item list, and the `scan-face` icon sentence on the
      `Good` and `Sharp only` items) and the matching sentences in
      `README.ja.md` use `OK` / `Bad`.
    - `docs/plans/20261008-burst-keep-score/plan.md`, the note that
      `20261010-sharp-only-filter` rewrote the `AF eye` text: updated to name
      the current labels (`Good` / `OK` / `Bad` / `Unknown`) so a later step of
      that plan does not reintroduce `Sharp only`.
    - `rg "Sharp only"` finds nothing outside `docs/plans/_archived/`,
      `docs/plans/review-history/` and this plan folder.
    - Icons and colors (added after the rename was implemented, at the user's
      request): the strip cell and the filter menu item show the same icon per
      state: `good` the Lucide `scan-face` in bright green, `candidate_only`
      (`OK`) the Lucide `scan-eye` in bright green, `not_candidate` (`Bad`)
      the Lucide `scan` in gray, and `unknown` no icon (the menu keeps an
      empty slot for alignment). Today only `good` frames get the strip icon
      and only `Good` / `OK` items get a menu icon.
    - `FOCUS_MARK_COLORS` in `crates/app/ui/src/focus.ts` matches: the focus
      mark of a `candidate_only` frame is the same bright green as `good`, and
      a `not_candidate` frame's mark is the same gray as its icon (`unknown`
      stays white). Pick one gray that reads on the dark UI and on photos.
    - `scan-eye` and `scan` are inlined in `crates/app/ui/src/icons.ts` like
      `SCAN_FACE_SVG` (Lucide, same license handling).
    - The docs (`usage.md` / `usage.ja.md`, `README.md` / `README.ja.md`)
      describe the mark colors and the strip / menu icons as they now are,
      including the sentence about the `scan-face` icon.
    - Tests pin the state -> icon / color mapping where a pure function holds
      it (`focus.test.ts` for `FOCUS_MARK_COLORS` / `markState`, `strip`
      helpers if pure).
    - `mise run ci` passes.
  - Implementation approach:
    - The rename part is a label change only. The icon part changes only the
      colors in `FOCUS_MARK_COLORS` and the icon rendering; do not touch the
      focus.ts thresholds, `photoTier` / `markState` logic, `AfEye` /
      `FocusCandidate` values, `crates/core/src/eyes.rs`,
      `crates/cli/src/main.rs` or `docs/plans/20261010-mesh-roll/`.
    - Keep each English / Japanese doc pair in sync in the same PR.
    - Leave the other `Soft` hits alone: `crates/app/src/exif.rs` (Sony scene
      modes), archived plans, review history, and the older `Sharp` / `Soft`
      mentions inside `docs/plans/20261008-burst-keep-score/` that record
      earlier steps; only the #756 note is forward-looking.

## Trade-offs and risks

- **Other UI surfaces.** None label the same four classes: the meta pane shows
  `AF eye in focus` as a percentage, the strip mark carries a per-state icon and color, and
  `companion.ts` / `mcp.rs` carry no `Sharp` / `Soft` text. So the rename stays
  in the filter menu and its docs.
- **History in the burst-keep-score plan.** Only the #756 note is updated;
  rewriting older mentions would falsify what those steps did at the time.
- **Good and OK share a color.** They differ by icon shape only (`scan-face`
  vs `scan-eye`); the user chose this, and moved the focus mark to the same
  colors so the mark and the icons agree.
- **Label meaning.** `OK` / `Bad` no longer say on their own what is judged;
  the section heading `AF eye` and the docs carry that.

## Progress

- (2026-10-10) Step 1 complete
