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

# Meta pane: flat EXIF / Maker note / Analysis groups

## Purpose

After #444 the meta pane shows an `EXIF` group split into `Standard` and
`Maker note` sub-sections, an `Analysis` group, and a thin divider above every
group heading but the first. The two heading levels and the border lines add
visual noise for little information. This work flattens the pane to three
same-level group headings, `EXIF` (standard tags), `Maker note` (the vendor
MakerNote rows) and `Analysis` (sharpness, focus), with no sub-headings and no
dividers:

```
DSC00001.ARW
EXIF
  Aperture ...
Maker note
  Shutter type ...
Analysis
  Sharpness ...
```

Each heading is styled like today's `.group` heading. A group with no rows is
dropped, as groups and sections are today.

## Steps

- [x] Step 1: Flatten `metaGroups` to three row-only groups, drop the section labels and the divider, and update tests and docs
  - Done when:
    - `metaGroups(sony, 303.54)` returns three groups in order `EXIF`,
      `Maker note`, `Analysis`, each `{ heading, rows }` with the same rows,
      in the same order, as today's corresponding section; the Leica fixture
      yields `EXIF` and `Maker note` (Focus distance); a fixture with every
      maker note field `null` yields no `Maker note` group; `empty` yields
      only `Analysis`; `meta === null` yields only `Analysis`; no score and no
      focus yields no `Analysis` group; the existing focus / eye-sharpness
      cases hold with `groups[0].rows`.
    - `renderMeta()` in `main.ts` appends `line("group", heading)` and one
      `<dl>` per group; no `line("section", ...)` remains.
    - `style.css` no longer has the `#meta .group ~ .group` divider rule or
      the `#meta .section` rule; `#meta .group` is unchanged apart from its
      comment.
    - `STANDARD_LABEL` and the `MetaSection` type / `section()` helper are
      gone; `MAKER_NOTE_HEADING` (renamed from `MAKER_NOTE_LABEL`, to match
      `EXIF_HEADING` / `ANALYSIS_HEADING`) is exported and used by the test.
    - `docs/usage.md`, `CLAUDE.md`, `README.md` and `README.ja.md` describe
      the new layout (see approach); `mise run ci` passes.
  - Implementation approach:
    - `crates/app/ui/src/meta.ts`: make `MetaGroup = { heading: string; rows: MetaRow[] }`
      and delete `MetaSection`. Sections become pointless once every group
      has exactly one unlabeled section, so the flat shape is the minimal data
      shape. Replace `section()` / `group()` with one `group(heading, rows)`
      helper that does today's null-drop, and filter the result with
      `rows.length > 0`. Push `EXIF` and `Maker note` inside the
      `if (meta !== null)` block, then `Analysis` unconditionally, as today.
      Drop any mention of sections from comments.
    - `crates/app/ui/src/main.ts` `renderMeta()`: drop the inner `sections`
      loop and the `line("section", ...)` call; one `<dl>` per group.
    - `crates/app/ui/style.css`: delete the `#meta .group ~ .group` rule and
      the `#meta .section` rule; update the `#meta .group` comment to name all
      three headings.
    - `crates/app/ui/src/meta.test.ts`: import `MAKER_NOTE_HEADING` instead of
      `MAKER_NOTE_LABEL` / `STANDARD_LABEL`; rewrite expectations to the flat
      shape; the "leaves out the maker note section" test becomes "leaves out
      the Maker note group".
    - Docs: `docs/usage.md` Meta pane bullet drops the divider and the
      `Standard` / `Maker note` sub-section wording and describes **EXIF**,
      **Maker note** and **Analysis** as three groups. `CLAUDE.md`'s
      description of `src/meta.ts` becomes "(EXIF, Maker note and Analysis,
      whose rows include ...)". `README.md` "`Maker note` section" becomes
      "`Maker note` group" and `README.ja.md` "`Maker note` セクション"
      becomes "`Maker note` グループ", in the same PR.
    - Commit: `feat(app): flatten the meta pane into EXIF, Maker note and Analysis groups`.

## Trade-offs and risks

- Flat `{ heading, rows }` vs. keeping `sections`: the flat shape deletes the
  `MetaSection` type, the `section()` helper and the label branch in the
  renderer; keeping sections would leave a one-element array with a `null`
  label in every group. Chosen: flat.
- `docs/plans/20260925-sony-af-meta` is complete (all steps merged) but not
  archived; its wrap-up only moves files under `docs/plans/`, so no conflict
  is expected.

## Progress

- (2026-09-25) Step 1 complete
