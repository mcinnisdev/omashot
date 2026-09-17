#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod export;
mod model;
mod overlay;

use capture::Frame;
use export::Export;
use image::RgbaImage;
use model::{Purpose, Session, Shot, ShotKind};
use serde::Serialize;
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
const HK_RECORD: &str = "CommandOrControl+Shift+R";

#[derive(Default)]
struct Inner {
    session: Option<Session>,
    /// Frozen monitor frames, held only while the capture overlay is up.
    frames: Vec<(Frame, RgbaImage)>,
    /// True from the moment a capture is requested until its frames are
    /// stored, so a key-repeat cannot start a second grab mid-way.
    capturing: bool,
    /// The shot awaiting a note: (group index, shot id).
    pending: Option<(usize, String)>,
    /// A recording in progress and the shot it will fill in.
    recording: Option<(capture::Recording, usize, String)>,
    /// True when the session has changed since it was last written out, so
    /// "New bundle" knows whether there is anything to save first.
    dirty: bool,
    last_export: Option<Export>,
}

/// Everything the bundle window needs in one call.
#[derive(Clone, Serialize)]
struct AppState {
    session: Option<Session>,
    last_export: Option<Export>,
    dirty: bool,
    custom_prompt: String,
}

type Shared = Mutex<Inner>;

fn base_dir(app: &AppHandle) -> std::path::PathBuf {
    app.path()
        .home_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("QACut")
}

/// Returns the live session, starting one if there is none. A session only
/// ends when the user explicitly starts a new bundle; finishing writes it out
/// but leaves it open so more shots can be added and it can be written again.
fn ensure_session<'a>(app: &AppHandle, inner: &'a mut Inner) -> Result<&'a mut Session, String> {
    if inner.session.is_none() {
        let s = Session::start(&base_dir(app)).map_err(|e| e.to_string())?;
        inner.session = Some(s);
        inner.last_export = None;
        inner.dirty = false;
    }
    Ok(inner.session.as_mut().expect("just ensured"))
}

/// The user's own prompt template lives next to the bundles so it is easy
/// to find and edit by hand.
fn custom_prompt_path(app: &AppHandle) -> std::path::PathBuf {
    base_dir(app).join("custom-prompt.txt")
}

fn load_custom_prompt(app: &AppHandle) -> String {
    std::fs::read_to_string(custom_prompt_path(app)).unwrap_or_default()
}

/// Fills `{root}` and `{name}` in a custom template. A template that never
/// mentions the folder gets it appended, so the agent can always find it.
fn fill_custom_prompt(template: &str, root: &str, name: &str) -> String {
    let mut out = template.trim().replace("{root}", root).replace("{name}", name);
    if !template.contains("{root}") {
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(&format!("The bundle is at {root}. Start with bundle.md."));
    }
    out
}

/// The instruction handed to an agent alongside the folder path.
fn agent_prompt(root: &str, name: &str, purpose: Purpose, custom: &str) -> String {
    match purpose {
        Purpose::Custom => fill_custom_prompt(custom, root, name),
        Purpose::Fix => [
            &format!("Work through the QA bundle at {root}. "),
            "Start with bundle.md: each group is a page or area, its quoted master note ",
            "applies to every screenshot under it, and each screenshot's note says what is ",
            "wrong. Open each screenshot, and the key frames of any recording, before ",
            "changing anything.",
        ]
        .concat(),
        Purpose::Document => [
            &format!("Using the QA bundle at {root}, write a step-by-step process document "),
            "for the workflow it shows. Read bundle.md first: each group is a stage, the ",
            "quoted note under it describes that stage, and each screenshot or recording ",
            "is one step with the reviewer's note saying what is happening. Open every ",
            "screenshot and every recording's key frames before writing. Write the steps ",
            "in second person, embed each image and GIF where it belongs using its ",
            "relative path, and keep the file names so the document can live next to ",
            "the folder.",
        ]
        .concat(),
    }
}

// ---------------------------------------------------------------- triggers
//
// Hotkey and tray callbacks arrive on the main thread, inside a WndProc. If a
// window is built there, wry pumps a nested message loop while it waits for
// the WebView2 controller, and any queued window close that lands during that
// pump can deadlock the app. So every trigger runs on a worker thread; window
// creation is then queued to the event loop and handled at the top level like
// any other Tauri window.

fn off_main(app: &AppHandle, f: fn(&AppHandle)) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || f(&app));
}

fn trigger_capture(app: &AppHandle) {
    open_overlay(app, "shot");
}

/// Toggles: starts a recording via the overlay, or stops the one running.
fn trigger_record(app: &AppHandle) {
    let state: State<Shared> = app.state();
    let active = state.lock().unwrap().recording.is_some();
    if active {
        finish_recording(app);
    } else {
        open_overlay(app, "record");
    }
}

/// Freezes every monitor and opens the selection overlay in `mode`.
fn open_overlay(app: &AppHandle, mode: &str) {
    let state: State<Shared> = app.state();

    // Don't stack overlays if one is already up or on its way, and don't
    // start a still capture while a recording is running.
    {
        let mut inner = state.lock().unwrap();
        if inner.capturing || !inner.frames.is_empty() || inner.recording.is_some() {
            return;
        }
        inner.capturing = true;
    }
    overlay::close_note(app);
    overlay::close_peek(app);

    let frames = match capture::freeze_all(&capture::scratch_dir()) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("qacut: capture failed: {e}");
            state.lock().unwrap().capturing = false;
            return;
        }
    };

    let meta: Vec<Frame> = frames.iter().map(|(f, _)| f.clone()).collect();
    {
        let mut inner = state.lock().unwrap();
        inner.capturing = false;
        if let Err(e) = ensure_session(app, &mut inner) {
            eprintln!("qacut: could not start session: {e}");
            return;
        }
        inner.frames = frames;
    }

    if let Err(e) = overlay::open_capture(app, &meta, mode) {
        eprintln!("qacut: could not open overlay: {e}");
        let mut inner = state.lock().unwrap();
        inner.frames.clear();
    }
}

/// Stops the running recording, files what it produced, and asks for a note.
fn finish_recording(app: &AppHandle) {
    let state: State<Shared> = app.state();
    let Some((rec, group, id)) = state.lock().unwrap().recording.take() else {
        return;
    };
    overlay::close_rec_badge(app);

    match rec.stop() {
        Ok(done) => {
            let mut inner = state.lock().unwrap();
            if let Some(session) = inner.session.as_mut() {
                if let Some(shot) = session.shot_mut(group, &id) {
                    shot.width = done.width;
                    shot.height = done.height;
                    shot.duration_ms = done.duration_ms;
                    shot.frames = done.frames;
                }
            }
            inner.pending = Some((group, id));
            inner.dirty = true;
        }
        Err(e) => {
            eprintln!("qacut: recording failed: {e}");
            let mut inner = state.lock().unwrap();
            if let Some(session) = inner.session.as_mut() {
                session.remove_shot(group, &id);
            }
            let _ = app.emit("session-changed", ());
            return;
        }
    }

    let _ = app.emit("session-changed", ());
    if let Err(e) = overlay::open_note(app, "recording", None) {
        eprintln!("qacut: could not open note box: {e}");
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

fn trigger_new_bundle(app: &AppHandle) {
    if let Err(e) = do_new_bundle(app) {
        eprintln!("qacut: new bundle failed: {e}");
    }
}

/// Closes the current session. Unsaved work is written out first; a session
/// with no shots is deleted rather than left as an empty folder.
fn do_new_bundle(app: &AppHandle) -> Result<(), String> {
    overlay::close_note(app);
    let state: State<Shared> = app.state();
    {
        let mut inner = state.lock().unwrap();
        let dirty = inner.dirty;
        if let Some(session) = inner.session.as_mut() {
            if session.shot_count() == 0 {
                let _ = std::fs::remove_dir_all(&session.root);
            } else if dirty {
                export::write_bundle(session).map_err(|e| e.to_string())?;
            }
        }
        inner.session = None;
        inner.pending = None;
        inner.last_export = None;
        inner.dirty = false;
    }
    let _ = app.emit("session-changed", ());
    Ok(())
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
        inner.dirty = false;
        inner.last_export = Some(ex.clone());
        ex
    };

    match action {
        "markdown" => {
            app.clipboard()
                .write_text(result.markdown.clone())
                .map_err(|e| e.to_string())?;
        }
        "prompt" => {
            let (purpose, name) = state
                .lock()
                .unwrap()
                .session
                .as_ref()
                .map(|s| (s.purpose, s.title()))
                .unwrap_or((Purpose::Fix, String::new()));
            let custom = load_custom_prompt(app);
            if purpose == Purpose::Custom && custom.trim().is_empty() {
                return Err("write a custom prompt first".into());
            }
            app.clipboard()
                .write_text(agent_prompt(&result.root, &name, purpose, &custom))
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

fn shot_id(group: usize) -> String {
    format!("{group}-{}", chrono::Local::now().timestamp_micros())
}

// ---------------------------------------------------------------- commands
//
// Any command that opens or closes a window is `async`. Sync commands run on
// the main thread, and on Windows creating a window from there deadlocks
// (the builder waits on the event loop it is blocking).

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
async fn commit_selection(
    app: AppHandle,
    state: State<'_, Shared>,
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
        let (group, file, abs) = session.reserve_shot("png");

        let (pw, ph) = capture::crop_selection(&frame, &image, x, y, width, height, &abs)
            .map_err(|e| e.to_string())?;

        let id = shot_id(group);
        session.current().shots.push(Shot {
            id: id.clone(),
            file,
            abs_path: abs.to_string_lossy().to_string(),
            title: String::new(),
            note: String::new(),
            width: pw,
            height: ph,
            captured_at: chrono::Local::now().to_rfc3339(),
            kind: ShotKind::Image,
            duration_ms: 0,
            frames: Vec::new(),
        });

        inner.pending = Some((group, id));
        inner.frames.clear();
        inner.dirty = true;

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

/// The overlay, in record mode, hands over the region to record. The shot
/// is filed straight away with placeholder size and filled in on stop.
#[tauri::command]
async fn start_recording(
    app: AppHandle,
    state: State<'_, Shared>,
    monitor: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    overlay::close_capture(&app);

    let badge_at = {
        let mut inner = state.lock().unwrap();
        if inner.recording.is_some() {
            return Err("already recording".into());
        }

        let frame = inner
            .frames
            .iter()
            .find(|(f, _)| f.monitor_id == monitor)
            .map(|(f, _)| f.clone())
            .ok_or_else(|| "that monitor is no longer frozen".to_string())?;
        inner.frames.clear();

        let session = ensure_session(&app, &mut inner)?;
        let (group, file, abs) = session.reserve_shot("gif");
        let frames_dir = abs.with_file_name(format!(
            "{}-frames",
            abs.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
        ));

        let rec = capture::start_recording(
            app.clone(),
            &frame,
            x,
            y,
            width,
            height,
            abs.clone(),
            frames_dir,
        )
        .map_err(|e| e.to_string())?;

        let id = shot_id(group);
        session.current().shots.push(Shot {
            id: id.clone(),
            file,
            abs_path: abs.to_string_lossy().to_string(),
            title: String::new(),
            note: String::new(),
            width: 0,
            height: 0,
            captured_at: chrono::Local::now().to_rfc3339(),
            kind: ShotKind::Recording,
            duration_ms: 0,
            frames: Vec::new(),
        });
        inner.recording = Some((rec, group, id));
        inner.dirty = true;

        // Badge just above the region, or just below if that is off-screen.
        let bx = (frame.x as f64 + x).min(frame.x as f64 + frame.width as f64 - 240.0);
        let above = frame.y as f64 + y - 42.0;
        let by = if above >= frame.y as f64 { above } else { frame.y as f64 + y + height + 8.0 };
        (bx.max(frame.x as f64), by)
    };

    capture::clear_scratch();
    let _ = app.emit("session-changed", ());
    if let Err(e) = overlay::open_rec_badge(&app, badge_at.0, badge_at.1) {
        eprintln!("qacut: could not show recording badge: {e}");
    }
    Ok(())
}

#[tauri::command]
async fn stop_recording(app: AppHandle) {
    finish_recording(&app);
}

#[tauri::command]
async fn cancel_capture(app: AppHandle, state: State<'_, Shared>) -> Result<(), String> {
    overlay::close_capture(&app);
    {
        let mut inner = state.lock().unwrap();
        inner.frames.clear();
    }
    capture::clear_scratch();
    Ok(())
}

/// Commits the note for the shot that is waiting, plus the shot and group
/// names typed into the note box header. Empty values are fine.
#[tauri::command]
async fn save_note(
    app: AppHandle,
    state: State<'_, Shared>,
    note: String,
    title: String,
    group_title: String,
) -> Result<(), String> {
    {
        let mut inner = state.lock().unwrap();
        let Some((group, id)) = inner.pending.take() else {
            return Ok(());
        };
        if let Some(session) = inner.session.as_mut() {
            if let Some(shot) = session.shot_mut(group, &id) {
                shot.note = note.trim().to_string();
                shot.title = title.trim().to_string();
            }
            if let Some(g) = session.group_mut(group) {
                g.title = group_title.trim().to_string();
            }
        }
        inner.dirty = true;
    }
    overlay::close_note(&app);
    let _ = app.emit("session-changed", ());
    Ok(())
}

/// Discards the shot the note box was attached to, file and all.
#[tauri::command]
async fn discard_pending(app: AppHandle, state: State<'_, Shared>) -> Result<(), String> {
    {
        let mut inner = state.lock().unwrap();
        if let Some((group, id)) = inner.pending.take() {
            if let Some(session) = inner.session.as_mut() {
                session.remove_shot(group, &id);
            }
            inner.dirty = true;
        }
    }
    overlay::close_note(&app);
    let _ = app.emit("session-changed", ());
    Ok(())
}

/// Wraps up the current group with its name and master note and, if it has
/// shots, opens the next one.
#[tauri::command]
async fn close_group(
    app: AppHandle,
    state: State<'_, Shared>,
    title: String,
    master_note: String,
) -> Result<usize, String> {
    let index = {
        let mut inner = state.lock().unwrap();
        let session = ensure_session(&app, &mut inner)?;
        let index = session
            .close_group(&title, &master_note)
            .map_err(|e| e.to_string())?;
        inner.dirty = true;
        index
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
fn get_state(app: AppHandle, state: State<Shared>) -> AppState {
    let inner = state.lock().unwrap();
    AppState {
        session: inner.session.clone(),
        last_export: inner.last_export.clone(),
        dirty: inner.dirty,
        custom_prompt: load_custom_prompt(&app),
    }
}

/// Saves the user's prompt template. Not part of the session: it is meant to
/// be written once and reused across bundles.
#[tauri::command]
fn set_custom_prompt(app: AppHandle, text: String) -> Result<(), String> {
    let path = custom_prompt_path(&app);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, text).map_err(|e| e.to_string())
}

/// Points new captures at an existing group.
#[tauri::command]
fn set_current_group(app: AppHandle, state: State<Shared>, group: usize) -> Result<(), String> {
    {
        let mut inner = state.lock().unwrap();
        let session = inner.session.as_mut().ok_or("nothing captured yet")?;
        if !session.set_current(group) {
            return Err("no such group".into());
        }
    }
    let _ = app.emit("session-changed", ());
    Ok(())
}

/// Labels the bundle and renames its folder to match.
#[tauri::command]
fn rename_bundle(app: AppHandle, state: State<Shared>, name: String) -> Result<(), String> {
    {
        let mut inner = state.lock().unwrap();
        let session = inner.session.as_mut().ok_or("nothing captured yet")?;
        session.rename(&name).map_err(|e| e.to_string())?;
        let root = session.root.to_string_lossy().to_string();
        if let Some(ex) = inner.last_export.as_mut() {
            ex.root = root;
        }
        inner.dirty = true;
    }
    let _ = app.emit("session-changed", ());
    Ok(())
}

#[tauri::command]
async fn new_bundle(app: AppHandle) -> Result<(), String> {
    do_new_bundle(&app)
}

#[tauri::command]
fn set_purpose(app: AppHandle, state: State<Shared>, purpose: String) -> Result<(), String> {
    let p = Purpose::parse(&purpose).ok_or("unknown purpose")?;
    {
        let mut inner = state.lock().unwrap();
        let session = inner.session.as_mut().ok_or("nothing captured yet")?;
        session.purpose = p;
        inner.dirty = true;
    }
    let _ = app.emit("session-changed", ());
    Ok(())
}

#[tauri::command]
fn set_shot_note(
    app: AppHandle,
    state: State<Shared>,
    group: usize,
    shot: String,
    note: String,
    title: String,
) {
    {
        let mut inner = state.lock().unwrap();
        if let Some(session) = inner.session.as_mut() {
            if let Some(s) = session.shot_mut(group, &shot) {
                s.note = note.trim().to_string();
                s.title = title.trim().to_string();
            }
        }
        inner.dirty = true;
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
        inner.dirty = true;
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
        inner.dirty = true;
    }
    let _ = app.emit("session-changed", ());
}

#[tauri::command]
async fn finish(app: AppHandle, action: String) -> Result<Export, String> {
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
async fn start_capture(app: AppHandle) {
    trigger_capture(&app);
}

#[tauri::command]
async fn start_group(app: AppHandle) {
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
                        off_main(&app, trigger_capture);
                    } else if matches(HK_GROUP) {
                        off_main(&app, trigger_group);
                    } else if matches(HK_PEEK) {
                        off_main(&app, trigger_peek);
                    } else if matches(HK_FINISH) {
                        off_main(&app, trigger_finish);
                    } else if matches(HK_RECORD) {
                        off_main(&app, trigger_record);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            frame_for,
            commit_selection,
            start_recording,
            stop_recording,
            cancel_capture,
            save_note,
            discard_pending,
            close_group,
            get_session,
            get_state,
            set_current_group,
            rename_bundle,
            new_bundle,
            set_purpose,
            set_custom_prompt,
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

            for spec in [HK_CAPTURE, HK_RECORD, HK_GROUP, HK_PEEK, HK_FINISH] {
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
            let record_i = MenuItem::with_id(app, "record", "Record region / stop", true, Some(HK_RECORD))?;
            let group_i = MenuItem::with_id(app, "group", "Wrap up group", true, Some(HK_GROUP))?;
            let peek_i = MenuItem::with_id(app, "peek", "Show bundle", true, Some(HK_PEEK))?;
            let finish_i = MenuItem::with_id(app, "finish", "Finish and copy path", true, Some(HK_FINISH))?;
            let new_i = MenuItem::with_id(app, "new", "New bundle", true, None::<&str>)?;
            let folder_i = MenuItem::with_id(app, "folder", "Open QACut folder", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit QACut", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&capture_i, &record_i, &group_i, &peek_i, &finish_i, &new_i, &folder_i, &quit_i],
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
                    "capture" => off_main(app, trigger_capture),
                    "record" => off_main(app, trigger_record),
                    "group" => off_main(app, trigger_group),
                    "peek" => off_main(app, trigger_peek),
                    "finish" => off_main(app, trigger_finish),
                    "new" => off_main(app, trigger_new_bundle),
                    "folder" => {
                        let dir = base_dir(app);
                        let _ = std::fs::create_dir_all(&dir);
                        let _ = app.opener().open_path(dir.to_string_lossy().to_string(), None::<&str>);
                    }
                    "quit" => {
                        // Let a running recording write its trailer first.
                        let state: State<Shared> = app.state();
                        let rec = state.lock().unwrap().recording.take();
                        if let Some((rec, _, _)) = rec {
                            let _ = rec.stop();
                        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_prompt_fills_placeholders_and_always_names_the_folder() {
        let filled = fill_custom_prompt("Review {name} at {root}.", "C:/b", "Sprint 4");
        assert_eq!(filled, "Review Sprint 4 at C:/b.");

        let appended = fill_custom_prompt("Fix everything you see.", "C:/b", "x");
        assert!(appended.starts_with("Fix everything you see."));
        assert!(appended.ends_with("The bundle is at C:/b. Start with bundle.md."));

        assert_eq!(
            fill_custom_prompt("", "C:/b", "x"),
            "The bundle is at C:/b. Start with bundle.md."
        );
    }
}
