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

# Pin the status lines to the bottom of the meta pane

## Purpose

`renderMeta()` in `crates/app/ui/src/main.ts` rebuilds the right-hand
`#meta` pane on every redraw: the file name, the `<dl>` of shooting
settings, and then, in the same flow, the transient lines: the `note` set by
`setStatus` (an invoke error), the scan progress (`scanning 1449 / 1795`, or
`N failed` once the scan ends), the `1:1` zoom indicator, and the sticky
`errors.list()` entries with their dismiss buttons. Because they follow the
metadata list, they sit right under it and jump around as the list grows or
shrinks.

After this work the status lines are rendered in their own block anchored to
the bottom of the right pane, separate from the metadata list, which is
otherwise unchanged.

## Steps

- [x] Step 1: Render the status lines in a bottom-anchored block of the right pane
  - Done when:
    - The scan progress, the `note`, the sticky errors and the `1:1`
      indicator are no longer children of the metadata block; they render in
      a separate element pinned to the lower end of the right pane.
    - The file name and the `<dl>` of shooting settings render exactly as
      before (same elements, classes, order, styling).
    - The metadata block still scrolls on its own when it is taller than
      the pane; the status block stays visible at the bottom and does not
      scroll away with it.
    - Error dismiss buttons still work (clicking removes the entry and
      redraws).
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/app/ui/index.html` line 172: replace `<div id="meta"></div>`
      with a wrapper that holds two children, e.g.
      `<div id="info"><div id="meta"></div><div id="meta-status"></div></div>`
      (avoid a bare `#status`, which `settings.html` / `settings.ts` already
      use). Keep `#meta` as the id of the metadata block so `metaEl` and the
      `#meta .name` / `#meta dl` / `#meta dt` / `#meta dd` CSS rules stay
      untouched.
    - `crates/app/ui/style.css` (`#meta` block, lines 461-512): move the
      `flex: 0 0 220px`, `background`, `font-size`, `line-height`,
      `box-sizing` and pane-level padding to the wrapper and make the wrapper
      `display: flex; flex-direction: column; min-height: 0`. Give `#meta`
      `flex: 1; min-height: 0; overflow-y: auto`, and the status block
      `flex: none`. Retarget `#meta .note`, `#meta .error` and
      `#meta .error button` to the new status element.
    - `crates/app/ui/src/main.ts`: add a second element lookup next to
      `metaEl` (line 84); in `renderMeta()` call `replaceChildren()` on both,
      append the name and `<dl>` to `metaEl` as now, and append the `note`,
      `scanning`, `1:1` and `errors` lines to the status element instead.
      Update the comments above `renderMeta` and on `note` (line ~128) to
      describe the new placement. Do not change `setStatus`, the scan event
      handlers, or `ErrorList`.
    - No existing test covers `renderMeta` or the `#meta` DOM, so no test
      update is needed; do not add a DOM test harness for this change.

## Trade-offs and risks

- The `1:1` zoom indicator moves to the bottom block too (user decision), so
  the metadata block contains only the name and settings.
- A long sticky error list grows upward and shrinks the metadata area;
  capping it is a possible follow-up, not part of this change.

## Progress

- (none yet)
