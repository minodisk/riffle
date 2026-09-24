#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod exif;
mod index;
mod mcp;
mod shortcuts;
mod sidecar;
mod trash;
mod update;
mod watch;

mod app_menu {
    use tauri::image::Image;
    #[cfg(target_os = "macos")]
    use tauri::menu::IconMenuItem;
    #[cfg(not(target_os = "macos"))]
    use tauri::menu::MenuItem;
    use tauri::menu::{AboutMetadata, Menu, MenuEvent, MenuItemKind, PredefinedMenuItem, Submenu};
    use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, Wry};
    use tauri_plugin_opener::OpenerExt;

    const OPEN_FOLDER_ID: &str = "open-folder";
    const RELOAD_FOLDER_ID: &str = "reload-folder";
    const TRASH_REJECTED_ID: &str = "trash-rejected";
    const OPEN_LOG_FOLDER_ID: &str = "open-log-folder";
    const SETTINGS_ID: &str = "open-settings";
    const UNDO_ID: &str = "undo";
    const REDO_ID: &str = "redo";
    const CHECK_UPDATES_ID: &str = "check-for-updates";

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

    /// The app menu, with the `Open Folder…`, `Undo` and `Redo` accelerators
    /// the keymap currently gives those actions.
    pub fn build(
        handle: &AppHandle,
        keymap: &crate::shortcuts::Keymap,
    ) -> tauri::Result<Menu<Wry>> {
        let open = keymap.accelerator_for("open");
        let open = open.as_deref();
        let undo_key = keymap.accelerator_for("undo");
        let undo_key = undo_key.as_deref();
        let redo_key = keymap.accelerator_for("redo");
        let redo_key = redo_key.as_deref();
        // The default menu carries the platform's standard items (Quit, Copy,
        // ...), which setting a menu at all would otherwise replace.
        let menu = Menu::default(handle)?;
        // The default About's metadata carries no icon, so macOS's About
        // panel and GTK's About dialog show none; it is rebuilt with the same
        // metadata plus the app icon (Windows' MessageBox ignores it).
        let about = PredefinedMenuItem::about(
            handle,
            None,
            Some(AboutMetadata {
                name: Some(handle.package_info().name.clone()),
                version: Some(handle.package_info().version.to_string()),
                copyright: handle.config().bundle.copyright.clone(),
                authors: handle.config().bundle.publisher.clone().map(|p| vec![p]),
                icon: Some(Image::from_bytes(include_bytes!("../icons/128x128.png"))?),
                ..Default::default()
            }),
        )?;
        #[cfg(target_os = "macos")]
        let about_menu = match menu.items()?.into_iter().next() {
            Some(MenuItemKind::Submenu(app)) => Some(app),
            _ => None,
        };
        #[cfg(not(target_os = "macos"))]
        let about_menu = submenu(&menu, "Help")?;
        if let Some(about_menu) = about_menu {
            if let Some(MenuItemKind::Predefined(_)) = about_menu.items()?.into_iter().next() {
                about_menu.remove_at(0)?;
                about_menu.insert(&about, 0)?;
            }
        }
        // `folder.png` is deliberately shared with Help > Open Log Folder:
        // the two menus are never open at the same time.
        #[cfg(target_os = "macos")]
        let open_folder = IconMenuItem::with_id(
            handle,
            OPEN_FOLDER_ID,
            "Open Folder…",
            true,
            Some(Image::from_bytes(include_bytes!(
                "../icons/menu/folder.png"
            ))?),
            open,
        )?;
        #[cfg(not(target_os = "macos"))]
        let open_folder = MenuItem::with_id(handle, OPEN_FOLDER_ID, "Open Folder…", true, open)?;
        // No accelerator: a destructive action, reached deliberately through
        // the menu and its confirmation.
        #[cfg(target_os = "macos")]
        let trash_rejected = IconMenuItem::with_id(
            handle,
            TRASH_REJECTED_ID,
            "Move Rejected to Trash…",
            true,
            Some(Image::from_bytes(include_bytes!(
                "../icons/menu/trash.png"
            ))?),
            None::<&str>,
        )?;
        #[cfg(not(target_os = "macos"))]
        let trash_rejected = MenuItem::with_id(
            handle,
            TRASH_REJECTED_ID,
            "Move Rejected to Trash…",
            true,
            None::<&str>,
        )?;
        // A fixed accelerator, like Settings: reloading is not a
        // culling action, so it is not part of the rebindable keymap.
        #[cfg(target_os = "macos")]
        let reload_folder = IconMenuItem::with_id(
            handle,
            RELOAD_FOLDER_ID,
            "Reload Folder",
            true,
            Some(Image::from_bytes(include_bytes!(
                "../icons/menu/arrow.clockwise.png"
            ))?),
            Some("CmdOrCtrl+R"),
        )?;
        #[cfg(not(target_os = "macos"))]
        let reload_folder = MenuItem::with_id(
            handle,
            RELOAD_FOLDER_ID,
            "Reload Folder",
            true,
            Some("CmdOrCtrl+R"),
        )?;
        #[cfg(target_os = "macos")]
        let settings = IconMenuItem::with_id(
            handle,
            SETTINGS_ID,
            "Settings...",
            true,
            Some(Image::from_bytes(include_bytes!("../icons/menu/gear.png"))?),
            Some("CmdOrCtrl+,"),
        )?;
        #[cfg(not(target_os = "macos"))]
        let settings = MenuItem::with_id(
            handle,
            SETTINGS_ID,
            "Settings...",
            true,
            Some("CmdOrCtrl+,"),
        )?;
        #[cfg(target_os = "macos")]
        let check_updates = IconMenuItem::with_id(
            handle,
            CHECK_UPDATES_ID,
            "Check for Updates…",
            true,
            Some(Image::from_bytes(include_bytes!(
                "../icons/menu/square.and.arrow.down.png"
            ))?),
            None::<&str>,
        )?;
        #[cfg(not(target_os = "macos"))]
        let check_updates = MenuItem::with_id(
            handle,
            CHECK_UPDATES_ID,
            "Check for Updates…",
            true,
            None::<&str>,
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
        file.prepend_items(&[
            &open_folder,
            &reload_folder,
            &trash_rejected,
            &PredefinedMenuItem::separator(handle)?,
        ])?;
        // On macOS both go in the app menu: Check for Updates joins About above
        // the default menu's first separator, since both are app-identity
        // items, and Settings gets a section of its own below it, as the
        // platform convention has it. Elsewhere they go at the end of File's
        // own items, above Close Window and Quit.
        #[cfg(target_os = "macos")]
        if let Some(MenuItemKind::Submenu(app)) = menu.items()?.into_iter().next() {
            app.insert_items(&[&check_updates], 1)?;
            app.insert_items(&[&settings, &PredefinedMenuItem::separator(handle)?], 3)?;
        }
        #[cfg(not(target_os = "macos"))]
        file.insert_items(
            &[
                &settings,
                &check_updates,
                &PredefinedMenuItem::separator(handle)?,
            ],
            2,
        )?;
        // `Help` may be missing from the default menu (Linux), in which case
        // it's created here; empty on macOS; holding About elsewhere, where
        // the separator keeps the two apart.
        let help = match submenu(&menu, "Help")? {
            Some(help) => help,
            None => {
                let help = Submenu::new(handle, "Help", true)?;
                menu.append(&help)?;
                help
            }
        };
        #[cfg(target_os = "macos")]
        let open_log_folder = IconMenuItem::with_id(
            handle,
            OPEN_LOG_FOLDER_ID,
            "Open Log Folder",
            true,
            Some(Image::from_bytes(include_bytes!(
                "../icons/menu/folder.png"
            ))?),
            None::<&str>,
        )?;
        #[cfg(not(target_os = "macos"))]
        let open_log_folder = MenuItem::with_id(
            handle,
            OPEN_LOG_FOLDER_ID,
            "Open Log Folder",
            true,
            None::<&str>,
        )?;
        if help.items()?.is_empty() {
            help.prepend(&open_log_folder)?;
        } else {
            help.prepend_items(&[&open_log_folder, &PredefinedMenuItem::separator(handle)?])?;
        }
        // `Edit` opens with the predefined Undo and Redo, which only act on
        // editable content (neither window has any) and would own Cmd+Z.
        if let Some(edit) = submenu(&menu, "Edit")? {
            for _ in 0..2 {
                if let Some(MenuItemKind::Predefined(_)) = edit.items()?.into_iter().next() {
                    edit.remove_at(0)?;
                }
            }
            #[cfg(target_os = "macos")]
            let undo = IconMenuItem::with_id(
                handle,
                UNDO_ID,
                "Undo",
                true,
                Some(Image::from_bytes(include_bytes!(
                    "../icons/menu/arrow.uturn.backward.png"
                ))?),
                undo_key,
            )?;
            #[cfg(not(target_os = "macos"))]
            let undo = MenuItem::with_id(handle, UNDO_ID, "Undo", true, undo_key)?;
            edit.insert(&undo, 0)?;
            #[cfg(target_os = "macos")]
            let redo = IconMenuItem::with_id(
                handle,
                REDO_ID,
                "Redo",
                true,
                Some(Image::from_bytes(include_bytes!(
                    "../icons/menu/arrow.uturn.forward.png"
                ))?),
                redo_key,
            )?;
            #[cfg(not(target_os = "macos"))]
            let redo = MenuItem::with_id(handle, REDO_ID, "Redo", true, redo_key)?;
            edit.insert(&redo, 1)?;
        }
        Ok(menu)
    }

    /// Make the keymap-derived accelerators match the keymap. muda's macOS
    /// `set_accelerator(None)` does not clear a key equivalent, so macOS
    /// replaces the whole menu. Elsewhere a runtime `set_menu` turns the
    /// Windows dark menu bar white, so an existing menu is patched in place.
    pub fn refresh(app: &AppHandle, keymap: &crate::shortcuts::Keymap) -> tauri::Result<()> {
        #[cfg(not(target_os = "macos"))]
        if let Some(menu) = app.menu() {
            for (id, action) in [
                (OPEN_FOLDER_ID, "open"),
                (UNDO_ID, "undo"),
                (REDO_ID, "redo"),
            ] {
                let item = menu
                    .items()?
                    .iter()
                    .filter_map(|kind| kind.as_submenu()?.get(id))
                    .find_map(|kind| kind.as_menuitem().cloned());
                let Some(item) = item else {
                    log::warn!("menu item {id} not found; accelerator not updated");
                    return Ok(());
                };
                item.set_accelerator(keymap.accelerator_for(action).as_deref())?;
            }
            return Ok(());
        }
        let menu = build(app, keymap)?;
        app.set_menu(menu)?;
        Ok(())
    }

    pub fn on_event(app: &AppHandle, event: MenuEvent) {
        // The frontend owns which folder is open, so it does the invoking.
        if event.id() == OPEN_FOLDER_ID {
            let _ = app.emit("open-folder", ());
        }
        if event.id() == RELOAD_FOLDER_ID {
            let _ = app.emit("reload-folder", ());
        }
        if event.id() == TRASH_REJECTED_ID {
            let _ = app.emit("trash-rejected", ());
        }
        if event.id() == UNDO_ID {
            let _ = app.emit("undo", ());
        }
        if event.id() == REDO_ID {
            let _ = app.emit("redo", ());
        }
        if event.id() == CHECK_UPDATES_ID {
            crate::update::spawn(app.clone(), true);
        }
        if event.id() == OPEN_LOG_FOLDER_ID {
            if let Err(e) = open_log_folder(app) {
                log::error!("failed to open the log folder: {e}");
            }
        }
        if event.id() == SETTINGS_ID {
            if let Err(e) = open_settings(app) {
                log::error!("failed to open the settings window: {e}");
            }
        }
    }

    /// Show the folder holding `Riffle.log` in the platform's file manager.
    fn open_log_folder(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let dir = app.path().app_log_dir()?;
        // The plugin creates the folder on its first write, which has happened
        // by the time a menu can be clicked; this only covers a log-less run.
        std::fs::create_dir_all(&dir)?;
        app.opener()
            .open_path(dir.to_string_lossy(), None::<&str>)?;
        Ok(())
    }

    /// Focus the settings window, or open it when it is not open yet.
    fn open_settings(app: &AppHandle) -> tauri::Result<()> {
        if let Some(window) = app.get_webview_window(super::SETTINGS_WINDOW) {
            return window.set_focus();
        }
        let window = WebviewWindowBuilder::new(
            app,
            super::SETTINGS_WINDOW,
            WebviewUrl::App("settings.html".into()),
        )
        .title("Settings")
        .inner_size(480.0, 640.0)
        .build()?;
        // On Windows and Linux a window built without `.menu()` inherits the
        // app-wide menu. Menu events arrive on the main thread, where
        // `remove_menu` runs inline, so it is gone before the first paint. On
        // macOS the menu is app-wide and `Window::remove_menu` is unsupported.
        #[cfg(not(target_os = "macos"))]
        window.remove_menu()?;
        #[cfg(target_os = "macos")]
        let _ = window;
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

/// Write a frontend timing line to `Riffle.log`, so it can be read without
/// DevTools.
#[tauri::command]
fn log_timing(line: String) {
    log::info!("{line}");
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

/// Save the format picked in the main window's first-launch dialog. Unlike
/// `set_sidecar_format` it saves an unchanged format too, so the dialog does
/// not come back on the next launch, and a failed save is returned so the
/// dialog stays up.
#[tauri::command]
async fn choose_sidecar_format(app: AppHandle, format: String) -> Result<(), String> {
    let format = SidecarFormat::from_setting(Some(&format));
    let unchanged = *index::lock(&app.state::<commands::AppSidecarFormat>().0) == format;
    let chosen = app.clone();
    tauri::async_runtime::spawn_blocking(move || commands::choose_sidecar_format(&chosen, format))
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
/// written. The writer retries it a few times with a growing delay, and the
/// judgment stays in the index to be written on the next open if all fail.
#[derive(Clone, serde::Serialize)]
struct SidecarError {
    path: String,
    message: String,
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
                // The plugin's default cap is 40 KB, which a single ~30s
                // scan can fill on its own: `folder_entries` is re-read
                // while a `scan-progress` stream leaves the focused row
                // missing, so its `open entries` line alone runs to ~300
                // lines of ~120 bytes. The timing lines are the whole point
                // of the file, so the cap is raised rather than the lines
                // thinned out.
                .max_file_size(1_000_000)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::new().build());
    builder
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
            let reader = index
                .as_ref()
                .map(|index| match index::Index::open_reader(&path) {
                    Ok(reader) => Arc::new(Mutex::new(reader)),
                    Err(e) => {
                        log::error!("failed to open the index reader at {}: {e}", path.display());
                        index.clone()
                    }
                });
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
            let (format, keymap, auto_advance, label_names, mcp_enabled) =
                commands::load_settings(app.handle());
            app.manage(TimingLogs(AtomicBool::new(false)));
            app.manage(commands::AppSidecarFormat(Mutex::new(format)));
            app.manage(commands::AppLabelNames(Mutex::new(label_names)));
            app.manage(commands::AppAutoAdvance(AtomicBool::new(auto_advance)));
            if let Err(e) = app_menu::refresh(app.handle(), &keymap) {
                log::error!("failed to set the app menu: {e}");
            }
            app.manage(commands::AppKeymap(Mutex::new(keymap)));
            app.manage(commands::AppSwitchLock(Mutex::new(())));
            app.manage(commands::AppWriter(writer));
            app.manage(commands::AppIndex(index));
            app.manage(commands::AppIndexReader(reader));
            app.manage(commands::Scans::default());
            app.manage(watch::Watch::spawn(app.handle().clone()));
            commands::spawn_eviction(app.handle().clone());
            app.manage(update::UpdateRun::default());
            update::spawn(app.handle().clone(), false);
            app.manage(mcp::AppMcp::default());
            // A failed bind (the port in use) is reported in the settings
            // window rather than stopping the app from launching.
            if mcp_enabled {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    mcp::switch(&handle, true).await;
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::pick_folder,
            commands::list_arw,
            commands::remember_folder,
            commands::last_folder,
            commands::sort_order,
            commands::set_sort_order,
            commands::preview,
            commands::preview_pixel_limit,
            commands::dropped_folder,
            commands::scan_folder,
            commands::start_scan,
            commands::folder_entries,
            commands::thumbnail,
            commands::metadata,
            commands::focus_crop,
            commands::set_rating,
            commands::sidecar_format,
            commands::sidecar_format_saved,
            commands::shortcuts,
            commands::add_shortcut_key,
            commands::remove_shortcut_key,
            commands::reset_shortcut,
            commands::reset_shortcuts,
            commands::auto_advance,
            commands::set_auto_advance,
            commands::mcp_enabled,
            commands::set_mcp_enabled,
            commands::label_names,
            commands::set_label_names,
            commands::scan_running,
            commands::index_size,
            commands::clear_index,
            commands::trash_rejected,
            set_sidecar_format,
            choose_sidecar_format,
            debug_build,
            timing_logs,
            set_timing_logs,
            log_timing
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // A judgment may still be inside the writer's debounce window
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
            // The folder watcher needs nothing here: it holds nothing
            // pending on disk, and its thread ends with the app.
            if let RunEvent::ExitRequested { .. } = event {
                if let Some(writer) = &app.state::<commands::AppWriter>().0 {
                    writer.flush(sidecar::DRAIN_TIMEOUT);
                }
                update::install_pending(app);
                tauri::async_runtime::block_on(mcp::stop(&app.state::<mcp::AppMcp>()));
            }
        });
}
