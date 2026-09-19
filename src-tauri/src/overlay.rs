use crate::capture::Frame;
use anyhow::Result;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder,
};

pub const NOTE: &str = "note";
pub const PEEK: &str = "peek";
pub const REC: &str = "rec";
pub const EDIT: &str = "edit";
pub const STUDIO: &str = "studio";

/// Every window gets the same browser arguments (WebView2 fixes them for
/// the process at the first window). Tauri's defaults, plus: no permission
/// prompt for the microphone and camera, since the only pages that ask are
/// QACut's own, and no gesture needed for the camera preview to play.
#[cfg(windows)]
const BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection \
                            --use-fake-ui-for-media-stream \
                            --autoplay-policy=no-user-gesture-required";

fn builder<'a>(
    app: &'a AppHandle,
    label: &str,
    url: WebviewUrl,
) -> WebviewWindowBuilder<'a, tauri::Wry, AppHandle> {
    let b = WebviewWindowBuilder::new(app, label, url);
    #[cfg(windows)]
    let b = b.additional_browser_args(BROWSER_ARGS);
    b
}

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

        let win = builder(app, &label, WebviewUrl::App(url.into()))
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

/// The note box. `mode` is "shot", "quick", "recording" or "group". Anchored under the selection
/// when we have one, otherwise centred on the focused monitor.
pub fn open_note(app: &AppHandle, mode: &str, anchor: Option<(f64, f64)>) -> Result<()> {
    if let Some(w) = app.get_webview_window(NOTE) {
        let _ = w.close();
    }

    let (w, h) = match mode {
        "group" => (480.0, 236.0),
        "quick" => (480.0, 232.0),
        _ => (480.0, 190.0),
    };
    let url = format!("note.html?mode={mode}");

    let win = builder(app, NOTE, WebviewUrl::App(url.into()))
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
    let win = builder(app, PEEK, WebviewUrl::App(url.into()))
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
#[allow(clippy::too_many_arguments)]
pub fn open_rec_badge(
    app: &AppHandle,
    frame: &Frame,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    countdown_ms: u64,
    studio: bool,
) -> Result<()> {
    close_rec_badge(app);
    let url = format!(
        "rec.html?countdown={countdown_ms}&x={}&y={}&w={}&h={}&studio={}",
        x.round(),
        y.round(),
        w.round(),
        h.round(),
        if studio { 1 } else { 0 }
    );
    let win = builder(app, REC, WebviewUrl::App(url.into()))
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

    // A studio source is the whole monitor, so keep this window out of it:
    // the tint, the outline, the badge and the camera preview stay on the
    // screen for the operator and off the recording. (Auto-capture grabs
    // only the region, which the overlay never covers.)
    #[cfg(windows)]
    if studio {
        if let Ok(hwnd) = win.hwnd() {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE,
            };
            let ok = unsafe { SetWindowDisplayAffinity(hwnd.0 as _, WDA_EXCLUDEFROMCAPTURE) };
            if ok == 0 {
                eprintln!("qacut: could not exclude the overlay from capture; it will be in the source");
            }
        }
    }
    Ok(())
}

pub fn close_rec_badge(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(REC) {
        let _ = w.close();
    }
}

/// The markup editor for one PNG. Sized to the image plus its chrome, capped
/// to something that fits on the screen; the canvas scales down inside.
pub fn open_editor(app: &AppHandle, path: &str, label: &str, img_w: u32, img_h: u32) -> Result<()> {
    if let Some(w) = app.get_webview_window(EDIT) {
        let _ = w.close();
    }
    let w = (img_w as f64 + 40.0).clamp(560.0, 1400.0);
    let h = (img_h as f64 + 118.0).clamp(380.0, 900.0);
    let url = format!(
        "edit.html?path={}&label={}",
        urlencode(path),
        urlencode(label)
    );
    let win = builder(app, EDIT, WebviewUrl::App(url.into()))
        .title("QACut edit")
        .inner_size(w, h)
        .min_inner_size(480.0, 320.0)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(true)
        .build()?;
    let _ = win.center();
    let _ = win.set_focus();
    Ok(())
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// QACut Studio, on a project folder or on the list of recordings.
pub fn open_studio(app: &AppHandle, project_dir: Option<&str>) -> Result<()> {
    if let Some(w) = app.get_webview_window(STUDIO) {
        let _ = w.close();
    }
    let url = match project_dir {
        Some(d) => format!("studio.html?project={}", urlencode(d)),
        None => "studio.html".to_string(),
    };
    let win = builder(app, STUDIO, WebviewUrl::App(url.into()))
        .title("QACut Studio")
        .inner_size(1360.0, 860.0)
        .min_inner_size(960.0, 600.0)
        .decorations(false)
        .focused(true)
        .build()?;
    let _ = win.center();
    let _ = win.set_focus();
    Ok(())
}
