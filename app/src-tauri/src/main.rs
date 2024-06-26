// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    mycap_server::setup_log();
    tauri::async_runtime::spawn(async {
        if let Err(error) = mycap_server::start_server().await {
            log::error!("Server error: {error:?}");
        }
    });

    tauri::Builder::default()
        .setup(|app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
