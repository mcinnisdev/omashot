#!/usr/bin/env bash
# Regenerates every app icon from the pixel grid in logo.py.
#
# Sizes that are whole multiples of the 16x16 grid are drawn directly, so
# they are pixel-exact. The Windows Store tiles are not multiples of 16, so
# those are point-sampled down from a large exact render: still crisp, just
# with the odd uneven pixel row, which nothing on Omarchy ever sees.
#
# The brand is one decision in one place. Colours live here rather than in logo.py because this is the brand, and
# the brand is one decision in one place. The tray icon is re-rendered in
# the live theme accent at runtime by omashot-theme-apply; this is the
# fallback for before a theme has ever been set.
set -euo pipefail

cd "$(dirname "$0")/.."

ACCENT="${OMASHOT_ACCENT:-#ff9e64}" # pixel amber
INK="${OMASHOT_INK:-#16181d}"       # the dark the mark is knocked out of

logo() { python3 scripts/logo.py --mark handoff "$@"; }

echo "brand assets"
logo --style tile --size 1024 --color "$ACCENT" --ink "$INK" --out assets/tile.png
logo --style mark --size 1024 --color "$ACCENT" --out assets/logo.png
logo --style mark --size 512 --color "$ACCENT" --out assets/mark.png

echo "app icons"
mkdir -p src-tauri/icons
for size in 32 64 128 256 512; do
  case $size in
  256) name="128x128@2x" ;;
  512) name="icon" ;;
  *) name="${size}x${size}" ;;
  esac
  logo --style mark --size "$size" --color "$ACCENT" \
    --out "src-tauri/icons/${name}.png"
done

# The tray sits on the bar at whatever height the bar is, so it ships bare
# and monochrome-friendly rather than as a tile.
logo --style mark --size 512 --color "$ACCENT" --out src-tauri/icons/tray.png

echo "windows tiles"
for size in 30 44 71 89 107 142 150 284 310; do
  magick src-tauri/icons/icon.png -filter point -resize "${size}x${size}" \
    "src-tauri/icons/Square${size}x${size}Logo.png"
done
magick src-tauri/icons/icon.png -filter point -resize 50x50 src-tauri/icons/StoreLogo.png

echo "container formats"
magick src-tauri/icons/icon.png -define icon:auto-resize=256,128,64,48,32,16 \
  src-tauri/icons/icon.ico
magick src-tauri/icons/icon.png -define icon:auto-resize=512,256,128,64,32,16 \
  src-tauri/icons/icon.icns 2>/dev/null ||
  echo "  icns skipped (ImageMagick without icns support; macOS is not a target)"

echo "done"
