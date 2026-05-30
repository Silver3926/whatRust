use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Recent desktop Chrome UA. WhatsApp Web rejects the default WebKitGTK/Safari UA.
/// Bump the major version occasionally.
pub const CHROME_UA: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub fn create_main_window(app: &AppHandle, start_hidden: bool) -> tauri::Result<WebviewWindow> {
    let url = "https://web.whatsapp.com/".parse().expect("valid url");
    WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
        .title("whatRust")
        .inner_size(1100.0, 800.0)
        .min_inner_size(560.0, 480.0)
        .user_agent(CHROME_UA)
        .visible(!start_hidden)
        .build()
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Opens (or focuses) the local settings window. Fully wired in Task 9.
pub fn open_settings_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("whatRust — Settings")
        .inner_size(440.0, 560.0)
        .resizable(false)
        .build();
}
