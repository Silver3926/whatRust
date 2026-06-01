#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    Available,
    NotConfigured,
    Unsupported,
}

pub fn availability() -> Availability {
    Availability::Unsupported
}

pub fn authenticate(_app: &tauri::AppHandle, _reason: &str) -> Result<bool, String> {
    Err("biometric authentication is not available on this platform".into())
}

pub fn label() -> &'static str {
    "biometric"
}
