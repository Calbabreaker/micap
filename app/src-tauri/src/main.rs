// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    mycap_server::setup_log();
    tauri::async_runtime::spawn(mycap_server::start_server());

    tauri::Builder::default()
        .setup(|app| {
            let webview = app.get_webview_window("main").unwrap();
            webview.eval(&format!(
                "location.search='?websocket_port={}'",
                mycap_server::WEBSOCKET_PORT.to_string()
            ));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
