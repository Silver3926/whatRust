#!/bin/sh
set -eu

APP_ID="io.github.karem505.whatRust"
BASE_URL="https://github.com/karem505/whatRust/releases/latest/download"

if ! command -v flatpak >/dev/null 2>&1; then
  printf '%s\n' "Flatpak is required. Install it from https://flatpak.org/setup/ and run this installer again." >&2
  exit 1
fi
if ! command -v curl >/dev/null 2>&1; then
  printf '%s\n' "curl is required to download whatRust." >&2
  exit 1
fi

case "$(uname -m)" in
  x86_64|amd64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *)
    printf 'Unsupported architecture: %s\n' "$(uname -m)" >&2
    exit 1
    ;;
esac

TMP="$(mktemp "${TMPDIR:-/tmp}/whatrust-flatpak.XXXXXX")"
trap 'rm -f "$TMP"' EXIT HUP INT TERM

flatpak remote-add --if-not-exists --user flathub \
  https://dl.flathub.org/repo/flathub.flatpakrepo
curl -fL --retry 3 --progress-bar \
  "$BASE_URL/whatRust_${ARCH}.flatpak" -o "$TMP"
flatpak install --user --or-update -y "$TMP"

printf '%s\n' "whatRust is installed. Launch it from your application menu or run:"
printf '  flatpak run %s\n' "$APP_ID"
