#[tauri::command]
pub fn notify(app: tauri::AppHandle, title: String, body: String) {
    if crate::settings::load(&app).notifications {
        crate::notify::show(&app, &title, &body);
    }
}

#[tauri::command]
pub fn set_unread(app: tauri::AppHandle, title: String) {
    let count = crate::unread::parse_unread(&title);
    crate::tray::update_badge(&app, count);
}

use crate::settings::Settings;

/// The settings commands must NOT be reachable from the remote WhatsApp page
/// (window label "main"). Only local windows (e.g. "settings") may use them.
/// Tauri injects the calling `window`; the remote page cannot forge its label.
fn is_remote(window: &tauri::Window) -> bool {
    window.label() == "main"
}

#[tauri::command]
pub fn get_settings(window: tauri::Window, app: tauri::AppHandle) -> Result<Settings, String> {
    if is_remote(&window) {
        return Err("forbidden".into());
    }
    Ok(crate::settings::load(&app))
}

#[tauri::command]
pub fn set_settings(window: tauri::Window, app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    if is_remote(&window) {
        return Err("forbidden".into());
    }
    crate::settings::save(&app, &settings).map_err(|e| e.to_string())?;
    crate::settings::apply(&app, &settings);
    Ok(())
}

#[tauri::command]
pub fn open_settings(window: tauri::Window, app: tauri::AppHandle) {
    if is_remote(&window) {
        return;
    }
    crate::window::open_settings_window(&app);
}
