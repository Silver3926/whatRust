# whatRust

A RAM-friendly, cross-platform WhatsApp Web desktop client built with **Rust + Tauri v2**.
Uses your OS's native webview (not a bundled Chromium), so idle RAM is typically 5–10×
lower than the official Electron-based WhatsApp Desktop.

## Install (one line)

**Linux / macOS**
```bash
curl -fsSL https://raw.githubusercontent.com/karem505/whatRust/master/install.sh | sh
```

**Windows** (PowerShell)
```powershell
irm https://raw.githubusercontent.com/karem505/whatRust/master/install.ps1 | iex
```

These download the latest published release (AppImage on Linux, `.app` → `/Applications`
on macOS, NSIS/MSI on Windows). No build toolchain required.

## Features
- WhatsApp Web in a lightweight native window (spoofed Chrome UA so it isn't rejected)
- System tray + close-to-tray + unread badge
- Native OS notifications
- Voice messages & calls (microphone/camera granted to the webview)
- Persistent login (no QR re-scan between restarts)
- Launch at startup, global show/hide shortcut (default `Ctrl/Cmd+Shift+W`)
- Single instance; remembers window size/position

## Develop
```bash
# Linux deps (Ubuntu):
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libhunspell-dev patchelf
cargo install tauri-cli --version "^2.0" --locked

cargo tauri dev      # run
cargo tauri build    # bundle (.deb + AppImage on Linux)
cd src-tauri && cargo test   # unit tests
```

> Requires **WebKitGTK ≥ 2.46.1** on Linux (older versions hang WhatsApp's QR-login).

## Releases
Push a tag `vX.Y.Z` (or run the **release** workflow manually) to build Linux/Windows/macOS
bundles via GitHub Actions.

## Known limitations
- **Windows unread count**: the system tray on Windows ignores text titles, so the unread
  *number* shows only in the icon hover-tooltip; the tray icon itself switches to a
  badged variant to indicate unread. macOS/Linux show the numeric count.
- **Notification click**: clicking a native notification does not yet focus the window
  (the tray icon and the global shortcut are the reliable ways to bring it forward).
- **macOS**: the build is unsigned. The installer clears the quarantine flag; if macOS
  still warns on first launch, right-click the app → **Open**.
- **Linux AppImage**: needs FUSE (`sudo apt install libfuse2`) or run with
  `--appimage-extract-and-run`.

## Notes
This is an unofficial wrapper around WhatsApp Web. It is not affiliated with or endorsed
by WhatsApp/Meta. The icon is a recreation of the WhatsApp mark, which is a trademark of
Meta Platforms, Inc.
