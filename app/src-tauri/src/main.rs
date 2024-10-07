// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // If LANG not set to en, it shows blank window for some reason
    std::env::set_var("LANG", "en");

    micap_server::setup_log();
    tauri::async_runtime::spawn(async {
        if let Err(error) = micap_server::start_server().await {
            let note = if std::env::var("RUST_BACKTRACE") != Ok("1".to_string()) {
                "Note: set environment variable RUST_BACKTRACE=1 to see the error backtrace"
            } else {
                ""
            };

            log::error!("Server error: {error:?}\n{note}");

            std::process::exit(1);
        }
    });

    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
