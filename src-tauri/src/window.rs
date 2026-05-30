use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Recent desktop Chrome UA. WhatsApp Web rejects the default WebKitGTK/Safari UA.
/// Bump the major version occasionally.
pub const CHROME_UA: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

const BRIDGE_JS: &str = include_str!("../resources/bridge.js");

pub fn create_main_window(app: &AppHandle, start_hidden: bool) -> tauri::Result<WebviewWindow> {
    let url = "https://web.whatsapp.com/".parse().expect("valid url");
    let win = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
        .title("whatRust")
        .inner_size(1100.0, 800.0)
        .min_inner_size(560.0, 480.0)
        .user_agent(CHROME_UA)
        .initialization_script(BRIDGE_JS)
        .visible(!start_hidden)
        .build()?;

    let app_handle = app.clone();
    win.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            // Re-read settings so the toggle takes effect without a restart.
            if crate::settings::load(&app_handle).close_to_tray {
                if let Some(w) = app_handle.get_webview_window("main") {
                    let _ = w.hide();
                }
                api.prevent_close();
            }
        }
    });

    enable_webview_media(&win);
    Ok(win)
}

/// Grant microphone/camera + WebRTC to WhatsApp Web. The system webview denies
/// getUserMedia by default, which blocks voice messages and calls (the
/// "Allow microphone" prompt). We enable the media settings and auto-approve
/// the webview's permission requests for the WhatsApp window.
fn enable_webview_media(win: &WebviewWindow) {
    #[cfg(target_os = "linux")]
    {
        use webkit2gtk::glib::prelude::ObjectExt;
        use webkit2gtk::{PermissionRequestExt, WebViewExt};
        let _ = win.with_webview(|webview| {
            let wv = webview.inner();
            if let Some(settings) = WebViewExt::settings(&wv) {
                settings.set_property("enable-media-stream", true);
                settings.set_property("enable-mediasource", true);
                settings.set_property("enable-webrtc", true);
                settings.set_property("enable-encrypted-media", true);
            }
            wv.connect_permission_request(|_wv, req| {
                req.allow();
                true
            });
        });
    }
    #[cfg(not(target_os = "linux"))]
    let _ = win;
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Opens (or focuses) the local settings window.
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
