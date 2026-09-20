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

# Native template menu icons (blocked) and a dependency refresh

## Purpose

The macOS menu bar mixes two icon sources: muda's `NativeIcon` for
`Open in DxO PhotoLab` and `Check for Updates…`, and three PNGs committed
under `crates/app/icons/menu/` for `Settings...`, `Open Log Folder` and
`Undo`. The goal was to move `Settings...` and `Undo` onto OS-provided
template images (`NSActionTemplate`, `NSTouchBarRotateLeftTemplate`) so they
tint with dark mode and menu highlight like their neighbours, and to drop the
two PNGs from the repo. That needs muda's `NativeIcon::Raw` /
`NativeIcon::from_name`, added in muda 0.20.0.

**Gate result (2026-09-20): blocked.** No released stable Tauri resolves
muda >= 0.20:

- `tauri 2.11.6` (latest stable) requires `muda ^0.19`; the lockfile has
  2.11.5, also `^0.19`. The chain is direct (`muda <- tauri <- riffle-app`;
  no wry / tauri-runtime-wry hop).
- Only `tauri 3.0.0-alpha.1` requires `muda ^0.20` (with
  `tauri-runtime ~3.0.0-alpha.0`, `tray-icon ^0.25`). A prerelease is not an
  acceptable base for this app, and vendoring, `[patch.crates-io]` or a git
  dependency on muda were ruled out up front.

The icon swap is therefore deferred (see "Deferred: the icon swap" below) and
this plan does only the part that can land cleanly today: refreshing the Rust
and frontend lockfiles to the newest compatible versions of Tauri, its
plugins and everything else, so the next re-check of the gate starts from an
up-to-date baseline.

## Steps

- [x] Step 1: Refresh the Rust and frontend lockfiles to the newest
      compatible versions (patch / minor only; no manifest changes)
  - Done when:
    - `Cargo.lock` has `tauri 2.11.6`, `tauri-plugin-log 2.9.2`,
      `tauri-plugin-single-instance 2.4.5`, `tauri-plugin-updater 2.12.0`,
      `cc 1.4.7`, `find-msvc-tools 0.1.13` (whatever `cargo update` resolves
      on the day; re-run `cargo update --dry-run` first and list the actual
      set in the PR body).
    - `pnpm-lock.yaml` has `@tauri-apps/cli 2.11.5` (and its platform
      binaries).
    - No `Cargo.toml` or `package.json` is modified. `cargo update --dry-run
      --verbose` after the change shows only transitive pins we do not own
      (`toml 0.8.x`, `toml_edit 0.20.x`, `generic-array 0.14.x`) as
      "unchanged, newer available".
    - `mise run ci` passes.
    - Manual sanity check on macOS (`mise run tauri:dev`): the app launches,
      the menu builds with all five icons rendering as before, and
      `Check for Updates…` (the one minor bump, `tauri-plugin-updater 2.12`)
      still completes its check without error. Record the result in
      `learnings.md`.
  - Implementation approach:
    - Rust: `cargo update` (the whole set is patch/minor within the existing
      `^2` / `^0.x` requirements). Do not edit version requirements in any
      `Cargo.toml`; every direct dependency is already at its latest stable
      (verified against crates.io on 2026-09-20: rusqlite 0.40.2,
      serde 1.0.229, serde_json 1.0.151, log 0.4.34, mozjpeg 0.10.13,
      mozjpeg-sys 2.2.3, anyhow 1.0.104, rayon 1.12.0, quick-xml 0.42.0,
      image 0.25.10, tauri-build 2.6.3, tauri-plugin-dialog 2.7.3,
      -opener 2.5.5, -store 2.4.5, -window-state 2.4.1).
    - Frontend: `pnpm update @tauri-apps/cli` (or `pnpm update` and confirm
      only `@tauri-apps/cli` moved). `vitest` stays pinned to `4.1.11` and
      `@types/node` stays on 24 (see exclusions).
    - Commit as `chore(deps): refresh the Rust and frontend lockfiles`.
    - Regression surface: this touches Tauri itself (2.11.5 -> 2.11.6), so
      the whole app is nominally affected; the practical check is the launch
      + menu + updater sanity run above. `mise run ci` covers build, clippy
      and the test suites on both sides.

## Deliberate exclusions (major bumps not attempted)

- `vitest 4.1.11 -> 5.0.1`: `vite-plus 0.3.3` (already the latest) hard-pins
  `vitest` and every `@vitest/*` package to `4.1.11`; the root
  `package.json` pins `vitest` to exactly that version so `vp test` and the
  direct dependency agree. Bumping to 5 would run a different vitest than
  `vp test` uses. Revisit when a `vite-plus` release moves to vitest 5.
- `@types/node 24.13.6 -> 26.6.2`: `mise.toml` pins Node 24.21.0; the types
  major must match the runtime major.
- `tauri 3.0.0-alpha.1` (and the alpha `tauri-runtime`, `tauri-build`,
  `@tauri-apps/cli` that go with it): prerelease. This is also the only
  thing that would unlock muda 0.20; see below.
- Transitive `toml 0.8.2` / `toml_edit 0.20.2` / `generic-array 0.14.7`: held
  by upstream pins inside the Tauri tree; not ours to override.

## Deferred: the icon swap (do not start until the gate passes)

Re-check the gate with
`curl -s -A riffle https://crates.io/api/v1/crates/tauri/<version>/dependencies`
(look for the `muda` requirement) or `cargo tree -i muda` after a trial
`cargo update`. Once a **stable** Tauri release requires `muda ^0.20` or
later, the swap is a single PR:

- `crates/app/src/main.rs`, in `build` (lines ~40–195):
  - `Settings...`: replace `IconMenuItem::with_id(... Some(Image::from_bytes(include_bytes!("../icons/menu/gearshape.png"))?) ...)`
    with `IconMenuItem::with_id_and_native_icon(... Some(NativeIcon::from_name("NSActionTemplate")) ...)`
    (or `NativeIcon::Raw("NSActionTemplate".into())`, whichever reads
    better next to the existing `NativeIcon::Refresh` usage).
  - `Undo`: same, with `NSTouchBarRotateLeftTemplate`.
  - Both stay behind the existing `#[cfg(target_os = "macos")]` /
    `#[cfg(not(target_os = "macos"))]` split; the non-macOS `MenuItem`
    twins are untouched. Note `NativeIcon::Raw` is `String` on non-Windows
    and `i32` on Windows, but the code is macOS-only so the type
    difference does not surface.
- Delete `crates/app/icons/menu/gearshape.png` and
  `crates/app/icons/menu/arrow.uturn.backward.png`. `folder.png` stays
  (see below), so `use tauri::image::Image;` and the `image-png` Tauri
  feature in `crates/app/Cargo.toml` remain in use — do not remove them.
- `Open Log Folder` keeps `folder.png` **on purpose**: the only built-in
  candidate, `NSFolder`, is a colour Finder folder (`isTemplate == false`),
  which would clash with the surrounding template icons. Reviewers should
  not "fix" this.
- `tools/macos/export-menu-icons.swift` (line 21,
  `let symbols = ["gearshape", "arrow.uturn.backward", "folder"]`): drop the
  two removed symbols so a rerun does not recreate the PNGs.
- Docs: update the "Menu icons: native where one exists, a bundled SF
  Symbol otherwise" entry in `docs/agents/tauri-app.md` (around line 286)
  and the matching sentence in `todo.md` (line 184) to describe the new
  state (only `folder.png` is bundled; the tinting limitation now applies
  to it alone).
- Facts verified with a swift + AppKit script on this machine (macOS 26.6):
  `NSActionTemplate` and `NSTouchBarRotateLeftTemplate` resolve via
  `NSImage(named:)` with `isTemplate == true`; `NSImage(named: "gearshape")`
  is nil, so SF Symbols are unreachable through muda (it never calls
  `NSImage(systemSymbolName:)`).
- Acceptance for that PR: `mise run ci` passes; `crates/app/icons/menu/`
  contains only `folder.png`; no `include_bytes!` of the two PNGs remains;
  on macOS, Settings shows a gear and Undo a u-turn arrow, both tinting as
  template images in light and dark mode and under menu highlight.

## Trade-offs and risks

- **Blocked vs. alpha vs. abandon.** The gate rules out the icon swap on a
  stable Tauri today. The user chose to land the dependency refresh now and
  wait for a stable Tauri that ships muda 0.20; the swap recipe above is
  kept ready for that moment. The alternatives — adopting
  `tauri 3.0.0-alpha.1` (prerelease across the whole runtime, plugins and
  CLI) or abandoning the idea and keeping the PNGs — were declined.
- **Lockfile-only refresh has limited value on its own.** It does not move
  the gate. It is the one step because it is what can land safely today and
  it keeps the baseline current for the next gate re-check.
- **`tauri-plugin-updater 2.11 -> 2.12` is the only minor bump.** Changes in
  a minor could touch the update dialog/flow; the manual `Check for
  Updates…` run covers it. Everything else is patch-level.
- **Frontend majors are intentionally left.** `vitest 5` and
  `@types/node 26` are both real migrations (test runner major; Node major
  the project does not run). Listed above as exclusions.

## Progress

- (2026-09-20) Step 1 complete: `cargo update` and
  `pnpm update @tauri-apps/cli` landed (`a41b0db`); `mise run ci` passes.
  The manual sanity check was blocked at first by a port-1420 conflict, then
  run successfully once the stale dev server (this worktree's own) was
  cleared: the app launches, the menu builds, `Check for Updates…` completes
  under `tauri-plugin-updater 2.12.0`, and all five menu icons render. That
  look at the menu also surfaced two pre-existing icon issues (PNG-backed
  icons render at muda's hardcoded 18pt while native ones sit at ~14–16pt,
  and `Open Folder…` has no icon); both are recorded in `learnings.md` as
  follow-ups rather than fixed here.
