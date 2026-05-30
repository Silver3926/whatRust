use tauri::AppHandle;

#[tauri::command]
pub fn notify(app: AppHandle, title: String, body: String) {
    if crate::settings::load(&app).notifications {
        crate::notify::show(&app, &title, &body);
    }
}

#[tauri::command]
pub fn set_unread(app: AppHandle, title: String) {
    let count = crate::unread::parse_unread(&title);
    crate::tray::update_badge(&app, count);
}

use crate::settings::Settings;

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    crate::settings::load(&app)
}

#[tauri::command]
pub fn set_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    crate::settings::save(&app, &settings).map_err(|e| e.to_string())?;
    crate::settings::apply(&app, &settings);
    Ok(())
}

#[tauri::command]
pub fn open_settings(app: AppHandle) {
    crate::window::open_settings_window(&app);
}
