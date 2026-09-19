#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod exif;
mod index;
mod shortcuts;
mod sidecar;

mod app_menu {
    use tauri::menu::{Menu, MenuEvent, MenuItem, MenuItemKind, PredefinedMenuItem, Submenu};
    use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, Wry};

    const PHOTOLAB_ID: &str = "open-in-photolab";
    const SETTINGS_ID: &str = "open-settings";
    const UNDO_ID: &str = "undo";

    /// The default menu's submenu titled `title`, if the platform has one.
    fn submenu(menu: &Menu<Wry>, title: &str) -> tauri::Result<Option<Submenu<Wry>>> {
        for item in menu.items()? {
            if let MenuItemKind::Submenu(submenu) = item {
                if submenu.text()? == title {
                    return Ok(Some(submenu));
                }
            }
        }
        Ok(None)
    }

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
        let settings = MenuItem::with_id(
            handle,
            SETTINGS_ID,
            "Settings...",
            true,
            Some("CmdOrCtrl+,"),
        )?;
        // Linux's default menu has no File submenu, so one is added there.
        let file = match submenu(&menu, "File")? {
            Some(file) => file,
            None => {
                let file = Submenu::new(handle, "File", true)?;
                menu.prepend(&file)?;
                file
            }
        };
        file.prepend_items(&[&photolab, &PredefinedMenuItem::separator(handle)?])?;
        // macOS puts Settings in the app menu, right after About; elsewhere it
        // goes at the end of File's own items, above Close Window and Quit.
        #[cfg(target_os = "macos")]
        if let Some(MenuItemKind::Submenu(app)) = menu.items()?.into_iter().next() {
            app.insert_items(&[&settings, &PredefinedMenuItem::separator(handle)?], 2)?;
        }
        #[cfg(not(target_os = "macos"))]
        file.insert_items(&[&settings, &PredefinedMenuItem::separator(handle)?], 2)?;
        // `Edit` opens with the predefined Undo and Redo, which only act on
        // editable content (neither window has any) and would own Cmd+Z.
        if let Some(edit) = submenu(&menu, "Edit")? {
            for _ in 0..2 {
                if let Some(MenuItemKind::Predefined(_)) = edit.items()?.into_iter().next() {
                    edit.remove_at(0)?;
                }
            }
            let undo = MenuItem::with_id(handle, UNDO_ID, "Undo", true, Some("CmdOrCtrl+Z"))?;
            edit.insert(&undo, 0)?;
        }
        Ok(menu)
    }

    pub fn on_event(app: &AppHandle, event: MenuEvent) {
        // The frontend owns which folder is open, so it does the invoking.
        if event.id() == PHOTOLAB_ID {
            let _ = app.emit("open-in-photolab", ());
        }
        if event.id() == UNDO_ID {
            let _ = app.emit("undo", ());
        }
        if event.id() == SETTINGS_ID {
            if let Err(e) = open_settings(app) {
                log::error!("failed to open the settings window: {e}");
            }
        }
    }

    /// Focus the settings window, or open it when it is not open yet.
    fn open_settings(app: &AppHandle) -> tauri::Result<()> {
        if let Some(window) = app.get_webview_window(super::SETTINGS_WINDOW) {
            return window.set_focus();
        }
        WebviewWindowBuilder::new(
            app,
            super::SETTINGS_WINDOW,
            WebviewUrl::App("settings.html".into()),
        )
        .title("Settings")
        .inner_size(480.0, 640.0)
        .build()?;
        Ok(())
    }
}

use std::sync::{Arc, Mutex};

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};

use sidecar::SidecarFormat;

const SETTINGS_WINDOW: &str = "settings";

/// Whether timing logs are on; the settings window toggles it and the main
/// window logs by it, so it is kept here where both can reach it.
struct TimingLogs(AtomicBool);

#[tauri::command]
fn timing_logs(state: tauri::State<TimingLogs>) -> bool {
    state.0.load(Ordering::Relaxed)
}

#[tauri::command]
fn set_timing_logs(app: AppHandle, enabled: bool) {
    app.state::<TimingLogs>()
        .0
        .store(enabled, Ordering::Relaxed);
    let _ = app.emit("debug", enabled);
}

/// Switch the sidecar format from the settings window. The backend then emits
/// `sidecar-format` so the frontend reopens the folder in the new format.
#[tauri::command]
async fn set_sidecar_format(app: AppHandle, format: String) -> Result<(), String> {
    let format = SidecarFormat::from_setting(Some(&format));
    let unchanged = *index::lock(&app.state::<commands::AppSidecarFormat>().0) == format;
    // The switch drains the writer and touches SQLite and the settings file,
    // none of which may block the main thread.
    let switched = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        commands::switch_sidecar_format(&switched, format)
    })
    .await
    .map_err(|e| e.to_string())??;
    if !unchanged {
        let _ = app.emit("sidecar-format", format.setting());
    }
    Ok(())
}

/// Whether this is a development build, which shows the settings window's
/// Debug section; a distributable build leaves the `devtools` feature off.
#[tauri::command]
fn debug_build() -> bool {
    cfg!(any(feature = "devtools", debug_assertions))
}

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
            let (format, overrides, keymap, auto_advance) = commands::load_settings(app.handle());
            app.manage(TimingLogs(AtomicBool::new(false)));
            app.manage(commands::AppSidecarFormat(Mutex::new(format)));
            app.manage(commands::AppAutoAdvance(AtomicBool::new(auto_advance)));
            app.manage(commands::AppKeymap(Mutex::new(keymap)));
            app.manage(commands::AppShortcutOverrides(Mutex::new(overrides)));
            app.manage(commands::AppSwitchLock(Mutex::new(())));
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
            commands::sidecar_format,
            commands::shortcuts,
            commands::set_shortcut,
            commands::reset_shortcut,
            commands::reset_shortcuts,
            commands::open_in_photolab,
            commands::auto_advance,
            commands::set_auto_advance,
            set_sidecar_format,
            debug_build,
            timing_logs,
            set_timing_logs
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // A judgement may still be inside the writer's debounce window
            // when the user quits, so the exit waits on an explicit drain
            // rather than on the writer's `Drop`, which is not guaranteed to
            // run during teardown.
            // The settings window serves the main one, so it closes with it
            // rather than keeping the app running on its own.
            if let RunEvent::WindowEvent {
                label,
                event: WindowEvent::Destroyed,
                ..
            } = &event
            {
                if label == "main" {
                    if let Some(settings) = app.get_webview_window(SETTINGS_WINDOW) {
                        let _ = settings.close();
                    }
                }
            }
            if let RunEvent::ExitRequested { .. } = event {
                if let Some(writer) = &app.state::<commands::AppWriter>().0 {
                    writer.flush(sidecar::DRAIN_TIMEOUT);
                }
            }
        });
}
