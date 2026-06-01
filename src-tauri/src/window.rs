use crate::accounts::{self, Account, ActiveAccount};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Recent desktop Chrome UA. WhatsApp Web rejects the default WebKitGTK/Safari UA.
/// Bump the major version occasionally.
pub const CHROME_UA: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

const BRIDGE_JS: &str = include_str!("../resources/bridge.js");
const APP_ICON: &[u8] = include_bytes!("../icons/128x128.png");

/// Open (or focus, if it already exists) the window for `account`. The label is
/// `wa-<id>`; everything the single-account window carried is preserved (Chrome UA,
/// `bridge.js`, app icon, sizes, close-to-tray), plus per-account session isolation
/// for non-default accounts.
pub fn open_account_window(
    app: &AppHandle,
    account: &Account,
    start_hidden: bool,
) -> tauri::Result<WebviewWindow> {
    let label = accounts::window_label(&account.id);

    // Reuse the existing window if it is already open.
    if let Some(w) = app.get_webview_window(&label) {
        return Ok(w);
    }

    let url = "https://web.whatsapp.com/".parse().expect("valid url");
    let icon = tauri::image::Image::from_bytes(APP_ICON)?;

    let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(url))
        .title(format!("whatRust — {}", account.name))
        .inner_size(1100.0, 800.0)
        .min_inner_size(560.0, 480.0)
        .icon(icon)?
        .user_agent(CHROME_UA)
        .initialization_script(BRIDGE_JS)
        .visible(!start_hidden);

    let builder = apply_isolation(builder, account, app);
    let win = builder.build()?;

    // Close-to-tray (reads the live setting so the toggle takes effect without a restart).
    let app_handle = app.clone();
    let label_for_close = label.clone();
    win.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            if crate::settings::load(&app_handle).close_to_tray {
                let lc = crate::applock::load(&app_handle);
                if lc.is_active() && lc.lock_on_hide {
                    crate::lock::lock_now(&app_handle);
                } else if let Some(w) = app_handle.get_webview_window(&label_for_close) {
                    let _ = w.hide();
                }
                api.prevent_close();
            }
        }
    });

    register_focus_listener(app, &win);
    enable_webview_media(&win);
    Ok(win)
}

/// Track the last-focused account window in `ActiveAccount`. Registered once per
/// window inside `open_account_window`, so startup *and* dynamically-added windows
/// get it exactly once.
fn register_focus_listener(app: &AppHandle, win: &WebviewWindow) {
    let app_handle = app.clone();
    let label = win.label().to_string();
    win.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(true) = event {
            if let Some(active) = app_handle.try_state::<ActiveAccount>() {
                *active.lock().unwrap() = label.clone();
            }
        }
    });
}

/// Apply per-account session isolation to a window builder.
///
/// The `default` account uses the default webview store (no override) so the
/// pre-multi-account login is preserved. Additional accounts get an isolated store:
/// `data_directory` on Linux/Windows, `data_store_identifier` (the persisted v4 UUID)
/// on macOS. `data_directory`/`data_store_identifier` compile on every platform in
/// tauri; only *which* one is called is cfg-gated, with a `compile_error!()` catch-all
/// so a future platform can't silently skip isolation.
fn apply_isolation<'a>(
    builder: WebviewWindowBuilder<'a, tauri::Wry, tauri::AppHandle<tauri::Wry>>,
    account: &Account,
    app: &AppHandle,
) -> WebviewWindowBuilder<'a, tauri::Wry, tauri::AppHandle<tauri::Wry>> {
    // The default account always uses the shared default store.
    if account.id == "default" {
        return builder;
    }

    #[cfg(any(target_os = "linux", windows))]
    {
        let _ = app;
        if let Ok(dir) = accounts::profile_dir(app, &account.id) {
            let _ = std::fs::create_dir_all(&dir);
            return builder.data_directory(dir);
        }
        builder
    }
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        // Non-default accounts carry a persisted, non-nil v4 UUID. Fall back to a
        // freshly generated non-nil UUID rather than risk the nil-UUID exception.
        let uuid = account
            .store_uuid
            .unwrap_or_else(accounts::gen_store_uuid);
        builder.data_store_identifier(uuid)
    }
    #[cfg(not(any(target_os = "linux", windows, target_os = "macos")))]
    {
        let _ = (builder, account, app);
        compile_error!("per-account session isolation is not implemented for this platform");
    }
}

/// Whether the platform can isolate additional accounts. macOS needs >= 14 for
/// `data_store_identifier`. Linux/Windows always can. Returns an `Err` message
/// suitable for surfacing in the Accounts UI.
pub fn ensure_isolation_supported() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use objc2_foundation::{NSOperatingSystemVersion, NSProcessInfo};
        // data_store_identifier requires macOS >= 14.
        let required = NSOperatingSystemVersion {
            majorVersion: 14,
            minorVersion: 0,
            patchVersion: 0,
        };
        let ok = NSProcessInfo::processInfo().isOperatingSystemAtLeastVersion(required);
        if ok {
            Ok(())
        } else {
            Err("Multiple accounts require macOS 14 or newer.".into())
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
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
    #[cfg(target_os = "windows")]
    {
        let _ = win.with_webview(enable_media_windows);
    }
    #[cfg(target_os = "macos")]
    {
        // wry already installs a WKUIDelegate that auto-grants requestMediaCapturePermission;
        // mic/camera are gated only by the Info.plist usage-description keys (see src-tauri/Info.plist).
        let _ = win;
    }
}

/// Windows (WebView2): auto-allow microphone/camera permission requests so WhatsApp
/// voice messages and calls work without a prompt (and can't be wedged by a prior "Block").
#[cfg(target_os = "windows")]
fn enable_media_windows(webview: tauri::webview::PlatformWebview) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2, ICoreWebView2PermissionRequestedEventArgs, COREWEBVIEW2_PERMISSION_KIND,
        COREWEBVIEW2_PERMISSION_KIND_CAMERA, COREWEBVIEW2_PERMISSION_KIND_MICROPHONE,
        COREWEBVIEW2_PERMISSION_STATE_ALLOW,
    };
    use webview2_com::PermissionRequestedEventHandler;

    // SAFETY: with_webview runs on the UI thread where the WebView2 controller lives;
    // these are standard WebView2 COM calls.
    unsafe {
        let controller = webview.controller();
        let core: ICoreWebView2 = match controller.CoreWebView2() {
            Ok(c) => c,
            Err(_) => return,
        };
        let handler = PermissionRequestedEventHandler::create(Box::new(
            move |_wv: Option<ICoreWebView2>,
                  args: Option<ICoreWebView2PermissionRequestedEventArgs>|
                  -> windows_core::Result<()> {
                if let Some(args) = args {
                    let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
                    args.PermissionKind(&mut kind)?;
                    if kind == COREWEBVIEW2_PERMISSION_KIND_MICROPHONE
                        || kind == COREWEBVIEW2_PERMISSION_KIND_CAMERA
                    {
                        args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?;
                    }
                }
                Ok(())
            },
        ));
        let mut token: i64 = 0;
        let _ = core.add_PermissionRequested(&handler, &mut token);
    }
}

/// Show + unminimize + focus an account window by its `wa-<id>` label.
pub fn show_account(app: &AppHandle, label: &str) {
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Show the active (last-focused) account window. Falls back to the first existing
/// account window, then the settings window.
pub fn show_active(app: &AppHandle) {
    // If the app is locked, any "reveal" request shows the lock screen, never an
    // account window. Covers tray click, global shortcut, single-instance, macOS Reopen.
    if !crate::lock::is_unlocked(app) {
        crate::lock::show_lock_window(app);
        return;
    }
    if let Some(active) = app.try_state::<ActiveAccount>() {
        let label = active.lock().unwrap().clone();
        if app.get_webview_window(&label).is_some() {
            show_account(app, &label);
            return;
        }
    }
    // Fall back to any account window.
    if let Some(label) = app
        .webview_windows()
        .keys()
        .find(|l| l.starts_with("wa-"))
        .cloned()
    {
        show_account(app, &label);
        return;
    }
    // Last resort: the settings window.
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// Backwards-compatible shim: single-instance + macOS Reopen call this; it now
/// targets the active account.
pub fn show_main(app: &AppHandle) {
    show_active(app);
}

/// What a toggle should do given the active window's visibility.
#[derive(Debug, PartialEq, Eq)]
pub enum ToggleAct {
    Hide,
    Show,
}

/// Pure toggle decision: a visible active window is hidden; anything else (a hidden
/// window, or no active window at all → `None`) is shown.
pub fn toggle_decision(active_visible: Option<bool>) -> ToggleAct {
    match active_visible {
        Some(true) => ToggleAct::Hide,
        _ => ToggleAct::Show,
    }
}

/// Toggle the active account window: hide it if visible, otherwise show + focus it.
/// The "show" path goes through `show_active`, which defers to the lock screen when
/// the app is locked — so a toggle (e.g. an OS-bound `whatrust --toggle` on Wayland,
/// where in-process global hotkeys can't fire) can NEVER reveal an account window
/// while locked. The "hide" path only triggers when an account window is visible,
/// which cannot happen while locked.
pub fn toggle_active(app: &AppHandle) {
    let label = app
        .try_state::<ActiveAccount>()
        .map(|a| a.lock().unwrap().clone());
    let visible = label
        .as_ref()
        .and_then(|l| app.get_webview_window(l))
        .map(|w| w.is_visible().unwrap_or(false));
    match toggle_decision(visible) {
        ToggleAct::Hide => {
            if let Some(l) = label {
                if let Some(w) = app.get_webview_window(&l) {
                    let _ = w.hide();
                }
            }
        }
        ToggleAct::Show => show_active(app),
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
        .inner_size(440.0, 680.0)
        .resizable(false)
        .build();
}

#[cfg(test)]
mod tests {
    use super::{toggle_decision, ToggleAct};

    #[test]
    fn visible_active_window_is_hidden() {
        assert_eq!(toggle_decision(Some(true)), ToggleAct::Hide);
    }

    #[test]
    fn hidden_active_window_is_shown() {
        assert_eq!(toggle_decision(Some(false)), ToggleAct::Show);
    }

    #[test]
    fn no_active_window_is_shown() {
        assert_eq!(toggle_decision(None), ToggleAct::Show);
    }
}
