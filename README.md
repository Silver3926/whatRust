<p align="center">
  <img src="src-tauri/icons/128x128.png" width="96" alt="whatRust app icon — a lightweight WhatsApp Web desktop client">
</p>

# whatRust — Lightweight WhatsApp Web Desktop Client (Rust + Tauri)

**whatRust is a free, open-source, lightweight desktop client for WhatsApp Web that runs on Linux, Windows, and macOS — a lean, native alternative to the official Electron-based WhatsApp Desktop app, built with Rust and Tauri v2.**

![Latest release](https://img.shields.io/github/v/release/karem505/whatRust?label=release)
![License: MIT](https://img.shields.io/github/license/karem505/whatRust)
![Platforms: Linux, Windows, macOS](https://img.shields.io/badge/platforms-Linux%20%7C%20Windows%20%7C%20macOS-informational)
![Built with Rust and Tauri v2](https://img.shields.io/badge/built%20with-Rust%20%2B%20Tauri%20v2-orange)
![GitHub stars](https://img.shields.io/github/stars/karem505/whatRust?style=social)

> **Unofficial, independent project** — not affiliated with, endorsed by, or sponsored by WhatsApp or Meta. whatRust simply loads the official `web.whatsapp.com` interface in a native system webview.

## Contents

- [What is whatRust?](#what-is-whatrust)
- [Why whatRust? A lean, native WhatsApp Desktop alternative](#why-whatrust-a-lean-native-whatsapp-desktop-alternative)
- [Features](#features)
- [whatRust vs the official WhatsApp Desktop (Electron)](#whatrust-vs-the-official-whatsapp-desktop-electron)
- [Requirements](#requirements)
- [Installation](#installation)
- [Getting started](#getting-started)
- [FAQ](#faq)
- [Limitations](#limitations)
- [Contributing](#contributing)
- [Disclaimer](#disclaimer)
- [License](#license)

## What is whatRust?

whatRust is an open-source **WhatsApp Web desktop client** for Linux, Windows, and macOS. It wraps the official `web.whatsapp.com` in your operating system's native webview and adds the desktop conveniences a browser tab can't — a system tray, native notifications, persistent login, global shortcuts, and microphone/camera access for voice messages and calls.

It is an **unofficial, open-source WhatsApp client** and a practical **WhatsApp Desktop alternative** for people who want a fast, native app with minimal overhead instead of the heavier Electron-based official build. It is not affiliated with WhatsApp or Meta.

## Why whatRust? A lean, native WhatsApp Desktop alternative

**whatRust's native app shell is small — typically around 90 MB** — because it reuses the webview that already ships with your OS instead of bundling its own browser engine the way Electron apps do. That gives it a much lighter baseline than the official Electron-based WhatsApp Desktop, which ships an entire Chromium runtime on top of the same WhatsApp Web page.

The official WhatsApp Desktop app is built on Electron, which packs an entire Chromium browser inside every app. whatRust instead renders WhatsApp Web through the OS-native webview — **WebKitGTK** on Linux, **WebView2** on Windows, and **WKWebView** on macOS — via [Tauri v2](https://tauri.app). The result is a fast, native WhatsApp desktop app with a small footprint of its own.

> **A fair caveat on total memory:** your overall RAM use is dominated by **WhatsApp Web itself** and grows with how many chats, groups, and media you keep open — commonly a few hundred MB up to ~1 GB for busy accounts. That cost is roughly the same in any browser-based client (whatRust, the official app, or a plain Chrome tab); whatRust's advantage is the lean native shell, not lighter web content.

## Features

- **System tray** icon with **close-to-tray** and an **unread message badge**
- **Native OS notifications** for new messages
- **Persistent login** — scan the QR code once, stay signed in across restarts
- **Voice messages, voice calls, and video calls** — microphone and camera support
- **Launch at startup** (auto-start), optional
- **Global keyboard shortcut** to show/hide the window (default `Ctrl/Cmd+Shift+W`)
- **Single instance** — relaunching focuses the running window instead of opening a second
- **Remembers window size and position**
- **One-line install** on every platform
- **Cross-platform**: Linux, Windows, and macOS from one Rust + Tauri codebase

## whatRust vs the official WhatsApp Desktop (Electron)

| Feature | whatRust | Official WhatsApp Desktop (Electron) |
|---|---|---|
| App-shell memory overhead | Lean native shell (~90 MB), no bundled browser | Heavier — bundles a full Chromium + Node runtime |
| Total RAM (with WhatsApp Web loaded) | Dominated by WhatsApp Web (similar across clients) | Dominated by WhatsApp Web **+** Electron runtime |
| Rendering engine | OS-native webview (WebKitGTK / WebView2 / WKWebView) | Bundled Chromium (Electron) |
| Built with | Rust + Tauri v2 | Electron (Chromium + Node.js) |
| Open source | ✅ Yes | ❌ No |
| Native Linux app | ✅ Yes | ⚠️ Limited |
| Windows / macOS | ✅ Yes | ✅ Yes |
| System tray + close to tray | ✅ Yes | ⚠️ Partial |
| Unread message badge | ✅ Yes | ✅ Yes |
| Native notifications | ✅ Yes | ✅ Yes |
| Voice messages & calls (mic/camera) | ✅ Yes | ✅ Yes |
| Global show/hide shortcut | ✅ Yes | ❌ No |
| Launch at startup | ✅ Yes | ✅ Yes |
| Affiliated with Meta | ❌ No (unofficial) | ✅ Yes |

## Requirements

| OS | Webview engine | Notes |
|---|---|---|
| **Linux** | WebKitGTK | Requires WebKitGTK **≥ 2.46.1** (older versions hang WhatsApp's QR login). AppImage may need `libfuse2`. |
| **Windows 10/11** | WebView2 | Uses the Evergreen WebView2 runtime (preinstalled on Windows 11). |
| **macOS** | WKWebView | macOS 12+; the current build is Apple Silicon (arm64). |

## Installation

**Linux / macOS** — one line:
```bash
curl -fsSL https://raw.githubusercontent.com/karem505/whatRust/master/install.sh | sh
```
Installs the AppImage to `~/.local/bin` on Linux (with an application-menu entry), or the `.dmg` app into `/Applications` on macOS (Apple Silicon). The macOS build is unsigned — if it warns on first launch, right-click the app → **Open**.

**Windows** — one line (PowerShell):
```powershell
irm https://raw.githubusercontent.com/karem505/whatRust/master/install.ps1 | iex
```
Downloads and runs the latest NSIS installer (`.exe`); an `.msi` is also available on the release page.

**Manual download** — grab a `.AppImage`/`.deb`, `.dmg`, `.exe`, or `.msi` from the [latest release](https://github.com/karem505/whatRust/releases/latest).

<details>
<summary><b>Build from source</b> (Rust + Cargo + Tauri CLI)</summary>

```bash
# Linux build dependencies (Ubuntu/Debian)
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libhunspell-dev patchelf

cargo install tauri-cli --version "^2.0" --locked

cargo tauri dev      # run in development
cargo tauri build    # build installers for the current OS
cd src-tauri && cargo test   # run the unit tests
```
</details>

## Getting started

1. Launch whatRust. The WhatsApp Web QR screen appears.
2. On your phone, open **WhatsApp → Linked Devices → Link a Device** and scan the QR code.
3. You're in. Login persists, so you won't need to scan again next launch.
4. Closing the window hides whatRust to the tray (toggle this in Settings). Use the tray icon or the global shortcut to bring it back.

## FAQ

### What is whatRust?
whatRust is a free, open-source, lightweight WhatsApp Web desktop client built with Rust and Tauri v2 for Linux, Windows, and macOS.

### Is whatRust an official WhatsApp app?
No. whatRust is unofficial and independent — not affiliated with WhatsApp or Meta. It loads the official `web.whatsapp.com` in a native webview.

### How is whatRust different from the official WhatsApp Desktop app?
whatRust uses your OS's native webview instead of bundling a Chromium engine (as Electron does), which makes it considerably lighter. See the [comparison table](#whatrust-vs-the-official-whatsapp-desktop-electron).

### How much RAM does whatRust use?
whatRust's own native shell is small — around 90 MB. Your **total** memory use is mostly WhatsApp Web's own footprint and scales with how many chats and how much media you keep open — commonly a few hundred MB up to ~1 GB for busy accounts, similar to WhatsApp Web in a browser tab.

### Is whatRust lighter than the official WhatsApp Desktop app?
Its native shell is lighter because it doesn't bundle a Chromium browser engine the way the Electron-based official app does, so it has less baseline overhead. The WhatsApp Web content itself uses a similar amount in either app.

### Which operating systems does whatRust support?
Linux (WebKitGTK), Windows 10/11 (WebView2), and macOS 12+ (WKWebView).

### Is whatRust free and open source?
Yes — whatRust is free and open source under the MIT License. The source is on [GitHub](https://github.com/karem505/whatRust).

### Do voice messages, voice calls, and video calls work in whatRust?
Yes. whatRust grants the webview microphone and camera access, so voice messages and calls work the same as in WhatsApp Web.

### Does whatRust support the system tray and close-to-tray?
Yes. It adds a system tray icon with an unread-message badge, can close to the tray, and forwards new messages to native OS notifications.

### Do I have to log in every time I open whatRust?
No. Login is persistent — scan the QR code once via Linked Devices and you stay signed in across restarts.

### Is whatRust safe? Does it read my messages?
whatRust only loads the official `web.whatsapp.com` in a native webview and adds no message-handling layer of its own. It requests only the webview, microphone, and camera access that WhatsApp Web itself needs, and it is open source, so the code can be audited.

## Limitations

- **Windows unread count**: Windows tray icons ignore text labels, so the unread *number* appears only in the hover tooltip (the icon still switches to a badged variant). macOS and Linux show the count.
- **Notification click** does not yet focus the window — use the tray icon or the global shortcut.
- **macOS** builds are unsigned and currently Apple Silicon (arm64) only.

## Contributing

Contributions to this open-source WhatsApp client are welcome — open an issue or a pull request on [GitHub](https://github.com/karem505/whatRust).

## Disclaimer

whatRust is an unofficial, independent project. It is **not affiliated with, endorsed by, or sponsored by WhatsApp LLC or Meta Platforms, Inc.** "WhatsApp" is a trademark of its respective owner. whatRust only loads the official `web.whatsapp.com` interface in a native webview and does not modify or intercept WhatsApp's services.

## License

Released under the [MIT License](LICENSE).

## Built with

[Rust](https://www.rust-lang.org/) · [Tauri v2](https://tauri.app/) · [WhatsApp Web](https://web.whatsapp.com/)
