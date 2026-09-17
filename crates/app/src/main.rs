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
            let index = index::Index::open(&path)?;
            app.manage(commands::AppIndex(Arc::new(Mutex::new(index))));
            app.manage(commands::Scans::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            commands::pick_folder,
            commands::list_arw,
            commands::preview,
            commands::scan_folder,
            commands::folder_entries,
            commands::thumbnail
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
