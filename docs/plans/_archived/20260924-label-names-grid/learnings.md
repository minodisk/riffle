# Learnings

## Step 1

- The grid rule is scoped as `#label-names:not([hidden])`: a plain
  `#label-names { display: grid; }` has id specificity and would override the
  global `[hidden] { display: none; }`, leaving the block visible for `.dop`.
- Each `<label>` row is a subgrid spanning both columns, so the implicit
  label/input association stays intact with no markup change; the first column
  is `max-content`, so it tracks the widest color name.
- Verification of the rendered layout was not possible from the agent (no
  display, no DOM tests for the settings window); only `mise run ci` was run.
  The visual check (alignment, hiding on `.dop`, persistence) remains manual.
- `mise run fmt` failed first with `Command "vp" not found` because the fresh
  worktree had no `node_modules`; `pnpm install --frozen-lockfile` fixed it.
