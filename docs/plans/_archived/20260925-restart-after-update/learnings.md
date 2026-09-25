# Learnings: restart-after-update

## Step 1

- `tauri-plugin-updater-2.12.0` verifies the signature in `Update::download`
  (`verify_signature` at the end of `download`), and the Windows
  `install_inner` only uses the bytes plus `self.context` (install mode,
  installer args, `restart_after_install`, current exe args). So installing
  kept bytes with a different `Update` value is safe.
- `Update` has its own `restart_after_install(self, bool) -> Self` setter
  (`updater.rs:783`, inside `impl Update`), not only the builder's. The plan's
  Windows re-check with a rebuilt updater was therefore unnecessary: the
  `Restart Now` path maps the pending `(Update, Vec<u8>)` to
  `(update.restart_after_install(true), bytes)` and calls `app.exit(0)`. This
  drops the extra manifest request and the version-mismatch error dialog the
  plan described. `plan.md` records the change.
- `AppHandle::request_restart` (`tauri-2.11.6/src/app.rs`) sets
  `restart_on_exit` and requests exit with `RESTART_EXIT_CODE`; `main.rs`'s
  `RunEvent::ExitRequested { .. }` arm matches any code and never calls
  `prevent_exit`, so the sidecar flush, `install_pending` (a no-op off
  Windows) and `mcp::stop` run unchanged before the relaunch.
- `MessageDialogBuilder::show` reports `true` for `OkCancelCustom` when the
  pressed button's label equals the ok label, so closing the dialog maps to
  `Later`.
- Tested: `mise run ci` on Windows only (compiles the `#[cfg(windows)]`
  path). The `#[cfg(not(windows))]` `restart` was not compiled locally, and
  neither path was exercised against a real install.
- `index::lock(&app.state::<UpdateRun>().pending)` held across statements
  fails with E0716 (the `State` temporary dies at the end of the `let`); bind
  the state first.
- Local `mise run ci` on Windows fails in `lint` on a file this step never
  touched: lychee's `--exclude-path docs/plans/review-history` does not match
  the backslash paths lychee reports on Windows, so the excluded
  `review-history` file with a stale `#running-the-app` fragment gets
  checked. Every other lint step (rerun with `--exclude-path review-history`)
  and `mise run test` (clippy `-D warnings`, `cargo test`, `vp test`) pass.

## Deferred issues (todo candidates)

- The `lint` task's lychee `--exclude-path docs/plans/review-history`
  (`mise.toml`) does not exclude that directory on Windows (lychee reports
  `docs\plans\review-history\...`), so local `mise run ci` on Windows fails on
  `docs/plans/review-history/thumbnail-cache-filmstrip-step-8/review-20260918-0253.md`
  (`#running-the-app`, Cannot find fragment). Basis: Step 1's local
  `mise run ci` run. A separator-agnostic pattern (e.g. `review-history`)
  would fix it.
