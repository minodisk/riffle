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

# Thin scrollbar on the strip's menus

## Purpose

The strip's filter and sort menus (`#filter-menu`, `#sort-menu`) show the
thick default scrollbar on Windows when their item list overflows their
`max-height`. `#strip` and `#folders` already use a thin scrollbar
(`scrollbar-width: thin; scrollbar-color: #444 transparent;`), so the menus
look out of place. Giving the menus the same thin scrollbar makes the strip's
popups consistent with the rest of the UI. The context menu (`#context-menu`)
shares the same rule block, so it gets the same treatment.

## Steps

- [x] Step 1: Add the thin scrollbar to the filter, sort and context menus
  - Done when:
    - The `#filter-menu, #sort-menu, #context-menu { ... }` block in
      `crates/app/ui/style.css` (around line 108, the one declaring
      `position: absolute` and `overflow-y: auto`) also declares
      `scrollbar-width: thin;` and `scrollbar-color: #444 transparent;`
    - `mise run ci` passes
  - Implementation approach:
    - CSS only; the change is two added lines in that one block, after
      `overflow-y: auto;`, matching the values used by `#strip`
      (`style.css` lines 269-270) and `#folders` (lines 288-289)
    - Keep it unconditional, like the existing `#strip` / `#folders` rules;
      no platform-specific override exists in the codebase
    - Do not extract a shared scrollbar rule or touch the neighbouring blocks
    - No documentation update: `README.md`, `README.ja.md` and
      `docs/agents/tauri-app.md` do not describe the scrollbar styling of the
      menus (the only scrollbar mention in `tauri-app.md`, line 1137, is
      about the strip's gutter and layout, not the menus)
    - Commit: `fix(app): use a thin scrollbar on the strip's menus`

## Progress

- (none yet)
