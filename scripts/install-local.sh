#!/usr/bin/env bash
# Installs a locally built Omashot into ~/.local, no root needed.
#
# This is the path for running from source. Packaged installs (PKGBUILD)
# put the same files under /usr instead. `--uninstall` takes them back out.
#
# It deliberately does not touch your Omarchy config -- that is
# omarchy/install.sh, which is a separate decision from having the binary.
set -euo pipefail

cd "$(dirname "$0")/.."

PREFIX="${PREFIX:-$HOME/.local}"
BIN="$PREFIX/bin"
APPS="$PREFIX/share/applications"
ICONS="$PREFIX/share/icons/hicolor"
BUILT="src-tauri/target/release/omashot"

say() { printf '  %s\n' "$*"; }

uninstall() {
  rm -f "$BIN/omashot" "$BIN/omashot-edit" "$APPS/omashot.desktop"
  local size
  for size in 32 64 128 256 512; do
    rm -f "$ICONS/${size}x${size}/apps/omashot.png"
  done
  update-desktop-database "$APPS" 2>/dev/null || true
  say "removed Omashot from $PREFIX"
  say "its Omarchy wiring is separate: omarchy/install.sh --uninstall"
}

if [[ ${1:-} == "--uninstall" ]]; then
  echo "Removing Omashot:"
  uninstall
  exit 0
fi

if [[ ! -x $BUILT ]]; then
  echo "omashot: no release build at $BUILT" >&2
  echo "Build it first:  npm install && npm run tauri build" >&2
  exit 1
fi

echo "Installing Omashot into $PREFIX:"

install -Dm755 "$BUILT" "$BIN/omashot"
say "binary -> $BIN/omashot"

install -Dm755 packaging/omashot-edit "$BIN/omashot-edit"
say "screenshot-editor shim -> $BIN/omashot-edit"

install -Dm644 packaging/omashot.desktop "$APPS/omashot.desktop"
say "launcher -> $APPS/omashot.desktop"

# hicolor wants one file per size, each named for the app, so the bar, the
# launcher and the app library all resolve Icon=omashot.
for size in 32 64 128 256 512; do
  case $size in
  256) src="src-tauri/icons/128x128@2x.png" ;;
  512) src="src-tauri/icons/icon.png" ;;
  *) src="src-tauri/icons/${size}x${size}.png" ;;
  esac
  install -Dm644 "$src" "$ICONS/${size}x${size}/apps/omashot.png"
done
say "icons -> $ICONS/*/apps/omashot.png"

update-desktop-database "$APPS" 2>/dev/null || true
gtk-update-icon-cache -f -t "$ICONS" 2>/dev/null || true

if ! command -v omashot >/dev/null; then
  say "note: $BIN is not on your PATH; the key bindings will not find omashot"
fi

echo
say "next: ./omarchy/install.sh  (theme, keys, menu, bar plugin)"
