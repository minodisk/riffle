# Learnings: startup-window-flash

## Step 1

- Checked `tauri-plugin-window-state` 2.4.1 `restore_state` in the cargo
  registry: `should_show` starts `true`, becomes the saved `state.visible`, and
  `show()` + `set_focus()` run only if it is still `true` and nothing before it
  returned through `?` (`set_decorations`, `set_position`, `set_size`,
  `maximize`, `set_fullscreen`). That matches the plan, so the `setup` show
  stays unconditional.
- The config window has no `label`, so it is `"main"`, the same label the
  single-instance callback already looks up.
- The manual checks were not run by the implementation agent (no GUI session
  here); the checkbox was ticked on the automated criteria (`mise run ci`).

## Deferred issues (todo candidates)

- **Pending manual check (Windows), startup window flash.** Basis: plan Step 1
  Done-when; files `crates/app/tauri.conf.json`, `crates/app/src/main.rs`.
  On a built binary (`pnpm tauri build`; the `pnpm tauri dev` binary is
  acceptable, note which was used):
  1. With a saved `.window-state.json` (app config dir) at a non-default size
     and position, launch: the window appears directly there, no 1280x800
     frame first.
  2. Delete `.window-state.json`, launch: the window appears at 1280x800,
     OS-placed, focused.
  3. Edit `.window-state.json` so `main.visible` is `false`, launch: the
     window still appears (the fallback `show()` path).
  4. With the app running (once normal, once minimized), launch it again: the
     existing window is unminimized and focused, no second window.
  5. Quit maximized, relaunch: it comes back maximized with no normal-size
     frame first.
  6. Note whether a white (unpainted WebView) frame shows before the dark UI;
     if it does and bothers the user, the follow-up is
     `"backgroundColor": "#0a0a0a"` on the window entry (plan Trade-offs).
  The step's checkbox was ticked on the automated criteria only.
- **Pending manual check (macOS), saved `visible: false`.** Basis: plan Step 1
  Done-when; files as above. On a Mac: minimize the window, quit with Cmd+Q,
  relaunch; the window appears. Not checked (no macOS machine here). The
  step's checkbox was ticked on the automated criteria only.
