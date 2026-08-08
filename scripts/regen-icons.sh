#!/usr/bin/env bash
# Regenerate all app + tray icons from the single SVG source.
# Requires: cargo tauri (tauri-cli v2) and ImageMagick (`magick`).
set -euo pipefail
cd "$(dirname "$0")/.."

SVG="src-tauri/icons/app-icon.svg"

# Full cross-platform app icon set (png sizes, .ico, .icns, Square* logos).
cargo tauri icon "$SVG" -o src-tauri/icons

# Tray icon = 128px render; unread = same + a red badge dot (top-right).
cp src-tauri/icons/128x128.png src-tauri/icons/tray.png
magick src-tauri/icons/tray.png -fill '#EA4335' -stroke white -strokewidth 3 \
  -draw 'circle 99,29 99,9' src-tauri/icons/tray-unread.png

# macOS menu bar wants a monochrome template image instead of the colour mark.
# Derived from the 512px render of the same SVG; needs only node (no ImageMagick).
node scripts/make-tray-template.mjs

echo "Icons regenerated from $SVG"
