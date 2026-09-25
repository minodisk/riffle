# Learnings

## Step 1

- `#meta .group ~ .group` works because every line `renderMeta` appends
  (`div.name`, `div.group`, `div.section`, `dl`) is a direct child of `#meta`;
  the general sibling combinator does not care that the `dl`s are a different
  tag. `:first-of-type` would not work since all lines are `div`s.
- `#meta .section` kept its `margin-top`; its `padding-top` only spaced the
  label off the old border, so it went with the border.
- Interaction with `docs/plans/20260925-sony-af-meta/plan.md`: that plan's
  later steps edit the `Maker note` section and mention "under the
  `Maker note` divider". Their test expectations now need `STANDARD_LABEL`
  (not `null`) for the EXIF group's first section and `ANALYSIS_HEADING`
  (formerly `RIFFLE_HEADING`) for the analysis group, and doc wording should
  say "under `Maker note`" rather than "divider".
- `README.md` / `README.ja.md` do not describe the meta pane layout, so they
  were left alone; `docs/usage.md` had two mentions of the Riffle group (the
  meta pane bullet and the sharpness bullet).
- The first `mise run fmt` in this worktree failed with `Command "vp" not
  found` (environmental, the frontend toolchain was not ready yet); a re-run
  passed without any change on our side.
