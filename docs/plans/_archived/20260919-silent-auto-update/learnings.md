# Learnings: silent-auto-update

## Step 1

- The flow lives in `crates/app/src/update.rs` (`spawn(app, interactive)`),
  called from the `setup` hook (silent) and the `Check for Updates…` menu
  item (dialogs). `UpdateRun` holds the concurrency flag and the version
  installed by this process.
- Menu click while a run is in flight: shows the dialog "An update check is
  already running." rather than ignoring the click, so the click always gets
  feedback. The startup run is silent regardless.
- After an install, later menu clicks show the "installed, used next launch"
  dialog for the stored version without touching the network, since
  `Updater::check` compares against the running binary's version.
- Sidecar flush on install: verified in
  `tauri-plugin-updater-2.11.0/src/updater.rs` that the macOS `install_inner`
  only extracts the tarball into a temp dir and renames bundles
  (`std::fs::rename` of `extract_path`, with an authorized fallback); it does
  not exit or signal the process, so the writer thread and the
  `ExitRequested` flush are unaffected. Same for the Linux AppImage path.
- Windows limitation: the running exe is locked, so the Windows
  `install_inner` launches the installer and calls `std::process::exit(0)`,
  skipping `ExitRequested` and the sidecar flush, and the "used on next
  launch" promise does not hold there (the app quits mid-session). An
  `on_before_exit` hook flushes the sidecar writer (compiled, untested).
  A possible future approach: download in the background and install only on
  quit (`Update::download` then `Update::install` from `ExitRequested`).
- Manual verification: the install path was not exercised (it needs a real
  older build against the release endpoint).

## Deferred issues (todo candidates)

- Windows self-update exits the app mid-session (installer + `process::exit`).
  Basis: plan Step 1 "Windows" note; `crates/app/src/update.rs`. Consider
  downloading in the background and installing on quit.
