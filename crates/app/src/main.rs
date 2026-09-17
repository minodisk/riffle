#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

#[tauri::command]
fn ping() -> String {
    "pong".to_string()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            ping,
            commands::pick_folder,
            commands::list_arw,
            commands::preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
