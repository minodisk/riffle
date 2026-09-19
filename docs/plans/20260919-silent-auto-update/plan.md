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

# Silent auto-update with a "Check for Updates…" menu item

## Purpose

The in-window update bar (`#update` in `crates/app/ui/index.html`, driven by
`checkForUpdate` in `crates/app/ui/src/main.ts`) interrupts culling and
relaunches the app after installing. Replace it with an invisible flow: on
launch, an available update is downloaded and installed in the background and
simply used on the next launch (macOS replaces the `.app` bundle in place, so
the running process is unaffected). A native "Check for Updates…" item in the
app menu lets the user trigger the same flow by hand and learn the outcome
through native dialogs. Once done the window has no update UI at all and the
update logic lives in one place, in Rust.

## Steps

- [x] Step 1: Move the update flow to Rust, drop the update bar, add the menu item
  - Done when:
    - `crates/app/ui/index.html` has no `#update` / `#update-text` /
      `#update-install`; `crates/app/ui/style.css` has no `#update` rules;
      `checkForUpdate` and its call are gone from `crates/app/ui/src/main.ts`;
      `TauriDownloadEvent`, `TauriUpdate`, `updater` and `process` are gone
      from `crates/app/ui/src/tauri.d.ts`.
    - On launch the app checks the updater endpoint in a background task; if
      an update exists it is downloaded and installed with no UI. Failures
      are only logged (`log::warn!` / `log::error!`, via the existing
      `tauri-plugin-log`). The app is never relaunched.
    - The macOS app menu (the `File` menu elsewhere, matching how
      `Settings...` is placed) has a `Check for Updates…` item. Clicking it:
      no update -> native dialog "Riffle is up to date."; update found ->
      download + install, then native dialog saying Riffle <version> was
      installed and will be used the next time Riffle launches; any error ->
      native dialog with the error message.
    - A startup check and a menu click cannot run the updater concurrently.
    - `tauri-plugin-process` is removed from `crates/app/Cargo.toml`
      (`Cargo.lock` updated), `.plugin(tauri_plugin_process::init())` is gone
      from `main.rs`, and `process:allow-restart` and `updater:default` are
      removed from `crates/app/capabilities/default.json` (the frontend no
      longer calls either plugin).
    - README's "Updating:" paragraph describes the new behaviour (silent
      install on launch, used on next launch, plus the menu item) instead of
      the bar.
    - `mise run ci` passes.
  - Implementation approach:
    - Put the flow in Rust only; no frontend involvement. Use
      `tauri_plugin_updater::UpdaterExt` (`app.updater()` or
      `app.updater_builder().build()`), `Updater::check().await`, then
      `Update::download_and_install(|_, _| {}, || {}).await`. `check` and
      `download` are async, and `install` on macOS does blocking file work,
      so run the whole thing inside `tauri::async_runtime::spawn` from both
      the `setup` hook (startup) and the menu event handler (which runs on
      the main thread and must not block).
    - A small module (e.g. `crates/app/src/update.rs`, or a `mod update`
      next to `mod app_menu` in `main.rs`, matching the existing style) with
      one async `fn run(app: AppHandle, interactive: bool)` that both callers
      share, so the check/download/install sequence is written once; only the
      reporting differs (dialogs when `interactive`, logs otherwise).
    - Dialogs: `tauri-plugin-dialog` is already a dependency and initialised.
      `app.dialog().message(text).title("Riffle").kind(MessageDialogKind::Info | Error).show(|_| {})`
      is non-blocking and marshals itself to the main thread, so it is safe to
      call from the async task. Do not use `blocking_show` (see the note on
      `pick_folder` in `commands.rs`).
    - Concurrency guard: managed state such as `struct UpdateRun(AtomicBool)`
      claimed with `compare_exchange(false, true)` at the start of `run` and
      cleared at the end (including on error). When the menu click finds the
      flag already set, either show a dialog ("An update check is already
      running.") or ignore the click; pick one and note it in `learnings.md`.
    - Repeat checks after an install: `Updater::check` compares the endpoint
      against the *running* binary's version, so once an update has been
      installed a later menu click would report the same version again and
      re-download it. Keep the installed version in the same state (e.g.
      `Mutex<Option<String>>`); when set, the menu click shows the "installed,
      used next launch" dialog for that version without touching the network.
    - Menu placement: add a `CHECK_UPDATES_ID` `MenuItem` in `app_menu::build`.
      On macOS insert it into the app submenu together with `Settings...`
      (e.g. `Check for Updates…` right after `Settings...`, keeping one
      separator group); elsewhere put it in `File` next to `Settings...`.
      Dispatch in `app_menu::on_event` by id.
    - Sidecar writer: no flush is needed on install. The writer thread and
      its `ExitRequested` flush in `main.rs` are tied to process lifetime,
      which the macOS/Linux install does not touch (the plugin's
      `install_inner` for macOS only extracts and renames the bundle, in
      `tauri-plugin-updater-2.11.0/src/updater.rs`). Verify by reading that
      path during implementation and record it in `learnings.md`. Windows is
      the exception (below).
    - Windows: `install` launches the installer and calls
      `std::process::exit(0)`, skipping `ExitRequested` and the sidecar flush.
      Out of scope beyond compiling; document as a known limitation in
      learnings. If it is cheap, pass `updater_builder().on_before_exit(...)`
      that flushes the writer, but do not test it.
    - Do not touch `plugins.updater` in `tauri.conf.json` (endpoint, pubkey,
      Windows `installMode`) or the release workflow.
    - Run `mise run fmt` (cargo fmt + oxfmt) before `mise run ci`.
    - Manual verification: `mise run tauri:dev` shows the menu item, and
      clicking it with the current version reports "Riffle is up to date."
      The install path can only be exercised against a real older build;
      note that it was not exercised if so.

## Trade-offs and risks

- Rust vs frontend: Rust chosen because the menu event already arrives in
  Rust, the dialog plugin is already initialised there, and the frontend
  would otherwise need the `updater` and `dialog:allow-message` capabilities
  and a duplicate of the flow. Cost: none of the update code is unit-testable
  without network; the previous frontend code was not either.
- Menu click while the startup run is in flight: show a dialog vs ignore.
  Decision left to the implementer, recorded in `learnings.md`.
- Startup check racing app quit: a half-download does not corrupt the
  installed bundle; the update is retried on the next launch.
- Dropping `updater:default` and `process:allow-restart` capabilities and the
  `tauri-plugin-process` crate: orphans of this change, removed.
- Windows: install exits the app without the sidecar flush; known limitation.
- Linux `.deb` / `.rpm` installs cannot self-update (unchanged from today);
  the menu click there surfaces the plugin's error in a dialog.

## Progress

- (none yet)
