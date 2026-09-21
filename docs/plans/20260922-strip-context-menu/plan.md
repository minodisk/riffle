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

# Strip context menu for flags

## Purpose

Right-clicking a photo in the left pane (the strip, `crates/app/ui/src/strip.ts`)
currently shows the WebView's default menu (Back / Reload / Inspect), which is
useless while culling. After this work a right-click on a strip cell opens the
app's own menu with the flag actions (Pick, Reject, Unflag), each showing its
current shortcut from the keymap (defaults plus the user's overrides), and
choosing an item does exactly what the key does: same sidecar write, same
undo entry, same auto-advance.

## Background (from investigation)

- The app has a single current file (`index` in `main.ts`); there is no
  multi-selection. A left click on a cell calls `strip.init`'s `select`
  callback, which sets `index` and calls `show()`. **Decision: a right-click
  first selects the clicked photo the same way, then the menu acts on the
  current file.** That keeps "what the menu acts on" identical to "what the
  strip highlights" and lets the menu reuse the keyboard code unchanged.
- All flag logic is inline in the `keydown` listener at the bottom of
  `main.ts` (the `switch (action)` and the auto-advance check after it).
  The `pick` case silently does nothing under XMP (`effectivePick`).
- The frontend already holds the resolved keymap (`keyBindings`, updated by
  `applyKeymap` from `shortcuts` / `shortcuts-changed`) and `keys.ts`
  exports `displayKey`. No new Tauri command is needed.
- **HTML menu, not a native `tauri::menu` popup.** `shortcuts::accelerator`
  deliberately maps modifier-less keys to `None`, so a native popup would
  show no shortcut for the default flag keys `p` / `x` / `u`. A native popup
  would also need a new command, menu-event routing back to the frontend and
  a duplicated label table, while `#filter-menu` / `#sort-menu` already
  establish an HTML `role="menu"` pattern (markup in `index.html`, shared
  rules in `style.css`, open / outside-mousedown-close / Escape-close in
  `main.ts`).
- **Default WebView menu is suppressed only inside `#strip`.** Elsewhere it
  is left alone (Inspect stays reachable in dev builds); see trade-offs.
- Vitest runs with `environment: "node"`, so tests cover pure functions
  only; the DOM wiring is verified manually (GUI automation does not work,
  `docs/agents/tauri-app.md`).

## Steps

- [x] Step 1: Extract the keydown action dispatch into a reusable `runAction`
  - Done when:
    - `main.ts` has a function (e.g. `runAction(action: string): boolean`)
      holding the whole `switch (action)` body and the auto-advance check
      that follows it, returning whether the action was handled.
    - The `keydown` listener only resolves `keyName` -> `keymap.get(key)`,
      handles the Escape-closes-menu cases, calls `runAction`, and calls
      `event.preventDefault()` when it returns `true` — behaviour is
      unchanged (the `pick`-under-XMP early `return` must still leave the
      default key behaviour alone, i.e. return `false`).
    - `mise run ci` passes; no new tests (pure refactor of DOM-bound code).
  - Implementation approach:
    - Keep the current-path capture (`const current = files[index]` before
      the switch) inside `runAction` so the "file dropped out of the filter
      already moved the cursor" rule (`docs/agents/tauri-app.md`, "A
      judgement's own move must not double up") is preserved verbatim.
    - No other behavioural change; do not touch strip.ts in this step.

- [ ] Step 2: Pure helpers for the flag menu, with tests
  - Done when:
    - A new module (e.g. `crates/app/ui/src/context.ts`) exports:
      - `flagMenuItems(bindings: Binding[], sidecarFormat: string)` returning
        an ordered list of `{ action, label, shortcut }` for `pick`,
        `reject`, `unflag` (labels: "Pick", "Reject", "Unflag"), where
        `shortcut` is `displayKey` of the action's first key (or `""` when
        the action is absent), and `pick` is omitted unless
        `effectivePick(true, sidecarFormat)` (see trade-offs).
      - `menuPosition(x, y, width, height, viewportWidth, viewportHeight)`
        returning the top-left corner clamped so the menu stays inside the
        viewport.
    - `context.test.ts` covers: default keys show `p` / `x` / `u`; an
      override changes the shown key; the first key is shown when an action
      has several; `pick` is dropped under `xmp` and present under `dop`;
      an action missing from the bindings gives an empty shortcut; the
      clamp flips / shifts near the right and bottom edges and leaves an
      interior point alone.
    - `mise run ci` passes.
  - Implementation approach:
    - Import `Binding` and `displayKey` from `./keys.js`, `effectivePick`
      from `./pick.js` (relative imports with `.js`, per the guide).
    - No DOM access in this module (tests run under `environment: node`).

- [ ] Step 3: Wire the context menu into the strip and the main window
  - Done when:
    - `strip.ts`: each cell listens for `contextmenu`, calls
      `event.preventDefault()`, and reports `(index, clientX, clientY)`
      through a new callback (extend `init` with a second argument or add
      `onContextMenu`, whichever is smaller). `#strip` also prevents the
      default on `contextmenu` so empty strip space does not show the
      WebView menu either.
    - `index.html` has a `<div id="context-menu" role="menu" hidden>`
      directly under `<body>` (not inside `#side`, whose comment forbids a
      stacking context / overflow there); `style.css` styles it
      `position: fixed` with the same look as `#filter-menu` / `#sort-menu`
      (extend the shared selector list), items as `<button role="menuitem">`
      with the label on the left and the shortcut right-aligned in a muted
      colour.
    - `main.ts`: on the strip callback it sets `index = selected; show()`
      (skipping when already current, as the click callback does), rebuilds
      the items from `flagMenuItems(keyBindings, sidecarFormat)`, positions
      the menu via `menuPosition` using the menu's measured size, and shows
      it. An item click closes the menu and calls `runAction(action)`; the
      menu closes on `mousedown` outside it (same `document` pattern as the
      other menus), on Escape (add a case beside the filter/sort ones in the
      keydown listener), and is closed by `openDirectory` / when `files`
      becomes empty so it never floats over a changed folder.
    - Choosing Reject / Pick from the menu produces the same `set_rating`
      invoke, the same undo entry and the same auto-advance as the key
      (guaranteed by going through `runAction`).
    - The WebView default menu no longer appears anywhere over `#strip`; it
      is unchanged over the viewer, meta pane and tool buttons.
    - `mise run ci` passes; the PR description carries a manual checklist:
      right-click a non-current cell selects it and opens the menu at the
      pointer; items show `p` / `x` / `u` by default and the overridden key
      after changing Pick in Settings (menu must reflect the change without
      restart); Pick is absent under XMP; Reject from the menu advances
      when auto-advance is on and `Edit > Undo` reverts it; Escape / click
      outside closes; the menu is not clipped near the bottom of the
      window; right-click on the viewer still shows the default menu.
  - Implementation approach:
    - Step 1 and Step 2 are merged first; this step is DOM-only glue.
    - Follow the `setFilterMenuOpen` / `setSortMenuOpen` shape
      (`hidden` toggle + `aria-expanded` is not needed since there is no
      toggle button). Items are rebuilt on every open, so a keymap change
      needs no extra listener.
    - Keep `strip.ts` free of judgement knowledge: it only reports the
      index and pointer position.

- [ ] Step 4: Document the decision
  - Done when:
    - `CLAUDE.md`'s layout paragraph mentions the new `context.ts` module
      in one clause, in the same style as the existing entries.
    - `docs/agents/tauri-app.md` gains a short Frontend item recording that
      the strip uses an HTML context menu because `accelerator()` cannot
      show modifier-less keys natively, and that the default WebView menu
      is suppressed only inside `#strip` (tag it Inferred / Hit as
      appropriate from what Step 3 found).
    - `mise run ci` passes (lychee link check included).

## Trade-offs and risks

- **HTML menu vs native `tauri::menu` popup.** HTML chosen (reasons above:
  native accelerators cannot show `p` / `x` / `u`; no new IPC; matches the
  existing filter/sort menus). Cost: it does not look like the OS menu and
  has no native keyboard navigation. If native fidelity is preferred, the
  shortcut display would have to be baked into the item *title*
  (`"Pick (P)"`), plus a `popup_flags` command and menu-event routing —
  roughly triple the code.
- **Scope of default-menu suppression.** Only `#strip`. Alternative: a
  document-level `contextmenu` preventDefault, which also hides Inspect in
  devtools-enabled builds and touches areas the requirement did not ask
  about. Minimal scope preferred; easy to widen later in one line.
- **Pick under XMP: omit vs disabled.** Plan omits the item (the key is a
  silent no-op there). Showing it disabled with a "needs .dop" hint is the
  other option; it explains more but adds a state and styling for one item.
- **Which key to show when an action has several.** First key in stored
  order, matching `Keymap::accelerator_for`. Alternative: join all keys with
  ", ". First-key keeps the menu narrow.
- **Auto-advance also applies to the menu.** Because the menu goes through
  `runAction`, Reject / Pick from the menu advances when auto-advance is on,
  exactly like the key. If a mouse action should not advance, `runAction`
  needs an `advance: boolean` parameter (small change).
- **Right-click changes the current file.** Necessary because judgements
  act on `files[index]`; a right-click that did not select would need the
  whole `judge` path parameterised by path.
- **Items limited to flags.** Stars and colour labels are deliberately not
  in the menu; `flagMenuItems` can grow later without changing the wiring.
- **Risk: `contextmenu` on macOS with Ctrl+click.** Ctrl+click also fires
  `contextmenu`; verify in Step 3's manual check that Ctrl+click opens the
  menu without a double select (the `selected === index` guard makes the
  second a no-op).

## Progress

- (2026-09-22) Plan approved by the user
