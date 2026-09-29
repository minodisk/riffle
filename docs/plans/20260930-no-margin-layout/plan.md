<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Lay the frontend out with flex / grid and gap, not margin

## Purpose

`crates/app/ui/style.css` spaces siblings with `margin` in 24 places (the
meta pane's group headings, the strip bar's `#position` and the folder
tree's `.count` pushed right with `margin-left: auto`, the dialogs'
paragraphs and lists, the settings chips) and leans on the user agent's
default margins for the `<p>`, `<h2>` and `<ul>` inside the settings
panels. Every parent is already (or trivially becomes) a flex or grid
container, so the same spacing can come from `gap` on the parent and
`padding` inside a component, with a global reset so no spacing is
implicit. Once done, spacing is readable from the parent alone, an element
can be added or hidden without its neighbors' margins leaking, and the
styling guide gains a grep-checkable rule: no `margin` outside the reset and
one documented exception, no `float`, no `.style.margin` in TypeScript.

The audit (selector, current value, replacement) is in [`audit.md`](./audit.md).
There is no `float` in the stylesheet, no `.style.margin` / `.style.float`
in `crates/app/ui/src/*.ts`, and no `style=` attribute in
`crates/app/ui/index.html`, so the work is entirely in `style.css` plus two
small DOM changes (`main.ts` and, if needed, `settings.ts`).

Flexbox `gap` is already load-bearing in the current stylesheet (`.button`,
`.menu-item`, `.folder`, `.sidenav`, `.dialog-actions`, `#settings-body`,
`#format-choices`, `.chip`, `.copyable`, `#meta-status .error`), so this
plan adds no new engine requirement. `safari13` in `vite.config.ts` is
Lightning CSS's syntax target; it neither down-levels nor warns on a layout
feature. Flex gap needs Safari 14.1 (macOS 11, or 10.15 with the Safari
14.1 update) and WebKitGTK 2.32; grid gap is older and safe everywhere;
WebView2 is evergreen. The de-facto minimum WebKit of the app is therefore
already 14.1 / WebKitGTK 2.32, and the guide records that.

## Decisions

Made by the user on 2026-09-30, before implementation:

- One step, one PR: the reset, the dialogs, the main window and the guide
  land together (the reset cannot be separated from the dialogs, which lean
  on UA margins today).
- Settings panels: `gap: 0.5rem` everywhere. The paragraphs get tighter
  than their UA `1em` margins gave; this visible change is accepted.
- `.folder`: a grid (`14px minmax(0, 1fr) max-content`), not
  `.name { flex: 1 }`, so the name span keeps hugging its text and the
  rename double-click target does not grow.
- `td.keys`: try `display: flex` on the `td` itself; if the shortcuts
  table misaligns in the Windows check, fall back to a `div.keys` wrapper
  inside the cell (`settings.ts`) and record which was used in
  `learnings.md`.
- Meta pane: wrap each heading + `dl` in a `div.group` in `renderMeta`.
- `.menu-separator`'s `margin: 4px -4px` stays as the one documented
  exception (a negative-margin bleed across the menu's padding).
- `todo.md` is not edited in the step. The open macOS / Linux checks (and
  the Windows check if it cannot be run) go under
  `## Deferred issues (todo candidates)` in `learnings.md`; the `develop`
  wrap-up's todo-curator files them.

## Steps

- [x] Step 1: Replace every layout margin with flex / grid + gap, add the reset, and write the rule into the styling guide
  - Done when:
    - **Reset.** A `/* Reset */` block at the top of `style.css` (before
      the component block) sets `margin: 0` on `body, h1, h2, h3, p, dl,
      dd, ul, ol, hr, pre, figure, table, input, button, select`, with a
      comment saying spacing comes only from `gap` and `padding`.
      `html, body` loses its `margin: 0`; `.dialog-title`,
      `#format-box .note`, `.folder input.name`, `.cell input.name`,
      `#meta dl` and `#meta dd` lose their `margin: 0` line (their other
      declarations stay).
    - **Format dialog.** `#format-box` is a flex column with
      `gap: 0.75rem`; the `#format-box p`, `#format-choices`
      `margin-bottom` and `#format-error` margin rules are gone.
    - **Trash and sequence dialogs.** `#sequence-box` / `#trash-box`
      (already flex columns) get `gap: 0.5rem`; the `#sequence-title,
      #trash-title`, `#sequence-box p, #trash-box p`, `#sequence-rows,
      #trash-rows` and `#sequence-failed, #trash-failed` margin
      declarations are gone (other declarations stay).
    - **Settings.** `#settings-box` gets `gap: 0.5rem` (replacing
      `#settings-header`'s `margin-bottom`); `#settings-content` gets
      `gap: 0.5rem` (replacing `#settings-status`'s `margin-top`); each
      `#settings-dialog [role="tabpanel"]:not([hidden])` is a flex column
      with `gap: 0.5rem`; `#label-names:not([hidden])` gets
      `row-gap: 0.5rem`; `#clear-index-note` and
      `#settings-dialog .copyable pre` lose their margins; `#mcp-examples`
      and each example block it holds (given a class in `settings.ts`'s
      `showMcpState`, e.g. `example`) are flex columns with `gap: 0.5rem`.
      `#settings-dialog .chip` loses its `margin`; the `td.keys` cell (or
      its `div.keys` fallback) lays chips, the `Press a key...` prompt and
      the `+` button out with `display: flex; flex-wrap: wrap;
      align-items: center; gap: 0.2rem 0.3rem`, and its vertical padding
      grows by the 0.1rem the chip's margin used to add so a one-line row
      keeps its height.
    - **Strip bar.** `#strip-bar` has `justify-content: space-between`;
      `#position` loses `margin-left: auto` (its `flex: none`, padding,
      font and color stay); `#tools` is unchanged.
    - **Folder tree.** `.folder` is `display: grid;
      grid-template-columns: 14px minmax(0, 1fr) max-content;
      align-items: center` with the existing `2px` gap as `column-gap`;
      `.folder .name` is `justify-self: start` (still `min-width: 0`,
      ellipsis); `.folder input.name` stretches its column; `.folder .count`
      loses `margin-left: auto`. The depth padding, line height and
      `white-space` are untouched; `tree.test.ts` / `treekeys.test.ts` are
      unchanged.
    - **Meta pane.** In `main.ts` `renderMeta`, each provenance group is a
      `div.group` holding a `div.group-title` (the heading) and its `dl`;
      `#meta` is a flex column with `gap: 0.75rem`, `#meta .group` a flex
      column with `gap: 0.125rem`; the `margin: 0.75rem 0 0.125rem` rule
      is gone and the heading's font size and `--muted-foreground` move to
      `.group-title`. `#meta-status` is confirmed margin-free and left as
      is. `meta.test.ts` is unchanged (it tests `metaGroups`, not the DOM).
    - **Untouched.** `#strip`, `#strip-inner`, `.cell` and its children
      (absolute placement against `--cell-width` / `--cell-gap`,
      `.cell.burst::before`'s negative offsets), `#canvas`, `#empty`'s
      `inset`, the menus' `position` / `z-index`, `.menu-separator`.
    - **Greps.** `grep -nE 'margin|float' crates/app/ui/style.css`
      matches only the reset block and `.menu-separator`;
      `grep -nE '\.style\.(margin|float)|margin|float' crates/app/ui/src/*.ts crates/app/ui/index.html`
      matches nothing.
    - **Guide.** `docs/agents/ui-styling.md` has a new "Layout" section
      (between "Selection" and "CSS features"): layout is flex or grid,
      siblings are spaced with `gap` on the parent, spacing inside a
      component is `padding`, `margin` and `float` are not used for
      layout, UA margins are zeroed by the reset block (list its
      selectors), and a right-aligned item is pushed by
      `justify-content: space-between`, a `flex: 1` spacer or a grid
      column, never `margin-left: auto`; the exception list names
      `.menu-separator`'s `margin: 4px -4px` and why. "CSS features" notes
      flex `gap` as in use, its floor (Safari 14.1 / macOS 11, WebKitGTK
      2.32, Inferred) and that `safari13` is a syntax target that does
      not down-level or warn on layout features; grid `gap` is older and
      safe. "Before opening a PR" gains the two greps above and their
      expected matches.
    - **`audit.md`** in this folder holds the audit table plus the "how
      each was replaced" outcome if it differed.
    - `mise run ci` passes.
    - **Manual GUI check on Windows** (`mise run tauri:dev`), recorded in
      `learnings.md` with the settings-panel spacing before / after (measure
      the old `p`-to-`p` and label-to-`#label-names` distances in devtools
      first):
      - Dialogs: the first-run format dialog (clear `sidecarFormat` or use a
        fresh profile; the error line under the note when a write fails);
        `Move Rejected to Trash…` on a folder with rejects and one without
        (rows, the empty note, the failed list, the total, the button row);
        `Sequence JPEG Timestamps…` on a JPEG folder (source / output lines,
        rebuild note, rows, count, the running note during a run).
      - Settings: Sidecar with XMP and `.dop` so `#label-names` shows and
        hides; Culling; Keyboard Shortcuts with a chip removed and a key
        captured on a row with enough keys to wrap (note whether
        `display: flex` on the `td` aligned or the `div.keys` fallback was
        needed); Cache with the clear note visible during a scan; MCP with
        the endpoint and both example blocks.
      - Strip bar: the `N / M` counter at the right edge, the toggles at
        the left, also with `· K selected`; the filter / sort / context
        menus open and their separators still bleed edge to edge.
      - Folder tree: a long name ellipsizing before its badge, the badge at
        the right edge, the open folder bold, a collapsed folder without a
        badge, an inline rename filling the name column without the row
        jumping, the keyboard cursor ring.
      - Meta pane: the file name, then the EXIF / Maker note / Analysis
        headings with 0.75rem above and 0.125rem below as before; the
        status notes (scan progress, `1:1`, Compare, an error with its
        `×`) unchanged.
      - Empty states: no folder, no files, filtered out.
    - `learnings.md` has a `## Deferred issues (todo candidates)` section
      listing the same pass for macOS WKWebView and Linux WebKitGTK (no
      device here), including confirming the flex-gap floor, and the
      Windows pass itself if it could not be run.
  - Implementation approach (as far as it is known; omit if unknown):
    - The reset and the dialog gaps must land together: the settings
      panels' `<p>` / `<h2>` / `<ul>` spacing is UA margins today.
    - A rule that sets `display` on something that can be `hidden` is
      scoped `:not([hidden])` (`docs/agents/ui-styling.md`): the tab
      panels and `#label-names` are hidden through `.dialog-box [hidden]`,
      which an id-prefixed `display: flex` would otherwise beat.
      `#format-error`, `#sequence-rebuild`, `#sequence-running`,
      `#trash-empty`, `#clear-index-note` are `hidden` and
      `#trash-rows:empty` / `#sequence-failed:empty` / `#trash-failed:empty`
      are `display: none`, so they contribute no gap; that is why one `gap`
      reproduces the per-element margins exactly in the dialogs.
    - `.folder` carries `.sidebar-item` (`display: flex` in the component
      block); `.folder`'s rule is later at equal specificity, so
      `display: grid` wins. The `.name` span owns the row's `click` /
      `dblclick` handlers (`folders.ts`), which is why it must not grow.
    - `td.keys` as a flex container: the engine wraps it in an anonymous
      table cell, so the cell padding may sit outside the flex box; watch
      for a shift against the label and Reset columns.
    - `renderMeta` uses `line(className, text)` for the heading; keep it
      for `.group-title` and add the wrapper around it and the `dl`.
      Nothing else in `main.ts` reads `.group` on the meta pane (the other
      `group` uses are the filter menu's `dataset.group`).
    - Guide voice: Hit / Measured / Inferred tags; the engine floor is
      Inferred, the Windows pass Measured.
    - Files: `crates/app/ui/style.css`, `crates/app/ui/src/main.ts`,
      `crates/app/ui/src/settings.ts`, `docs/agents/ui-styling.md`,
      `docs/plans/20260930-no-margin-layout/audit.md`, `learnings.md`.

## Trade-offs and risks

- **`display: flex` on `td.keys`** works in Chromium and WebKit but moves
  the cell padding onto an anonymous wrapper; the `div.keys` fallback is
  ready if it misaligns.
- **Settings rhythm changes.** `gap: 0.5rem` tightens paragraph spacing
  from UA `1em` (16px) to 8px and spreads the radio / checkbox labels from
  their ~0.2rem padding to 0.5rem. Accepted by the user; the Windows check
  is where a wrong-looking panel gets caught, and the before / after values
  are recorded.
- **Meta pane DOM change.** The `div.group` wrapper changes the pane's
  structure; no test or other code reads it, and the MCP companion
  (`companion.ts`) answers over view state, not the DOM. Verify with a grep
  for `group` before merging.
- **Engine floor.** No new risk: flex `gap` is already relied on by every
  button and menu item. The guide now states the real floor (Safari 14.1 /
  WebKitGTK 2.32) instead of implying Safari 13 works.
- **Out of scope, noticed:** `#label-names > label` uses
  `grid-template-columns: subgrid` (Chromium 117+, Safari 16+), beyond the
  `chrome105` target; fine on evergreen WebView2, not mentioned in the
  guide. Not touched here.

## Progress

- (not started)
