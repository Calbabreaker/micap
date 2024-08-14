// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    micap_server::setup_log();
    tauri::async_runtime::spawn(async {
        if let Err(error) = micap_server::start_server().await {
            log::error!("Server error: {error:?}");
            if std::env::var("RUST_BACKTRACE") != Ok("1".to_string()) {
                log::error!(
                    "Note: set environment variable RUST_BACKTRACE=1 to see the error backtrace"
                )
            }

            std::process::exit(1);
        }
    });

    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
