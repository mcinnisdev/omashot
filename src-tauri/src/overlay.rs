use crate::capture::Frame;
use anyhow::Result;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder,
};

pub const NOTE: &str = "note";
pub const PEEK: &str = "peek";
pub const REC: &str = "rec";

fn capture_label(monitor_id: &str) -> String {
    format!("capture-{monitor_id}")
}

/// Opens one borderless, always-on-top window per monitor, each sized and
/// positioned to cover that monitor exactly. The window shows the frozen
/// frame, so no transparency is needed and this behaves the same on X11,
/// Wayland, Windows and macOS. `mode` is "shot" or "record" and decides
/// what the overlay does with the selection.
pub fn open_capture(app: &AppHandle, frames: &[Frame], mode: &str) -> Result<()> {
    close_capture(app);

    for frame in frames {
        let label = capture_label(&frame.monitor_id);
        let url = format!("capture.html?m={}&mode={mode}", frame.monitor_id);

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
    open_peek(app, None)?;
    Ok(true)
}

/// Opens (or reopens) the bundle window. `focus` is "name" to land in the
/// bundle name field or "open" to show the list of past bundles.
pub fn open_peek(app: &AppHandle, focus: Option<&str>) -> Result<()> {
    if let Some(w) = app.get_webview_window(PEEK) {
        let _ = w.close();
    }
    let url = match focus {
        Some(f) => format!("peek.html?focus={f}"),
        None => "peek.html".to_string(),
    };
    let win = WebviewWindowBuilder::new(app, PEEK, WebviewUrl::App(url.into()))
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
    Ok(())
}

pub fn close_peek(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(PEEK) {
        let _ = w.close();
    }
}

/// The recording overlay: a transparent, click-through window over the
/// whole monitor that tints everything outside the region, outlines the
/// region (just outside its edge, so the line is not recorded), and shows
/// the countdown and then the elapsed time. `x, y, w, h` are the region in
/// logical pixels relative to the monitor.
pub fn open_rec_badge(
    app: &AppHandle,
    frame: &Frame,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    countdown_ms: u64,
) -> Result<()> {
    close_rec_badge(app);
    let url = format!(
        "rec.html?countdown={countdown_ms}&x={}&y={}&w={}&h={}",
        x.round(),
        y.round(),
        w.round(),
        h.round()
    );
    let win = WebviewWindowBuilder::new(app, REC, WebviewUrl::App(url.into()))
        .title("QACut recording")
        .position(frame.x as f64, frame.y as f64)
        .inner_size(frame.width as f64, frame.height as f64)
        .decorations(false)
        .resizable(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .focused(false)
        .build()?;
    let _ = win.set_ignore_cursor_events(true);
    Ok(())
}

pub fn close_rec_badge(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(REC) {
        let _ = w.close();
    }
}
