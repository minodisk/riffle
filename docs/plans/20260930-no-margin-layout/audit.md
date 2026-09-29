# Audit: `margin` and `float` in the frontend (2026-09-30)

Scope: `crates/app/ui/style.css`, `crates/app/ui/index.html`,
`crates/app/ui/src/*.ts`. Result: 24 `margin*` declarations in `style.css`;
no `float` anywhere; no `.style.margin` / `.style.float` in TypeScript; no
`style=` attribute in `index.html`. Line numbers are those of `style.css`
at commit `9571a0e`.

| # | Line | Selector | Current | Class | Replacement |
| --- | --- | --- | --- | --- | --- |
| 1 | 204 | `.dialog-title` | `margin: 0` | UA reset (h2 / p) | drop; covered by the global reset |
| 2 | 271 | `.menu-separator` | `margin: 4px -4px` | negative-margin trick | **documented exception**: the `hr` bleeds across `.menu`'s 4px padding |
| 3 | 395 | `html, body` | `margin: 0` | UA reset | `body` moves into the reset; drop the line here |
| 4 | 609 | `#position` | `margin-left: auto` | push | `#strip-bar { justify-content: space-between }` (children: `#tools`, `#position`) |
| 5 | 666 | `.folder input.name` | `margin: 0` | UA reset (input) | drop |
| 6 | 680 | `.folder .count` | `margin-left: auto` | push | `.folder { display: grid; grid-template-columns: 14px minmax(0, 1fr) max-content; align-items: center }`, `.folder .name { justify-self: start }` |
| 7 | 778 | `.cell input.name` | `margin: 0` | UA reset | drop (absolute placement untouched) |
| 8 | 961 | `#format-box p` | `margin: 0 0 0.75rem` | sibling spacing | `#format-box { display: flex; flex-direction: column; gap: 0.75rem }` |
| 9 | 967 | `#format-choices` | `margin-bottom: 0.75rem` | sibling spacing | same gap |
| 10 | 976 | `#format-box .note` | `margin: 0` | UA reset | drop |
| 11 | 981 | `#format-error` | `margin: 0.75rem 0 0` | sibling spacing | same gap (`hidden` → `display: none` → no gap while hidden) |
| 12 | 1000 | `#settings-header` | `margin-bottom: 0.5rem` | sibling spacing | `#settings-box { gap: 0.5rem }` |
| 13 | 1042 | `#settings-dialog .chip` | `margin: 0.1rem 0.3rem 0.1rem 0` | sibling spacing | `td.keys` (or `div.keys`) `display: flex; flex-wrap: wrap; align-items: center; gap: 0.2rem 0.3rem`; the 0.1rem vertical goes into the cell padding |
| 14 | 1056 | `#settings-status` | `margin: 0.5rem 0 0` | sibling spacing | `#settings-content { gap: 0.5rem }` |
| 15 | 1061 | `#clear-index-note` | `margin: 0.5rem 0` | sibling spacing | the `#cache` panel's column gap |
| 16 | 1095 | `#settings-dialog .copyable pre` | `margin: 0 0 0.5rem` | sibling spacing | the `#mcp` panel's column gap; `#mcp-examples` and each `.example` block flex columns with `gap: 0.5rem` |
| 17 | 1119 | `#sequence-title, #trash-title` | `margin-bottom: 0.5rem` | sibling spacing | `#sequence-box, #trash-box { gap: 0.5rem }` |
| 18 | 1124 | `#sequence-box p, #trash-box p` | `margin: 0 0 0.5rem` | sibling spacing | same gap (`:empty` / `[hidden]` siblings are `display: none`) |
| 19 | 1138 | `#sequence-rows, #trash-rows` | `margin: 0 0 0.5rem` | sibling spacing | same gap |
| 20 | 1167 | `#sequence-failed, #trash-failed` | `margin: 0 0 0.5rem` | sibling spacing | same gap |
| 21 | 1233 | `#meta .group` | `margin: 0.75rem 0 0.125rem` | sibling spacing | `#meta { display: flex; flex-direction: column; gap: 0.75rem }`; heading + `dl` wrapped in `div.group { display: flex; flex-direction: column; gap: 0.125rem }` (`main.ts` `renderMeta`) |
| 22 | 1242 | `#meta dl` | `margin: 0` | UA reset | drop |
| 23 | 1250 | `#meta dd` | `margin: 0` | UA reset | drop |

## UA-default spacing with no rule today

Removed by the reset and replaced explicitly:

- `<p>` in the settings panels (`#index-size`, the Cache prose,
  `#mcp-status`, the MCP prose, `#label-names > p`, the example titles):
  `#settings-dialog [role="tabpanel"]:not([hidden]) { display: flex;
  flex-direction: column; gap: 0.5rem }` and `#label-names:not([hidden])
  { row-gap: 0.5rem }`. Decision: `0.5rem` everywhere (was UA `1em`).
- `<h2>` (`.dialog-title`): already `margin: 0`; now from the reset.
- `<ul>` / `<ol>` / `<pre>` in the dialogs: already had explicit margins
  (rows 16, 19, 20).

## Confirmed margin-free, no change

`#meta-status` and its `.note` / `.error` children, `#strip-bar`, `#tools`,
`#filter` / `#sort`, `#empty`, `.field`, `.sidenav`, the menus and their
items, `.cell` and its children, `#canvas`, `#viewer`.

## Compatibility

Flex `gap` is already used by `.button`, `.menu-item`, `.folder`,
`.sidenav`, `.sidenav-item`, `.dialog-actions`, `#settings-body`,
`#format-choices`, `.chip`, `.copyable`, `#meta-status .error` and
`#filter-toggle`; grid `gap` by `#meta dl` and `#label-names`. Lightning CSS
at `safari13` is a syntax target and neither down-levels nor warns on a
layout feature. Floor (Inferred): Safari 14.1 (macOS 11, or 10.15 with the
Safari 14.1 update), WebKitGTK 2.32; grid gap is older; WebView2 is
evergreen. No grid fallback for flex gap is needed.

## Outcome (how each was replaced, where it differed)

Everything above landed as planned, with these differences:

- Row 13: the `div.keys` wrapper inside the `td` (`settings.ts`
  `renderShortcuts`), not `display: flex` on the `td` itself. A flex `td`
  stops being a table cell and gets wrapped in an anonymous one, which
  loses the row's vertical centering and can shift the column against the
  label and Reset cells; the wrapper keeps the `td` a real cell. The cell
  keeps its `0.2rem 0.5rem` padding and the wrapper adds `0.1rem` above and
  below, the chips' old vertical margin. The `td` no longer carries the
  `keys` class; `#settings-dialog .keys .add` still matches the `+` button.
- `#label-names:not([hidden])` has `gap: 0.5rem` (its existing
  `column-gap: 0.5rem` plus the new `row-gap: 0.5rem`) in one declaration.
- `.folder input.name` is `justify-self: stretch; width: 100%` (the
  `width` so the input fills the column rather than keeping its intrinsic
  `size` width).
- Row 21: the heading is `div.group-title` inside `div.group`; the old
  `#meta .group` rule's font size and color moved to `#meta .group-title`.
