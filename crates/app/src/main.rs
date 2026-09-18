#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod index;
mod sidecar;

// The Debug menu exists only in a development build; a distributable build
// leaves the `devtools` feature off and drops it entirely.
#[cfg(any(feature = "devtools", debug_assertions))]
mod debug_menu {
    use tauri::menu::{CheckMenuItem, Menu, MenuEvent, Submenu};
    use tauri::{AppHandle, Emitter, Manager, Wry};

    const TIMING_ID: &str = "debug-timing";

    /// Holds the item so the event handler can read back the checked state
    /// the platform toggled for us.
    struct TimingItem(CheckMenuItem<Wry>);

    pub fn append(handle: &AppHandle, menu: &Menu<Wry>) -> tauri::Result<()> {
        let timing =
            CheckMenuItem::with_id(handle, TIMING_ID, "Timing logs", true, false, None::<&str>)?;
        let debug = Submenu::with_items(handle, "Debug", true, &[&timing])?;
        menu.append(&debug)?;
        handle.manage(TimingItem(timing));
        Ok(())
    }

    pub fn on_event(app: &AppHandle, event: &MenuEvent) {
        if event.id() != TIMING_ID {
            return;
        }
        let checked = app.state::<TimingItem>().0.is_checked().unwrap_or(false);
        let _ = app.emit("debug", checked);
    }
}

mod app_menu {
    use tauri::menu::{Menu, MenuEvent, MenuItem, Submenu};
    use tauri::{AppHandle, Emitter, Wry};

    const PHOTOLAB_ID: &str = "open-in-photolab";

    pub fn build(handle: &AppHandle) -> tauri::Result<Menu<Wry>> {
        // The default menu carries the platform's standard items (Quit, Copy,
        // ...), which setting a menu at all would otherwise replace.
        let menu = Menu::default(handle)?;
        let photolab = MenuItem::with_id(
            handle,
            PHOTOLAB_ID,
            "Open in DxO PhotoLab",
            true,
            None::<&str>,
        )?;
        let folder = Submenu::with_items(handle, "Folder", true, &[&photolab])?;
        menu.append(&folder)?;
        #[cfg(any(feature = "devtools", debug_assertions))]
        super::debug_menu::append(handle, &menu)?;
        Ok(menu)
    }

    pub fn on_event(app: &AppHandle, event: MenuEvent) {
        // The frontend owns which folder is open, so it does the invoking.
        if event.id() == PHOTOLAB_ID {
            let _ = app.emit("open-in-photolab", ());
        }
        #[cfg(any(feature = "devtools", debug_assertions))]
        super::debug_menu::on_event(app, &event);
    }
}

use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager, RunEvent};

/// The payload of the `sidecar-error` event: a sidecar that could not be
/// written. The judgement stays in the index and is retried on the next open.
#[derive(Clone, serde::Serialize)]
struct SidecarError {
    path: String,
    message: String,
}

#[tauri::command]
fn ping() -> String {
    "pong".to_string()
}

fn main() {
    let builder = tauri::Builder::default()
        // Registered first, as the plugin requires: a second launch would
        // contend with this one for the SQLite index and the sidecar writer,
        // so it focuses the existing window and exits instead.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::new().build());
    builder
        .menu(app_menu::build)
        .on_menu_event(app_menu::on_event)
        .setup(|app| {
            let path = app.path().app_cache_dir()?.join("index.sqlite");
            // The index is a thumbnail/metadata cache, not required data: an
            // unwritable cache dir degrades to "no thumbnails" rather than
            // stopping the app from launching.
            let index = match index::Index::open(&path) {
                Ok(index) => Some(Arc::new(Mutex::new(index))),
                Err(e) => {
                    log::error!("failed to open the index cache at {}: {e}", path.display());
                    None
                }
            };
            let writer = index.as_ref().map(|index| {
                let handle = app.handle().clone();
                // `Emitter` is safe from any thread, as `run_scan` already
                // relies on, so the writer thread reports its own failures.
                sidecar::Writer::spawn(index.clone(), move |path, message| {
                    let _ = handle.emit(
                        "sidecar-error",
                        SidecarError {
                            path: path.to_string_lossy().into_owned(),
                            message: message.to_string(),
                        },
                    );
                })
            });
            let format = commands::load_settings(app.handle());
            app.manage(commands::AppSidecarFormat(Mutex::new(format)));
            app.manage(commands::AppWriter(writer));
            app.manage(commands::AppIndex(index));
            app.manage(commands::Scans::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            commands::pick_folder,
            commands::list_arw,
            commands::remember_folder,
            commands::last_folder,
            commands::preview,
            commands::dropped_folder,
            commands::scan_folder,
            commands::start_scan,
            commands::folder_entries,
            commands::thumbnail,
            commands::metadata,
            commands::focus_crop,
            commands::set_rating,
            commands::open_in_photolab
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // A judgement may still be inside the writer's debounce window
            // when the user quits, so the exit waits on an explicit drain
            // rather than on the writer's `Drop`, which is not guaranteed to
            // run during teardown.
            if let RunEvent::ExitRequested { .. } = event {
                if let Some(writer) = &app.state::<commands::AppWriter>().0 {
                    writer.flush(sidecar::DRAIN_TIMEOUT);
                }
            }
        });
}
