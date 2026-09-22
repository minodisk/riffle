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

# About dialog app icon

## Purpose

Tauri's `Menu::default` builds the predefined About item with an
`AboutMetadata` that carries the name, version, copyright and publisher but
no `icon`, so the About panel on macOS and the GTK About dialog on Linux show
no app icon (on Linux, GTK only sets the logo when one is passed). Replacing
that one item with the same metadata plus the app icon makes About show the
icon on the two platforms whose About dialog supports it. Windows renders
About as a MessageBox and ignores `icon`, so it is left as is.

## Steps

- [x] Step 1: Replace the default About item with one that carries the app icon
  - Done when:
    - `app_menu::build` in `crates/app/src/main.rs` replaces the predefined
      About item of `Menu::default` with
      `PredefinedMenuItem::about(handle, None, Some(AboutMetadata { .. }))`
      at the same position (item 0 of the app submenu on macOS, item 0 of
      `Help` elsewhere), with the same `name` / `version` / `copyright` /
      `authors` as `Menu::default` and `icon` set to the app icon.
    - Every other menu item and its order is unchanged on all three platforms;
      Windows behaviour is unchanged (About is rebuilt there too, with an icon
      Windows ignores).
    - No custom About window is added.
    - `mise run ci` passes.
    - Manual check described in the PR (the implementer cannot run the GUI):
      on macOS, `mise run tauri:dev` and a bundled build both show the icon in
      Riffle > About Riffle, with name and version still present; on Linux,
      Help > About opens the GTK dialog with the icon as its logo. On Windows,
      Help > About still shows the same MessageBox.
  - Implementation approach:
    - Do the replacement right after `let menu = Menu::default(handle)?;`,
      before the app's own items are inserted, so the existing index-based
      inserts (`app.insert_items(.., 1)`, `.., 3)` on macOS; the Help
      `prepend` on the others) keep working unchanged.
    - Build the metadata like `Menu::default` does
      (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tauri-2.11.6/src/menu/menu.rs:142-`):
      `name: Some(handle.package_info().name.clone())`,
      `version: Some(handle.package_info().version.to_string())`,
      `copyright: handle.config().bundle.copyright.clone()`,
      `authors: handle.config().bundle.publisher.clone().map(|p| vec![p])`,
      `icon: Some(..)`, `..Default::default()`. Import `tauri::menu::AboutMetadata`
      (and `tauri::image::Image` on non-macOS too; currently it is
      macOS-only in `app_menu`).
    - Icon source: `Image::from_bytes(include_bytes!("../icons/128x128.png"))?`,
      matching the existing `include_bytes!("../icons/menu/*.png")` items in
      the same function; the `image-png` feature is already on in
      `crates/app/Cargo.toml`. Do not use `handle.default_window_icon()`: it is
      always `Some` but on Unix it is the first `.png` listed in `bundle.icon`,
      i.e. `icons/32x32.png`, which is too small for the ~64 pt macOS About
      panel and for GTK, which draws the logo at native size.
    - Locate and replace About with the same guard the Edit > Undo/Redo
      replacement uses: only `remove_at(0)` + `insert(&about, 0)` when item 0
      is `MenuItemKind::Predefined(_)`. macOS: the first submenu of
      `menu.items()?`; non-macOS: `submenu(&menu, "Help")?`. Use two branches
      (`macos` / `not(macos)`), not a Linux-only cfg.
    - Add a short comment in the style of the neighbours explaining why the
      default About is swapped (default metadata carries no icon; Windows
      ignores it).
    - Optional: add one clause to the "App items go into the default menu's own
      submenus" section of `docs/agents/tauri-app.md` noting About is rebuilt
      with the icon.

## Trade-offs and risks

- **Windows**: decided to run the replacement on every non-macOS platform;
  muda never reads `icon` on Windows, so behaviour there is identical and the
  code needs one fewer cfg branch.
- **Icon size.** 128x128.png is the middle ground. If the manual check on
  macOS shows 128 px blurry, switch to a per-platform choice (256 px on macOS,
  128 px on Linux).
- **Dev vs bundled on macOS.** A bundled `.app` may already pick up the
  `.icns` without `icon`; `tauri:dev` does not. Passing it explicitly makes
  both consistent.
- Manual GUI verification cannot be done by the implementer; CI only proves
  it compiles, formats and passes clippy/tests on all three OSes.

## Progress

- (none yet)
