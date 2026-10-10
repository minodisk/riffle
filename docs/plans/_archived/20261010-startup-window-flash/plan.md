<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Create the main window hidden so the restored size and position show without a default-size flash

## Purpose

On startup (most visibly on Windows) the main window appears for a moment at
the `tauri.conf.json` default of 1280x800 at the OS-chosen position, then
jumps to the size and position `tauri-plugin-window-state` restores. The
window entry in `crates/app/tauri.conf.json` has no `visible` key, so Tauri
creates it visible; the plugin's `restore_state` then moves and resizes a
window that is already on screen.

Creating the window hidden (`"visible": false`) lets the plugin apply the saved
geometry first. The plugin (2.4.1, `StateFlags::all()` by default, which
includes `VISIBLE`) ends `restore_state` with `show()` + `set_focus()` when
there is no saved state (first launch: `should_show = true`) and when the
saved state has `visible: true`. It does not show the window when the saved
state has `visible: false` (on macOS `NSWindow.isVisible` is false while
miniaturized, so quitting with the window minimized saves that) or when an
earlier call in `restore_state` fails (`set_position` / `set_size` /
`set_fullscreen` return through `?` before `show()`, and the plugin swallows the
error with `let _ =`). Those paths need an explicit show so the window can never
stay hidden.

Tauri 2.11.6 builds the config windows inside its own `setup` before the app's
`.setup(...)` closure, and the plugin's `on_window_ready` -> `restore_state`
runs inline on the main thread during the window build
(`tauri-runtime-wry` `send_user_message` handles the message directly when
already on the main thread). So a `show()` in the app's `setup` closure runs
after the geometry is restored and cannot bring the flash back.

## Steps

- [x] Step 1: Create the main window hidden and show it after the window-state restore
  - Done when:
    - `crates/app/tauri.conf.json`: the main window entry has `"visible": false`.
    - `crates/app/src/main.rs`: the `.setup(...)` closure shows the main
      window (`app.get_webview_window("main")` -> `show()`), with a short
      comment naming why the plugin's own show is not enough (saved
      `visible: false` from a quit while minimized on macOS; a failed
      `set_position` / `set_size` / `set_fullscreen` aborting
      `restore_state` before its `show()`). `set_focus` is not needed here
      (the plugin focuses on the normal path; the OS focuses a newly shown
      window).
    - `docs/agents/tauri-app.md` gets one short **Inferred** entry under the
      Rust side (or a config section if one fits better) recording that the
      main window is created hidden, that `tauri-plugin-window-state` shows it
      after restoring, and the two paths that make the explicit `show()` in
      `setup` necessary, with the plugin version checked.
    - `mise run ci` passes.
    - Manual check on Windows, on a built binary (the dev binary from
      `pnpm tauri dev` is acceptable if `tauri build` is not practical, but
      note which was used in `learnings.md`):
      1. With a saved `.window-state.json` (app config dir) at a non-default
         size and position: launch; the window appears directly at the saved
         size/position with no 1280x800 frame visible first.
      2. First launch: delete `.window-state.json`, launch; the window
         appears (1280x800, OS-placed) and is focused.
      3. Saved `visible: false`: edit `.window-state.json` so `main.visible`
         is `false`, launch; the window still appears (this is the fallback
         `show()` path).
      4. Second instance: with the app running (also once minimized), launch
         it again; the existing window is unminimized and focused, no second
         window.
      5. Maximized state: quit maximized, relaunch; it comes back maximized
         without a normal-size frame first.
      6. Note whether a white (unpainted WebView) frame is still visible
         before the dark UI paints; see Trade-offs for what to do if so.
    - Manual check on macOS if a machine is available (otherwise record it as
      not checked in `learnings.md`): minimize the window, quit with Cmd+Q,
      relaunch; the window appears (this is the real-world source of saved
      `visible: false`).
  - Implementation approach:
    - Keep it to these two code touches; no new plugin flags, capabilities or
      frontend changes. `with_state_flags` stays at the default so `VISIBLE`
      keeps the plugin's own show on the common path.
    - Put the `show()` near the top of the `setup` closure, before the index
      open and the other `app.manage` calls, so a slow index open does not
      delay the window; ignore its `Result` with `let _ =` as the
      single-instance callback does.
    - `tauri.conf.json` is a release-shaping file; the `visible` key is read
      only at window creation and does not affect bundling or the updater,
      but run the check on a real binary rather than only `vp dev`.

## Trade-offs and risks

- **Fallback show in Rust `setup` (chosen) vs. deferring the show to the
  frontend's first paint.** The frontend route (plugin flags without
  `VISIBLE`, `core:window:allow-show` in `crates/app/capabilities/default.json`,
  a `show()` on the `getCurrentWindow()` typing in `crates/app/ui/src/tauri.d.ts`,
  a call after first paint in `main.ts`, and still a Rust fallback in case the
  frontend fails to load) would also hide any white WebView frame, but it is
  far more code for a cosmetic gain not yet observed. Chosen: the Rust
  fallback only.
- **If the Windows check still shows a white frame**, the minimal follow-up is
  `"backgroundColor": "#0a0a0a"` (the UI's `--background` in
  `crates/app/ui/style.css`) on the window entry in `tauri.conf.json`, which
  Tauri 2 supports as `WindowConfig.background_color`. Do it in the same step
  only if the user confirms the flash bothers them; otherwise leave it noted
  in `learnings.md`.
- **A hidden window that never shows is the failure mode to guard against.**
  The explicit `show()` runs unconditionally in `setup`, which runs on every
  launch after the window exists, so there is no launch path that reaches
  the event loop with the window hidden unless `setup` itself errors (which
  already aborts the app).
- **First-launch cache quirk.** With the window created hidden, the plugin
  records `visible: false` in its in-memory cache on first launch; a normal
  exit overwrites it with the live `is_visible()` (true) before writing the
  file. A crash before exit writes nothing. Either way the fallback `show()`
  covers a stale `false`.

## Progress

- (2026-10-10) Step 1 complete
