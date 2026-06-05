// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_app_lib::FileResult;

#[tauri::command]
async fn check_folder(
    folder: String,
    expected_encoding: String,
    expected_newline: String,
    exclude_dirs: Vec<String>,
    exclude_exts: Vec<String>,
) -> Result<Vec<FileResult>, String> {
    tauri_app_lib::check_folder_logic(folder, expected_encoding, expected_newline, exclude_dirs, exclude_exts).await
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![check_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
