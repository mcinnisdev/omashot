//! Omashot takes its colours from whatever Omarchy theme is set.
//!
//! Omarchy renders every `*.tpl` in `~/.config/omarchy/themed/` on a theme
//! change and drops the result beside the theme's other generated configs.
//! Ours lands at `<theme>/omashot.css` as a `:root` block, which is exactly
//! the shape of the token block the app's own stylesheet opens with, so
//! appending it overrides every token at once.
//!
//! Nothing here needs a theme-set hook: the file's own timestamp is the
//! signal. That keeps the install surface down to a single template.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use tauri::{AppHandle, Emitter};

/// How often the rendered stylesheet is checked. A theme change is a
/// human-scale event and this is one `stat` per tick.
const POLL: Duration = Duration::from_millis(1000);

/// Emitted to every window when the palette changes. The payload is the CSS.
pub const CHANGED: &str = "theme-changed";

pub fn css_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    let p = PathBuf::from(home)
        .join(".local/state/omarchy/current/theme/omashot.css");
    p.is_file().then_some(p)
}

pub fn read_css() -> String {
    css_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default()
}

/// Pulls `--signal` out of the rendered block. The tray mark is drawn in it,
/// so the icon belongs to the theme the same way the windows do.
pub fn accent(css: &str) -> Option<[u8; 3]> {
    let after = css.split("--signal:").nth(1)?;
    let value = after.split(';').next()?.trim();
    let hex = value.strip_prefix('#')?;
    let hex = match hex.len() {
        3 => hex.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 | 8 => hex[..6].to_string(),
        _ => return None,
    };
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

/// Recolours the tray mark. The shipped PNG is a single-colour glyph on
/// transparency, so only the RGB changes and the edges keep their alpha.
fn tint(png: &[u8], rgb: [u8; 3]) -> anyhow::Result<tauri::image::Image<'static>> {
    let mut img = image::load_from_memory(png)?.into_rgba8();
    for px in img.pixels_mut() {
        px.0 = [rgb[0], rgb[1], rgb[2], px.0[3]];
    }
    let (w, h) = (img.width(), img.height());
    Ok(tauri::image::Image::new_owned(img.into_raw(), w, h))
}

/// Applies the current accent to the tray icon, if there is one to apply.
pub fn retint_tray(app: &AppHandle, css: &str) {
    let Some(rgb) = accent(css) else { return };
    let Some(tray) = app.tray_by_id("main") else { return };
    match tint(super::TRAY_PNG, rgb) {
        Ok(icon) => {
            if let Err(e) = tray.set_icon(Some(icon)) {
                eprintln!("omashot: could not re-tint the tray icon: {e}");
            }
        }
        Err(e) => eprintln!("omashot: could not re-tint the tray icon: {e}"),
    }
}

fn stamp() -> Option<SystemTime> {
    std::fs::metadata(css_path()?).ok()?.modified().ok()
}

/// Watches the rendered stylesheet and re-skins the app when it moves.
///
/// The path is re-resolved every tick rather than held, because the file does
/// not exist until a theme has been set at least once since Omashot was
/// installed, and it should start working at that moment rather than at the
/// next launch.
pub fn watch(app: AppHandle) {
    std::thread::spawn(move || {
        let mut seen = stamp();
        // Whatever is on disk at launch is already applied by each window as
        // it opens, so only report changes from here on.
        loop {
            std::thread::sleep(POLL);
            let now = stamp();
            if now == seen {
                continue;
            }
            seen = now;
            let css = read_css();
            retint_tray(&app, &css);
            if let Err(e) = app.emit(CHANGED, &css) {
                eprintln!("omashot: could not announce the new theme: {e}");
            }
        }
    });
}

/// The stylesheet each window appends to its own on load.
#[tauri::command]
pub fn theme_css() -> String {
    read_css()
}
