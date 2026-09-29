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
classes are built on them in shadcn's variants, the existing dialogs,
inputs, buttons and menus are migrated to those classes, and the rule is
written down in a guide so later UI work reuses them. The look changes on
purpose (a darker, cooler neutral; check-mark menu selection instead of a
blue fill). Nothing is added to `package.json`; no React, no Tailwind, no
runtime CSS. Behavior does not change: only the appearance.

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
  (`.folder.selected`) both use `--accent` / `--accent-foreground`; they
  stay distinguishable by font weight (`.current` is `font-weight: 600`,
  `.selected` is normal weight), the way shadcn's sidebar marks the active
  item with `font-medium` on the same accent surface. A checked menu item
  (`aria-checked="true"` on a `menuitemcheckbox` / `menuitemradio`) shows a
  check-mark indicator in a leading column instead of a background fill,
  as shadcn's `DropdownMenuCheckboxItem` / `RadioItem` do; the mark is
  Lucide's `check` icon inlined the way `SCAN_FACE_SVG` in
  `crates/app/ui/src/strip.ts` inlines `scan-face` (same `/*! ... */`
  license header, covered by `crates/app/ui/LICENSE-lucide`).
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

Ground truth gathered at planning time (2026-09-29):

- The frontend is vanilla TypeScript; `index.html` holds the static dialogs
  (`#format-dialog`, `#sequence-dialog`, `#trash-dialog`, `#settings-dialog`)
  and menus (`#filter-menu`, `#sort-menu`, `#context-menu`), and the
  dynamically built controls live in `src/settings.ts` (shortcut chips,
  `+` / `×` / `Reset` buttons, `Copy`, `.copyable` blocks), `src/main.ts`
  (`showMenu`'s `#context-menu` items with a `.shortcut` span, the
  `#filter-exif` items, the `#meta-status .error` dismiss button, the
  sequence preview rows), `src/folders.ts` (the tree's inline rename
  `input.name`) and `src/strip.ts` (the cell's inline rename `input.name`).
  `src/modal.ts`, `src/firstrun.ts`, `src/trash.ts`, `src/sequence.ts` and
  `src/rename.ts` are DOM-free and need no change.
- No test asserts on a dialog / button / input / menu class name (the only
  class-name assertions are on `.cell` / `.folder` state classes, which
  keep their names). `modal.test.ts` tests `SettingsModal.key` /
  `cycleFocus` only.
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
  user; this plan's guide is a requirement of the work, so Step 4 creates
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

- [ ] Step 2: Migrate the first-run format dialog, the trash confirmation and the sequence dialog
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

- [ ] Step 3: Migrate the menus, the strip bar toggles, the folder tree selection, the inline rename editors and the remaining hardcoded colors
  - Done when:
    - `.menu` (surface `--popover` / `--popover-foreground`, `1px solid
      var(--border)`, `--radius-md`, the shadow, `padding: 4px`) and
      `.menu-item` (full-width flex row, `--radius-sm`, hover / focus on
      `--accent` / `--accent-foreground`, a fixed-width leading indicator
      column) exist in the component block; `#filter-menu`, `#sort-menu`
      and `#context-menu` carry `menu` in `index.html`; every item carries
      `menu-item`, both the static ones in `index.html` and the ones built
      in `main.ts` (`showMenu`, the `#filter-exif` builder). The id rules
      keep only positioning, `z-index`, `min-width`, `max-height`,
      `overflow` and the scrollbar rules; the `hr` separator, `.heading`
      and `.shortcut` read from the tokens (`--border`,
      `--muted-foreground`).
    - A `[aria-checked="true"]` item shows Lucide's `check` in its leading
      column (a `CHECK_SVG` constant next to `SCAN_FACE_SVG` in `strip.ts`,
      or a new `icons.ts` if `strip.ts` is the wrong home; same license
      header; `index.html`'s static items get the same markup by the
      builder or a small `main.ts` pass at startup, whichever keeps one
      source of truth) and no background fill; an unchecked item leaves
      the column empty so labels stay aligned. `#2f4a6a` is gone. The
      filter menu's `.dot`, `.face` and star spans keep working next to
      the indicator column.
    - `#filter-toggle` and `#sort-toggle` are `button outline`; `.active`
      uses `--primary` for its color and border; `#sort-toggle:disabled`
      relies on the button's `:disabled`.
    - `.folder.current` and `.folder.selected` use `--accent` /
      `--accent-foreground`, `.current` additionally `font-weight: 600`;
      `.folder:hover` uses `--muted`; `#2d4059` and `#2a2f36` are gone.
      `#folders:focus .folder.cursor`, the inline rename outlines and
      `body.dragging #main` use `--ring`.
    - The inline rename inputs (`.folder input.name`, `.cell input.name`)
      keep their borderless in-place layout and take `color` /
      `background` from `--foreground` / `--background`.
    - `#meta-status .error button` is `button ghost` (set in `main.ts`);
      `#meta-status .note`, `#meta .group`, `#meta dt`, `#position`,
      `#empty`, `.folder`, `.folder .count`, `.folder .expander` and the
      scrollbar colors read from the tokens (`--muted-foreground`,
      `--muted`, `--border`).
    - No hex color remains in `style.css` outside `:root`, the two
      hover-at-90% button hexes, and the strip cell rules that encode
      per-cell semantics (`.cell.failed`'s `#402020`, the `text-shadow: 0
      0 2px #000` halos, `.cell.burst::before`'s tint, `.cell.current` /
      `.cell.selected` borders, which are the strip's own selection and
      are left as they are); `grep -E '#[0-9a-f]{3,6}\b'` confirms and
      each deliberate exception is listed in `learnings.md`.
    - `mise run ci` passes.
    - Manual GUI check on Windows: open a folder; open the filter and sort
      menus and toggle a few items (hover, the check mark appears and
      disappears, the labels stay aligned, the `AF eye` heading, the star
      rows, the `#filter-exif` groups); right-click a cell for the context
      menu (the `.shortcut` column, a radio item's check); select two
      folders and open one (the open one is bold on the same accent);
      rename a folder and a file inline; press `×` on a meta status error;
      drag a folder over the window for the drop outline. Record the
      result and the open macOS / Linux checks in `learnings.md`.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Steps 1 and 2 are merged.
    - Do not change any positioning, `z-index`, `overflow`, `max-height`
      or size in the menus: the comment on `#film` explains why the menus
      must not get a stacking context, and `#filter-menu` / `#sort-menu`
      open upward with a computed `max-height`.
    - The indicator column: shadcn's item reserves `pl-8` and absolutely
      positions the check at `left-2`; the same shape here is
      `.menu-item { position: relative; padding-left: 1.6rem }` with the
      check `position: absolute; left: 0.5rem; width: 12px; height: 12px`
      and `display: none` unless `[aria-checked="true"]`. Because the
      check is inside the button, it needs no `main.ts` logic beyond the
      markup; the `aria-checked` toggling in `main.ts` already drives it.
    - `.folder.current` vs `.selected` weight: `.folder` rows are `0.8rem`
      with `line-height: 1.4`; check that `600` weight does not change the
      row's width enough to clip a long name differently (the `.name` span
      already ellipsizes).

- [ ] Step 4: Write the styling guide and point CLAUDE.md at it
  - Done when:
    - `docs/agents/ui-styling.md` exists, opens like the other guides
      ("Read this before touching `crates/app/ui/style.css`, `index.html`
      or a `.ts` file that creates a control"), and states the rule: UI
      code uses the `:root` tokens and the shared component classes, never
      an ad-hoc color, radius or border; carries the Decisions table (the
      tokens, the shadcn oklch source and the hex used, with what each
      paints); lists every component class and its variants with the
      element it goes on and which controls use which variant (the
      `primary` only on `Run` rule, `destructive` on `Move to Trash`,
      `outline` for the rest, `ghost` for icon-only), and that dynamically
      created controls in `.ts` set `className` to the same classes; the
      selection conventions (accent + weight for the tree, check mark for
      menus, the strip cell's own border scheme); the CSS features in use
      and the ones to avoid (anchor positioning, `base-select`,
      `color-mix()`, `oklch()`, nesting, `:has()`, `@layer`) with the
      webview reason (WebView2 / WKWebView / WebKitGTK, `safari13` build
      target); and the exceptions (the strip cell's per-cell semantics,
      the `--label-*` / `--stars-color` / `--pick-color` /
      `--reject-color` tokens).
    - `docs/agents/tauri-app.md`'s "Frontend" section gets one cross-link
      line to the new guide; its two styling entries ("Scope an id's
      `display` override to `:not([hidden])`", "Style the strip
      placeholder on `.cell img:not([src])`") stay where they are.
    - `CLAUDE.md`'s layout paragraph for `crates/app/ui` gains one clause
      naming `style.css` as the shadcn Neutral tokens and shared component
      classes (with the guide's path), in the paragraph's existing style.
    - `mise run lint` (lychee) passes on the new links.
    - The open manual checks (macOS WKWebView and Linux WebKitGTK visual
      pass over the settings modal, the three dialogs, the menus, the tree
      selection and the inline editors) are listed under `## Deferred
      issues (todo candidates)` in `learnings.md` so the wrap-up's todo
      curation files them the way `todo.md`'s other `### App: ... manual
      checks` entries are filed.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Steps 1–3 are merged, so the guide lists the final names.
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
  fainter. If a separator (the tab list's bottom border, the menu `hr`)
  disappears on the Windows check, the step may raise that one rule to
  `--input` (15%) rather than invent a color; record it.
- **`--muted-foreground` (`#a1a1a1`) is lighter than today's `#999` /
  `#888`.** Secondary text (meta labels, notes, the position counter)
  gains contrast; intended, but it is the most widespread visual change
  outside the dialogs.
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
- **Visual regressions cannot be unit-tested.** Each step carries a Windows
  manual check; macOS and Linux go to `todo.md` through the wrap-up. A
  reviewer should read the `style.css` diff rule by rule against the
  removed values rather than trust CI.
- **Scope creep risk in Step 3.** "Remaining hardcoded colors" touches the
  folder tree, the meta pane and the strip bar. Keep it to color / border
  / radius / indicator swaps; any layout tweak noticed on the way is a
  separate PR.

## Progress

- (none yet)
