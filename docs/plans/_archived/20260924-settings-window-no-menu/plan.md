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

# Remove the inherited menu bar from the settings window

## Purpose

On Windows and Linux the Settings window shows the app's menu bar (File /
Edit / Help ...). Tauri 2.11.6 gives a window built without `.menu(...)` the
app-wide menu (`tauri-2.11.6/src/window/mod.rs:393-398`,
`.or_else(|| self.manager.app_handle().menu())`), and `app_menu::refresh`
sets that menu with `app.set_menu` during `setup` on non-macOS, so the settings
window built in `app_menu::open_settings` (`crates/app/src/main.rs:362-374`)
inherits it. The settings window must never show a menu bar; the main window
keeps its menu and macOS (app-wide menu) is unchanged.

## Steps

- [x] Step 1: Remove the menu from the settings window right after building it
  - Done when:
    - `app_menu::open_settings` in `crates/app/src/main.rs` keeps the
      `WebviewWindowBuilder::build()?` result and, under
      `#[cfg(not(target_os = "macos"))]`, calls `window.remove_menu()?` on it
      before returning. No `.visible(false)` / `show()` dance and no
      window-state plugin change.
    - A short comment above the call explains the platform quirk in the style
      of the surrounding `app_menu` comments: on Windows/Linux a window built
      without `.menu()` inherits the app-wide menu; `remove_menu` runs inline
      here because menu events arrive on the main thread; on macOS the menu is
      app-wide and `Window::remove_menu` is unsupported, hence the cfg.
    - `docs/agents/tauri-app.md`: add one or two sentences to the existing
      settings-window paragraph in "App items go into the default menu's own
      submenus" (~line 271-280), or a small new Hit item, saying that windows
      built without `.menu()` inherit the app menu on Windows/Linux and the
      settings window removes it after `build()`; note that `AppHandle::set_menu`
      would re-attach it to menu-less windows, which is why the non-macOS
      `refresh` must keep patching items in place instead of calling
      `set_menu` again.
    - `mise run ci` passes on Linux (`cargo build` for `crates/app` compiles the
      non-macOS branch here). The Windows check (open Settings from the menu,
      no menu bar; main window still has its menu) is a manual step for the
      user and is written in the PR description as such.
  - Implementation approach:
    - Verified in the sources: `Window::remove_menu`
      (`tauri-2.11.6/src/window/mod.rs:1321-1353`) dispatches via
      `run_on_main_thread`, and tauri-runtime-wry 2.11.4's `send_user_message`
      (`src/lib.rs:235-255`) runs the closure inline when the caller is on the
      main thread. `open_settings` is reached from `on_menu_event`, so the
      native menu is removed in the same event-loop turn as window creation,
      before a paint; a visible flash is not expected.
    - Verified: `remove_menu_from_stash_by_id`
      (`tauri-2.11.6/src/manager/mod.rs:701-712`) keeps the menu in the stash
      because the app-wide menu is still in use, so the main window's menu and
      the later in-place `set_accelerator` patching in `refresh` are unaffected.
    - Do not use `.visible(false)` + `show()`: `tauri-plugin-window-state`
      (`2.4.1/src/lib.rs:407-430, 262-265`) calls `show()` and `set_focus()`
      in `on_window_ready` for every window not on its denylist /
      `skip_initial_state`, i.e. inside `build()`, so the window would be shown
      before `remove_menu` regardless. Making it work would need
      `skip_initial_state("settings")`, which also drops the settings window's
      position/size restore. Only revisit if the user reports a flash on
      Windows.
    - Commit as `fix(app): remove the inherited menu bar from the settings
      window` (Conventional Commits, English).

## Trade-offs and risks

- `remove_menu` vs `hide_menu`: `hide_menu` (`window/mod.rs:1359`) keeps the
  menu attached and merely hides it; `AppHandle::set_menu` would still treat
  the window as having the app-wide menu. `remove_menu` is chosen because the
  window should have no menu at all, and `Window::menu()` becomes `None`,
  which is the honest state. Downside: any future `AppHandle::set_menu` on
  non-macOS re-attaches the app menu to windows whose `menu().is_none()`
  (`app.rs:970-980`). Today `refresh` only calls `set_menu` once, before the
  settings window can exist, so this is documented rather than guarded.
- Passing an empty `Menu::new(app)?` to `.menu(...)` was considered and
  rejected: on Windows/GTK an empty menu still renders an empty menu bar strip.
- If the user still sees a flash on Windows after this change, the fallback is
  `.visible(false)` plus `tauri_plugin_window_state::Builder::skip_initial_state("settings")`
  and an explicit `show()` after `remove_menu`; that costs the settings
  window's saved position/size and is a separate decision.

## Progress

- (2026-09-24) Step 1 complete
