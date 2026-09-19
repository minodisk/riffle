//! The self-update flow: check, download and install in the background, with
//! the new version used on the next launch. The app is never relaunched. On
//! Windows the install exits the process, so it is deferred to quit.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

use crate::index;

/// Guards against a startup check and a menu click running the updater at
/// once, and remembers the version installed (or, on Windows, downloaded) by
/// this process: the updater compares against the running binary, so a later
/// check would otherwise download the same update again.
#[derive(Default)]
pub struct UpdateRun {
    running: AtomicBool,
    installed: Mutex<Option<String>>,
    pending: Mutex<Option<(tauri_plugin_updater::Update, Vec<u8>)>>,
}

/// Install the update downloaded this session, if any. Called on quit after
/// the sidecar flush; on Windows `install` launches the installer and calls
/// `process::exit(0)` itself.
pub fn install_pending(app: &AppHandle) {
    let Some((update, bytes)) = index::lock(&app.state::<UpdateRun>().pending).take() else {
        return;
    };
    if let Err(e) = update.install(bytes) {
        log::warn!("installing the update failed: {e}");
    }
}

/// Run the update flow in the background. `interactive` reports the outcome
/// through native dialogs (the menu item); otherwise it is only logged.
pub fn spawn(app: AppHandle, interactive: bool) {
    tauri::async_runtime::spawn(run(app, interactive));
}

async fn run(app: AppHandle, interactive: bool) {
    let state = app.state::<UpdateRun>();
    if let Some(version) = index::lock(&state.installed).clone() {
        if interactive {
            message(&app, &installed_text(&version), MessageDialogKind::Info);
        }
        return;
    }
    if state
        .running
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        if interactive {
            message(
                &app,
                "An update check is already running.",
                MessageDialogKind::Info,
            );
        }
        return;
    }
    let result = check_and_update(&app).await;
    if let Ok(Some(version)) = &result {
        *index::lock(&state.installed) = Some(version.clone());
    }
    state.running.store(false, Ordering::Release);
    match result {
        Ok(None) => {
            if interactive {
                message(&app, "Riffle is up to date.", MessageDialogKind::Info);
            }
        }
        Ok(Some(version)) => {
            log::info!("{}", installed_log(&version));
            if interactive {
                message(&app, &installed_text(&version), MessageDialogKind::Info);
            }
        }
        Err(e) => {
            log::warn!("update failed: {e}");
            if interactive {
                message(
                    &app,
                    &format!("Update failed: {e}"),
                    MessageDialogKind::Error,
                );
            }
        }
    }
}

async fn check_and_update(app: &AppHandle) -> Result<Option<String>, tauri_plugin_updater::Error> {
    let updater = app.updater_builder().restart_after_install(false).build()?;
    let Some(update) = updater.check().await? else {
        return Ok(None);
    };
    #[cfg(windows)]
    {
        let bytes = update.download(|_, _| {}, || {}).await?;
        *index::lock(&app.state::<UpdateRun>().pending) = Some((update.clone(), bytes));
    }
    #[cfg(not(windows))]
    update.download_and_install(|_, _| {}, || {}).await?;
    Ok(Some(update.version))
}

#[cfg(windows)]
fn installed_text(version: &str) -> String {
    format!("Riffle {version} was downloaded and will be installed when Riffle quits.")
}

#[cfg(not(windows))]
fn installed_text(version: &str) -> String {
    format!("Riffle {version} was installed and will be used the next time Riffle launches.")
}

#[cfg(windows)]
fn installed_log(version: &str) -> String {
    format!("downloaded Riffle {version}; it is installed when Riffle quits")
}

#[cfg(not(windows))]
fn installed_log(version: &str) -> String {
    format!("installed Riffle {version}; it is used on the next launch")
}

fn message(app: &AppHandle, text: &str, kind: MessageDialogKind) {
    app.dialog()
        .message(text)
        .title("Riffle")
        .kind(kind)
        .show(|_| {});
}
