# Learnings: no-margin-layout

## Step 1

- `td.keys`: went straight to the `div.keys` wrapper (`settings.ts`
  `renderShortcuts`) instead of trying `display: flex` on the `td`. The
  Windows GUI check could not be run from the implementation agent, so the
  choice was made by reasoning (Inferred): a `td` with `display: flex` is no
  longer a table cell; the engine wraps it in an anonymous cell, so the
  tbody's `vertical-align: middle` no longer centers the chips against the
  label and Reset cells, and the padding sits on the flex box rather than
  the cell. The wrapper keeps the `td` a real cell with its
  `0.2rem 0.5rem` padding and adds `0.1rem` above and below itself, the
  vertical margin the chips used to carry.
- The destructured `keys` of each binding shadows a `keys` wrapper variable
  in `renderShortcuts`; the wrapper is `keyList`.
- The guide's `margin|float` grep also matches comments, so the comment on
  the `.keys` rule was reworded to avoid the word; only the reset block's
  own comment and `margin: 0`, and `.menu-separator`, match.
- `#format-box p` used to out-rank `.dialog-title` (id + element over a
  class), so the title `p` also had the `0.75rem` bottom margin; the column
  gap reproduces it.
- Settings spacing, before / after (Inferred from the stylesheet, at the
  dialog's 16px body font; the devtools measurement is part of the pending
  Windows check below):
  - `p` to `p` in the Cache and MCP panels: UA `1em` = 16px collapsed ->
    `gap` 8px.
  - Last format radio label to `#label-names`' first paragraph: the `p`'s
    16px top margin (not collapsed through the grid container) -> 8px.
  - Radio / checkbox labels to each other: 0 (only `.field`'s 0.1rem
    padding) -> 8px apart.
  - The shortcuts table to `Reset all`, and the Cache prose to
    `Clear Cache`: 0 and 16px -> 8px.
  - MCP example title to its code block: 16px -> 8px; code block to the
    next block: the `pre`'s 0.5rem bottom margin plus the next title's
    16px -> 8px.
- `.folder` as a three-column grid keeps the `2px` column gap before the
  empty count column of a row with no badge, so such a row's name
  ellipsizes 2px earlier than before. Not visible in practice; left as is.

## Deferred issues (todo candidates)

- **Pending manual check (Windows, WebView2)**: `mise run tauri:dev` and go
  through the pass listed in Step 1's Done-when of
  `docs/plans/20260930-no-margin-layout/plan.md`. Measure in devtools the
  settings spacing listed above (old values from `main` first) and record
  it. Expected: the dialogs (format with its error line, Move Rejected to
  Trash with and without rejects, Sequence JPEG Timestamps including the
  running note) spaced as before; the settings panels at an even 8px rhythm
  with `#label-names` showing and hiding; the Keyboard Shortcuts rows with
  chips, the `Press a key...` prompt and `+` vertically centered against
  the label and Reset cells, wrapping with 0.2rem / 0.3rem gaps, a one-line
  row keeping its height (this validates the `div.keys` choice); the strip
  bar counter at the right edge (also with `· K selected`) and the menus'
  separators bleeding edge to edge; the folder tree's long name
  ellipsizing before its badge, the badge at the right edge, bold open
  folder, an inline rename filling the name column without the row
  jumping, the keyboard cursor ring; the meta pane's EXIF / Maker note /
  Analysis headings 0.75rem below the previous block and 0.125rem above
  their lists; the status notes and the empty states unchanged. Step 1's
  checkbox was ticked on the automated criteria (greps, `mise run ci`).
  Basis: Step 1 of this plan; files `crates/app/ui/style.css`,
  `crates/app/ui/src/main.ts`, `crates/app/ui/src/settings.ts`.
- **Pending manual check (macOS WKWebView)**: the same pass as the Windows
  one above on macOS, plus confirming flex `gap` renders (the guide's floor:
  Safari 14.1 / macOS 11). Step 1's checkbox was ticked on the automated
  criteria. Basis: Step 1 of this plan; `docs/agents/ui-styling.md`
  "CSS features".
- **Pending manual check (Linux WebKitGTK)**: the same pass on Linux, plus
  confirming flex `gap` renders (the guide's floor: WebKitGTK 2.32). Step
  1's checkbox was ticked on the automated criteria. Basis: Step 1 of this
  plan; `docs/agents/ui-styling.md` "CSS features".
- `#label-names > label` uses `grid-template-columns: subgrid` (Chromium
  117+, Safari 16+), beyond the `chrome105` / `safari13` targets and not
  mentioned in `docs/agents/ui-styling.md`. Basis: the plan's "Out of
  scope, noticed"; file `crates/app/ui/style.css`.
