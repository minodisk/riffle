# Learnings: ui-design-tokens

## Step 1: tokens, shared component classes, settings modal

- The oklch-to-hex check of the Decisions table (a small Node script using
  the OKLab to linear sRGB matrices and the sRGB transfer function, run on
  2026-09-29): every gray is exact (`0.145` → `#0a0a0a` (10.04), `0.985` →
  `#fafafa` (249.97), `0.205` → `#171717` (23.08), `0.922` → `#e5e5e5`
  (229.03), `0.269` → `#262626` (38.09), `0.708` → `#a1a1a1` (160.69),
  `0.556` → `#737373` (115.14)); `--destructive`
  `oklch(0.704 0.191 22.216)` → `#ff6467` (255.00 clipped, 99.74, 102.98),
  matching Tailwind v4's `red-400`.
- `pnpm exec vp build` (no `TAURI_ENV_PLATFORM`, so the `safari13` target)
  emits `:root` with every custom property kept and in source order; the
  `calc()` radius expressions are untouched (`calc(var(--radius) * .6)`).
  Lightning CSS rewrites each `rgb(255 255 255 / 10%)` to 8-digit hex
  (`#ffffff1a`, `#ffffff26`) and `#808080` to `gray`; this is the same
  rewrite the pre-existing `rgb(0 0 0 / 60%)` backdrop already got
  (`#0009`), and 4 / 8-digit hex is in Safari 10+, so it is safe. `inset: 0`
  on `.dialog:not([hidden])` is expanded to `top/right/bottom/left`, as
  before.
- The hover-at-90% hexes are computed over `--card` (the surface both
  buttons sit on): `--primary` 90% over `#171717` → `#d0d0d0`,
  `--destructive` 90% over `#171717` → `#e85c5f`.
- `.outline` and `.input` / `.select` use `rgb(255 255 255 / 4.5%)` as the
  surface: shadcn's dark `bg-input/30` is 30% of `--input`'s 15%.
- `.select option` gets `--popover` / `--popover-foreground` so the native
  dropdown list does not render the select's light text on the OS's light
  list background.
- `#settings-dialog label` became `.field` on every settings label;
  `#label-names > label`'s `display: grid` still wins by specificity.
  `#settings-dialog .keys .add` keeps only its padding (the `add` class is
  kept); the hover moved to `.button.ghost`.
- In the Neutral palette `--muted` and `--accent` are the same `#262626`, so
  the selected tab and a hovered tab look alike, and a chip's ghost `×`
  (on a `--muted` chip) has no visible hover fill. Watch for this in the
  manual check; if it reads badly, a later step can lift the chip `×` hover
  to `--input` rather than invent a color.

## Step 2: first-run format dialog, trash confirmation, sequence dialog

- No TypeScript change was needed: `main.ts` finds the format buttons by
  `button[data-format]` and the others by id, and nothing selects the old
  `.actions` class, so the two `.actions` rows became `.dialog-actions`.
- `#sequence-title` / `#trash-title` keep a `margin-bottom: 0.5rem` rule:
  `.dialog-title` sets `margin: 0` (the settings title gets its spacing
  from `#settings-header`), and dropping the per-id rule entirely would put
  the title flush against the first line. It is spacing only, no color.
  `#format-title` needs none because `#format-box p` already gives it
  `margin: 0 0 0.75rem` (it is a `<p>`).
- `#format-box` keeps its own `padding: 1.25rem 1.5rem` and `max-width`,
  which override `.dialog-box`'s padding by id specificity;
  `#format-choices button` keeps `flex: 1; padding: 0.5rem` over `.button`.
- `.dialog-box [hidden]` has specificity (0,2,0), lower than the
  `#settings-dialog [hidden]` (1,1,0) it replaces, so it no longer beats an
  id-scoped `display`. Checked every element that is ever `hidden` inside a
  dialog box (`#format-error`, `#sequence-rebuild`, `#sequence-running`,
  `#trash-empty`, `#label-names`, `#clear-index-note`, `#tab-debug`, the
  tab panels): none has an id rule that sets `display` on its hidden state
  (`#label-names` is scoped `:not([hidden])`), so the fold is safe. A later
  id rule that sets `display` on something inside a dialog box must scope
  itself `:not([hidden])` the same way.
- The failed lists and `#format-error` moved from `--reject-color` to
  `--destructive`; `#sequence-rows .changed` stays on `--pick-color`.

## Step 3: menus, strip bar, filmstrip cells

- The check mark is a `span.menu-check` holding `CHECK_SVG`, prepended to
  each item by `addMenuCheck` in `main.ts`: a startup loop over
  `.menu-item[role^="menuitem"]` covers `index.html`'s static items, and
  `showMenu` / `rebuildExifMenu` call it on the items they build. It is
  absolutely positioned in the item's `1.6rem` left padding, so it takes no
  flex gap and `#filter-menu [data-stars] { gap: 0 }` is unaffected; CSS
  alone shows it on `[aria-checked="true"]`, so the existing `aria-checked`
  toggling drives it. `#filter-reset` gets the (never shown) mark too, since
  it matches the selector.
- `check` is one of Lucide's Feather-derived icons (listed in
  `LICENSE-lucide`), so the `icons.ts` header also names Feather's MIT
  notice.
- `.menu-label` is indented to the item labels (`1.6rem`, shadcn's `inset`
  label) rather than to the menu edge, so the `AF eye` and EXIF headings
  line up with their items.
- `.menu-separator` uses `margin: 4px -4px` (shadcn's `-mx-1 my-1`) so the
  line spans the menu's new `4px` side padding.
- The unlit stars (`.off`) are `--muted-foreground` at `opacity: 0.4`, not
  `--muted`: `--muted` is the hover fill `#262626`, so the stars would
  vanish on a hovered row.
- The menu item's keyboard focus is shown as the hover fill
  (`:focus-visible` on `--accent`, no outline), as shadcn's item does.
- `#sort-toggle:disabled`'s `0.4` opacity became `.button:disabled`'s `0.5`.
- `.cell span.name.labeled`'s `#fff` became `--foreground` (`#fafafa`), so
  Step 5's hex grep does not have to come back to a cell rule; its `#000`
  halo stays.
- The strip bar toggles now inherit `--foreground` instead of `#999`, as
  shadcn's outline button does; `.active` is `--primary` for color and
  border and wins over the outline hover by id specificity.

## Deferred issues (todo candidates)

- **Pending manual check (Windows, Step 1 of ui-design-tokens):** the user
  runs `mise run tauri:dev`, opens Settings and walks every tab (Sidecar
  with both XMP and `.dop` so `#label-names` shows and hides, Culling,
  Keyboard Shortcuts with a chip removed and a key captured, Cache, MCP with
  `Copy`). Expected: the modal reads as shadcn Neutral dark (near-black
  backdrop over the `#0a0a0a` window, `#171717` box, hairline
  `rgb(255 255 255 / 10%)` borders, `#fafafa` text, outline buttons,
  `#737373` focus ring), every control is consistent with the others, the
  Tab trap cycles through the same controls, and the modal's size and
  scrolling (a long shortcuts table) are unchanged; also check the
  `--muted` / `--accent` tab and chip `×` hover noted above. Step 1's
  checkbox was ticked on the automated criteria only. Basis: plan Step 1
  "Done when"; files `crates/app/ui/style.css`, `crates/app/ui/index.html`,
  `crates/app/ui/src/settings.ts`.
- **Pending manual check (Windows, Step 2 of ui-design-tokens):** the user
  runs `mise run tauri:dev`; clears the `sidecarFormat` key (or uses a fresh
  profile) to see the first-run dialog and picks a format; opens `Move
  Rejected to Trash` from the folder tree's right-click on a folder with
  rejects and one without; opens `Sequence JPEG Timestamps…` on a JPEG
  folder and runs it. Expected: the format dialog is on the `#171717` card
  with three outline choices; the trash dialog shows its rows on `--muted`,
  the total, a red destructive `Move to Trash` and an outline `Cancel`; the
  sequence preview rows show changed (pick color) and unchanged
  (`#a1a1a1`) lines, a light primary `Run` and an outline `Cancel`; the
  disabled buttons during a run read as disabled (half opacity); all four
  dialogs (with Settings) look like one family. Step 2's checkbox was
  ticked on the automated criteria only. Basis: plan Step 2 "Done when";
  files `crates/app/ui/style.css`, `crates/app/ui/index.html`.
- **Pending manual check (Windows, Step 3 of ui-design-tokens):** the user
  runs `mise run tauri:dev` and opens a folder; opens the filter and sort
  menus and toggles a few items; right-clicks a cell for the context menu;
  clicks a cell and shift-clicks another; watches a fresh folder's
  placeholders fill in; renames a file inline. Expected: menu items hover on
  `#262626`, a checked item shows the check mark in its leading column with
  no fill and it disappears when unchecked, labels stay aligned, nothing
  wraps and the filter menu still fits above the strip bar; the `AF eye`
  heading, the star rows (unlit stars dim), the `#filter-exif` groups and
  the separators are visible on `#171717` (if a separator vanishes, raise
  `.menu-separator` to `--input`); the context menu's `.shortcut` column is
  `#a1a1a1` and a radio item shows its check; the filter / sort toggles
  are outline buttons, the lit filter toggle is `#e5e5e5`, and the sort
  toggle is half-opacity in a JPEG folder; the focused cell has the light
  `--primary` border on `#262626`, the other selected cells the `#737373`
  border on the same fill, and the two read as different; the rejected
  dimming, the burst band, the badges and the failed red are as before; the
  placeholders are `#262626`; the inline rename field is `#0a0a0a` with a
  `#737373` outline; the strip and menu scrollbars are thin and visible.
  Step 3's checkbox was ticked on the automated criteria only. Basis: plan
  Step 3 "Done when"; files `crates/app/ui/style.css`,
  `crates/app/ui/index.html`, `crates/app/ui/src/main.ts`,
  `crates/app/ui/src/icons.ts`, `crates/app/ui/src/strip.ts`.
- **Pending manual check (macOS WKWebView and Linux WebKitGTK, Steps 1 to
  3):** the same Settings walk-through as the Step 1 Windows check, the
  same first-run / trash / sequence dialog pass as the Step 2 Windows check,
  and the same menu / strip bar / cell pass as the Step 3 Windows check, on
  each platform (no device here); also that `scrollbar-color` is honored or
  harmlessly ignored. Also confirm that the native
  checkboxes / radios take `accent-color` (Safari 15.4+ / WebKitGTK 2.36+;
  older engines show the UA control) and that `:focus-visible` rings
  appear. The three steps' checkboxes were ticked on the automated criteria
  only. Basis: plan Steps 1 to 3 "Done when"; files
  `crates/app/ui/style.css`, `crates/app/ui/index.html`,
  `crates/app/ui/src/settings.ts`, `crates/app/ui/src/main.ts`,
  `crates/app/ui/src/icons.ts`, `crates/app/ui/src/strip.ts`.
