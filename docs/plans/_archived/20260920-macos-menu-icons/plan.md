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

# macOS menu bar icons for Riffle's own menu items

## Purpose

Recent macOS apps (System Settings, Ghostty, ...) show an icon next to most
menu bar items. Riffle adds four items of its own to the default menu
(`Open in DxO PhotoLab`, `Settings...`, `Check for Updates…`, `Undo`) and they
appear without one, which looks unfinished beside the system's items. This
work gives those four items an icon on macOS, using AppKit's standard named
images where one fits and a bundled SF Symbol rendering otherwise. Other
platforms keep the plain `MenuItem`.

Decisions already taken by the user (do not revisit):

- `tauri::menu::IconMenuItem` + `tauri::menu::NativeIcon` first.
- Items with no suitable `NativeIcon` get an SF Symbol exported ahead of time
  as a PNG, checked into the repository, and passed as `tauri::image::Image`.
- No runtime `objc` calls (no `NSImage(systemSymbolName:)` at run time).
- Icons are macOS only; the other platforms keep `MenuItem` behind `cfg`.
- `Open in DxO PhotoLab` uses `NativeIcon::FollowLinkFreestanding`.
- The bundled PNGs are rendered in a neutral gray (`#8E8E93`, roughly
  `NSColor.systemGray`) so they stay legible in both light and dark
  appearance, accepting that they do not tint like the native icons beside
  them.

Investigation results (Tauri 2.11.5, muda 0.19.3; see
`~/.cargo/registry/src/*/tauri-2.11.5/src/menu/icon.rs` and
`~/.cargo/registry/src/*/muda-0.19.3/src/platform_impl/macos/mod.rs`):

- `IconMenuItem::with_id_and_native_icon(manager, id, text, enabled, Option<NativeIcon>, Option<accel>)`
  and `IconMenuItem::with_id(manager, id, text, enabled, Option<Image<'_>>, Option<accel>)`
  both exist and return `tauri::Result<IconMenuItem<R>>`. An `IconMenuItem`
  implements `IsMenuItem`, so `prepend_items` / `insert_items` / `insert`
  accept it exactly like the current `MenuItem`.
- `NativeIcon` has no undo and no modern gear-style template image.
- Verified through AppKit on this machine: `Refresh`,
  `FollowLinkFreestanding`, `Share`, `GoLeft` are template images (they tint
  with the menu, so they work in dark mode). `PreferencesGeneral`, `Info`,
  `Advanced` are legacy 32px color bitmaps (`isTemplate == false`), which
  look out of place next to the system's monochrome symbols.
- muda sets a native icon's size to 18x18 pt; a custom `Image` is converted
  to PNG and given an `NSImage` with height 18 pt (`to_nsimage(Some(18.))`).
  muda never calls `setTemplate`, and Tauri has no template flag for menu
  items (only for the tray icon). A bundled PNG is therefore drawn as-is and
  is **not** tinted for dark mode.
- `tauri::image::Image::from_bytes` (PNG decoding) requires the `image-png`
  Tauri cargo feature, which is not in Tauri's defaults; `Image::new` (raw
  RGBA) does not. The `png` crate is already in the dependency tree through
  muda, so enabling `image-png` adds no new crate.
- SF Symbols exist for the two items without a native icon: `gearshape`
  (Settings) and `arrow.uturn.backward` (Undo). `swift` is available
  (`/Library/Developer/CommandLineTools/usr/bin/swift`); `rsvg-convert` and
  ImageMagick are not, so the reproducible export path is a Swift script.

## Steps

- [x] Step 1: Use `IconMenuItem` with a `NativeIcon` for the items that have a template AppKit image on macOS
  - Done when:
    - On macOS, `Check for Updates…` shows `NativeIcon::Refresh` and
      `Open in DxO PhotoLab` shows `NativeIcon::FollowLinkFreestanding` next
      to the label, in both light and dark appearance.
    - `Settings...` and `Undo` are unchanged in this step (no legacy color
      bitmap is used for them).
    - Non-macOS builds still compile with the existing `MenuItem` (verify
      with `cargo check -p riffle-app --target x86_64-unknown-linux-gnu` if
      the target is installed, otherwise reason from the `cfg` structure and
      note it in `learnings.md`).
    - `mise run ci` passes.
  - Implementation approach:
    - Only `crates/app/src/main.rs`, module `app_menu`, changes.
    - Keep the existing structure: the four `with_id` calls at the top of
      `build`, then the `File` / app-menu / `Edit` placement code. Only the
      construction of the item changes; placement code is untouched because
      `IconMenuItem` implements `IsMenuItem` like `MenuItem`.
    - Use `#[cfg(target_os = "macos")]` / `#[cfg(not(target_os = "macos"))]`
      on the two `let` bindings (same pattern as the existing Settings
      placement block). Two items change in this step, two more in Step 2;
      the implementer decides whether a tiny macOS-only helper
      (`fn item(handle, id, text, accel, icon: NativeIcon)`) is worth it once
      all four are known. Do not add a helper for other platforms.
    - Import `IconMenuItem` and `NativeIcon` under `cfg(target_os = "macos")`
      so clippy's `-D warnings` (unused import) stays clean on other
      platforms.
    - `IconMenuItem::with_id_and_native_icon` is documented "Windows / Linux:
      Unsupported" (silently no icon), which is why the non-macOS branch keeps
      `MenuItem` rather than relying on that.
    - Take a screenshot of both menus in light and dark mode for the PR.
- [x] Step 2: Bundle SF Symbol renderings for `Settings...` and `Undo`, with a reproducible export script
  - Done when:
    - `Settings...` shows `gearshape` and `Undo` shows
      `arrow.uturn.backward` on macOS, sized to match the Step 1 native icons
      (18 pt), legible in light and dark mode.
    - The PNGs are committed under `crates/app/icons/menu/` (e.g.
      `gearshape.png`, `arrow.uturn.backward.png`), rendered at 2x
      (36x36 px) so they are crisp on Retina; muda's `setSize(18)` handles
      the point size.
    - A Swift script (`tools/macos/export-menu-icons.swift`, run with
      `swift tools/macos/export-menu-icons.swift`) regenerates exactly those
      files from the symbol names, point size, weight and color it hardcodes.
      Running it on a clean checkout produces no git diff (byte-identical
      output on the same macOS version; if AppKit output differs across OS
      versions, record the version used in the script header).
    - The procedure (which script, when to rerun it, the dark-mode
      limitation) is written down: script header comment plus a short
      "Menu icons" entry in `docs/agents/tauri-app.md` (tag: Inferred/Hit as
      appropriate). `lychee` in `mise run lint` checks Markdown links, so
      any path referenced must exist.
    - `mise run ci` passes; non-macOS builds compile (the `include_bytes!`
      and `Image::from_bytes` code is under `cfg(target_os = "macos")`).
  - Implementation approach:
    - Assumes Step 1 is merged (same `cfg` structure and, if introduced, the
      helper).
    - `crates/app/Cargo.toml`: add `"image-png"` to the `tauri` features.
    - `crates/app/src/main.rs`: `IconMenuItem::with_id(handle, SETTINGS_ID,
      "Settings...", true, Some(Image::from_bytes(include_bytes!("../icons/menu/gearshape.png"))?), Some("CmdOrCtrl+,"))`
      and the same for Undo. `Image::from_bytes` returns `tauri::Result`, so
      `?` fits `build`'s signature. `include_bytes!` is relative to the
      source file (`crates/app/src/`), hence `../icons/menu/...`.
    - The export script uses `NSImage(systemSymbolName:accessibilityDescription:)`
      with `NSImage.SymbolConfiguration(pointSize:weight:)` (medium weight
      roughly matches the system menu symbols), draws into an
      `NSBitmapImageRep` at 2x scale in `#8E8E93`, and writes PNG via
      `representation(using: .png, properties: [:])`. The script must run
      only by a developer, never at build or run time (the user's constraint
      is about runtime objc; a developer-run export is the agreed fallback).
      The script header states the macOS version it was last run on.
    - Do not put the script under `.claude/skills` (the shellcheck target
      list in `mise run lint` only covers `*.sh`; a `.swift` file elsewhere is
      not linted, which is fine).

## Trade-offs and risks

- **Dark mode for the bundled PNGs**: muda never marks a custom menu image as
  a template and Tauri exposes no way to do so, so a black-on-transparent PNG
  is invisible on a dark menu and a white one on a light menu. The user chose
  a neutral gray (`#8E8E93`) that is legible on both, accepting that it does
  not exactly match the tinted native icons beside it. Adding `setTemplate`
  support upstream in muda / Tauri is the proper fix; it is out of scope here
  and should be recorded in `learnings.md` as a follow-up.
- **Reproducibility across macOS versions**: AppKit's symbol rasterization
  can change between OS releases, so "rerun the script, no diff" may only
  hold on the recorded macOS version. Recording the version in the script
  header and treating the committed PNGs as the source of truth (rerun only
  when a symbol changes) keeps this honest.
- **`image-png` feature**: pulls the `png` decoder into Tauri's `Image`. The
  crate is already compiled for muda, so build time and binary size impact
  is negligible; the alternative (decoding the PNG ourselves to RGBA and
  calling `Image::new`) would add code for no benefit.
- **Step count**: two steps only. Step 1 is a pure code change verifiable
  with no assets; Step 2 adds the asset pipeline, the feature flag and the
  docs. Merging them would put the "does the fallback look right in dark
  mode" question in the same review as the trivial native-icon change.

## Progress

- Step 1 (`3298ea4`): `Open in DxO PhotoLab` and `Check for Updates…` now use
  `IconMenuItem` with `NativeIcon::FollowLinkFreestanding` /
  `NativeIcon::Refresh` on macOS, behind `#[cfg(target_os = "macos")]`; other
  platforms keep the plain `MenuItem`. A non-macOS cross-check
  (`cargo check --target x86_64-unknown-linux-gnu`) was not possible because
  only `aarch64-apple-darwin` is installed locally; the non-macOS path is
  unchanged from the previously compiling code, so it was verified by
  inspection instead. See `learnings.md` for details.
- (2026-09-20) Step 1 complete
- Step 2 (`a50a077`): `Settings...` and `Undo` now use `IconMenuItem` with the
  bundled `gearshape` / `arrow.uturn.backward` PNGs (SF Symbols rendered at
  18 pt 2x in `#8E8E93`), exported by the developer-run
  `tools/macos/export-menu-icons.swift`; the `image-png` tauri feature was
  added, and `docs/agents/tauri-app.md` gained a "Menu icons" entry. With all
  four items on the macOS icon path, `MenuItem` moved to a
  `#[cfg(not(target_os = "macos"))]` import.
- (2026-09-20) Step 2 complete
