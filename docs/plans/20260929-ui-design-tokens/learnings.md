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
- **Pending manual check (macOS WKWebView and Linux WebKitGTK, Step 1):**
  the same Settings walk-through as above on each platform (no device
  here). Also confirm that the native checkboxes / radios take
  `accent-color` (Safari 15.4+ / WebKitGTK 2.36+; older engines show the UA
  control) and that `:focus-visible` rings appear. Step 1's checkbox was
  ticked on the automated criteria only. Basis: plan Step 1 "Done when";
  same files.
