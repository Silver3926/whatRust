use crate::accounts::{self, ActiveAccount, UnreadMap};
use crate::settings::Settings;
use serde::Serialize;
use tauri::Manager;

/// Account-management commands must NOT be reachable from a remote WhatsApp page.
/// Account windows carry the `wa-<id>` label; the trusted local `settings` window
/// does not. Tauri injects the calling `window`; the remote page cannot forge its
/// label. WhatsApp pages keep `notify`/`set_unread` (they need them) but are denied
/// every account-management command.
fn is_remote(window: &tauri::Window) -> bool {
    is_remote_label(window.label())
}

/// Pure predicate behind `is_remote`, broken out so it can be unit-tested without a
/// live `tauri::Window`.
fn is_remote_label(label: &str) -> bool {
    label.starts_with("wa-")
}

#[tauri::command]
pub fn notify(window: tauri::Window, app: tauri::AppHandle, title: String, body: String) {
    if !crate::settings::load(&app).notifications {
        return;
    }
    // Prefix the account name when more than one account exists, so notifications
    // are attributable (e.g. "Work: New message").
    let f = accounts::load(&app);
    let title = if f.accounts.len() > 1 {
        if let Some(id) = accounts::id_from_label(window.label()) {
            if let Some(acct) = f.accounts.iter().find(|a| a.id == id) {
                format!("{}: {}", acct.name, title)
            } else {
                title
            }
        } else {
            title
        }
    } else {
        title
    };
    crate::notify::show(&app, &title, &body);
}

#[tauri::command]
pub fn set_unread(window: tauri::Window, app: tauri::AppHandle, title: String) {
    let count = crate::unread::parse_unread(&title);
    let Some(id) = accounts::id_from_label(window.label()) else {
        return;
    };

    // Update the per-account count and compute the aggregate, then drop all
    // UnreadMap guards BEFORE calling tray::rebuild_menu (which re-locks the map)
    // to avoid a deadlock.
    let total = {
        let state = app.state::<UnreadMap>();
        let mut map = state.lock().unwrap();
        map.insert(id.to_string(), count);
        accounts::aggregate_unread(&map)
    };

    crate::tray::update_badge(&app, total);
    crate::tray::rebuild_menu(&app);
}

#[tauri::command]
pub fn get_settings(window: tauri::Window, app: tauri::AppHandle) -> Result<Settings, String> {
    if is_remote(&window) {
        return Err("forbidden".into());
    }
    Ok(crate::settings::load(&app))
}

#[tauri::command]
pub fn set_settings(
    window: tauri::Window,
    app: tauri::AppHandle,
    settings: Settings,
) -> Result<(), String> {
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

// ---------------------------------------------------------------------------
// Account-management commands (local-only).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct AccountView {
    pub id: String,
    pub name: String,
    pub order: u32,
    pub unread: u32,
    pub open: bool,
}

fn account_views(app: &tauri::AppHandle) -> Vec<AccountView> {
    let f = accounts::load(app);
    let map = app.state::<UnreadMap>();
    let map = map.lock().unwrap();
    let mut views: Vec<AccountView> = f
        .accounts
        .iter()
        .map(|a| AccountView {
            id: a.id.clone(),
            name: a.name.clone(),
            order: a.order,
            unread: map.get(&a.id).copied().unwrap_or(0),
            open: app
                .get_webview_window(&accounts::window_label(&a.id))
                .is_some(),
        })
        .collect();
    views.sort_by_key(|v| v.order);
    views
}

#[tauri::command]
pub fn list_accounts(
    window: tauri::Window,
    app: tauri::AppHandle,
) -> Result<Vec<AccountView>, String> {
    if is_remote(&window) {
        return Err("forbidden".into());
    }
    Ok(account_views(&app))
}

#[tauri::command]
pub fn add_account(
    window: tauri::Window,
    app: tauri::AppHandle,
    name: String,
) -> Result<AccountView, String> {
    if is_remote(&window) {
        return Err("forbidden".into());
    }
    // macOS < 14 cannot isolate additional accounts (no data_store_identifier).
    crate::window::ensure_isolation_supported()?;

    let name = name.trim();
    if name.is_empty() {
        return Err("account name cannot be empty".into());
    }

    let mut f = accounts::load(&app);
    let acct = accounts::add(&mut f, name);
    accounts::save(&app, &f).map_err(|e| e.to_string())?;

    crate::window::open_account_window(&app, &acct, false).map_err(|e| e.to_string())?;
    crate::tray::rebuild_menu(&app);

    Ok(AccountView {
        id: acct.id,
        name: acct.name,
        order: acct.order,
        unread: 0,
        open: true,
    })
}

#[tauri::command]
pub fn remove_account(
    window: tauri::Window,
    app: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    if is_remote(&window) {
        return Err("forbidden".into());
    }

    let mut f = accounts::load(&app);
    let removed = accounts::remove(&mut f, &id)?;
    accounts::save(&app, &f).map_err(|e| e.to_string())?;

    // Close the window if open.
    if let Some(w) = app.get_webview_window(&accounts::window_label(&removed.id)) {
        let _ = w.destroy();
    }
    // Drop the per-account unread count.
    {
        let state = app.state::<UnreadMap>();
        let mut map = state.lock().unwrap();
        map.remove(&removed.id);
    }
    accounts::delete_profile(&app, &removed.id);

    // Recompute the aggregate badge and the menu.
    let total = {
        let state = app.state::<UnreadMap>();
        let map = state.lock().unwrap();
        accounts::aggregate_unread(&map)
    };
    crate::tray::update_badge(&app, total);
    crate::tray::rebuild_menu(&app);
    Ok(())
}

#[tauri::command]
pub fn rename_account(
    window: tauri::Window,
    app: tauri::AppHandle,
    id: String,
    name: String,
) -> Result<(), String> {
    if is_remote(&window) {
        return Err("forbidden".into());
    }
    let name = name.trim();
    if name.is_empty() {
        return Err("account name cannot be empty".into());
    }

    let mut f = accounts::load(&app);
    accounts::rename(&mut f, &id, name)?;
    accounts::save(&app, &f).map_err(|e| e.to_string())?;

    if let Some(w) = app.get_webview_window(&accounts::window_label(&id)) {
        let _ = w.set_title(&format!("whatRust — {name}"));
    }
    crate::tray::rebuild_menu(&app);
    Ok(())
}

#[tauri::command]
pub fn open_account(window: tauri::Window, app: tauri::AppHandle, id: String) -> Result<(), String> {
    if is_remote(&window) {
        return Err("forbidden".into());
    }
    let f = accounts::load(&app);
    let Some(acct) = f.accounts.iter().find(|a| a.id == id) else {
        return Err(format!("unknown account: {id}"));
    };
    // Open the window if it was closed, then show + focus it.
    if app
        .get_webview_window(&accounts::window_label(&acct.id))
        .is_none()
    {
        crate::window::open_account_window(&app, acct, false).map_err(|e| e.to_string())?;
    }
    crate::window::show_account(&app, &accounts::window_label(&acct.id));
    // Track as the active account.
    if let Some(active) = app.try_state::<ActiveAccount>() {
        *active.lock().unwrap() = accounts::window_label(&acct.id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::is_remote_label;

    #[test]
    fn is_remote_wa_prefix_is_true() {
        assert!(is_remote_label("wa-default"));
    }

    #[test]
    fn is_remote_wa_acct_is_true() {
        assert!(is_remote_label("wa-acct-2"));
    }

    #[test]
    fn is_remote_settings_is_false() {
        assert!(!is_remote_label("settings"));
    }
}
