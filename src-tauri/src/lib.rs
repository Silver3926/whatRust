mod window;
mod unread;
mod settings;
mod tray;
mod commands;
mod notify;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // single-instance MUST be registered first.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show_main(app);
        }));
    }

    builder = builder.plugin(tauri_plugin_notification::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_window_state::Builder::default().build());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            commands::notify,
            commands::set_unread,
        ])
        .setup(|app| {
            let handle = app.handle();
            let s = settings::load(handle);
            let args: Vec<String> = std::env::args().collect();
            let start_hidden = s.start_minimized || args.iter().any(|a| a == "--minimized");

            window::create_main_window(handle, start_hidden)?;
            tray::setup(handle)?;
            settings::apply(handle, &s);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running whatRust");
}
