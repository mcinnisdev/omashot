#!/usr/bin/env bash
# Wires Omacut into Omarchy: the theme template, the keys and the menu.
#
# Everything here is additive and idempotent, and nothing touches a file you
# have edited yourself. Run it again after an update; it will only fill in
# what is missing. `--uninstall` takes it all back out.
set -euo pipefail

OMARCHY_CONFIG="$HOME/.config/omarchy"
HYPR_CONFIG="$HOME/.config/hypr"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

MARK_BEGIN="-- >>> omacut >>>"
MARK_END="-- <<< omacut <<<"

say() { printf '  %s\n' "$*"; }
skip() { printf '  %s (already done)\n' "$*"; }

need_omarchy() {
  if [[ ! -d $OMARCHY_CONFIG ]]; then
    echo "omacut: $OMARCHY_CONFIG not found -- this installer is for Omarchy." >&2
    echo "The app itself runs on any Wayland compositor; only this wiring is Omarchy-specific." >&2
    exit 1
  fi
}

# ------------------------------------------------------------------- install

install_theme() {
  local dest="$OMARCHY_CONFIG/themed/omacut.css.tpl"
  mkdir -p "$(dirname "$dest")"
  if [[ -e $dest ]] && ! cmp -s "$HERE/themed/omacut.css.tpl" "$dest"; then
    skip "theme template (yours differs -- leaving it alone)"
    return
  fi
  install -m 644 "$HERE/themed/omacut.css.tpl" "$dest"
  say "theme template -> $dest"
}

# Omarchy only renders templates when the theme changes, so a fresh install
# has no stylesheet until the user switches themes. Render once now by
# re-setting the theme that is already set.
render_theme() {
  local current
  current=$(omarchy theme current 2>/dev/null || true)
  if omarchy theme refresh >/dev/null 2>&1; then
    say "rendered the palette from your ${current:-current} theme"
  else
    say "could not refresh the theme; switch themes once and Omacut will pick it up"
  fi
}

install_bindings() {
  append_block "$HYPR_CONFIG/bindings.lua" "$HERE/bindings.lua" "key bindings"
}

install_windowrules() {
  append_block "$HYPR_CONFIG/looknfeel.lua" "$HERE/looknfeel.lua" "window rules"
}

# True when a JSONC file holds anything beyond comments and an empty object.
has_entries() {
  local body
  body=$(sed 's|//.*||' "$1" | tr -d '[:space:]')
  [[ -n $body && $body != "{}" ]]
}

# Appends a marked block to a Lua config, once.
append_block() {
  local dest=$1 src=$2 what=$3
  [[ -f $dest ]] || { say "no $dest; skipping $what"; return; }
  if grep -qF -- "$MARK_BEGIN" "$dest"; then
    skip "$what"
    return
  fi
  {
    printf '\n%s\n' "$MARK_BEGIN"
    grep -v '^--' "$src" | sed '/^$/d'
    printf '%s\n' "$MARK_END"
  } >>"$dest"
  say "$what -> $dest"
}

install_menu() {
  local dest="$OMARCHY_CONFIG/extensions/omarchy-menu.jsonc"
  mkdir -p "$(dirname "$dest")"
  if [[ -f $dest ]] && grep -q '"omacut"' "$dest"; then
    skip "menu entries"
    return
  fi
  # The file Omarchy ships is all comments around an empty object. Anything
  # else is the user's, and merging JSONC safely from bash is not worth
  # getting wrong.
  if [[ -f $dest ]] && has_entries "$dest"; then
    say "menu: $dest has your own entries -- merge $HERE/omarchy-menu.jsonc in by hand"
    return
  fi
  install -m 644 "$HERE/omarchy-menu.jsonc" "$dest"
  say "menu entries -> $dest"
}

install_screenshot_editor() {
  cat <<'EOF'

  One more, if you want it: make Omacut the editor for Omarchy's own
  screenshot key, so every SUPER+SHIFT+S flows into markup, a note and a
  hand-off. Add this to ~/.config/environment.d/omacut.conf and log back in:

      OMACUT_SCREENSHOT_EDITOR=1
      OMARCHY_SCREENSHOT_EDITOR=omacut-edit

EOF
}

# ----------------------------------------------------------------- uninstall

uninstall() {
  local tpl="$OMARCHY_CONFIG/themed/omacut.css.tpl"
  # `set -e` would take a missing file as a failure to abort on, and there is
  # nothing wrong with uninstalling something that was never installed.
  if [[ -e $tpl ]]; then
    rm -f "$tpl"
    say "removed $tpl"
  fi

  local f
  for f in "$HYPR_CONFIG/bindings.lua" "$HYPR_CONFIG/looknfeel.lua"; do
    [[ -f $f ]] && grep -qF -- "$MARK_BEGIN" "$f" || continue
    sed -i "/^$(printf '%s' "$MARK_BEGIN" | sed 's/[]\/$*.^[]/\\&/g')$/,/^$(printf '%s' "$MARK_END" | sed 's/[]\/$*.^[]/\\&/g')$/d" "$f"
    say "removed the Omacut block from $f"
  done

  local menu="$OMARCHY_CONFIG/extensions/omarchy-menu.jsonc"
  if [[ -f $menu ]] && grep -q '"omacut"' "$menu"; then
    say "menu: remove the \"omacut\" keys from $menu by hand"
  fi

  echo
  say "done. The app itself is untouched; remove it with your package manager."
}

# ---------------------------------------------------------------------- main

if [[ ${1:-} == "--uninstall" ]]; then
  need_omarchy
  echo "Removing Omacut's Omarchy wiring:"
  uninstall
  exit 0
fi

need_omarchy
echo "Wiring Omacut into Omarchy:"
install_theme
render_theme
install_bindings
install_windowrules
install_menu
install_screenshot_editor
say "reload Hyprland (SUPER + ESCAPE) to pick up the keys."
