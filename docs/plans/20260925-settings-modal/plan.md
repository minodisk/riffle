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

# Settings as an in-app modal

## Purpose

Riffle opens its settings in a separate OS window (`crates/app/ui/settings.html`,
built by `open_settings` in `crates/app/src/main.rs`). Riffle is a
cross-platform app with its own UI, so a second native window looks foreign
next to it (VS Code, Lightroom (cloud) and Plasticity all keep settings inside
the main window), and every setting has to be mirrored across two JS contexts
through backend events (`shortcuts-changed`, `sidecar-format`, `label-names`,
`auto-advance`, `debug`, `scan-state`, `index-clearing`) plus the
window-lifetime plumbing (close with `main`, `remove_menu` on Windows/Linux,
the `settings` entry in the capability, the second Vite entry).

After this work, `Settings...` / `CmdOrCtrl+,` opens a modal overlay in the
main window, Escape or a close control dismisses it, the culling keymap is
blocked while it is open, and settings changes reach the main view directly
in the same JS context. The separate window, and everything that only served
it, is gone. Persisted settings keys and formats (`sidecarFormat`,
`labelNames`, `shortcuts`, `autoAdvance`) do not change.

## Steps

- [x] Step 1: Replace the settings window with a modal in the main window
  - Done when:
    - `Settings...` in the menu and `CmdOrCtrl+,` open a modal overlay inside
      the main window; no OS window is created. Opening it again while open
      does nothing (no second overlay).
    - The modal has the same tabs and controls as `settings.html` had
      (Sidecar with the label names block, Culling, Keyboard Shortcuts,
      Cache, dev-only Debug) and a visible close control; every setting
      persists under the same store keys as before and takes effect in the
      main view without restarting.
    - Escape closes the modal, except while a shortcut row is capturing a key,
      where Escape cancels the capture (as today) and a second Escape closes
      the modal. Adding a key still works: the captured keydown never reaches
      the culling keymap.
    - No culling shortcut fires while the modal is open, Tab / Shift+Tab stay
      inside the modal, and closing it returns focus to where it was.
    - The filter, sort and right-click menus are closed when the modal opens.
    - `settings.html`, `settings.css`, the `settings` Vite entry, the
      `settings` window label in `capabilities/default.json`, `open_settings`,
      `SETTINGS_WINDOW`, the `WindowEvent::Destroyed` close-along handler and
      the `WebviewWindowBuilder` / `WebviewUrl` imports are removed.
    - Frontend tests cover open / close / key-blocking (see approach) and
      `mise run ci` passes.
    - `CLAUDE.md`, `docs/usage.md` and `docs/agents/tauri-app.md` no longer
      describe a settings window (see the doc list below).
  - Implementation approach:
    - **Rust entry point**: keep the `Settings...` item and its accelerator in
      `app_menu` (`crates/app/src/main.rs`). Replace the `open_settings` call
      in the menu handler with `app.emit("open-settings", ())`, the same
      pattern as `open-folder` / `reload-folder` / `undo` / `redo` there. The
      accelerator is fixed, not part of the keymap, so nothing in
      `shortcuts.rs` changes.
    - **Rust removals**: `open_settings`, `SETTINGS_WINDOW`, the `Destroyed`
      arm in the `run` closure, and the now-unused imports. In
      `commands::clear_index`, parent the confirmation dialog on
      `get_webview_window("main")` instead of `"settings"`.
    - **Capability**: `crates/app/capabilities/default.json` -> `"windows":
      ["main"]`, and reword the description.
    - **Vite**: drop `build.rollupOptions.input` in `vite.config.ts` (the
      default entry is `root/index.html`) or leave only `main`; delete
      `crates/app/ui/settings.html` and `crates/app/ui/settings.css`.
    - **Markup**: move the body of `settings.html` into `index.html` as
      `<div id="settings-dialog" role="dialog" aria-modal="true"
      aria-labelledby="..." hidden>` with an inner box, a title and a close
      button, next to `#format-dialog`. Merge `settings.css` into
      `style.css` scoped under `#settings-dialog` (keep the
      `#label-names:not([hidden])` scoping and the `[hidden]` rule; see the
      "Scope an id's `display` override" note in `docs/agents/tauri-app.md`).
      Reuse the overlay / box styling of `#format-dialog`; the modal can be
      larger than the old 480x640 window, but keep the shortcuts table
      scrollable inside the box.
    - **Frontend structure**: `crates/app/ui/src/settings.ts` currently runs
      at import time against `document`. Turn it into a module `main.ts`
      calls (e.g. an exported `initSettings(...)` / `SettingsModal` that does
      the DOM lookups when invoked) so it can live in the main bundle. Put the
      logic that has to be tested in a pure, DOM-free module (tests run with
      `environment: "node"`, like `firstrun.ts` / `tabs.ts` / `keys.ts`):
      the open/closed state, the "which of these does a keydown do while
      open" decision (capturing: add key or cancel on Escape; Escape: close;
      Tab: cycle focus; anything else: swallowed, never a culling action),
      and "open while already open is a no-op". Name and shape are the
      implementer's choice; the constraint is that `main.ts`'s keydown
      handler consults it and that `*.test.ts` can exercise it without a DOM.
    - **Keydown routing in `main.ts`**: extend the existing early-return
      block for `formatDialog` (~line 2265): while the settings modal is
      open, hand the event to the modal (key capture, Escape, Tab trap over
      the modal's focusable elements, computed at open time or per keydown)
      and `return` before `keymap.get(key)`. The old `settings.ts`
      `window.addEventListener("keydown")` for capture and the tablist
      arrow-key handler fold into this path. Keep the tablist keyboard
      behavior (`nextTab`).
    - **Open / close**: listen for `open-settings` in `main.ts`; ignore it
      while `#format-dialog` is open (the first-run dialog is already modal).
      On open: close `filterMenu`, `sortMenu` and the context menu, remember
      `document.activeElement`, un-hide, focus the selected tab. On close:
      hide, cancel any capture, restore focus. Wire the close button and a
      click on the backdrop (outside the box) to close as well, if kept
      simple.
    - **Settings data flow in the same context**: the existing commands keep
      working unchanged from the main window (`shortcuts`,
      `add_shortcut_key`, ..., `sidecar_format`, `set_sidecar_format`,
      `label_names`, `set_label_names`, `auto_advance`, `set_auto_advance`,
      `debug_build`, `timing_logs`, `set_timing_logs`, `index_size`,
      `clear_index`, `scan_running`). In this step the modal may keep using
      the backend events it used before (they are delivered to the main
      window too); the cleanup is Step 2. Load the modal's values on open
      (not at startup) or at startup, either is fine, but a value changed by
      the backend while the modal is closed (e.g. `choose_sidecar_format`
      from the first-run dialog) must show correctly the next time it opens.
    - **Focus listener**: `main.ts` listens for `tauri://focus` on the
      current window to `resync()`. With the modal, the Clear Cache native
      dialog closing refocuses the main window and triggers a rescan; verify
      that `clear_index` (which refuses while `ScansState::scanning()`) still
      behaves: the dialog is answered before the focus returns, so the
      refusal path should not trigger, but check it by hand and note the
      result in `learnings.md`.
    - **Docs**: `CLAUDE.md` ("chosen in the settings window" ->
      the settings modal); `docs/usage.md` (Escape also closes the
      settings; "Settings window's tab strip" -> the settings tab
      strip); `docs/agents/tauri-app.md`: the "App items go into the default
      menu's own submenus" paragraphs that describe the separate window,
      `remove_menu` and the capability entry, the "Rebuild the
      menu on macOS" bullet whose Windows suspicion names the Settings
      window (keep the rule, drop or reword the suspicion), the
      "global `event.listen("tauri://focus")`" item (the rule
      stays, the example no longer applies), the Clear Cache reference
      and the "Vite+ config facts" bullet about `settings.html`.
      Do not touch `README.md` / `README.ja.md`: they only say
      "Settings".

- [ ] Step 2: Drop the cross-window sync the modal no longer needs
  - Done when:
    - Backend events that only existed to reach the other window are gone:
      `label-names`, `auto-advance`, `debug`, `shortcuts-changed`,
      `index-clearing`, and the `scan_running` command, together with their
      `emit` sites in `crates/app/src/commands.rs` / `main.rs` and their
      `listen` sites in the frontend. `sidecar-format`, `scan-state` and
      `index-cleared` stay (they are backend-driven state changes the main
      view reacts to, not window mirroring).
    - The modal applies results directly: the `Binding[]` returned by the
      shortcut commands goes to `applyKeymap`, the auto-advance checkbox and
      the timing-logs checkbox set `main.ts`'s `autoAdvance` /
      `debugLogging`, the label names come from `set_label_names`'s return
      value, and the Cache tab reads `main.ts`'s `scanRunning` instead of
      `scan_running` + its own `scan-state` listener.
    - `mise run ci` passes; `docs/agents/tauri-app.md` no longer lists the
      removed events.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - `TimingLogs` (`main.rs`) and `timing_logs` / `set_timing_logs` can stay
      as the Rust-held state that survives a webview reload (the comment says
      it exists so a reloaded main window stays in sync); only the `debug`
      emit goes. If the implementer finds nothing else reads it, removing the
      state and keeping `debugLogging` frontend-only is acceptable too;
      record the choice in `learnings.md`.
    - `set_label_names` still emits `sidecar-format` after the reset so the
      folder reopens; keep that.

## Trade-offs and risks

- **Two PRs**: the user chose to keep the two steps apart so Step 1's
  behavior change is reviewable on its own.
- **Where the tested logic lives**: tests run without a DOM, so the plan
  requires a DOM-free module for the modal's state and key routing. The
  alternative (adding `jsdom` / `happy-dom` to test `settings.ts` directly)
  changes the test environment for the whole suite or needs per-file
  environment pragmas; not recommended for this change.
- **Escape semantics while capturing**: the plan keeps today's behavior
  (Escape cancels the capture first, a second Escape closes the modal).
- **Clear Cache confirmation dialog**: it stays a native `tauri-plugin-dialog`
  message box, now parented on `main`. Making it an in-app confirm as well
  would be consistent but is out of scope; note if the native dialog looks
  odd over the modal.
- **`tauri://focus` rescan**: the old separate-window bug (Clear Cache
  dialog refocus colliding with the rescan) was solved by scoping the
  listener. With the dialog parented on `main`, closing it refocuses `main`
  and may trigger `resync()`; `clear_index` has already passed its guard by
  then. Verify by hand during Step 1.
- **Modal size**: the old window was 480x640; the shortcuts table is long.
  The modal needs internal scrolling (max-height relative to the viewport),
  otherwise a small main window hides the Reset all button.
- **In-progress plan `docs/plans/20260924-mcp-companion/plan.md`** says the
  MCP toggle is "turned on in the settings window"; after this lands it
  should add its tab/toggle to the modal instead. The two plans touch
  `settings.ts` / `index.html`, so expect a rebase.
- **Windows dark menu bar note** in `docs/agents/tauri-app.md` blamed the
  menu-less Settings window for `set_menu` turning the bar white. That
  suspicion becomes moot, but the rule (do not call `set_menu` at runtime on
  Windows) should stay until someone verifies on a device; reword, do not
  delete.

## Progress

- Step 1: Replaced the settings window with a modal in the main window
  (`0352c6a`). Fixed a review finding that the `tauri://focus` resync raced
  `clear_index` for the Scans lock while the settings modal was open; the
  listener now skips `resync()` while `settings.isOpen`.
- (2026-09-25) Step 1 complete
