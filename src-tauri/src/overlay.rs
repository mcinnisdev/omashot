use crate::capture::Frame;
use anyhow::Result;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder,
};

pub const NOTE: &str = "note";
pub const PEEK: &str = "peek";

fn capture_label(monitor_id: &str) -> String {
    format!("capture-{monitor_id}")
}

/// Opens one borderless, always-on-top window per monitor, each sized and
/// positioned to cover that monitor exactly. The window shows the frozen
/// frame, so no transparency is needed and this behaves the same on X11,
/// Wayland, Windows and macOS.
pub fn open_capture(app: &AppHandle, frames: &[Frame]) -> Result<()> {
    close_capture(app);

    for frame in frames {
        let label = capture_label(&frame.monitor_id);
        let url = format!("capture.html?m={}", frame.monitor_id);

        let win = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title("QACut capture")
            .position(frame.x as f64, frame.y as f64)
            .inner_size(frame.width as f64, frame.height as f64)
            .decorations(false)
            .resizable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .shadow(false)
            .focused(true)
            .build()?;

        let _ = win.set_cursor_grab(false);
        let _ = win.set_focus();
    }
    Ok(())
}

pub fn close_capture(app: &AppHandle) {
    let labels: Vec<String> = app
        .webview_windows()
        .keys()
        .filter(|l| l.starts_with("capture-"))
        .cloned()
        .collect();
    for label in labels {
        if let Some(w) = app.get_webview_window(&label) {
            let _ = w.close();
        }
    }
}

/// The note box. `mode` is "shot" or "group". Anchored under the selection
/// when we have one, otherwise centred on the focused monitor.
pub fn open_note(app: &AppHandle, mode: &str, anchor: Option<(f64, f64)>) -> Result<()> {
    if let Some(w) = app.get_webview_window(NOTE) {
        let _ = w.close();
    }

    let (w, h) = if mode == "group" { (480.0, 236.0) } else { (480.0, 190.0) };
    let url = format!("note.html?mode={mode}");

    let win = WebviewWindowBuilder::new(app, NOTE, WebviewUrl::App(url.into()))
        .title("QACut note")
        .inner_size(w, h)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(true)
        .build()?;

    match anchor {
        Some((x, y)) => {
            let _ = win.set_position(LogicalPosition::new(x, y));
        }
        None => {
            let _ = win.center();
        }
    }
    let _ = win.set_focus();
    Ok(())
}

pub fn close_note(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(NOTE) {
        let _ = w.close();
    }
}

/// The bundle overview. Toggles: a second press closes it.
pub fn toggle_peek(app: &AppHandle) -> Result<bool> {
    if let Some(w) = app.get_webview_window(PEEK) {
        let _ = w.close();
        return Ok(false);
    }

    let win = WebviewWindowBuilder::new(app, PEEK, WebviewUrl::App("peek.html".into()))
        .title("QACut bundle")
        .inner_size(900.0, 640.0)
        .min_inner_size(620.0, 420.0)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(true)
        .build()?;

    let _ = win.set_size(LogicalSize::new(900.0, 640.0));
    let _ = win.center();
    let _ = win.set_focus();
    Ok(true)
}

pub fn close_peek(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(PEEK) {
        let _ = w.close();
    }
}
