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

# Fix the stale keymap sentence in CLAUDE.md

## Purpose

The Layout section of `CLAUDE.md` says `crates/app/src/shortcuts.rs` holds "the
default keys, whose color label keys differ per sidecar format". That was true
when color labels landed (#118) but is no longer: `DEFAULTS` in
`shortcuts.rs` is a single static table (`red`..`purple` = `ctrl+alt+1`..`7`,
`clearlabel` = `ctrl+alt+0`), `Keymap::defaults()` takes no format argument,
and the file never references `SidecarFormat`. Removing the clause keeps the
agent-facing layout description accurate so nobody goes looking for a
per-format keymap that does not exist.

## Steps

- [x] Step 1: Remove the ", whose color label keys differ per sidecar format" clause from CLAUDE.md
  - Done when:
    - The `src/shortcuts.rs` description in the Layout section of `CLAUDE.md`
      reads "the keymap: the default keys and the user's overrides, persisted
      in the `shortcuts` key" (or equivalent) with no mention of color label
      keys differing per sidecar format
    - No other text in `CLAUDE.md` or `README.md` claims the default keys
      depend on the sidecar format (grep for "differ per sidecar" and
      "per sidecar format" returns nothing)
    - `mise run ci` passes
  - Implementation approach:
    - Only `CLAUDE.md` changes (the Layout section). Docs-only; no code change
    - Re-confirm before editing that `crates/app/src/shortcuts.rs` still has no
      `SidecarFormat` / `sidecar_format` reference and that `Keymap::defaults()`
      takes no argument
    - Commit as `docs: drop the stale per-format color label clause from CLAUDE.md`

## Trade-offs and risks

- Minimal deletion as asked, rather than a broader rewrite of the sentence.

## Progress

- (2026-09-22) Step 1 complete
