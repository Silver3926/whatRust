use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Persisted app-lock configuration. Stored in its own `app-lock.json` (NOT in
/// settings.json) so the password hash never rides along with the general settings
/// blob and is never serialized to the frontend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppLockConfig {
    pub enabled: bool,
    /// Argon2id PHC string (salt embedded). `None` when the lock is disabled.
    pub password_phc: Option<String>,
    pub biometric_enabled: bool,
    pub lock_on_launch: bool,
    pub lock_on_hide: bool,
    /// Idle auto-lock threshold in seconds. 0 == off.
    pub idle_secs: u32,
}

impl Default for AppLockConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            password_phc: None,
            biometric_enabled: false,
            lock_on_launch: true,
            lock_on_hide: false,
            idle_secs: 0,
        }
    }
}

impl AppLockConfig {
    /// The lock is only truly active when it is enabled AND a password exists. A
    /// bare `enabled` with no hash can never lock the user out.
    pub fn is_active(&self) -> bool {
        self.enabled && self.password_phc.is_some()
    }
}

fn config_path(app: &AppHandle) -> tauri::Result<PathBuf> {
    let dir = app.path().app_config_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("app-lock.json"))
}

pub fn load(app: &AppHandle) -> AppLockConfig {
    config_path(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, c: &AppLockConfig) -> tauri::Result<()> {
    let path = config_path(app)?;
    let json = serde_json::to_string_pretty(c).expect("serialize app-lock config");
    std::fs::write(path, json)?;
    Ok(())
}

/// Hash a passcode into an Argon2id PHC string. `Argon2::default()` is Argon2id,
/// version 0x13, with OWASP-baseline parameters. The random salt is embedded in the
/// returned PHC string, so nothing else needs storing.
pub fn hash_password(password: &str) -> Result<String, String> {
    use argon2::password_hash::rand_core::OsRng;
    use argon2::password_hash::{PasswordHasher, SaltString};
    use argon2::Argon2;
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

/// Verify a passcode against a stored PHC string. Returns false on any parse/verify
/// error (never panics, never leaks which step failed).
pub fn verify_password(password: &str, phc: &str) -> bool {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;
    match PasswordHash::new(phc) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Factory-reset everything the lock could gate: every account profile dir, the
/// default webview store, accounts.json, and app-lock.json. Used by the forgot-
/// password reset. Best-effort: ignores individual delete errors.
pub fn reset_all(app: &AppHandle) {
    // Per-account isolated profiles + the default shared store.
    let f = crate::accounts::load(app);
    for a in &f.accounts {
        crate::accounts::delete_profile(app, &a.id);
    }
    if let Ok(dir) = app.path().app_config_dir() {
        let _ = std::fs::remove_file(dir.join("accounts.json"));
        let _ = std::fs::remove_file(dir.join("app-lock.json"));
    }
    // The default account's webview data lives under the app data dir.
    if let Ok(data) = app.path().app_data_dir() {
        // Remove only known webview store dirs; do not nuke the whole data dir blindly.
        // NOTE: ["EBWebView", "default"] is a best-effort guess for the default-account
        // webview store path and must be confirmed on Linux in the Task 10 smoke test.
        for sub in ["EBWebView", "default"] {
            let _ = std::fs::remove_dir_all(data.join(sub));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let c = AppLockConfig::default();
        assert!(!c.enabled);
        assert!(c.password_phc.is_none());
        assert!(!c.biometric_enabled);
        assert!(c.lock_on_launch);
        assert!(!c.lock_on_hide); // user decision: hide-to-tray lock ships OFF
        assert_eq!(c.idle_secs, 0);
    }

    #[test]
    fn is_active_requires_enabled_and_hash() {
        let mut c = AppLockConfig::default();
        assert!(!c.is_active());
        c.enabled = true;
        assert!(!c.is_active(), "enabled without a hash must not be active");
        c.password_phc = Some("x".into());
        assert!(c.is_active());
    }

    #[test]
    fn partial_json_fills_defaults() {
        let c: AppLockConfig = serde_json::from_str(r#"{"enabled": true}"#).unwrap();
        assert!(c.enabled);
        assert!(c.lock_on_launch);
        assert!(!c.lock_on_hide);
        assert_eq!(c.idle_secs, 0);
    }

    #[test]
    fn empty_json_is_all_defaults() {
        let c: AppLockConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(c, AppLockConfig::default());
    }

    #[test]
    fn roundtrip() {
        let c = AppLockConfig { enabled: true, idle_secs: 300, ..Default::default() };
        let json = serde_json::to_string(&c).unwrap();
        let back: AppLockConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn hash_then_verify_roundtrips() {
        let phc = hash_password("correct horse").unwrap();
        assert!(phc.starts_with("$argon2id$"), "must be Argon2id: {phc}");
        assert!(verify_password("correct horse", &phc));
    }

    #[test]
    fn wrong_password_fails() {
        let phc = hash_password("secret").unwrap();
        assert!(!verify_password("nope", &phc));
    }

    #[test]
    fn verify_rejects_garbage_phc() {
        assert!(!verify_password("anything", "not-a-phc-string"));
    }
}
