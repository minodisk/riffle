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

# Rating and colour label in the right-click menu, also on the main view

## Purpose

The strip's HTML right-click menu (`crates/app/ui/src/context.ts`, opened by
`openContextMenu` in `crates/app/ui/src/main.ts`) only offers Pick / Reject /
Unflag. The user also wants to set the star rating (clear, 1–5) and the
colour label (Red, Orange, Yellow, Green, Blue, Pink, Purple, clear) from it,
on the same targets the flag items act on. The same menu must also open on
the main photo view (`#canvas`), which today shows the webview's native
context menu; in compare mode (`v`) a right-click on one pane must target that
pane's photo, the way a left-click makes it the active pane.

Every menu item runs the same `runAction(action)` the keyboard uses, so the
`judge` path (selection targets, or the active compare pane), sidecar writing
(`set_rating`, XMP or `.dop`), undo/redo and auto-advance behave exactly like
the keys.

## Decisions (agreed with the user)

- The menu marks the focused file's current flag, rating and label with
  `aria-checked="true"` (colour label actions are toggles, so the marking makes
  that self-explanatory).
- In compare mode, a right-click in the gap between panes or outside every pane
  does nothing (no menu).
- Two steps, two PRs.

## Steps

- [ ] Step 1: Rating and colour label items in the strip's context menu
  - Done when:
    - Right-clicking a strip cell shows, in order and separated by `<hr>`:
      Pick / Reject / Unflag; 1 star … 5 stars, No stars (`clear`); Red,
      Orange, Yellow, Green, Blue, Pink, Purple, No label (`clearlabel`).
    - Each item shows the first key of its binding as the flag items do
      (`displayKey`), empty when unbound.
    - The items matching the focused file's current flag, rating and label
      (`files[index]`, or `compareActivePath` when comparing) carry
      `aria-checked="true"` and are visibly marked; "No stars" / "No label" /
      "Unflag" are marked when the value is unset.
    - Choosing an item calls `runAction(<action>)` with the existing action
      names (`rate1`…`rate5`, `clear`, `red`…`purple`, `clearlabel`), so the
      change applies to the selection, is written to the sidecar, and is one
      undo entry, identical to the key.
    - `crates/app/ui/src/context.test.ts` covers the new items: labels,
      action names, order/grouping, checked state, default keys, an
      overridden key, and unbound actions.
    - `mise run ci` passes.
  - Implementation approach:
    - Generalise `flagMenuItems` in `crates/app/ui/src/context.ts` to return
      groups (one array per section) built from a static table of
      `[action, label]` pairs per section, keeping the
      `{ action, label, shortcut }` shape plus a `checked` boolean computed
      from a passed-in `{ rating, flag, label }`. Keep it free of DOM.
    - Do not add `clearall` or `rejectRest`.
    - In `openContextMenu` (`main.ts`) render an `<hr>` between groups and set
      `aria-checked`; add `#context-menu hr` and `#context-menu
      [aria-checked="true"]` rules to `crates/app/ui/style.css` next to the
      filter menu's equivalents.
    - No submenus; the menu is a flat `<div role="menu">` of buttons.
- [ ] Step 2: The same context menu on the main photo view, targeting the compare pane
  - Done when:
    - Right-clicking `#canvas` opens the same menu (`openContextMenu`) and
      suppresses the native webview menu (`preventDefault`); nothing opens
      when no files are shown.
    - Outside compare mode the menu acts on the current targets exactly as a
      keyboard judgement would; the selection is not changed by the
      right-click.
    - In compare mode a right-click on a pane first makes that pane active
      (`compareActivePath = frame.path`, `drawCompare()`, `renderMeta()`),
      then opens the menu, so `judge` applies the item to that photo. A
      right-click in the gap between panes or outside every pane does nothing
      (the native menu is still suppressed).
    - `mise run ci` passes.
  - Implementation approach:
    - The `canvas` `click` handler in `main.ts` hit-tests the compare grid
      inline. Extract that into a helper (e.g. `comparePaneAt(clientX,
      clientY)`) used by both the `click` and the new `contextmenu` handler.
      If the pure grid maths moves into `crates/app/ui/src/compare.ts`, add a
      unit test in `compare.test.ts`.
    - The strip's own `contextmenu` handlers stay as they are.
    - Manual check to record in `learnings.md`: compare mode with 2 / 3 / 4
      frames, right-click each pane, verify the meta pane switches and the
      chosen rating/label lands on that file and is undoable; verify a
      right-click on the single-image view with a multi-selection applies to
      the whole selection.

## Trade-offs and risks

- Labels keep the keyboard's toggle semantics (reusing `runAction`); the
  checked marking makes this visible.
- Auto-advance after `rate1`–`rate5` / `pick` / `reject` applies from the menu
  too, as it already does for the menu's Pick / Reject.
- Menu height (~17 items): `menuPosition` already flips and clamps and
  `#context-menu` scrolls; verify on a small window in Step 1.

## Progress

- (none yet)
