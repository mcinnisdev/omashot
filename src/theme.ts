import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/// Omashot's stylesheet opens with a `:root` block of tokens and nothing else
/// names a colour. Omarchy renders the same block from the current theme, so
/// appending it re-skins the window: same properties, later in the cascade.
///
/// Every window calls this once on load. Until a theme has been set at least
/// once since install there is nothing to append, and the built-in tokens in
/// styles.css stand in.
const ELEMENT_ID = "omashot-theme";

function apply(css: string): void {
  let style = document.getElementById(ELEMENT_ID);
  if (!css) {
    style?.remove();
    return;
  }
  if (!style) {
    style = document.createElement("style");
    style.id = ELEMENT_ID;
    // Last in head, so it wins over the stylesheet the page linked.
    document.head.append(style);
  }
  style.textContent = css;
}

export async function applyTheme(): Promise<void> {
  // Listen before the first read, so a theme change during start-up is not
  // missed in the gap between them.
  await listen<string>("theme-changed", (e) => apply(e.payload));
  try {
    apply(await invoke<string>("theme_css"));
  } catch (err) {
    // A window with no palette is still a usable window: styles.css already
    // carries a full set of tokens.
    console.warn("omashot: could not load the theme", err);
  }
}
