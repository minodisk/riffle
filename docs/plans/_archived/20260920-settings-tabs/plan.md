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

# Settings window tabs

## Purpose

The settings window (`crates/app/ui/settings.html`) stacks Sidecar, Culling,
Keyboard Shortcuts and (in debug builds) Debug in one scrolling column. The
Shortcuts table dominates it and the short sections above it read as noise.
Splitting the sections into macOS-preferences-style top tabs shows one section
at a time, keeps each tab short, and lets the Debug tab appear only when the
debug section is enabled, without changing how any setting is read or written.

Current state the plan is based on:

- `settings.html` has three `<h2>` sections followed by
  `<section id="debug" hidden>`, then a shared `#status` error line.
  `settings.css` has a `#debug[hidden] { display: none }` rule.
- `crates/app/ui/src/settings.ts` wires each control to Tauri commands; the
  `debug_build` command sets `#debug.hidden`. A `window` keydown listener adds
  the pressed key to the capturing Shortcuts row while `capturing !== null`
  (Escape cancels); it `preventDefault`s every key it consumes.
- The window is opened by `open_settings` in `crates/app/src/main.rs` with a
  fixed `inner_size(480, 640)` and is closed together with the main window.
- Vitest runs under `environment: "node"` (root `vite.config.ts`), so tests
  cannot touch the DOM. The repo pattern is a pure module plus a `*.test.ts`
  next to it (`advance.ts`, `keys.ts`, `sort.ts`).

## Steps

- [x] Step 1: Split the settings window into top tabs (Sidecar, Culling, Keyboard Shortcuts, Debug when enabled)
  - Done when:
    - `settings.html` has a tab bar at the top (`role="tablist"` with one
      `role="tab"` button per section, each `aria-controls` its panel and
      carries `aria-selected`) and each section is a `role="tabpanel"`
      element with `aria-labelledby` pointing at its tab; only the selected
      panel is visible (the others carry `hidden`). Sidecar is selected on
      open.
    - The Debug tab button is `hidden` by default and shown only when
      `debug_build` resolves `true` (the same command that shows the debug
      section today); its panel keeps `id="debug"` and the existing controls.
    - Clicking a tab switches panels. With focus on a tab, ArrowLeft /
      ArrowRight move to the previous / next visible tab (wrapping), Home /
      End to the first / last visible tab, and the newly focused tab is
      activated (automatic activation, roving `tabindex`: selected tab
      `tabindex="0"`, others `-1`).
    - Every existing control behaves as before: sidecar format radios,
      Auto-advance checkbox, shortcuts table incl. key capture, Reset / Reset
      all, Timing logs checkbox, `#status` error line, and the live updates
      from the `sidecar-format` / `auto-advance` events.
    - The tab-selection logic lives in a pure module
      `crates/app/ui/src/tabs.ts` (no DOM), with `tabs.test.ts` covering
      arrow wrap-around in both directions, Home / End, an unknown key
      returning the current tab unchanged, and a tab list with and without
      Debug.
    - `mise run ci` passes; the change is verified by hand in
      `cargo tauri dev` (debug build shows the Debug tab; confirm a release
      build or a stubbed `debug_build=false` hides it).
  - Implementation approach:
    - Files: `crates/app/ui/settings.html`, `crates/app/ui/settings.css`,
      `crates/app/ui/src/settings.ts`, new `crates/app/ui/src/tabs.ts` and
      `crates/app/ui/src/tabs.test.ts`. No Rust changes (the window size stays
      as is; see trade-offs).
    - `tabs.ts` exports something like
      `nextTab(tabs: readonly string[], current: string, key: string): string`
      that maps `ArrowLeft` / `ArrowRight` / `Home` / `End` to the target tab
      id among the *visible* tabs (the caller passes the visible list, so
      Debug is simply omitted when hidden). Keep it to one function.
    - In `settings.ts`, a small `selectTab(id)` toggles `aria-selected`,
      `tabindex` and panel `hidden` for all tabs; click handlers and a
      `keydown` handler on the tablist call it. Attach the arrow-key handler
      to the tablist element, not `window`, so it does not interact with the
      Shortcuts key-capture listener (capture is started from a button inside
      the Shortcuts panel, so focus is never on a tab while capturing; verify
      this by hand). Coerce `hidden` with `!!` per `docs/agents/tauri-app.md`.
    - Keep `#status` outside the panels so an error is visible regardless of
      the active tab; keep the `.actions` Reset-all row inside the Shortcuts
      panel.
    - CSS: generalize `#debug[hidden]` to `[hidden] { display: none }` so
      panels and the Debug tab button hide reliably; style the tab bar as a
      simple row of buttons (selected one highlighted, matching the existing
      `#222` / `#333` / `#444` palette). Drop the per-section `<h2>` inside
      panels only if the tab label already names the section; otherwise keep
      them. Keep the CSS minimal, no animation.
    - Do not persist the selected tab (see trade-offs).
    - Add a short note to `docs/agents/tauri-app.md` only if something
      non-obvious is learned.

## Trade-offs and risks

- Window size per tab: not done. The window keeps its fixed 480x640 from
  `open_settings`; the short tabs show empty space below their content, and
  Shortcuts scrolls inside the window as it does today. Resizing per tab would
  need either a Tauri `setSize` call from the frontend (a new capability
  permission and a visible jump on every tab switch) or a Rust command, for
  little gain.
- Remembering the last tab: not done. The window always opens on Sidecar;
  persisting would need a settings-store key and a command pair. Revisit only
  if users report friction.
- Automatic vs manual tab activation: automatic (focus moves and activates),
  as macOS preferences and most tab bars do.
- The Debug tab appears asynchronously (after `debug_build` resolves), as the
  section does today.
- The key-capture `window` keydown listener `preventDefault`s while a row is
  capturing. If hand testing shows arrow keys being swallowed while capturing,
  the tablist handler should early-return when a capture is active.

## Progress

- Step 1: Split the settings window into top tabs. Also fixed a review
  finding (round 1): clicking a tab while a Shortcuts row was capturing left
  `capturing` set and the panel hidden, so the next keypress silently added a
  binding; `selectTab` now cancels any active capture.
- (2026-09-20) Step 1 complete
