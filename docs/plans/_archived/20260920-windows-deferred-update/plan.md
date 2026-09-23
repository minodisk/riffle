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

# Windows self-update: download in the background, install on quit

## Purpose

`crates/app/src/update.rs` runs `check` → `download_and_install` on every
platform. On macOS and Linux (AppImage) the plugin's install only renames
bundles, so the running session is untouched and the new version is used on
the next launch. On Windows the running exe is locked, so
`tauri-plugin-updater-2.11.0`'s `install_inner`
(`src/updater.rs:835-876`) extracts the installer, runs the `on_before_exit`
hook, launches the installer with `ShellExecuteW` and calls
`std::process::exit(0)`: the silent startup update quits Riffle mid-session.

Split the flow on Windows into `Update::download` (in the background, at
launch or from the menu item) and `Update::install` (from
`RunEvent::ExitRequested` in `main.rs`, after the existing sidecar flush).
Once done, a Windows session is never interrupted by the updater, the update
is applied when the user quits, and every pending sidecar write is on disk
before the installer starts. This resolves the `todo.md` item "App: Windows
self-update exits the app mid-session".

## Decisions

- Only Windows defers the install; macOS/Linux keep `download_and_install`
  (deferring on macOS would run `install_inner`'s `run_on_main_thread`
  fallback from the main-thread run-loop callback at quit, a deadlock risk).
- `restart_after_install(false)`: quitting means quitting; the new version is
  used on the next launch.
- No "Quit and Install" button in the Check for Updates… dialog.
- The downloaded update is kept as a plain `(Update, Vec<u8>)` slot; no
  abstraction is added just to unit-test it.

## Steps

- [x] Step 1: Defer the Windows install to quit
  - Done when:
    - On Windows, `update::run` only downloads (and signature-verifies, which
      `Update::download` does) the update and keeps it in managed state; the
      process is never exited by the update flow while the app is in use.
    - `main.rs`'s `RunEvent::ExitRequested` handler installs the kept update
      *after* `writer.flush(sidecar::DRAIN_TIMEOUT)`, so the sidecar writer is
      drained before the installer runs. The install is attempted at most
      once; an install error is logged (`log::warn!`) and the quit proceeds.
    - The updater is built with `restart_after_install(false)` (the Windows
      context defaults to `true`, `updater.rs:197`).
    - The `on_before_exit` flush hook in `update.rs` is removed: the flush now
      runs in `main.rs` before `install`.
    - macOS and Linux keep the current behavior.
    - The startup log line and the menu item's dialog on Windows say the
      update was downloaded and will be installed when Riffle quits (not
      "installed"); a later menu click during the same session reports the
      same without re-downloading, as the current `installed` state already
      does for macOS/Linux.
    - README's "Updating:" paragraph gains a Windows sentence: the download
      still happens quietly in the background, and the installer runs when
      Riffle quits, so the next launch is the new version.
    - The `docs/agents/tauri-app.md` section "Self-update's install step
      bypasses the sidecar flush on Windows only" is rewritten to describe the
      new mechanism (download in the background, `install` from
      `ExitRequested` after the flush, `restart_after_install(false)`), keeping
      the "why" (locked exe, `process::exit(0)` inside `install`). Do not touch
      other sections.
    - Unit tests for whatever logic is testable without the network;
      `mise run ci` passes (CI compiles the `#[cfg(windows)]` branch on
      `windows-latest`).
  - Implementation approach:
    - Files: `crates/app/src/update.rs`, `crates/app/src/main.rs`,
      `README.md`, `docs/agents/tauri-app.md`. No change to
      `crates/app/tauri.conf.json`, `Cargo.toml`, capabilities, or the release
      workflow.
    - State: extend `UpdateRun` with a slot for the downloaded update, e.g.
      `pending: Mutex<Option<(tauri_plugin_updater::Update, Vec<u8>)>>`
      (`Update` is `Clone`). Keep `index::lock` for the mutexes. Consider
      renaming `installed` (it now means "installed or downloaded"); keep the
      rename minimal.
    - Flow: in `check_and_install` (rename to fit), after `updater.check()`,
      branch with `#[cfg(windows)]`: `update.download(|_, _| {}, || {}).await?`
      and store `(update.clone(), bytes)` in `pending`; otherwise keep
      `download_and_install`. Build with
      `app.updater_builder().restart_after_install(false).build()?`.
    - Quit: add `pub fn install_pending(app: &AppHandle)` in `update.rs` that
      takes the slot and calls `update.install(bytes)`, logging `Err`. Compile
      it on every platform so `main.rs` stays `cfg`-free; call it right after
      the writer flush in the `ExitRequested` arm. Comment that on Windows
      `install` launches the installer and calls `process::exit(0)` itself.
    - `ExitRequested` fires with `code: None` on a user quit and `Some(code)`
      from `AppHandle::exit`; both reach the install. A download still in
      flight at quit is dropped and retried on the next launch.
    - Texts: split `installed_text` per platform (Windows: "Riffle {version}
      was downloaded and will be installed when Riffle quits."). The startup
      `log::info!` likewise.
    - Tests: `Update` cannot be constructed outside the plugin, so the slot is
      compile-checked only. Record in `learnings.md` that the Windows install
      path is not exercised end-to-end; the existing `todo.md` verification
      item tracks it.
    - Run `mise run fmt` before `mise run ci`.

## Trade-offs and risks

- End-to-end verification needs a real older Windows build against the
  release endpoint; it stays under the existing `todo.md` verification item.
- Installer bytes are held in memory for the session (tens of MB).
- Quit latency on Windows grows by the installer extraction plus the bounded
  flush; acceptable for a quit.

## Progress

- (2026-09-20) Step 1 complete
