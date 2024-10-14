// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use state::AppState;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

mod state;

fn main() {
    // If LANG not set to en, it shows blank window for some reason
    std::env::set_var("LANG", "en");

    micap_server::setup_log();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(AppState::new(app));

    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        handle_window_events(app);
        create_system_tray(app)?;
    }

    // Start server
    let dialog = app.dialog().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = start_server().await {
            let note = if std::env::var("RUST_BACKTRACE") != Ok("1".to_string()) {
                "Note: set environment variable RUST_BACKTRACE=1 to see the error backtrace"
            } else {
                ""
            };

            let description = format!("{error:?}\n\n{note}");
            log::error!("Server error: {description}");
            dialog
                .message(description)
                .title("Server error")
                .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                .buttons(tauri_plugin_dialog::MessageDialogButtons::Ok)
                .blocking_show();

            std::process::exit(1);
        }
    });

    Ok(())
}

// Start two nested tasks to listen for panics
async fn start_server() -> anyhow::Result<()> {
    tauri::async_runtime::spawn(async { micap_server::start_server().await }).await??;
    Ok(())
}

fn handle_window_events(app: &tauri::App) {
    let state = app.state::<AppState>();
    let w = state.window.clone();
    state.window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            w.state::<AppState>().set_visible(false).unwrap();
        }
    });
}

fn create_system_tray(app: &tauri::App) -> tauri::Result<tauri::tray::TrayIcon> {
    let state = app.state::<AppState>();
    let quit_item = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = tauri::menu::Menu::with_items(app, &[&quit_item, &state.toggle_item])?;

    tauri::tray::TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "toggle" => {
                let state = app.state::<AppState>();
                state.toggle_visible().unwrap();
            }
            _ => {
                log::error!("Unknown menu id {:?}", event.id);
            }
        })
        .build(app)
}
