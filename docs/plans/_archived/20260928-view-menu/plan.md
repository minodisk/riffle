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

# Menu items for shortcut-only actions

## Purpose

Some keymap actions can only be reached by their key. The panel toggles
(`toggleStrip` F6, `toggleLeft` F7, `toggleRight` F8, `toggleSides` Tab) and
the view toggles (`focus` f, `zoom` z, `compare` v) have no item in the native
menu bar, so on a keyboard without function keys (reported by the user) the
filmstrip cannot be hidden or shown at all, and none of them is discoverable
from the menu. This adds a `View` menu whose items run the same actions as the
keys, mirroring the action's (rebindable) accelerator the way
`File > Open Folder…` and `Edit > Undo` / `Redo` / `Select All` already do.

Of the 39 keymap actions (`crates/app/src/shortcuts.rs`, `DEFAULTS`), four
have a native menu item today (`open`, `undo`, `redo`, `selectAll`). The
other 35 are sorted as follows:

- **Get a View item**: `toggleLeft`, `toggleRight`, `toggleStrip`,
  `toggleSides`, `focus`, `zoom`, `compare`.
- **Excluded, with reasons** (decided with the user):
  - `previous`, `next`, `burstPrevious`, `burstNext`, `burstFramePrevious`,
    `burstFrameNext`: cursor moves the strip already offers by click and
    wheel; as menu items they would be noise, and their arrow keys exist on
    every keyboard.
  - `extendPrevious`, `extendNext`: selection growth by one file, a
    keyboard-only gesture whose mouse equivalent is `Shift+click`.
  - `grayscale`: a hold-to-preview (keydown starts, keyup ends, in
    `main.ts` `setGrayscale`); a menu click cannot be held.
  - Judgments (`pick`, `reject`, `unflag`, `rate1`–`rate5`, `clear`, the
    seven color labels, `clearlabel`, `rejectRest`, `clearall`): in the
    strip's right-click menu (`crates/app/ui/src/context.ts`, all but
    `rejectRest` / `clearall`) and on plain keys reachable on any keyboard.
    The user chose not to add a native `Photo` menu.
  - `open`, `undo`, `redo`, `selectAll`: already in the menu.

## Steps

- [x] Step 1: Add a `View` menu with the panel and view toggles, its tests and docs
  - Reopened after Round 1 review (the `menu-action` listener missed the
    first-launch format dialog check and did not close the strip's context
    menu first); fixed in the Round 1 addressal commit, and Rounds 2–4 raised
    nothing further on the gates, so the step is closed again.
  - Done when:
    - The menu bar has a `View` submenu with, in this order: `Left Pane`,
      `Right Pane`, `Both Side Panes`, `Filmstrip`, a separator, `Focus Mark`,
      `1:1 Zoom`, `Compare` (bare nouns, chosen by the user; plain items, no
      check marks). On macOS they go above the default `Enter Full Screen`,
      separated from it; on Windows / Linux a `View` submenu is created and
      inserted right after `Edit`.
    - Clicking each item does exactly what its key does (same `runAction`
      arm), under the same gates as the key: nothing while the settings /
      sequence modal or a rename is open (`modalOpen()`), and while the
      folder tree has the keyboard only the actions `treeGate` lets through
      (the four panel toggles) run.
    - Each item shows the action's first accelerator-convertible key
      (`Keymap::accelerator_for`), so it is blank by default (`f6`, `tab`,
      `f`, … are modifier-less) and follows a rebinding, updated in place on
      Windows / Linux and by rebuild on macOS as the existing items are.
    - A Rust test asserts every action in `Keymap::defaults()` is either
      in the menu's action table or in an explicit exclusion list (with the
      reasons above as comments), so a new action cannot be added without
      deciding its menu placement.
    - `docs/usage.md`, `README.md`, `README.ja.md` and
      `docs/agents/tauri-app.md` mention the View menu where the panel keys
      and the menu are documented. `mise run ci` passes.
    - Manually verified on the dev machine (Windows) by the user after the PR
      is up: each View item works by mouse; rebinding `toggleStrip` to a
      modified key (e.g. `ctrl+alt+s`) shows it on the item and one press
      toggles exactly once (no double fire from keydown + accelerator);
      `Reset` blanks it again.
  - Implementation approach:
    - PR #537 (`feature/file-menu-folder-items`, removing the folder-targeted
      File items) has merged; this branch starts after it. Re-read
      `app_menu` as it is now rather than the line numbers below.
    - `crates/app/src/main.rs`, `mod app_menu`: add a table
      `const VIEW_ITEMS: &[(&str, &str, &str)]` of `(menu id, action, label)`
      (ids like `toggle-left`, `toggle-strip`, `focus-mark`, `zoom`,
      `compare`). `build` creates one plain `MenuItem::with_id(handle, id,
      label, true, keymap.accelerator_for(action).as_deref())` per row (no
      macOS icon; `MenuItem` is imported on every platform, so no `cfg` twin
      is needed, unlike the `IconMenuItem` items). On macOS
      `submenu(&menu, "View")?` exists: `prepend_items` the items plus a
      separator. Elsewhere build `Submenu::new(handle, "View", true)?`,
      append the items, and insert it right after the `Edit` submenu found by
      scanning `menu.items()` (fall back to `append` when not found).
    - `refresh` (the non-macOS patch path): extend its `(id, action)` table
      with `VIEW_ITEMS` so `set_accelerator` covers them; a missing item is
      logged as today.
    - `on_event`: when `event.id()` matches a `VIEW_ITEMS` id, emit one
      event `menu-action` with the action name as payload (one generic
      event rather than seven named ones; the frontend already routes by
      action name). The existing named events stay as they are.
    - `crates/app/src/commands.rs`: `accelerators()` must include the seven
      actions (turn the fixed array into a list built from a shared table of
      menu-backed actions), otherwise a rebinding of a View action never
      refreshes the menu.
    - `crates/app/src/shortcuts.rs`: no keymap change.
    - Tests: a `#[cfg(test)]` coverage test (`menu_covers_every_action`):
      the set of `open`, `undo`, `redo`, `selectAll` + `VIEW_ITEMS` actions +
      a documented `MENU_LESS` const equals the actions of
      `Keymap::defaults()` with no duplicates.
    - `crates/app/ui/src/main.ts`: `listen<string>("menu-action", …)` next to
      the `undo` / `redo` listeners: return if `modalOpen()`; if
      `folders.hasFocus() && treeGate(payload) === "swallow"` return; else
      `runAction(payload)`. If a pure gate helper is extracted, give it a
      Vitest test next to `treekeys.test.ts`.
    - Docs: `docs/usage.md` Panels bullet and the Focus mark / 1:1 / Compare
      bullets gain `View > …`; the Keys table rows for `F6`–`Tab`, `f`, `z`,
      `v` note the View item like the `Edit > Undo` rows; the paragraph
      listing the rebindable menu accelerators adds the View items.
      `README.md` and `README.ja.md` (the Filmstrip bullet) mention the View
      menu, both in this PR. `docs/agents/tauri-app.md`: extend "App items go
      into the default menu's own submenus" (View is the one submenu created
      on Windows / Linux, prepended into on macOS) and the `refresh` section's
      list of patched items.
    - `todo.md`: note that the View items ship without macOS SF Symbol icons
      (the export script needs macOS).

## Trade-offs and risks

- **Plain items vs. check items.** Check items would need the backend to
  track the frontend-owned panel state and re-apply it after every macOS
  rebuild; plain items with bare-noun labels were chosen.
- **Double fire on a rebound modified key.** The existing items rely on the
  frontend keydown running the key while the menu accelerator only mirrors
  it. A toggle fired twice looks like "nothing happened". If it double-fires
  on the dev machine, pass `None` as the View items' accelerator and record
  it in `learnings.md`.
- **macOS icons.** The View items ship without SF Symbol icons; noted in
  `todo.md`.

## Wrap-up note

A separate issue found while verifying strip-keep-scroll-on-rescan: deleting
the focused file from outside the app moves focus to the first file instead
of its neighbour, because `resync` looks the vanished anchor up in the new
list (`anchorAfterFilter` in `crates/app/ui/src/filter.ts` gets index -1 and
picks the first passing file). The user wants it fixed as separate work; the
wrap-up files it in `todo.md`.

## Progress

- (2026-09-28) Step 1 complete
