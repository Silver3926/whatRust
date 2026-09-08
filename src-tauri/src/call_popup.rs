//! Windows: keep WebView2's default call popups, but cut their Win32 leash.
//!
//! Why NOT intercept `window.open()`: WhatsApp Web gates ALL calling behind a
//! popup probe — `window.open()` must return a live popup, or WhatsApp shows
//! "Make calls with the Windows app / Download WhatsApp for Windows" and
//! disables incoming AND outgoing calls entirely. This was observed in
//! practice: a blanket `NewWindowRequested` handler that called
//! `SetHandled(true)` (without opening anything) killed the popup probe and
//! with it the whole call feature. So the engine's default popup must stay.
//!
//! The one real problem with the default popup is Win32 OWNERSHIP: WebView2
//! creates it as an owned window of the account window, so minimizing the
//! account window minimizes the call with it.
//!
//! Fix: a background watcher enumerates this process's popup windows and
//! strips the owner (`SetWindowLongPtrW(GWLP_HWNDPARENT, 0)`) on any WebView2
//! popup owned by one of our windows. The popup becomes a true top-level
//! window — independent of the main window — while every JS-level
//! relationship (window.opener, the popup probe WhatsApp relies on) stays
//! intact, so calls keep working.
//!
//! Notes:
//! - Un-owned popups gain a taskbar button (owned windows don't get one).
//!   That is intended: an ongoing call is something the user should be able
//!   to find and click.
//! - The watcher is process-global and needs no wiring into window creation:
//!   popups from ANY account window (including accounts added later) are
//!   detached automatically.
//! - The same enumeration backs `call_in_progress()`, which pauses the idle
//!   auto-lock: a user mid-call is idle to mouse and keyboard but not away.
//!
//! Detection heuristic: a popup candidate is a VISIBLE, TOP-LEVEL window of
//! OUR process whose class is WebView2/Chromium's `Chrome_WidgetWin_1`. wry's
//! webview hosts are CHILD windows of the Tauri window (excluded by the
//! top-level check), and the Tauri/tao window itself has a different class.

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetAncestor, GetClassNameW, GetWindowLongPtrW, GetWindowThreadProcessId,
    IsWindowVisible, SetWindowLongPtrW, GA_ROOT, GWLP_HWNDPARENT,
};
// BOOL lives in windows-core in windows-rs 0.61 (moved out of Win32::Foundation).
use windows_core::BOOL;

/// WebView2/Chromium's window class for the web content host.
const POPUP_CLASS: PCWSTR = w!("Chrome_WidgetWin_1");
/// How often the watcher sweeps for new popups. A call opened up to ~300 ms
/// before its window is detached still minimizes with the main window for
/// that first instant only — a benign race.
const WATCH_INTERVAL_MS: u64 = 300;

/// The one decision the watcher makes, factored out pure for unit tests.
fn popup_candidate(visible: bool, top_level: bool, our_process: bool, class: &str) -> bool {
    visible && top_level && our_process && class == "Chrome_WidgetWin_1"
}

/// Class name of `hwnd`, lossy-decoded (never matches on decode failure → "").
fn class_name_of(hwnd: HWND) -> String {
    let mut buf = [0u16; 64];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    if n <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..n as usize])
}

/// `true` if `hwnd` belongs to this process.
fn is_our_process(hwnd: HWND) -> bool {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    pid != 0 && pid == unsafe { GetCurrentProcessId() }
}

unsafe extern "system" fn collect_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let out = &mut *(lparam.0 as *mut Vec<isize>);
    let visible = IsWindowVisible(hwnd).as_bool();
    let top_level = GetAncestor(hwnd, GA_ROOT) == hwnd;
    if popup_candidate(visible, top_level, is_our_process(hwnd), &class_name_of(hwnd)) {
        out.push(hwnd.0 as isize);
    }
    true.into()
}

/// All visible top-level WebView2 popups of this process, as raw HWNDs.
fn popup_hwnds() -> Vec<isize> {
    let mut out: Vec<isize> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(collect_cb),
            LPARAM(&mut out as *mut Vec<isize> as isize),
        );
    }
    out
}

/// Strip the owner from every popup still owned by one of our windows.
/// Returns how many were detached (for the diagnostic log).
fn unown_popups() -> usize {
    let mut detached = 0;
    for raw in popup_hwnds() {
        let hwnd = HWND(raw as *mut _);
        let owner = unsafe { GetWindowLongPtrW(hwnd, GWLP_HWNDPARENT) };
        if owner == 0 {
            continue; // already independent
        }
        // Only cut the leash we own: the owner must be one of our windows too.
        let owner_hwnd = HWND(owner as *mut _);
        if !is_our_process(owner_hwnd) {
            continue;
        }
        unsafe { SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, 0) };
        detached += 1;
    }
    detached
}

/// Long-running watcher: detach any WebView2 popup from its owner. Spawned
/// once at startup (see lib.rs); process-global, needs no Tauri state.
pub fn watch() {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(WATCH_INTERVAL_MS));
        let n = unown_popups();
        if n > 0 {
            crate::dlog::log(&format!("call popup: detached {n} popup window(s) from their owner"));
        }
    }
}

/// Whether a WebView2 popup (call or otherwise) is currently visible. Pure
/// Win32; the idle-lock watcher calls this every few seconds, so the cost of
/// one enumeration per poll is fine.
pub fn call_in_progress() -> bool {
    !popup_hwnds().is_empty()
}

#[cfg(test)]
mod tests {
    use super::popup_candidate;

    #[test]
    fn a_visible_toplevel_our_process_webview2_window_is_a_candidate() {
        assert!(popup_candidate(true, true, true, "Chrome_WidgetWin_1"));
    }

    #[test]
    fn child_windows_our_tao_window_and_other_classes_are_not() {
        // wry's webview host is a CHILD window of the Tauri window.
        assert!(!popup_candidate(true, false, true, "Chrome_WidgetWin_1"));
        // The Tauri/tao main window has its own class.
        assert!(!popup_candidate(true, true, true, "Window Class"));
        // Hidden windows are ignored.
        assert!(!popup_candidate(false, true, true, "Chrome_WidgetWin_1"));
        // Another process's Chromium window (e.g. real Chrome/Edge) is not ours.
        assert!(!popup_candidate(true, true, false, "Chrome_WidgetWin_1"));
        // No class / decode failure yields "".
        assert!(!popup_candidate(true, true, true, ""));
    }
}
