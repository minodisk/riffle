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

# Offer a restart after a manual update

## Purpose

`Check for Updates…` installs (macOS/Linux) or downloads (Windows) the new
version and then only tells the user it will be used on the next launch. A
user who clicked the menu item usually wants the new version now. After the
manual check, offer to restart at once on every platform, while keeping
"later" identical to today's behavior and leaving the silent startup check
untouched.

## Steps

- [x] Step 1: Replace the interactive "installed" info dialog with a restart offer
  - Done when:
    - In `crates/app/src/update.rs`, both interactive branches that currently
      show `installed_text(version)` (the `state.installed` already-set branch
      and the `Ok(Some(version))` branch) show a two-button dialog instead:
      the same facts plus buttons `Restart Now` / `Later` on every platform,
      via `tauri_plugin_dialog::MessageDialogButtons::OkCancelCustom`.
    - macOS/Linux: `Restart Now` calls `app.request_restart()`, which goes
      through `RunEvent::ExitRequested` in `crates/app/src/main.rs`, so the
      sidecar flush, `install_pending` and `mcp::stop` run unchanged (verify
      by reading the arm, not by assumption).
    - Design change during implementation: `Update` itself has a
      `restart_after_install(self, bool) -> Self` setter
      (`tauri-plugin-updater-2.12.0/src/updater.rs:783`), so the Windows path
      below was simplified to swapping the pending `Update` for
      `update.restart_after_install(true)` (same bytes) and `app.exit(0)`,
      with no re-check, no extra request and no version-mismatch error path.
      See `learnings.md`.
    - Windows (original plan, superseded by the note above): `Restart Now` rebuilds the updater with
      `restart_after_install(true)`, calls `check()` again, and, when it
      returns the same version as the pending download, replaces the
      `Update` in `UpdateRun.pending` with the new one while keeping the
      already-downloaded bytes (no second download). Then `app.exit(0)`, so
      `ExitRequested` flushes sidecars and `install_pending` runs the
      installer with the relaunch arguments. If the re-check fails or returns
      a different / no version, show an error dialog and leave the pending
      download as it was (install on quit, no relaunch).
      - Before relying on this, confirm in
        `tauri-plugin-updater-2.12.0/src/updater.rs` that the signature is
        verified in `download` (not `install`) and that `install(bytes)` on a
        freshly checked `Update` needs no state from the original download.
        If that does not hold, stop and record it in `learnings.md`.
    - `Later` (or closing the dialog) does nothing; the version stays recorded
      in `UpdateRun.installed`, and on Windows the download stays in
      `UpdateRun.pending` for the quit-time install with no relaunch.
    - The non-interactive startup check still only logs (`installed_log`);
      "up to date", "already running" and error dialogs are untouched.
    - The module doc comment at the top of `update.rs` ("The app is never
      relaunched") is corrected.
    - `docs/usage.md` (the "Updating:" paragraph) describes the new dialog,
      and the "Self-update defers the install to quit on Windows only" entry
      in `docs/agents/tauri-app.md` gets a short note on the interactive
      restart and why the Windows path re-checks with
      `restart_after_install(true)` instead of building with `true` up front
      (that would relaunch after a normal quit following `Later`).
      `README.md` / `README.ja.md` only describe the startup check; touch
      them only if their wording becomes wrong, and keep the two in sync.
    - `mise run ci` passes.
  - Implementation approach:
    - `AppHandle::request_restart` (tauri 2.11.6, `app.rs:615`) emits
      `ExitRequested` with `RESTART_EXIT_CODE` and re-execs on `Exit`; do not
      use `AppHandle::restart`, which is `-> !` and parks the calling thread
      when called off the main thread (the dialog callback thread).
    - On Windows the installer only relaunches the app when the updater was
      built with `restart_after_install(true)`; the flag is captured in the
      `Update` at check time (`tauri-plugin-updater-2.12.0/src/updater.rs`
      ~896), hence the re-check. Keep `restart_after_install(false)` for the
      initial check so `Later` + a normal quit does not relaunch.
    - Add a small helper next to `message()` (e.g. a `confirm` taking the
      text, button labels and an on-ok closure) using
      `.buttons(MessageDialogButtons::OkCancelCustom(..)).show(move |ok| ..)`.
      The Windows re-check is async; spawn it with
      `tauri::async_runtime::spawn` from the callback. Gate platform code with
      `#[cfg(windows)]` like the existing `installed_text` / `installed_log`
      pairs. Match the surrounding style.
    - `installed_text` wording may be adjusted to read naturally above the
      buttons, but keep the facts.

## Trade-offs and risks

- Neither `request_restart` nor the Windows installer path is exercised by
  CI; real-install verification on each OS is manual. Record in
  `learnings.md` what was actually tested.
- The Windows `Restart Now` costs one extra `check()` request (the small
  update manifest). If the server has published a newer version between the
  download and the click, the versions differ and the relaunch is declined
  with an error dialog rather than installing mismatched bytes.
- The `state.installed` branch (second menu click in the same session) now
  also offers the restart; this is intended, so the user can change their
  mind after choosing `Later`.

## Progress

- (none yet)
