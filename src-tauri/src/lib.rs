mod window;
mod unread;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            window::create_main_window(app.handle(), false)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running whatRust");
}
