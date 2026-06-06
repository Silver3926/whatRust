use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub fn show(app: &AppHandle, title: &str, body: &str) {
    // Don't silently swallow failures: on Windows a toast can fail (e.g. an
    // unregistered AppUserModelID — see aumid.rs) and the only signal is this
    // Result. Logging it makes such failures diagnosable from the console.
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        eprintln!("[whatrust] failed to show notification: {e:?}");
    }
}
