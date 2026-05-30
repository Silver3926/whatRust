#[derive(Debug, PartialEq, Eq)]
pub enum BadgeState {
    Clear,
    Count(u32),
}

/// Decide what the tray should show for a given unread count.
pub fn badge_state(count: u32) -> BadgeState {
    if count == 0 {
        BadgeState::Clear
    } else {
        BadgeState::Count(count)
    }
}

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

const ICON_NORMAL: &[u8] = include_bytes!("../icons/tray.png");
const ICON_UNREAD: &[u8] = include_bytes!("../icons/tray-unread.png");

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItemBuilder::with_id("show", "Show / Hide").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
    let reload = MenuItemBuilder::with_id("reload", "Reload").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&show, &settings, &reload, &quit])
        .build()?;

    TrayIconBuilder::with_id("main-tray")
        .icon(Image::from_bytes(ICON_NORMAL)?)
        .tooltip("WhatsApp")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => toggle(app),
            "settings" => crate::window::open_settings_window(app),
            "reload" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.eval("window.location.reload()");
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Note: Linux does not deliver left-click tray events; use the menu there.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn toggle(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            crate::window::show_main(app);
        }
    }
}

/// Update the tray to reflect the current unread count.
pub fn update_badge(app: &AppHandle, count: u32) {
    let Some(tray) = app.tray_by_id("main-tray") else {
        return;
    };
    match badge_state(count) {
        BadgeState::Clear => {
            let _ = tray.set_title(None::<String>);
            let _ = tray.set_tooltip(Some("WhatsApp"));
            if let Ok(icon) = Image::from_bytes(ICON_NORMAL) {
                let _ = tray.set_icon(Some(icon));
            }
        }
        BadgeState::Count(c) => {
            let _ = tray.set_title(Some(c.to_string()));
            let _ = tray.set_tooltip(Some(format!("{c} unread — WhatsApp")));
            if let Ok(icon) = Image::from_bytes(ICON_UNREAD) {
                let _ = tray.set_icon(Some(icon));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{badge_state, BadgeState};

    #[test]
    fn zero_is_clear() {
        assert_eq!(badge_state(0), BadgeState::Clear);
    }

    #[test]
    fn positive_is_count() {
        assert_eq!(badge_state(1), BadgeState::Count(1));
        assert_eq!(badge_state(42), BadgeState::Count(42));
    }
}
