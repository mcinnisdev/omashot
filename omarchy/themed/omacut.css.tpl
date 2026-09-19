/* Omacut's palette, rendered from the current Omarchy theme.
 *
 * Omarchy fills the {{ ... }} placeholders whenever the theme changes and
 * writes the result next to the theme's other generated configs, at
 * ~/.local/state/omarchy/current/theme/omacut.css. The app watches that file
 * and re-skins every open window without restarting, so there is no hook.
 *
 * Omacut has no colours of its own. Every value here comes from the theme,
 * and the two that a theme does not name are mixed from the ones it does, so
 * this keeps working on a theme nobody has written yet — including a light
 * one, where mixing toward the foreground darkens rather than lightens.
 */

:root {
  --base: {{ background }};
  --text: {{ foreground }};
  --muted: {{ muted }};
  --signal: {{ accent }};
  --danger: {{ red }};

  /* Surfaces and borders are mixed rather than taken from the theme's own
   * lighter_background / darker_background, because a theme is free to set
   * those equal to the background — solitude does — and panels that do not
   * separate from what is behind them are worse than panels in the wrong
   * shade. Mixing toward the foreground always separates, and does the
   * right thing on a light theme too. */
  --raised: color-mix(in srgb, var(--base) 93%, var(--text));
  --sunken: color-mix(in srgb, var(--base) 96%, #000);
  --line: color-mix(in srgb, var(--base) 82%, var(--text));
  --line-soft: color-mix(in srgb, var(--base) 90%, var(--text));

  --faint: color-mix(in srgb, var(--muted) 62%, var(--base));
  --signal-dim: color-mix(in srgb, var(--signal) 14%, transparent);
}
