mod window;
mod unread;
mod settings;
mod tray;
mod commands;
mod notify;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::notify,
            commands::set_unread,
        ])
        .setup(|app| {
            window::create_main_window(app.handle(), false)?;
            tray::setup(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running whatRust");
}
