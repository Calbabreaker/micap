use tauri::Manager;

pub struct AppState {
    pub window: tauri::WebviewWindow,
    pub toggle_item: tauri::menu::MenuItem<tauri::Wry>,
}

impl AppState {
    pub fn new(app: &tauri::App) -> Self {
        Self {
            window: app.get_webview_window("main").unwrap(),
            toggle_item: tauri::menu::MenuItem::with_id(app, "toggle", "Show", true, None::<&str>)
                .unwrap(),
        }
    }

    pub fn toggle_visible(&self) -> tauri::Result<()> {
        self.set_visible(!self.window.is_visible()?)
    }

    pub fn set_visible(&self, visible: bool) -> tauri::Result<()> {
        if visible {
            self.window.show()?;
            self.toggle_item.set_text("Hide")
        } else {
            self.window.hide()?;
            self.toggle_item.set_text("Show")
        }
    }
}
