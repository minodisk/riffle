# Learnings

## Step 1

- All 33 `DEFAULTS` entries in `crates/app/src/shortcuts.rs` were already in
  `docs/usage.md`'s Keys table (the label keys in the separate label table), so
  the only addition was a "fixed keys" table.
- The fixed keys were confirmed in source: `Escape` in the `main.ts` keydown
  handler (filter, sort and context menus, Compare) and `settings.ts` (key
  capture); `Tab` trapping in the first-launch dialog in `main.ts`;
  `CmdOrCtrl+R` / `CmdOrCtrl+,` accelerators in `crates/app/src/main.rs`; the
  tab-strip keys in `crates/app/ui/src/tabs.ts`.
- The old usage.md Sharpness cue bullet predated face detection and eye-AF
  scoring; it now matches the README's scoring order.
- CI failure: lychee checks `docs/plans/**` too, and resolves relative links
  from the plan's own directory. An example sentence in plan.md containing
  `[docs/usage.md](./docs/usage.md)` failed; wrapping it in backticks fixed it.

## Deferred issues (todo candidates)

- Document that lychee resolves relative links in `docs/plans/**` from the
  linking file's own directory.
  - Change: add a short note for agents writing plan / learnings / doc
    Markdown, either as a new `docs/agents/docs-writing.md` or as a line in the
    `develop` / `planner` workflow docs. `mise run lint` runs lychee over
    `docs/plans/**`, so an illustrative link such as `./docs/usage.md` written
    inside a plan fails CI; use a correctly relative path, or wrap an example
    path in backticks when it is not a real link target.
  - Rationale: Step 1's first CI run failed on exactly this (see the last
    bullet of "## Step 1" above).
  - Done when: a guide or workflow section that an agent writing a plan.md or
    learnings.md with a doc-to-doc link would read beforehand carries this
    note.
