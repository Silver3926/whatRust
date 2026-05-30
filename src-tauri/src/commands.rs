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
