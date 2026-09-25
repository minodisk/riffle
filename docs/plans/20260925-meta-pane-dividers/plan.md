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

# Meta pane: divider between groups, `Standard` section label, `Analysis` heading

## Purpose

The meta pane's only divider today sits above the `Maker note` section label
(`#meta .section { border-top: 1px solid #3a3a3a }` in
`crates/app/ui/style.css`), so the divider separates two sections of the same
group while the two groups run together. This work moves the divider to where
the provenance actually changes (before every group heading except the first),
gives the standard EXIF tags an explicit `Standard` label so both EXIF
sections are named, and renames the `Riffle` group heading to `Analysis`,
which says what its rows are (values Riffle computes from the image, as
opposed to values the camera recorded). Target rendering:

```
DSC00001.ARW
EXIF
Standard
  Aperture  f/2.8
  ...
Maker note
  Shutter type  ...
────────────
Analysis
  Sharpness  303.5
  ...
```

## Steps

- [x] Step 1: Move the meta pane divider between groups, label the standard EXIF section `Standard`, and rename the `Riffle` heading to `Analysis`
  - Done when:
    - `crates/app/ui/src/meta.ts` exports `STANDARD_LABEL = "Standard"` next
      to `MAKER_NOTE_LABEL`, and the EXIF group's first section is built with
      `section(STANDARD_LABEL, ...)` instead of `section(null, ...)`. The
      analysis group's section stays `label: null`.
    - `RIFFLE_HEADING = "Riffle"` is renamed to
      `ANALYSIS_HEADING = "Analysis"` everywhere it is referenced (meta.ts,
      its tests, and any other importer), and comments that call the group
      "the Riffle group" are updated.
    - `crates/app/ui/style.css`: `#meta .section` loses `border-top` (keep
      its font size and color; drop `padding-top` if it only existed to space
      off the border). A new rule `#meta .group ~ .group` draws
      `border-top: 1px solid #3a3a3a` with a small `padding-top` (about
      `0.25rem`) before every group heading except the first (all `line()`
      elements are same-tag siblings under `#meta`, so `:first-of-type` does
      not work). Comments describe the new roles (section label inside a
      group; divider between groups).
    - No divider above `EXIF`, `Standard`, or `Maker note`; one divider above
      `Analysis` when both groups are shown; none when only one group is
      shown.
    - `crates/app/ui/src/meta.test.ts` expects `label: STANDARD_LABEL` for the
      EXIF group's first section (including the case with no maker note
      fields, which becomes `[STANDARD_LABEL]`), and `ANALYSIS_HEADING` for
      the analysis group.
    - Docs: `docs/usage.md` (which mentions a `Maker note` divider, and may
      name the `Riffle` heading) is reworded to match. `CLAUDE.md`'s
      description of `src/meta.ts` ("EXIF with its Maker note section, and
      Riffle, ...") becomes "EXIF with its Standard and Maker note sections,
      and Analysis, ...". `README.md` / `README.ja.md` are touched only if
      they describe this layout (keep them in sync if so).
    - `mise run ci` passes.
  - Implementation approach:
    - `main.ts` `renderMeta` already renders `line("section", label)` whenever
      `section.label !== null`, so it should need no change.
    - Commit message: `feat(app): divide the meta pane between groups, label the standard EXIF section, and rename Riffle to Analysis`.

## Trade-offs and risks

- If `#meta` children turn out to be wrapped, `#meta .group ~ .group` stops
  matching; fall back to a class set in `main.ts`.
- The in-flight plan `docs/plans/20260925-sony-af-meta/plan.md` also edits the
  `Maker note` section and mentions "under the `Maker note` divider". Its
  later steps' test expectations will need `STANDARD_LABEL` /
  `ANALYSIS_HEADING` once this lands; note the interaction in `learnings.md`.

## Progress

- (none yet)
