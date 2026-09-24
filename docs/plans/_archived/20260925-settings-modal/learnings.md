# Learnings: settings-modal

## Step 1: Replace the settings window with a modal in the main window

- The plan's tab list predates the MCP tab: `settings.html` on `main` already
  had an `MCP` tab (from #402). It moved into the modal along with the others;
  the plan's "same tabs and controls as `settings.html` had" covers it.
- Two ids from `settings.html` were too generic for the shared document and
  were renamed on the way in: `tabs` -> `settings-tabs`, `status` ->
  `settings-status`. The panel ids (`sidecar`, `culling`, `shortcuts`,
  `cache`, `mcp`, `debug`) did not collide with anything in `index.html`, so
  they and every `aria-controls` stay as they were.
- `settings.css` merged into `style.css` under `#settings-dialog`. Its bare
  `[hidden] { display: none; }` became `#settings-dialog [hidden]`, placed
  last so it wins over the scoped rules of equal specificity;
  `#label-names:not([hidden])` keeps its scoping. The box is a flex column
  with a fixed `min(640px, 85vh)` height; the header and tab strip stay put
  and only the tab panel scrolls, so Reset all is reachable in a small
  window.
- The DOM-free part is `crates/app/ui/src/modal.ts`: `SettingsModal` (open
  state, `open()` returning false when already open, the capturing row, and
  `key(name)` deciding add / cancel / close / focus / native from
  `keyName`'s output) and `cycleFocus` for the Tab trap. Deciding on the
  `keyName` string (`"tab"`, `"shift+tab"`, `"escape"`) rather than on the
  raw DOM event keeps it testable with plain strings.
- "native" means "do not `preventDefault`, but still return before the
  keymap", so Enter/Space on buttons, typing in the label-name inputs, and
  arrow keys on radios keep working. The tablist's arrow / Home / End handling
  runs on that path when the event target is inside the tablist.
- The Tab trap skips unchecked radios, so a radio group takes one Tab stop as
  it does natively, and skips hidden elements via `getClientRects().length`
  (panels behind other tabs, the hidden Debug tab).
- Menu accelerators bypass the keydown path: on macOS a key equivalent is
  handled by the menu before the webview sees the key, so the keymap-driven
  accelerators (`Open Folder…`, `Undo`, `Redo`) would still run culling
  actions behind the modal. `main.ts` now ignores the `open-folder`, `undo`
  and `redo` events while the modal is open. `Reload Folder` and
  `Move Rejected to Trash` are not keymap actions and are left alone.
- Values: the listeners (`sidecar-format`, `label-names`, `auto-advance`,
  `mcp-state`, `scan-state`, `index-clearing`) are registered once in
  `initSettings`, so they keep the controls current while the modal is
  closed. `open()` additionally re-reads `sidecar_format` and `index_size`,
  as a freshly opened window used to.
- Clear Cache / `tauri://focus` check: not verified by hand. This
  environment (WSL2, no display for the Tauri app) cannot run the app, so
  the step could not click Clear Cache. By the code path: `clear_index` checks
  `ScansState::scanning()` before showing the dialog and re-checks it under
  the `Scans` lock after the answer, while the `resync()` that the refocus
  triggers is a separate `scan_folder` call. If that rescan wins the lock
  race, `clear_index` returns `a scan is running` (the same refusal as
  before the listener was scoped). Needs a manual check on macOS / Windows.

## Step 2: Drop the cross-window sync the modal no longer needs

- Removed `label-names`, `auto-advance`, `debug`, `shortcuts-changed`,
  `index-clearing` and the `scan_running` command. The MCP companion's
  `mcp-state` / `mcp-request` stay: `mcp-state` is backend-driven (the server
  binding, failing, or being toggled), not mirroring for the old window.
- `initSettings` takes `SettingsHooks` (`applyKeymap`, `setAutoAdvance`,
  `setDebugLogging`) and returns `setScanRunning`; `main.ts` routes every
  change of its `scanRunning` through a `setScanRunning` helper that also
  tells the modal, which keeps the Clear Cache button and the post-scan size
  refresh the `scan-state` listener used to drive. `main.ts`'s `scanRunning`
  also covers the `scan_folder` prepare phase, so it is at least as wide as
  the backend's `ScansState::scanning()` for scans this window starts; a
  mismatch only makes `clear_index` refuse with its own error.
- `TimingLogs` and `timing_logs` / `set_timing_logs` stay as the Rust-held
  state that survives a webview reload (`main.ts` still reads it at startup);
  only the `debug` emit went. `set_timing_logs` now takes the state directly.
- Without `index-clearing` the modal cannot tell when the native confirmation
  was accepted, so the "Clearing the index cache…" text is gone; the Clear
  Cache button stays disabled while the clear runs and the size updates when
  it finishes. Showing the text at click time would show it while the
  confirmation is still up, so that was not done.
- The auto-advance checkbox applies to `main.ts` after `set_auto_advance`
  resolves; the timing-logs checkbox applies immediately, as the command
  cannot fail.

## Deferred issues (todo candidates)

- Manually verify Clear Cache from the settings modal on a real device: the
  native confirmation is now parented on `main`, and closing it refocuses
  `main`, whose `tauri://focus` listener runs `resync()`. Check that
  `clear_index` does not refuse with `a scan is running` and that the
  confirmation does not look odd over the modal. Basis: the plan's
  "Focus listener" approach item, which this step could not exercise
  (no display in the implementation environment). Files:
  `crates/app/src/commands.rs` (`clear_index`),
  `crates/app/ui/src/main.ts` (`tauri://focus` listener).
- `scan-state` has no frontend listener left after Step 2 (the settings
  modal was its only one; `main.ts` tracks scans through `scan-done`). The
  plan kept it as a backend-driven state change, so its emits (and the
  "emit under the `Scans` lock" rule in `docs/agents/tauri-app.md`) stay;
  decide whether to remove it or give it a consumer. Basis: Step 2
  implementation. Files: `crates/app/src/commands.rs` (`scan-state` emits),
  `docs/agents/tauri-app.md`.
