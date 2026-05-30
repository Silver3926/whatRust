# whatRust

A RAM-friendly, cross-platform WhatsApp Web desktop client built with **Rust + Tauri v2**.
Uses your OS's native webview (not a bundled Chromium), so idle RAM is typically 5–10×
lower than the official Electron-based WhatsApp Desktop.

## Features
- WhatsApp Web in a lightweight native window (spoofed Chrome UA so it isn't rejected)
- System tray + close-to-tray + unread badge
- Native OS notifications
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

## Notes
This is an unofficial wrapper around WhatsApp Web. It is not affiliated with or endorsed
by WhatsApp/Meta.
