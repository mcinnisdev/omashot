#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod export;
mod model;
mod overlay;

use capture::Frame;
use export::Export;
use image::RgbaImage;
use model::{Session, Shot};
use std::str::FromStr;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_opener::OpenerExt;

// Change these to retune the hotkeys; they are parsed at startup and any that
// fail to register are reported once in the console rather than killing boot.
const HK_CAPTURE: &str = "CommandOrControl+Shift+2";
const HK_GROUP: &str = "CommandOrControl+Shift+G";
const HK_PEEK: &str = "CommandOrControl+Shift+Q";
const HK_FINISH: &str = "CommandOrControl+Shift+Enter";

#[derive(Default)]
struct Inner {
    session: Option<Session>,
    /// Frozen monitor frames, held only while the capture overlay is up.
    frames: Vec<(Frame, RgbaImage)>,
    /// The shot awaiting a note: (group index, shot id).
    pending: Option<(usize, String)>,
    /// Set once a bundle has been written, so the next capture starts fresh.
    exported: bool,
    last_export: Option<Export>,
}

type Shared = Mutex<Inner>;

fn base_dir(app: &AppHandle) -> std::path::PathBuf {
    app.path()
        .home_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("QACut")
}

/// Returns the live session, starting one if there is none or if the previous
/// one has already been exported.
fn ensure_session<'a>(app: &AppHandle, inner: &'a mut Inner) -> Result<&'a mut Session, String> {
    let needs_new = inner.session.is_none() || inner.exported;
    if needs_new {
        let s = Session::start(&base_dir(app)).map_err(|e| e.to_string())?;
        inner.session = Some(s);
        inner.exported = false;
        inner.last_export = None;
    }
    Ok(inner.session.as_mut().expect("just ensured"))
}

// ---------------------------------------------------------------- triggers

fn trigger_capture(app: &AppHandle) {
    let state: State<Shared> = app.state();

    // Don't stack overlays if one is already up.
    {
        let inner = state.lock().unwrap();
        if !inner.frames.is_empty() {
            return;
        }
    }
    overlay::close_note(app);
    overlay::close_peek(app);

    let frames = match capture::freeze_all(&capture::scratch_dir()) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("qacut: capture failed: {e}");
            return;
        }
    };

    let meta: Vec<Frame> = frames.iter().map(|(f, _)| f.clone()).collect();
    {
        let mut inner = state.lock().unwrap();
        if let Err(e) = ensure_session(app, &mut inner) {
            eprintln!("qacut: could not start session: {e}");
            return;
        }
        inner.frames = frames;
    }

    if let Err(e) = overlay::open_capture(app, &meta) {
        eprintln!("qacut: could not open overlay: {e}");
        let mut inner = state.lock().unwrap();
        inner.frames.clear();
    }
}

fn trigger_group(app: &AppHandle) {
    overlay::close_peek(app);
    if let Err(e) = overlay::open_note(app, "group", None) {
        eprintln!("qacut: could not open group prompt: {e}");
    }
}

fn trigger_peek(app: &AppHandle) {
    if let Err(e) = overlay::toggle_peek(app) {
        eprintln!("qacut: could not open bundle view: {e}");
    }
}

fn trigger_finish(app: &AppHandle) {
    match do_finish(app, "path") {
        Ok(_) => {
            overlay::close_peek(app);
            let _ = overlay::toggle_peek(app);
        }
        Err(e) => eprintln!("qacut: finish failed: {e}"),
    }
}

fn do_finish(app: &AppHandle, action: &str) -> Result<Export, String> {
    let state: State<Shared> = app.state();
    let result = {
        let mut inner = state.lock().unwrap();
        let session = inner
            .session
            .as_mut()
            .ok_or_else(|| "nothing captured yet".to_string())?;
        let ex = export::write_bundle(session).map_err(|e| e.to_string())?;
        inner.exported = true;
        inner.last_export = Some(ex.clone());
        ex
    };

    match action {
        "markdown" => {
            app.clipboard()
                .write_text(result.markdown.clone())
                .map_err(|e| e.to_string())?;
        }
        "open" => {
            app.opener()
                .open_path(result.root.clone(), None::<&str>)
                .map_err(|e| e.to_string())?;
        }
        _ => {
            app.clipboard()
                .write_text(result.root.clone())
                .map_err(|e| e.to_string())?;
        }
    }

    let _ = app.emit("session-changed", ());
    Ok(result)
}

// ---------------------------------------------------------------- commands

/// The capture overlay asks for the frame belonging to its monitor.
#[tauri::command]
fn frame_for(state: State<Shared>, monitor: String) -> Option<Frame> {
    let inner = state.lock().unwrap();
    inner
        .frames
        .iter()
        .find(|(f, _)| f.monitor_id == monitor)
        .map(|(f, _)| f.clone())
}

/// Crops the selection, files it in the current group, and opens the note box.
#[tauri::command]
fn commit_selection(
    app: AppHandle,
    state: State<Shared>,
    monitor: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    overlay::close_capture(&app);

    let anchor = {
        let mut inner = state.lock().unwrap();

        let (frame, image) = inner
            .frames
            .iter()
            .find(|(f, _)| f.monitor_id == monitor)
            .map(|(f, i)| (f.clone(), i.clone()))
            .ok_or_else(|| "that monitor is no longer frozen".to_string())?;

        let session = ensure_session(&app, &mut inner)?;
        let (group, file, abs) = session.reserve_shot();

        let (pw, ph) = capture::crop_selection(&frame, &image, x, y, width, height, &abs)
            .map_err(|e| e.to_string())?;

        let id = format!("{group}-{}", chrono::Local::now().timestamp_micros());
        session.current().shots.push(Shot {
            id: id.clone(),
            file,
            abs_path: abs.to_string_lossy().to_string(),
            note: String::new(),
            width: pw,
            height: ph,
            captured_at: chrono::Local::now().to_rfc3339(),
        });

        inner.pending = Some((group, id));
        inner.frames.clear();

        // Anchor the note box just under the selection, nudged back on screen.
        let nx = frame.x as f64 + x;
        let ny = frame.y as f64 + y + height + 12.0;
        let max_y = frame.y as f64 + frame.height as f64 - 200.0;
        let max_x = frame.x as f64 + frame.width as f64 - 496.0;
        (nx.min(max_x).max(frame.x as f64 + 8.0), ny.min(max_y))
    };

    capture::clear_scratch();
    let _ = app.emit("session-changed", ());
    overlay::open_note(&app, "shot", Some(anchor)).map_err(|e| e.to_string())
}

#[tauri::command]
fn cancel_capture(app: AppHandle, state: State<Shared>) {
    overlay::close_capture(&app);
    {
        let mut inner = state.lock().unwrap();
        inner.frames.clear();
    }
    capture::clear_scratch();
}

/// Commits the note for the shot that is waiting. An empty note is fine.
#[tauri::command]
fn save_note(app: AppHandle, state: State<Shared>, note: String) -> Result<(), String> {
    {
        let mut inner = state.lock().unwrap();
        let Some((group, id)) = inner.pending.take() else {
            return Ok(());
        };
        if let Some(session) = inner.session.as_mut() {
            if let Some(shot) = session.shot_mut(group, &id) {
                shot.note = note.trim().to_string();
            }
        }
    }
    overlay::close_note(&app);
    let _ = app.emit("session-changed", ());
    Ok(())
}

/// Discards the shot the note box was attached to, file and all.
#[tauri::command]
fn discard_pending(app: AppHandle, state: State<Shared>) {
    {
        let mut inner = state.lock().unwrap();
        if let Some((group, id)) = inner.pending.take() {
            if let Some(session) = inner.session.as_mut() {
                session.remove_shot(group, &id);
            }
        }
    }
    overlay::close_note(&app);
    let _ = app.emit("session-changed", ());
}

#[tauri::command]
fn save_group(
    app: AppHandle,
    state: State<Shared>,
    title: String,
    master_note: String,
) -> Result<usize, String> {
    let index = {
        let mut inner = state.lock().unwrap();
        let session = ensure_session(&app, &mut inner)?;
        session
            .begin_group(&title, &master_note)
            .map_err(|e| e.to_string())?
    };
    overlay::close_note(&app);
    let _ = app.emit("session-changed", ());
    Ok(index)
}

#[tauri::command]
fn get_session(state: State<Shared>) -> Option<Session> {
    state.lock().unwrap().session.clone()
}

#[tauri::command]
fn get_last_export(state: State<Shared>) -> Option<Export> {
    state.lock().unwrap().last_export.clone()
}

#[tauri::command]
fn set_shot_note(app: AppHandle, state: State<Shared>, group: usize, shot: String, note: String) {
    {
        let mut inner = state.lock().unwrap();
        if let Some(session) = inner.session.as_mut() {
            if let Some(s) = session.shot_mut(group, &shot) {
                s.note = note.trim().to_string();
            }
        }
    }
    let _ = app.emit("session-changed", ());
}

#[tauri::command]
fn set_group_note(
    app: AppHandle,
    state: State<Shared>,
    group: usize,
    title: String,
    master_note: String,
) {
    {
        let mut inner = state.lock().unwrap();
        if let Some(session) = inner.session.as_mut() {
            if let Some(g) = session.group_mut(group) {
                g.title = title.trim().to_string();
                g.master_note = master_note.trim().to_string();
            }
        }
    }
    let _ = app.emit("session-changed", ());
}

#[tauri::command]
fn delete_shot(app: AppHandle, state: State<Shared>, group: usize, shot: String) {
    {
        let mut inner = state.lock().unwrap();
        if let Some(session) = inner.session.as_mut() {
            session.remove_shot(group, &shot);
        }
    }
    let _ = app.emit("session-changed", ());
}

#[tauri::command]
fn finish(app: AppHandle, action: String) -> Result<Export, String> {
    do_finish(&app, &action)
}

#[tauri::command]
fn copy_text(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_path(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn start_capture(app: AppHandle) {
    trigger_capture(&app);
}

#[tauri::command]
fn start_group(app: AppHandle) {
    trigger_group(&app);
}

// ------------------------------------------------------------------- boot

fn main() {
    tauri::Builder::default()
        .manage(Shared::default())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let pressed = shortcut.to_string();
                    let app = app.clone();
                    let matches = |spec: &str| {
                        Shortcut::from_str(spec)
                            .map(|s| s.to_string() == pressed)
                            .unwrap_or(false)
                    };

                    if matches(HK_CAPTURE) {
                        trigger_capture(&app);
                    } else if matches(HK_GROUP) {
                        trigger_group(&app);
                    } else if matches(HK_PEEK) {
                        trigger_peek(&app);
                    } else if matches(HK_FINISH) {
                        trigger_finish(&app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            frame_for,
            commit_selection,
            cancel_capture,
            save_note,
            discard_pending,
            save_group,
            get_session,
            get_last_export,
            set_shot_note,
            set_group_note,
            delete_shot,
            finish,
            copy_text,
            open_path,
            start_capture,
            start_group,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            for spec in [HK_CAPTURE, HK_GROUP, HK_PEEK, HK_FINISH] {
                match Shortcut::from_str(spec) {
                    Ok(sc) => {
                        if let Err(e) = handle.global_shortcut().register(sc) {
                            eprintln!("qacut: hotkey {spec} is taken by something else ({e})");
                        }
                    }
                    Err(e) => eprintln!("qacut: hotkey {spec} is not valid ({e})"),
                }
            }

            let capture_i = MenuItem::with_id(app, "capture", "Capture region", true, Some(HK_CAPTURE))?;
            let group_i = MenuItem::with_id(app, "group", "New group", true, Some(HK_GROUP))?;
            let peek_i = MenuItem::with_id(app, "peek", "Show bundle", true, Some(HK_PEEK))?;
            let finish_i = MenuItem::with_id(app, "finish", "Finish and copy path", true, Some(HK_FINISH))?;
            let folder_i = MenuItem::with_id(app, "folder", "Open QACut folder", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit QACut", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&capture_i, &group_i, &peek_i, &finish_i, &folder_i, &quit_i],
            )?;

            // A trimmed copy of the mark rather than the app icon, whose
            // margins cost a third of the glyph at menubar size.
            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;

            TrayIconBuilder::with_id("main")
                .icon(tray_icon)
                .tooltip("QACut")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "capture" => trigger_capture(app),
                    "group" => trigger_group(app),
                    "peek" => trigger_peek(app),
                    "finish" => trigger_finish(app),
                    "folder" => {
                        let dir = base_dir(app);
                        let _ = std::fs::create_dir_all(&dir);
                        let _ = app.opener().open_path(dir.to_string_lossy().to_string(), None::<&str>);
                    }
                    "quit" => {
                        capture::clear_scratch();
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // No visible window on launch. The tray is the app.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to start QACut")
        .run(|_app, event| {
            // `code` is None when the last window closed and Some when Quit
            // (or a restart) asked for it. Only the former should be swallowed;
            // the tray is the app, so closing overlays must not end it.
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
