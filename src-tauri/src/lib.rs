mod accounts;
mod window;
mod unread;
mod settings;
mod tray;
mod commands;
mod notify;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // single-instance MUST be registered first.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // A 2nd launch normally raises the active account window — but NOT an
            // autostart relaunch carrying --minimized (keep it hidden in the tray).
            // show_main is a show_active shim (correction #5).
            if !args.iter().any(|a| a == "--minimized") {
                window::show_main(app);
            }
        }));
    }

    builder = builder.plugin(tauri_plugin_notification::init());

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(
                // Restore size/position, but NOT visibility — otherwise the plugin
                // force-shows the window on launch and defeats start-minimized / --minimized.
                tauri_plugin_window_state::Builder::default()
                    .with_state_flags(
                        tauri_plugin_window_state::StateFlags::all()
                            & !tauri_plugin_window_state::StateFlags::VISIBLE,
                    )
                    .build(),
            )
            .plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(|app, _shortcut, event| {
                        if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                            // Toggle the active account window.
                            let label = app
                                .try_state::<accounts::ActiveAccount>()
                                .map(|a| a.lock().unwrap().clone());
                            let visible = label
                                .as_ref()
                                .and_then(|l| app.get_webview_window(l))
                                .map(|w| w.is_visible().unwrap_or(false));
                            match (label, visible) {
                                (Some(l), Some(true)) => {
                                    if let Some(w) = app.get_webview_window(&l) {
                                        let _ = w.hide();
                                    }
                                }
                                _ => window::show_active(app),
                            }
                        }
                    })
                    .build(),
            )
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec!["--minimized"]),
            ));
    }

    builder
        .manage(accounts::UnreadMap::default())
        .manage(accounts::ActiveAccount::new("wa-default".into()))
        .invoke_handler(tauri::generate_handler![
            commands::notify,
            commands::set_unread,
            commands::get_settings,
            commands::set_settings,
            commands::open_settings,
            commands::list_accounts,
            commands::add_account,
            commands::remove_account,
            commands::rename_account,
            commands::open_account,
        ])
        .setup(|app| {
            let handle = app.handle();
            let s = settings::load(handle);
            let args: Vec<String> = std::env::args().collect();
            let start_hidden = s.start_minimized || args.iter().any(|a| a == "--minimized");

            // Load accounts (seeds a single `default` on first run / corrupt file).
            let mut f = accounts::load(handle);

            // Backfill a persisted store_uuid for any non-default account missing one
            // (older state predating multi-account). Save only if something changed.
            let mut changed = false;
            for a in f.accounts.iter_mut() {
                if a.id != "default" && a.store_uuid.is_none() {
                    a.store_uuid = Some(accounts::gen_store_uuid());
                    changed = true;
                }
            }
            if changed {
                let _ = accounts::save(handle, &f);
            }

            // Open every account window so each one receives messages/notifications.
            for a in &f.accounts {
                window::open_account_window(handle, a, start_hidden)?;
            }

            tray::setup(handle)?;
            tray::rebuild_menu(handle);
            settings::apply(handle, &s);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building whatRust")
        .run(|_app_handle, _event| {
            // macOS: clicking the dock icon after hide-to-tray re-shows the window
            // (otherwise the app is only reachable via the menu-bar tray icon).
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { has_visible_windows, .. } = &_event {
                if !*has_visible_windows {
                    window::show_main(_app_handle);
                }
            }
        });
}
