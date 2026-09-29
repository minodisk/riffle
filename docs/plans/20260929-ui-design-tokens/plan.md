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

# Unify the frontend's look with shadcn/ui's Neutral dark theme tokens

## Purpose

`crates/app/ui/style.css` (1129 lines, hand-written, no framework) styles
every control per id: the settings modal, the first-run format dialog, the
trash confirmation, the sequence dialog, the filter / sort / context menus
and the inline rename editors each carry their own hardcoded grays
(`#222`, `#262626`, `#2a2a2a`, `#333`, `#3a3a3a`, `#444`, `#999`, `#aaa`,
`#888`, `#ccc`), radii (`3px`, `6px`, `0.3rem`, `8px`), the one accent blue
`#4a9eff` and three selection blues (`#2d4059`, `#2a2f36`, `#2f4a6a`).
Buttons in one dialog look different from buttons in the next
(`#format-choices button` has a border and hover; `#sequence-run` /
`#trash-run` / `Reset` / `Copy` / `Clear Cache` are unstyled native
buttons), and nothing tells a new feature which values to use.

This work adopts shadcn/ui's **Neutral dark** theme as the app's palette:
its token vocabulary (`--background`, `--foreground`, `--card`,
`--popover`, `--primary`, `--secondary`, `--muted`, `--accent`,
`--destructive`, `--border`, `--input`, `--ring`, `--sidebar`, `--radius`)
becomes a set of CSS custom properties on `:root`, a few shared component
classes are built on them in shadcn's variants, every view of the app (the
dialogs, the menus, the filmstrip, the folder tree, the viewer, the meta
pane, the empty states and the settings modal) is migrated to those classes,
and the rule is written down in a guide so later UI work reuses them. The
look changes on purpose (a darker, cooler neutral; check-mark menu selection
instead of a blue fill; a side-navigation settings modal). Nothing is added
to `package.json`; no React, no Tailwind, no runtime CSS. Behavior does not
change, and layout does not change except for the approved settings
side-nav: only the appearance.

## Decisions

Taken with the user on 2026-09-29:

- **Palette: shadcn/ui Neutral dark, as-is**, converted from oklch to hex /
  `rgb(... / ...)` because the macOS / Linux build target is `safari13`
  (`vite.config.ts`) and Lightning CSS cannot down-level `oklch()` with
  full fidelity there. Source: the `.dark` block of shadcn/ui's Theming
  page (<https://ui.shadcn.com/docs/theming>, fetched 2026-09-29; the
  Neutral base color, shadcn/ui v4-era Tailwind v4 tokens). Verbatim
  values and their hex conversions:

  | Token | shadcn (oklch) | Riffle (hex / rgb) |
  | --- | --- | --- |
  | `--background` | `oklch(0.145 0 0)` | `#0a0a0a` |
  | `--foreground` | `oklch(0.985 0 0)` | `#fafafa` |
  | `--card`, `--popover`, `--sidebar` | `oklch(0.205 0 0)` | `#171717` |
  | `--card-foreground`, `--popover-foreground`, `--sidebar-foreground` | `oklch(0.985 0 0)` | `#fafafa` |
  | `--primary` | `oklch(0.922 0 0)` | `#e5e5e5` |
  | `--primary-foreground` | `oklch(0.205 0 0)` | `#171717` |
  | `--secondary`, `--muted`, `--accent`, `--sidebar-accent` | `oklch(0.269 0 0)` | `#262626` |
  | `--secondary-foreground`, `--accent-foreground` | `oklch(0.985 0 0)` | `#fafafa` |
  | `--muted-foreground` | `oklch(0.708 0 0)` | `#a1a1a1` |
  | `--destructive` | `oklch(0.704 0.191 22.216)` | `#ff6467` (Tailwind v4 `red-400`) |
  | `--border`, `--sidebar-border` | `oklch(1 0 0 / 10%)` | `rgb(255 255 255 / 10%)` |
  | `--input` | `oklch(1 0 0 / 15%)` | `rgb(255 255 255 / 15%)` |
  | `--ring`, `--sidebar-ring` | `oklch(0.556 0 0)` | `#737373` |
  | `--radius` | `0.625rem` | `0.625rem` |
  | `--radius-sm` / `-md` / `-lg` | `calc(var(--radius) * 0.6)` / `* 0.8` / `var(--radius)` | same `calc()` expressions |

  The `--chart-*`, `--sidebar-primary*` and `--radius-xl` and larger
  tokens are not carried over (nothing uses them). Confirm each hex against
  the oklch value once at Step 1 (any oklch-to-sRGB converter; the grays
  are exact, `--destructive` rounds) and record the check in
  `learnings.md`. The panes (`#side`, `#film`, `#info`) use `--sidebar`.
  The app-semantic tokens `--label-*`, `--stars-color`, `--pick-color`
  and `--reject-color` stay unchanged.
- **Selection is shadcn-style; the old blues go away, no `--selection`
  token.** The open folder (`.folder.current`) and a multi-selected folder
  (`.folder.selected`) both use `--sidebar-accent` / `--sidebar-foreground`;
  they stay distinguishable by font weight (`.current` is `font-weight:
  600`, `.selected` is normal weight), the way shadcn's sidebar marks the
  active item with `font-medium` on the same accent surface. A checked
  menu item (`aria-checked="true"` on a `menuitemcheckbox` /
  `menuitemradio`) shows a check-mark indicator in a leading column instead
  of a background fill, as shadcn's `DropdownMenuCheckboxItem` /
  `RadioItem` do; the mark is Lucide's `check` icon inlined the way
  `SCAN_FACE_SVG` in `crates/app/ui/src/strip.ts` inlines `scan-face`
  (same `/*! ... */` license header, covered by
  `crates/app/ui/LICENSE-lucide`).
- **Filmstrip cells follow the same scheme** (added 2026-09-29):
  `.cell.current` gets `border-color: var(--primary)` and `background:
  var(--accent)`; `.cell.selected` gets `border-color: var(--ring)` and
  `background: var(--muted)`; `.cell`'s radius becomes `var(--radius-sm)`.
  `.cell.failed`'s red, the burst band tint and the `text-shadow` halos
  stay as app-semantic exceptions.
- **Buttons follow shadcn's variants**: `primary` (shadcn's default look,
  `--primary` on `--primary-foreground`) only on the sequence dialog's
  `Run`; `destructive` on `Move to Trash`; `outline` on `Cancel`, `Reset`,
  `Reset all`, `Clear Cache`, `Copy`, `#label-names-reset`, the three
  format choices and the strip bar's filter / sort toggles; `ghost` on the
  icon-only buttons (`#settings-close`, the chip `×`, the `+` key adder,
  the meta status dismiss `×`). `secondary` is added only if a control
  turns out to need it. The focus ring is `--ring`.
- **Menus get shared `.menu` / `.menu-item` classes** used by
  `#filter-menu`, `#sort-menu` and `#context-menu`, set in `index.html`
  and in `main.ts`'s two item builders (`showMenu` and the `#filter-exif`
  builder). Positioning, `z-index`, `overflow` and `max-height` stay in
  the id rules, untouched.
- **`#format-box` moves onto the card surface** like the other three
  dialog boxes.
- **Every view gets the treatment, not only the dialogs** (added
  2026-09-29): the folder tree pane, the strip bar and cells, the viewer's
  canvas overlays (compare labels, focus mark halo), the meta pane, the
  empty states, the status / progress lines and the scrollbars each map to
  a shadcn counterpart implemented as a plain CSS class on the tokens.
  The full part-by-part mapping is in [`audit.md`](./audit.md). No
  behavior or layout change except the settings side-nav below.
- **The settings modal grows and gets a side navigation** (added
  2026-09-29): a left column of category items and a right content pane,
  as in shadcn's sidebar settings dialog and macOS System Settings. It
  keeps `tablist` semantics with `aria-orientation="vertical"` (see Step 6
  for the reasoning) and adds `.sidenav` / `.sidenav-item` to the
  component block.

Ground truth gathered at planning time (2026-09-29):

- The frontend is vanilla TypeScript; `index.html` holds the static dialogs
  (`#format-dialog`, `#sequence-dialog`, `#trash-dialog`, `#settings-dialog`)
  and menus (`#filter-menu`, `#sort-menu`, `#context-menu`), and the
  dynamically built controls live in `src/settings.ts` (shortcut chips,
  `+` / `×` / `Reset` buttons, `Copy`, `.copyable` blocks), `src/main.ts`
  (`showMenu`'s `#context-menu` items with a `.shortcut` span, the
  `#filter-exif` items, the `#meta-status .error` dismiss button, the
  sequence preview rows, the `#meta` `dl` / `dt` / `dd` rows, the compare
  labels and the focus mark drawn on `#canvas`), `src/folders.ts` (the tree
  rows, expander, count and the inline rename `input.name`) and
  `src/strip.ts` (the cell, its badges and the inline rename `input.name`).
  `src/modal.ts`, `src/firstrun.ts`, `src/trash.ts`, `src/sequence.ts`,
  `src/rename.ts`, `src/empty.ts`, `src/tabs.ts`, `src/panels.ts`,
  `src/compare.ts`, `src/zoom.ts`, `src/focus.ts` and `src/meta.ts` are
  DOM-free.
- There are no panel splitters (the side panes are fixed at 220px), no
  tooltips beyond native `title` attributes, no progress bar (scan / faces
  progress is a `note` text line in `#meta-status`) and no toasts (errors
  are `#meta-status .error` rows with a dismiss `×`).
- No test asserts on a dialog / button / input / menu class name (the only
  class-name assertions are on `.cell` / `.folder` state classes, which
  keep their names). `modal.test.ts` tests `SettingsModal.key` /
  `cycleFocus` only; `tabs.test.ts` tests `nextTab`.
- `settings.ts`'s `focusables()` selects `button, input, select,
  [tabindex]` inside `#settings-box`, so adding classes does not change
  the Tab trap; the format / trash / sequence dialogs select by id and
  `button[data-format]`.
- The build target is `safari13` on macOS / Linux and `chrome105` on
  Windows; Vite+ runs the CSS through Lightning CSS against that target,
  so `color-mix()`, `oklch()`, relative colors and nesting are out, and
  plain hex / `rgb(... / ...)` (already in the file) are in.
- The dialogs already share one shape (fixed `inset: 0` backdrop at
  `rgb(0 0 0 / 60%)`, `z-index: 30`, a per-id `[hidden] { display: none }`,
  a box, a `600`-weight `1rem` title, a right-aligned `.actions` row) and
  the menus another (`#262626` surface, `1px solid #444`, `6px`, one
  shadow, `#3a3a3a` hover, `#2f4a6a` checked fill).
- `docs/agents/` holds three pitfall guides, each opening with "Read this
  before touching ..." and the Hit / Measured / Inferred legend linking to
  `tauri-app.md`. The `develop` skill's wrap-up defers *new* guides to the
  user; this plan's guide is a requirement of the work, so Step 7 creates
  the file directly.

## Steps

- [x] Step 1: Add the tokens and the shared component classes, and migrate the settings modal
  - Done when:
    - `:root` in `crates/app/ui/style.css` defines every token in the
      Decisions table with the listed value and a one-line comment naming
      what it paints, and keeps `--stars-color`, `--reject-color`,
      `--pick-color` and the `--label-*` set unchanged.
    - `html, body` paint `var(--background)` / `var(--foreground)`; `#side`,
      `#film` and `#info` paint `var(--sidebar)`.
    - The shared classes exist, are defined once in a `/* Components */`
      block right after `:root`, and are the only rules that set a
      control's colors, border and radius: `.button` with `.primary`,
      `.destructive`, `.outline`, `.ghost`; `.input`; `.select`;
      `.checkbox` (native checkbox and radio with `accent-color`); `.field`
      (a label wrapping its control, as `#label-names > label` and the
      checkbox labels do); `.dialog`, `.dialog-box`, `.dialog-title`,
      `.dialog-actions`. `.menu` / `.menu-item` are Step 3's.
    - The settings modal uses them: `#settings-dialog` / `#settings-box` /
      `#settings-title` / every `.actions` row in `index.html` carry the
      dialog classes; the buttons carry the variants from Decisions
      (`#settings-close`, chip `×`, `+` are `ghost`; `Reset`, `Reset all`,
      `Clear Cache`, `Copy`, `#label-names-reset` are `outline`); the
      `#label-name-*` inputs, `#label-names-language` and the checkboxes /
      radios carry `.input` / `.select` / `.checkbox`. The tabs, `.chip`,
      `.copyable pre`, `#settings-status`, `#clear-index-note` and
      `#mcp-status` keep their id-scoped rules but read their colors and
      radii from the tokens (tabs: selected tab on `--accent`, hover on
      `--muted`; `.chip` and `pre` on `--muted` with `--radius-sm`;
      `#settings-status` in `--destructive`; notes in
      `--muted-foreground`).
    - Every rule the migration made redundant under `#settings-dialog` /
      `#settings-box` is removed; `grep -E '#[0-9a-f]{3,6}\b'` finds no hex
      in the settings rules.
    - A comment block at the top of the component section lists the CSS
      features the components rely on and why each is safe in WebView2,
      WKWebView and WebKitGTK (see the implementation approach).
    - `pnpm exec vp build` was run once and the emitted CSS's `:root` block
      diffed against the source (custom properties and `rgb(... / ...)`
      untouched); the result is in `learnings.md`.
    - `mise run ci` passes.
    - Manual GUI check on Windows (`mise run tauri:dev`): open Settings,
      walk every tab (Sidecar with both XMP and `.dop` so `#label-names`
      shows and hides, Culling, Keyboard Shortcuts with a chip removed and
      a key captured, Cache, MCP with `Copy`), confirm the modal reads as
      shadcn Neutral dark (near-black backdrop over the `#0a0a0a` window,
      `#171717` box, hairline `rgb(255 255 255 / 10%)` borders, `#fafafa`
      text, outline buttons, `#737373` focus ring), that every control is
      consistent with the others, the Tab trap still cycles through the
      same controls, and the modal's size and scrolling (a long shortcuts
      table) are unchanged. Record the result in `learnings.md`; note macOS
      and Linux as unverified (no device here).
  - Implementation approach (as far as it is known; omit if unknown):
    - Values stay plain hex / `rgb(... / ...)`; no `oklch()`, `color-mix()`
      or hsl channel triplets. No nesting (the file has none today).
    - `.dialog` is `position: fixed; inset: 0; z-index: 30; display:
      flex; align-items: center; justify-content: center; background:
      rgb(0 0 0 / 60%)` scoped as `.dialog:not([hidden])`, per the guide
      entry "Scope an id's `display` override to `:not([hidden])`" in
      `docs/agents/tauri-app.md`, so the per-id `[hidden] { display:
      none }` rules can go. `.dialog-box` is `background: var(--card);
      color: var(--card-foreground); border: 1px solid var(--border);
      border-radius: var(--radius-lg); box-shadow: 0 6px 20px rgb(0 0 0 /
      50%)` plus the existing padding; the per-dialog width / height stay
      on the ids. Keep the per-box `[hidden]` rules that hide *children*
      (`#settings-dialog [hidden]`) until Step 2 folds them into one
      `.dialog-box [hidden]` rule.
    - Button base (shadcn `size="sm"`-ish): `display: inline-flex;
      align-items: center; justify-content: center; gap: 0.4rem; padding:
      0.3rem 0.75rem; font: inherit; font-size: 0.85rem; font-weight: 500;
      border-radius: var(--radius-md); border: 1px solid transparent;
      cursor: pointer`, `:disabled { opacity: 0.5; cursor: default }`
      (shadcn's `disabled:opacity-50`), `:focus-visible { outline: 2px
      solid var(--ring); outline-offset: 2px }`. `.primary`: `--primary`
      on `--primary-foreground`, hover at 90% (use a second solid hex,
      e.g. `#d4d4d4`, since `color-mix()` is out; comment it as
      "primary at 90%"). `.destructive`: `--destructive` on `#fafafa`,
      hover at 90% the same way. `.outline`: `border-color: var(--input);
      background: var(--input)`-ish per shadcn's dark `bg-input/30
      border-input`, hover `--accent` / `--accent-foreground`. `.ghost`: no
      border, no background, hover `--accent` / `--accent-foreground`.
    - Input / select base: `font: inherit; font-size: 0.85rem; color:
      var(--foreground); background: transparent` (shadcn dark
      `bg-input/30`, so `rgb(255 255 255 / 5%)` is an acceptable
      approximation); `border: 1px solid var(--input); border-radius:
      var(--radius-md); padding: 0.2rem 0.5rem`; `:focus-visible { border-
      color: var(--ring); outline: 2px solid var(--ring); outline-offset:
      0 }` (shadcn's `ring-[3px]` collapsed to an outline). `<select>`
      keeps `appearance: auto` (the native arrow); never
      `appearance: base-select` (Chromium-only).
    - Checkbox / radio: keep the native control; `accent-color:
      var(--primary)`. `accent-color` is in Chromium 93+, Safari 15.4+ and
      WebKitGTK 2.36+; an older engine ignores it and shows the UA control.
    - Features used and why they are safe (write into the comment block):
      CSS custom properties (universal); `:focus-visible` (Chromium 86+,
      Safari 15.4+, WebKitGTK 2.36+; an older WebKit falls back to its UA
      focus ring because the rule does not match); `accent-color` (above);
      `rgb(... / ...)` space syntax and `inset` (already in the file);
      `:not([hidden])` (universal); `calc()` in a custom property
      (universal). Nothing from CSS anchor positioning, `@layer`, `:has()`,
      nesting, `color-mix()`, `oklch()` or `base-select`.
    - Do not touch the strip cell geometry (`#strip-inner`, `.cell*`), the
      folder tree rows, the menus or the meta pane in this step; they are
      Step 3's. The pane backgrounds (`--sidebar`) and the body are the
      only rules outside the modal this step changes.
    - `settings.ts` changes are `className` assignments on the elements it
      creates (`remove.className = "button ghost"`, `add.className =
      "button ghost add"`, `reset.className = "button outline"`, the
      `Copy` button); keep the existing `add` class because
      `#settings-dialog .keys .add` is selected by it, or drop that rule
      if `.button.ghost` now covers it. No behavior change, no new test.

- [x] Step 2: Migrate the first-run format dialog, the trash confirmation and the sequence dialog
  - Done when:
    - `#format-dialog`, `#sequence-dialog`, `#trash-dialog` and their boxes,
      titles and `.actions` rows in `index.html` carry the dialog classes,
      `#format-box` sits on the card surface like the other three, and the
      per-id backdrop / box / title / actions / hidden rules in `style.css`
      are deleted; only the per-dialog width / height / list rules remain
      and read from the tokens (`#sequence-rows` / `#trash-rows` on
      `--muted` with `--radius-sm`; `#sequence-rows .unchanged`,
      `#sequence-rebuild`, `#sequence-running`, `#trash-empty`,
      `#format-box .note` in `--muted-foreground`; `#format-error` and the
      failed lists in `--destructive`).
    - `#sequence-run` is `button primary`; `#trash-run` is `button
      destructive`; `#sequence-cancel`, `#trash-cancel` and the three
      `#format-choices` buttons are `button outline`; `#format-choices
      button`'s own colors / hover / focus rules are removed (its `flex: 1;
      padding: 0.5rem` stays).
    - `#sequence-box [hidden]`, `#trash-box [hidden]` and `#settings-dialog
      [hidden]` are folded into one `.dialog-box [hidden] { display: none
      }` rule kept last in the component block (it must win over the
      `display` of earlier same-specificity rules; see the existing
      comment above `#settings-dialog [hidden]`).
    - `mise run ci` passes.
    - Manual GUI check on Windows: clear the `sidecarFormat` key (or use a
      fresh profile) to see the first-run dialog and pick a format; open
      `Move Rejected to Trash` from the folder tree's right-click on a
      folder with rejects and one without, confirm the rows, the total, the
      red destructive button and the outline Cancel; open `Sequence JPEG
      Timestamps…` on a JPEG folder, confirm the preview rows (changed /
      unchanged colors), the light primary Run and the outline Cancel; the
      disabled state during a run reads as disabled (half opacity). All
      four dialogs look like one family. Record the result and the open
      macOS / Linux checks in `learnings.md`.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Step 1 is merged.
    - The Tab traps and mutual exclusion (`FormatGate`, `TrashFlow`,
      `SequenceFlow` in `main.ts`) select by id and `button[data-format]`,
      so no TypeScript change is expected; verify by reading `main.ts`
      around `formatButtons`, `trashRunButton`, `sequenceRunButton` and
      with `pnpm exec vp test`.

- [x] Step 3: Migrate the menus, the strip bar and the filmstrip cells
  - Done when:
    - `.menu` (surface `--popover` / `--popover-foreground`, `1px solid
      var(--border)`, `--radius-md`, the shadow, `padding: 4px`) and
      `.menu-item` (full-width flex row, `--radius-sm`, hover / focus on
      `--accent` / `--accent-foreground`, a fixed-width leading indicator
      column), plus `.menu-separator` (the `hr`: `border-top: 1px solid
      var(--border)`) and `.menu-label` (the `.heading`:
      `--muted-foreground`, `0.7rem`), exist in the component block;
      `#filter-menu`, `#sort-menu` and `#context-menu` carry `menu` in
      `index.html`; every item carries `menu-item`, both the static ones in
      `index.html` and the ones built in `main.ts` (`showMenu`, the
      `#filter-exif` builder). The id rules keep only positioning,
      `z-index`, `min-width`, `max-height`, `overflow` and the scrollbar
      rules; `.shortcut` reads `--muted-foreground`.
    - A `[aria-checked="true"]` item shows Lucide's `check` in its leading
      column and no background fill; an unchecked item leaves the column
      empty so labels stay aligned. `#2f4a6a` is gone. The filter menu's
      `.dot` (`--muted-foreground` fill; the `none` label's ring uses
      `--border`), `.face` and star spans (`.off` in `--muted-foreground`
      at reduced opacity, or `--muted`; pick and record) keep working next
      to the indicator column. The SVG lives in a new
      `crates/app/ui/src/icons.ts` as `CHECK_SVG`, with `SCAN_FACE_SVG`
      moved there from `strip.ts` (re-exported or imports updated) under
      the same `/*! Lucide ... */` license header; `index.html`'s static
      items get the mark by a small startup pass in `main.ts` that appends
      it to every `.menu-item[role^="menuitem"]`, and the two builders
      append it themselves, so there is one source of truth.
    - `.scrollarea` exists in the component block (`scrollbar-width: thin;
      scrollbar-color: var(--border) transparent`, shadcn ScrollArea's
      look) and replaces the per-id `scrollbar-*` pairs on the menus and
      `#strip` (`#folders` and `#meta` take it in Steps 4 and 5).
    - `#filter-toggle` and `#sort-toggle` are `button outline` (their
      `padding: 0.2rem 0.4rem; gap: 2px` stay on the ids); `.active` uses
      `--primary` for its color and border; `#sort-toggle:disabled` relies
      on the button's `:disabled`. `#position` reads `--muted-foreground`.
    - `.cell` has `border-radius: var(--radius-sm)`; `.cell.current` is
      `border-color: var(--primary); background: var(--accent)`;
      `.cell.selected` is `border-color: var(--ring); background:
      var(--muted)`; `.cell img:not([src])` (the placeholder; shadcn
      Skeleton) is `--muted`; `.cell span.sharpness`'s neutral bar is
      `--muted-foreground`; `.cell span.count` is `--foreground`; `.cell
      input.name` takes `--foreground` / `--background` with an outline of
      `--ring`. `.cell.failed`'s `#402020`, `.cell.burst::before`'s tint,
      the `text-shadow: 0 0 2px #000` / `drop-shadow` halos, and the
      `--stars-color` / `--pick-color` / `--reject-color` / `--label`
      badge colors stay as they are (app semantics).
    - `mise run ci` passes; `strip.test.ts`-adjacent tests still pass (the
      `.cell` state class names are unchanged).
    - Manual GUI check on Windows: open a folder; open the filter and sort
      menus and toggle a few items (hover, the check mark appears and
      disappears, the labels stay aligned, the `AF eye` label, the star
      rows, the `#filter-exif` groups, the separators are visible on
      `#171717`); right-click a cell for the context menu (the `.shortcut`
      column, a radio item's check); click a cell and shift-click another
      (the focused cell has the light `--primary` border on the `--accent`
      fill, the others the `--ring` border on `--muted`; the rejected
      dimming, the burst band, the badges and the failed red are as
      before); watch a fresh folder's placeholders fill in; rename a file
      inline. Record the result and the open macOS / Linux checks in
      `learnings.md`.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Steps 1 and 2 are merged.
    - Do not change any positioning, `z-index`, `overflow`, `max-height`
      or size in the menus: the comment on `#film` explains why the menus
      must not get a stacking context, and `#filter-menu` / `#sort-menu`
      open upward with a computed `max-height`. Do not touch the cell
      geometry (`--cell-width`, `--cell-gap`, the 144px image box, the
      badge offsets).
    - The indicator column: shadcn's item reserves `pl-8` and absolutely
      positions the check at `left-2`; the same shape here is
      `.menu-item { position: relative; padding-left: 1.6rem }` with the
      check `position: absolute; left: 0.5rem; width: 12px; height: 12px`
      and `display: none` unless `[aria-checked="true"]`. Because the
      check is inside the button, it needs no `main.ts` logic beyond the
      markup; the `aria-checked` toggling in `main.ts` already drives it.
      `#filter-menu [data-stars]` sets `gap: 0`; keep the indicator outside
      that gap.
    - `.cell.current` on `--accent` (`#262626`) next to `.cell.selected` on
      `--muted` (the same `#262626`): the fills are identical in Neutral,
      so the border is what distinguishes them (`--primary` `#e5e5e5` vs
      `--ring` `#737373`); confirm in the manual check that it reads.

- [ ] Step 4: Migrate the folder tree pane to the sidebar scheme
  - Done when:
    - `.sidebar-item` exists in the component block (shadcn
      `SidebarMenuButton`: a flex row, `--radius-sm`, hover on
      `--sidebar-accent`, `[data-active]` / `.current` on `--sidebar-accent`
      + `--sidebar-foreground` + `font-weight: 600`, `.selected` the same
      surface at normal weight) and `.folder` carries it (set in
      `folders.ts`'s row builder, keeping `folder` and the state classes
      `current` / `selected` / `cursor` / `failed` the tests assert on);
      `.folder`'s own `padding-left: calc(4px + var(--depth) * 12px)`,
      `line-height` and `white-space` stay on `.folder`.
    - `.badge` exists in the component block (shadcn Badge `secondary`:
      `--secondary` / `--secondary-foreground`, `0.7rem`, pill radius) and
      `.folder .count` carries it (`folders.ts`); `.folder .expander` reads
      `--muted-foreground`; `.folder.failed .name` reads `--destructive`;
      `#folders:focus .folder.cursor`'s outline is `--sidebar-ring`;
      `.folder input.name` takes `--foreground` / `--background` with an
      outline of `--sidebar-ring`; `#folders` is a `.scrollarea`;
      `body.dragging #main`'s outline is `--ring`. `#ccc`, `#222`,
      `#2a2f36`, `#2d4059`, `#d66`, `#4a9eff`, `#888`, `#999`, `#2a2a2a`
      and `#111` are gone from the tree rules.
    - `mise run ci` passes; `tree.test.ts` / `treekeys.test.ts` are
      unchanged.
    - Manual GUI check on Windows: expand a few roots, hover rows, select
      two folders and open one (the open one is bold on the same accent;
      the keyboard cursor ring shows while the tree has focus), see a
      count badge on an expanded folder and a failed folder's red name,
      rename a folder inline, drag a folder over the window for the drop
      outline. Record the result and the open macOS / Linux checks in
      `learnings.md`.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Step 3 is merged (for `.scrollarea`).
    - `.folder.current` vs `.selected` weight: `.folder` rows are `0.8rem`
      with `line-height: 1.4`; check that `600` weight does not change the
      row's width enough to clip a long name differently (the `.name` span
      already ellipsizes).
    - Keep the id-scoped `#folders` rules for size and padding; only the
      colors move to the classes.

- [ ] Step 5: Migrate the viewer overlays, the meta pane, the empty states and the status lines
  - Done when:
    - `.empty` exists in the component block (shadcn Empty: centered,
      `--muted-foreground`, `0.9rem`) and `#empty` carries it in
      `index.html`; `#empty`'s id rule keeps `position: absolute; inset:
      0; padding` and the `data-state` cursor. The `openHint` text stays
      text (no `<kbd>` markup; see Trade-offs).
    - `.kbd` exists only if Step 3's `.shortcut` was turned into one; if
      not, no `.kbd` (nothing else shows a key hint as markup).
    - The meta pane: `#meta` is a `.scrollarea`; `#meta .group` (the
      provenance headings, shadcn's section label) reads
      `--muted-foreground`; `#meta dt` reads `--muted-foreground`;
      `#meta-status .note` (the scan / faces progress `scanning N / M`,
      `1:1`, `Compare · N frames`, view-only and held-operation notes) reads
      `--muted-foreground`; `#meta-status .error` reads `--destructive`
      (replacing the amber `#e0a040`; see Trade-offs) and its dismiss
      button is `button ghost` (set in `main.ts`'s `renderMeta`).
    - The canvas overlays in `main.ts` read their neutral colors from the
      tokens once at startup via `getComputedStyle(document.documentElement)
      .getPropertyValue(...)` into a small `const THEME = { ... }` (a
      `theme.ts` helper with a test that the reader trims and falls back
      when a property is missing is fine): the compare label bar
      (`#252525` → `--card`), its text (`#ddd` → `--foreground`), the frame
      border (`#444` → `--border`), the active frame ring (`#fff` →
      `--primary`); the best-frame green (`#244c31` / `#6bdc8a` =
      `--pick-color`) and `FOCUS_MARK_COLORS` stay app-semantic; the focus
      mark's black halo stays.
    - `grep -E '#[0-9a-f]{3,6}\b' crates/app/ui/style.css` matches only:
      the `:root` block; the two hover-at-90% button hexes in the
      component block; `.cell.failed` (`#402020`); the `#000` halos
      (`text-shadow` / `drop-shadow`). `grep -E '#[0-9a-f]{3,6}\b|rgba?\('
      crates/app/ui/src/*.ts` matches only `focus.ts`'s
      `FOCUS_MARK_COLORS`, the compare best-frame green pair and the
      focus mark halo in `main.ts`, and the `--label-*` / `--pick-color`
      strings if any are read by name. Each match is listed in
      `learnings.md` as a deliberate exception.
    - `mise run ci` passes.
    - Manual GUI check on Windows: no folder open (the empty hint), a
      filtered-out folder (`No files match`), an empty folder; a folder
      mid-scan (the `scanning N / M` then `focus N / M` notes); a sidecar
      write error (rename a sidecar read-only) and its dismiss `×`; the
      meta pane's EXIF / Maker note / Analysis groups; 1:1 zoom and
      Compare with three frames (label bars, the best frame's green, the
      active frame's light ring). Record the result and the open macOS /
      Linux checks in `learnings.md`.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Steps 3 and 4 are merged.
    - `renderMeta()` rebuilds `#meta-status` on every call, so the dismiss
      button's `className` is set where it is created (`main.ts`
      `renderMeta`), not toggled.
    - Read the tokens once after the stylesheet is applied (module top
      level of `main.ts` runs after `<link>` in `<head>` has loaded, since
      the script is `type="module"` at the end of `<body>`); do not read
      them per frame.

- [ ] Step 6: Enlarge the settings modal and replace its tabs with a side navigation
  - Done when:
    - `#settings-box` is `width: min(860px, 90vw); height: min(720px,
      80vh)` and lays out as header row, then a two-column body: a
      `.sidenav` column of fixed width (`180px`) on the left and the
      content pane on the right; the content pane (`[role="tabpanel"]`)
      scrolls independently (`flex: 1; min-height: 0; overflow-y: auto`,
      a `.scrollarea`) while the header and the nav stay put;
      `#settings-status` stays under the content pane.
    - `.sidenav` (a vertical flex column, `gap: 2px`, `padding-right:
      0.75rem`, `border-right: 1px solid var(--border)`) and
      `.sidenav-item` (shadcn `SidebarMenuButton`: a full-width flex row
      with `gap: 0.5rem`, `padding: 0.4rem 0.6rem`, `--radius-sm`, `font:
      inherit; font-size: 0.85rem`, `--muted-foreground` at rest, hover on
      `--accent` / `--accent-foreground`, `[aria-selected="true"]` on
      `--accent` + `--accent-foreground` + `font-weight: 600`, the same
      accent-plus-weight scheme as the folder tree, `:focus-visible` on
      `--ring`) are in the component block, and every `#settings-tabs`
      button carries `sidenav-item`; the horizontal `[role="tablist"]` /
      `[role="tab"]` id rules are deleted.
    - Each item shows a Lucide icon (16px, `currentColor`) before its
      label: `file-text` (Sidecar), `sliders-horizontal` (Culling),
      `keyboard` (Keyboard Shortcuts), `database` (Cache), `plug` (MCP),
      `bug` (Debug), added to `icons.ts` under the same license header and
      inserted by `settings.ts` at init by tab id, so `index.html` stays
      free of SVG paste.
    - ARIA and keyboard: the nav keeps `role="tablist"` with
      `aria-orientation="vertical"` and the items keep `role="tab"` /
      `aria-controls` / `aria-selected` / roving `tabindex`; `ArrowUp` /
      `ArrowDown` move between the visible items (wrapping), `Home` /
      `End` to the first / last, and `ArrowLeft` / `ArrowRight` are no
      longer handled (they fall through to `native`); Tab still leaves the
      nav for the content pane through the existing `focusables()` trap.
      `tabs.ts`'s `nextTab` takes an `orientation` argument (`"horizontal"`
      | `"vertical"`) or a new `nextVerticalTab` is added; `tabs.test.ts`
      covers `ArrowUp` / `ArrowDown` / `Home` / `End` / wrapping / the
      hidden Debug item and that the horizontal keys are ignored in the
      vertical case. No other `settings.ts` behavior changes (automatic
      activation on focus, `#tab-debug` hidden in release, the active tab
      not persisted).
    - `mise run ci` passes.
    - Manual GUI check on Windows: open Settings; the modal is wider and
      taller, the nav lists the five (six in a debug build) categories
      with icons, the active one is bold on the accent surface; `ArrowUp`
      / `ArrowDown` / `Home` / `End` move and activate, `ArrowLeft` /
      `ArrowRight` do nothing, Tab moves into the content, Shift+Tab back,
      Escape closes; the shortcuts table scrolls inside the content pane
      while the nav and header stay; the modal fits a 1280x720 window
      (`80vh`). Record the result and the open macOS / Linux checks in
      `learnings.md`.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Steps 1–5 are merged (for `.scrollarea`, `icons.ts` and the
      grep criterion, which this step must keep true).
    - Why keep `tablist` rather than a `<nav>` list: the existing
      `aria-controls` / `aria-selected` / roving-`tabindex` wiring,
      `selectTab`, the `focusables()` filter (which skips `tabindex="-1"`
      items) and the "focus follows selection" behavior all hang off
      `role="tab"`; a `<nav>` of buttons would need `aria-current`, a new
      focus model and a new panel-association, for no accessibility gain.
      WAI-ARIA APG's vertical tabs pattern is exactly `aria-orientation=
      "vertical"` with Up / Down, so screen readers still announce "tab,
      n of m" and the panel association.
    - Layout: `#settings-box { display: flex; flex-direction: column }`
      stays; add a `#settings-body { display: flex; flex: 1; min-height:
      0; gap: 0.75rem }` wrapper in `index.html` around the nav and the
      sections (the sections are siblings today; wrap them in a
      `#settings-content { flex: 1; min-width: 0; display: flex;
      flex-direction: column }` so `[role="tabpanel"]` keeps its `flex: 1;
      overflow-y: auto`). `selectTab` hides / shows the sections by id as
      today.
    - Icons are 16px `<span class="icon">` with `innerHTML` from
      `icons.ts` (the same pattern as `sharpFace.innerHTML =
      SCAN_FACE_SVG`), `aria-hidden` on the SVG as the Lucide strings
      already carry.

- [ ] Step 7: Write the styling guide and point CLAUDE.md at it
  - Done when:
    - `docs/agents/ui-styling.md` exists, opens like the other guides
      ("Read this before touching `crates/app/ui/style.css`, `index.html`
      or a `.ts` file that creates a control or draws on the canvas"), and
      states the rule: UI code uses the `:root` tokens and the shared
      component classes, never an ad-hoc color, radius or border; carries
      the Decisions table (the tokens, the shadcn oklch source and the hex
      used, with what each paints); lists every component class and its
      variants with the element it goes on and which controls use which
      variant (`button` primary / destructive / outline / ghost with the
      `primary` only on `Run` rule; `input`, `select`, `checkbox`, `field`;
      `dialog`, `dialog-box`, `dialog-title`, `dialog-actions`; `menu`,
      `menu-item`, `menu-separator`, `menu-label`; `scrollarea`;
      `sidebar-item`, `badge`; `empty`; `sidenav`, `sidenav-item`; `icon`),
      and that dynamically created controls in `.ts` set `className` to
      the same classes and canvas drawing reads the tokens through the
      Step 5 helper; the selection conventions (accent + weight for the
      tree and the settings nav, check mark for menus, `--primary` /
      `--ring` borders for the strip cells); the Lucide icon convention
      (`icons.ts`, the license header, `LICENSE-lucide`); the CSS features
      in use and the ones to avoid (anchor positioning, `base-select`,
      `color-mix()`, `oklch()`, nesting, `:has()`, `@layer`) with the
      webview reason (WebView2 / WKWebView / WebKitGTK, `safari13` build
      target); the grep criterion from Step 5 as the check to run before
      opening a PR; and the exceptions (the strip cell's per-cell
      semantics, the `--label-*` / `--stars-color` / `--pick-color` /
      `--reject-color` tokens, `FOCUS_MARK_COLORS`, the two hover hexes).
    - The audit table in [`audit.md`](./audit.md) is not copied into the
      guide; the guide links to its archived path.
    - `docs/agents/tauri-app.md`'s "Frontend" section gets one cross-link
      line to the new guide; its two styling entries ("Scope an id's
      `display` override to `:not([hidden])`", "Style the strip
      placeholder on `.cell img:not([src])`") stay where they are.
    - `CLAUDE.md`'s layout paragraph for `crates/app/ui` gains one clause
      naming `style.css` as the shadcn Neutral tokens and shared component
      classes, `src/icons.ts` as the inlined Lucide icons, and the guide's
      path, in the paragraph's existing style.
    - `mise run lint` (lychee) passes on the new links.
    - The open manual checks (macOS WKWebView and Linux WebKitGTK visual
      pass over every view touched by Steps 1–6) are listed under `##
      Deferred issues (todo candidates)` in `learnings.md` so the wrap-up's
      todo curation files them the way `todo.md`'s other `### App: ...
      manual checks` entries are filed.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Steps 1–6 are merged, so the guide lists the final names.
    - Keep it short (the three existing guides are pitfall lists; this one
      is a rule plus two tables). Tag the compatibility notes `Inferred`
      per the guides' legend, since nothing has broken on them.
    - Do not add a guide index to `CLAUDE.md`; it lists file layout, not
      guides. The one clause is enough to find the guide by name under
      `docs/agents/`.

## Trade-offs and risks

- **Hover shades without `color-mix()`.** shadcn's hover states are
  `bg-primary/90`, `bg-destructive/90`, which need `color-mix()` or an
  alpha on the color. Since `color-mix()` is out at `safari13`, Step 1
  uses a second solid hex per variant (commented as "at 90%"); this is
  the one place a non-token hex is allowed, and the guide lists it as
  such. An alternative is `opacity: 0.9` on hover, which also dims the
  label; note in `learnings.md` if that reads better.
- **Hairline borders on the dark panes.** `--border` is
  `rgb(255 255 255 / 10%)`; over `#171717` it is faint, over `#0a0a0a`
  fainter. If a separator (the settings nav's right border, the menu
  separator) disappears on the Windows check, the step may raise that one
  rule to `--input` (15%) rather than invent a color; record it.
- **`--muted`, `--accent`, `--secondary` and `--sidebar-accent` are all
  `#262626` in Neutral.** Hover and selected surfaces therefore look
  alike (already noted for the tabs and chips in Step 1's learnings), and
  `.cell.current` / `.cell.selected` fills are identical, so the border is
  what distinguishes them. This is shadcn's own choice; the plan
  compensates with weight (tree, nav), borders (cells) and the check mark
  (menus) rather than inventing colors.
- **`--muted-foreground` (`#a1a1a1`) is lighter than today's `#999` /
  `#888`.** Secondary text (meta labels, notes, the position counter)
  gains contrast; intended, but it is the most widespread visual change
  outside the dialogs.
- **The amber error line becomes `--destructive`.** `#meta-status .error`
  is `#e0a040` today (a warning tone for a failed sidecar write). shadcn
  has no warning token; Step 5 maps it to `--destructive`. If the user
  wants a softer tone for non-fatal errors, the alternative is an
  app-semantic `--warning` next to `--reject-color`; decide at Step 5 and
  record it.
- **`openHint`'s key names stay text, no `<kbd>`.** Wrapping the keys in
  `<kbd class="kbd">` would need `empty.ts` to return markup or `main.ts`
  to split the string, a behavior-adjacent change for a hint; the plan
  keeps it text. If Step 3 turns `.shortcut` into a `.kbd` and it reads
  well, Step 5 may revisit.
- **Canvas colors read from tokens.** `getComputedStyle` on
  `documentElement` returns the raw custom property string (`#171717`,
  or Lightning CSS's 8-digit hex for the alpha ones); Canvas 2D accepts
  both. A property that resolves empty (a typo) would paint transparent,
  hence the fallback in the helper and its test.
- **`:focus-visible` on old WebKit.** WKWebView on macOS < 12.3 and
  WebKitGTK < 2.36 do not match `:focus-visible`; the outline rule then
  never applies and the UA focus ring shows, which is today's behavior for
  every button except `#format-choices`. Acceptable; noted in the guide.
  Tauri's minimum macOS is not pinned in `tauri.conf.json`.
- **Lightning CSS at `safari13`.** Vite+ may rewrite or warn on syntax it
  down-levels. Step 1 diffs the emitted `:root` block; if
  `rgb(... / ...)` or `calc()` in a custom property were rewritten
  unexpectedly, record it in `learnings.md`.
- **Check-mark indicator vs the filter menu's existing leading glyphs.**
  The filter items already start with a `.dot` / `.face` / star spans; the
  new indicator column sits before them, widening every row by ~1.1rem
  inside a `min-width: 180px` menu. Step 3 must confirm nothing wraps and
  the menus still fit above the strip bar (`max-height: calc(100vh -
  240px)` is unchanged).
- **The side-nav modal on a small window.** `min(720px, 80vh)` on a
  720px-tall window leaves 576px; the shortcuts table scrolls, but the
  nav's six items plus header must fit without scrolling. Step 6 checks
  1280x720; if the nav overflows, reduce its item padding, not the modal.
- **Visual regressions cannot be unit-tested.** Each step carries a Windows
  manual check; macOS and Linux go to `todo.md` through the wrap-up. A
  reviewer should read the `style.css` diff rule by rule against the
  removed values rather than trust CI.
- **Scope creep risk in Steps 3–5.** They touch every pane. Keep each to
  color / border / radius / indicator swaps and `className` assignments;
  any layout tweak noticed on the way is a separate PR.

## Progress

- (2026-09-29) Step 1 complete
- (2026-09-29) Step 2 complete; the plan was extended to every view (audit.md) and a settings side-nav step, at the user's request
- (2026-09-29) Step 3 complete
