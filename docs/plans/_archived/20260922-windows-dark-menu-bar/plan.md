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

# Keep the Windows menu bar dark after a shortcut change

## Purpose

On Windows in dark mode the main window's menu bar (muda's owner-drawn dark
bar) turns white after the user changes the `Open Folder` or
`Open in DxO PhotoLab` shortcut in Settings. Those two are the only
shortcuts with a menu accelerator, so they are the only ones whose change
runs `update_keymap` -> `app_menu::refresh` -> `app.set_menu`
(`crates/app/src/commands.rs:1262`, `crates/app/src/main.rs:278-286`). The
same `refresh` in `setup` (`main.rs:496`) leaves the bar dark. After this
work a shortcut change updates the two accelerators without replacing the
menu on Windows and Linux, the bar stays dark, macOS keeps its full rebuild,
and the pitfall is written down in `docs/agents/tauri-app.md`.

## Background (from investigation)

- Ruled out by the user's diagnostics on Windows: the title change
  (`setTitle` runs at launch with a folder restored and on every page, bar
  stays dark) and the folder picker (open + cancel, bar stays dark).
- Versions: muda 0.19.3, tao 0.35.3, tauri 2.11.6, tauri-runtime-wry 2.11.4.
- Threading is identical in both `refresh` calls: the shortcut commands are
  synchronous, so they run on the main thread, and `run_on_main_thread`
  executes inline there (`tauri-runtime-wry/src/lib.rs:235-255`).
- Theme is identical: tauri's `window.set_menu`
  (`tauri/src/window/mod.rs:1275-1291`) passes `window.theme()` to muda's
  `init_for_hwnd_with_theme`, so the menu is pinned to `MenuTheme::Dark`
  both times.
- Subclass removal and re-add ordering in `app.set_menu` matches what muda
  #363 fixed in 0.19.3; on paper it is correct.
- The one structural difference at runtime is the **Settings window**. It
  is built without `.menu()`, so `WindowBuilder::build`
  (`tauri/src/window/mod.rs:392-404`) attaches the app-wide menu to it, and
  `app.set_menu` then `SetMenu`s the same muda `Menu` / `HMENU` on both
  top-level windows, each with its own `menu_subclass_proc` registration. In
  `setup` only the main window exists. A shared `HMENU` between two frames is
  the most plausible way the main window's bar ends up painted by the stock
  light path. This is not proven from documentation.
- muda's Windows `set_key_accelerator` (`platform_impl/windows/mod.rs:740`)
  rewrites the item label (`text\taccel`) and the `HACCEL` store, and
  `None` removes the entry, so patching accelerators in place is sound on
  Windows. The full rebuild exists for macOS, where `set_accelerator(None)`
  does not clear a key equivalent
  (`docs/plans/_archived/20260920-menu-accelerators/learnings.md`,
  `docs/agents/tauri-app.md` "Rebuild the menu with `set_menu`").
- tauri API for the in-place path: `app.menu()` ->
  `Menu::get(OPEN_FOLDER_ID)` -> `MenuItemKind::as_menuitem()` ->
  `MenuItem::set_accelerator(Option<&str>)`.

## Steps

- [x] Step 1: Patch the two accelerators in place on Windows and Linux instead of replacing the menu
  - Done when:
    - `mise run ci` passes on Linux, and the Windows / macOS CI jobs pass
      on the PR.
    - `docs/agents/tauri-app.md` "Rebuild the menu with `set_menu`, not
      `set_accelerator(None)`" is rewritten: macOS rebuilds, Windows/Linux
      patch in place, and why (a runtime `set_menu` turns the Windows dark
      bar white; suspected cause: the app-wide menu shared with the Settings
      window).
    - Manual Windows verification is deferred: the user checks it on a
      release build after merge (change the `open` and `photolab` shortcuts,
      the bar stays dark and `File` shows the new accelerators; also whether
      the Settings window carries a menu bar and whether merely opening
      Settings turns the main bar white). Do not wait for it or ask for it
      during implementation.
  - Implementation approach (as far as it is known; omit if unknown):
    - `app_menu::refresh` in `crates/app/src/main.rs`: keep the `build` +
      `app.set_menu` path for `#[cfg(target_os = "macos")]` and for the
      first call (when `app.menu()` is `None`, i.e. `setup`). Otherwise
      (`#[cfg(not(target_os = "macos"))]` and a menu already set) fetch the
      existing menu with `app.menu()`, look up `OPEN_FOLDER_ID` and
      `PHOTOLAB_ID` with `Menu::get`, and call
      `set_accelerator(keymap.accelerator_for("open").as_deref())` /
      `..("photolab")..` on the `MenuItemKind::MenuItem` values. Keep the
      function signature so `commands.rs:1262` and `main.rs:496` are
      untouched; a missing item is an early return with a `log::warn!`, not
      a panic.
    - Do not change `build`.
    - Do not remove the menu from the Settings window in this step; that is
      a follow-up only if the post-release check shows it is still needed.
    - No new dependency; nothing is `unsafe`; no Win32 calls.
    - Commit as `fix(app): patch menu accelerators in place to keep the
      Windows menu bar dark`.

## Trade-offs and risks

- **In-place patch vs repaint / re-theme after `set_menu`.** Patching in
  place removes the trigger whatever the mechanism; the cost is a second
  code path next to the macOS rebuild.
- **Settings window still shares the menu.** If merely opening Settings
  breaks the bar, a follow-up removes the menu from the Settings window.
- **Verification is manual and deferred** to the user's release build on
  Windows.

## Progress

- 2026-09-22: Step 1 done: `refresh` patches accelerators in place on
  non-macOS; Windows check deferred to the release build.
