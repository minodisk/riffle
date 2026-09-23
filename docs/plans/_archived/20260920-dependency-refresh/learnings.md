# Learnings

## Step 1: lockfile refresh

### What actually moved

`cargo update` resolved exactly the set the plan predicted (`Cargo.lock`):

- `cc 1.4.6 -> 1.4.7`
- `find-msvc-tools 0.1.12 -> 0.1.13`
- `tauri 2.11.5 -> 2.11.6`
- `tauri-plugin-log 2.9.1 -> 2.9.2`
- `tauri-plugin-single-instance 2.4.4 -> 2.4.5`
- `tauri-plugin-updater 2.11.0 -> 2.12.0`

`pnpm-lock.yaml`: `@tauri-apps/cli 2.11.4 -> 2.11.5` and its platform binaries
only. No other package moved.

`cargo update --dry-run --verbose` after the refresh reports only upstream-held
transitive pins as "unchanged, newer available": `generic-array 0.14.7`,
`toml 0.8.2`, `toml_edit 0.20.2` and — not named in the plan, same family —
`toml_datetime 0.6.3`.

### Trip-up: `pnpm update` rewrites `package.json`

`pnpm update @tauri-apps/cli` narrowed the `package.json` range from `^2` to
`^2.11.5`, which the step forbids. Fix: `git checkout package.json` followed by
`pnpm install --no-frozen-lockfile`, which keeps the already-resolved 2.11.5 in
the lockfile and restores the `specifier: ^2` entry. Net result is a
lockfile-only change.

### Manual sanity check

**Performed and passed** (2026-09-20, macOS 26.6, `mise run tauri:dev`).

The port-1420 conflict that blocked the first attempt turned out to be a stale
Vite dev server left behind by *this* worktree's own earlier run, not another
session's. Once it was cleared the dev command ran normally.

Verified from the run:

- The app launches (`target/debug/riffle-app`) and the menu builds with no
  panic or error.
- `Check for Updates…` completes end to end under `tauri-plugin-updater 2.12.0`
  — it fetches `latest.json` from GitHub, parses the release, and resolves the
  `darwin-aarch64-app` target against the current version.
- The folder index works (117 RAWs / 19 sidecars scanned and reconciled).
- All five menu icons render.

### Menu icon sizing: a pre-existing issue this run surfaced

Looking at the menu confirmed the icons render, but the three PNG-backed ones
(`Settings...`, `Undo`, `Open Log Folder`) are visibly **larger** than the two
`NativeIcon` ones (`Open in DxO PhotoLab`, `Check for Updates…`).

Cause, found in muda 0.19/0.20 (`src/platform_impl/macos/mod.rs`): a custom
icon goes through `icon.inner.to_nsimage(Some(18.))`, which **hardcodes an
18pt height** for every `Image`-backed menu item. A `NativeIcon` takes a
different path (`NSImage::imageNamed`) and is used at its natural size, which
for these templates is around 14–16pt. The PNG's own pixel dimensions (36×36,
i.e. 18pt @2x from `tools/macos/export-menu-icons.swift`) have no influence on
the final size — muda resizes to 18pt regardless.

This is **not a regression from the dependency refresh**; it has been true for
as long as the PNGs have been in place. The fix is available to us without any
upstream change: draw the glyph smaller inside the same canvas in
`tools/macos/export-menu-icons.swift`, so the transparent padding absorbs the
difference and the visible glyph lands at ~14pt once muda stretches the image
to 18pt.

Two related observations from the same look at the menu:

- `Open Folder…` has no icon at all — it is created as a plain `MenuItem` in
  `crates/app/src/main.rs`; none was ever assigned.
- `Toggle Full Screen` (and `Cut` / `Copy` / `Paste` / `Select All` / `Hide` /
  `Quit`) have icons we do not control. macOS inserts those items itself — the
  `View` menu does not appear anywhere in `crates/app/src/main.rs` — so muda
  and Tauri give no handle to set an icon on them.

Both are filed as follow-ups below rather than fixed here, to keep this PR to
the dependency refresh.

## Deferred issues (todo candidates)

- **Menu icons backed by PNGs render larger than the native ones.** muda
  hardcodes an 18pt height for every `Image`-backed menu item
  (`to_nsimage(Some(18.))`), while `NativeIcon` items are used at their natural
  ~14–16pt, so `Settings...`, `Undo` and `Open Log Folder` sit visibly bigger
  than `Open in DxO PhotoLab` and `Check for Updates…`. Fix without waiting on
  upstream: shrink the drawn glyph inside the canvas in
  `tools/macos/export-menu-icons.swift` so the padding absorbs muda's stretch.
  Done when the five icons look the same size in the menu. Related:
  `tools/macos/export-menu-icons.swift`, `crates/app/icons/menu/*.png`,
  `crates/app/src/main.rs`.
- **`Open Folder…` has no icon.** It is built as a plain `MenuItem` in
  `crates/app/src/main.rs` and no icon was ever assigned. `NativeIcon::Folder`
  exists but is the color Finder folder (`isTemplate == false`) and would
  clash; exporting an SF Symbol the way `Open Log Folder` does would match.
  Worth deciding together with the sizing fix above, since both touch the same
  export script. Related: `crates/app/src/main.rs`,
  `tools/macos/export-menu-icons.swift`.
