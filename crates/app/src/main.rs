#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod index;

use std::sync::{Arc, Mutex};

use tauri::Manager;

#[tauri::command]
fn ping() -> String {
    "pong".to_string()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let path = app.path().app_cache_dir()?.join("index.sqlite");
            // The index is a thumbnail/metadata cache, not required data: an
            // unwritable cache dir degrades to "no thumbnails" rather than
            // stopping the app from launching.
            let index = match index::Index::open(&path) {
                Ok(index) => Some(Arc::new(Mutex::new(index))),
                Err(e) => {
                    eprintln!("failed to open the index cache at {}: {e}", path.display());
                    None
                }
            };
            app.manage(commands::AppIndex(index));
            app.manage(commands::Running::default());
            app.manage(commands::Pending::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            commands::pick_folder,
            commands::list_arw,
            commands::preview,
            commands::scan_folder,
            commands::start_scan,
            commands::folder_entries,
            commands::thumbnail
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
