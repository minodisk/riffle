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

# Align the label-name inputs in the settings window

## Purpose

In the settings window's Sidecar tab, the `xmp:Label` name section
(`#label-names` in `crates/app/ui/settings.html`) renders five rows of
`<label>Red <input id="label-name-red" type="text" /></label>`. The global
`label { display: block; }` rule in `crates/app/ui/settings.css` lays the
input inline after the color word, so each input starts at a different x
position (Red, Yellow, Green, Blue, Purple have different widths). Aligning
the inputs in a single column makes the section read as a form instead of a
ragged list. Saving behaviour (`change` listeners on the inputs by id in
`crates/app/ui/src/settings.ts`) is untouched.

## Steps

- [x] Step 1: Lay out the label-name rows on a shared two-column grid so every input starts at the same x position
  - Done when:
    - In the Sidecar tab with `Lightroom (.xmp)` selected, the left edges of
      the five text inputs are vertically aligned, and the first column is
      as wide as the widest color name (no hard-coded width).
    - Each input is still labelled by its color name (the `<label>` wrapping
      the input stays the label; the accessibility tree still exposes
      "Red", "Yellow", ... as the input names).
    - Switching to `PhotoLab (.dop)` still hides `#label-names` (the
      `hidden` attribute keeps working).
    - The `<p>` explanation and the `.actions` button row still span the
      full width of the section.
    - No change to `crates/app/ui/src/settings.ts` or `labels.ts`; saving
      label names behaves as before.
    - `mise run ci` passes.
  - Implementation approach:
    - Files: `crates/app/ui/settings.css` (CSS only); touch
      `crates/app/ui/settings.html` only if wrapping the color word in a
      `<span>` turns out to be needed (see below).
    - Recommended layout: make `#label-names` a grid with
      `grid-template-columns: max-content 1fr` and a `column-gap`, and make
      each direct `<label>` child a subgrid row:
      `#label-names > label { display: grid; grid-template-columns: subgrid; grid-column: 1 / -1; align-items: center; }`.
      The `<label>` element stays a real box, so the implicit label/input
      association is unchanged. `subgrid` is supported by all three Tauri
      webviews (WKWebView, WebView2/Chromium >= 117, recent webkit2gtk).
      Fallback option if subgrid misbehaves on one platform: `display: contents`
      on the label with the color word wrapped in a `<span>` (markup change,
      and older webviews have dropped `display: contents` labels from the
      accessibility tree, so prefer subgrid).
    - Span the non-row children: `#label-names > p, #label-names > .actions { grid-column: 1 / -1; }`.
    - Hidden pitfall (must handle): a rule on `#label-names` that sets
      `display` has id specificity and overrides the global
      `[hidden] { display: none; }`, so the block would stop hiding when
      `.dop` is selected. Scope the grid rule as
      `#label-names:not([hidden])` or add `#label-names[hidden] { display: none; }`.
      Verify by toggling the sidecar-format radios.
    - The text node `Red ` ends with a space that grid trims; rely on
      `column-gap` (e.g. `0.5rem`, matching the existing `.actions` gap) for
      the spacing between name and input. Keep the existing `label` padding
      unless it visibly double-spaces the rows.
    - Verification is manual (no DOM tests exist for the settings window):
      run the app (`cargo tauri dev` or open `settings.html` via
      `pnpm exec vp dev`) and check alignment, hiding, and that editing a
      name still persists. Run `mise run ci` for fmt/lint/type-check/test.

## Trade-offs and risks

- Subgrid (recommended) vs `display: contents` on the label: subgrid keeps
  the `<label>` as a real box and needs no markup change; `display: contents`
  is simpler CSS but has a history of removing labels from the accessibility
  tree in older WebKit/Chromium, and needs a `<span>` wrapper around the
  color word to be robust. The plan picks subgrid; the caller may prefer the
  `display: contents` route if subgrid renders wrongly on the target Linux
  webkit2gtk version.
- Alternative not taken: a fixed-width `<span>` for the color name
  (`inline-block; width: 4.5em`). Simplest, but hard-codes a width that
  breaks if names change or the font differs; the acceptance criterion is
  "regardless of color-name length", so a `max-content` column is preferred.
- Alternative not taken: converting to `<label for=...>` + sibling inputs in
  one grid. Aligns cleanly but changes markup for all five rows and switches
  to explicit association for no extra benefit.
- Risk: the id-specificity vs `[hidden]` interaction described above; it is
  the one way this CSS-only change can break behaviour (the block failing
  to hide for `.dop`). Covered by the Done-when criteria.

## Progress

- (2026-09-24) Step 1 complete (subgrid layout for #label-names, scoped as
  `:not([hidden])`); visual check pending, see learnings.md
