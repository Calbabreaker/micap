// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

const RESTART_WAIT_TIME: Duration = Duration::from_secs(3);

fn main() {
    mycap_server::setup_log();
    tauri::async_runtime::spawn(async {
        loop {
            if let Err(error) = mycap_server::start_server().await {
                log::error!("Server error: {error:?}");
                log::error!("Restarting in {RESTART_WAIT_TIME:?}...");
                tokio::time::sleep(RESTART_WAIT_TIME).await;
            }
        }
    });

    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
