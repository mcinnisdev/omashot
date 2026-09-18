#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod export;
mod model;
mod mp4;
mod overlay;
mod studio;

use capture::Frame;
use export::{BrandKit, Export};
use image::RgbaImage;
use model::{BundleInfo, DocFormat, Purpose, Session, Shot, ShotKind};
use serde::Serialize;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::menu::{CheckMenuItem, Menu, MenuItem};
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
const HK_NEW: &str = "CommandOrControl+Shift+N";
/// v2: record a source for the studio ("for people") rather than a GIF.
const HK_STUDIO: &str = "CommandOrControl+Shift+3";

/// Time between confirming a recording region and the first frame, so the
/// user can get windows and the mouse into place.
const RECORD_COUNTDOWN_MS: u64 = 3000;

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
    /// Set while the pre-recording countdown runs; storing true cancels it.
    countdown: Option<Arc<AtomicBool>>,
    /// A studio recording in progress.
    studio: Option<studio::Active>,
    /// A studio recording whose screen side has stopped; the overlay is
    /// flushing the camera track before the project is finalised.
    studio_finishing: Option<studio::Finishing>,
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
    brand: BrandKit,
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

/// Logo, colours, fonts, style guides, voice notes: whatever the user drops
/// here is copied into each bundle so the agent's output matches the brand.
fn brand_dir(app: &AppHandle) -> std::path::PathBuf {
    base_dir(app).join("brand")
}

/// The sentence added to the built-in prompts when a brand kit rides along.
const BRAND_LINE: &str = " The brand/ folder holds the business's brand kit and voice notes; match \
                          them in anything you produce.";

/// Where the prompt is going: a CLI agent that can open the folder, or a
/// chat agent that only sees an uploaded ZIP.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Target {
    Cli,
    Chat,
}

/// Fills `{root}` and `{name}` in a custom template. A template that never
/// mentions the folder gets it appended, so the agent can always find it.
fn fill_custom_prompt(template: &str, root: &str, name: &str, target: Target) -> String {
    let location = match target {
        Target::Cli => root.to_string(),
        Target::Chat => "the attached ZIP".to_string(),
    };
    let mut out = template
        .trim()
        .replace("{root}", &location)
        .replace("{name}", name);
    if !template.contains("{root}") {
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(&match target {
            Target::Cli => format!("The bundle is at {root}. Start with bundle.md."),
            Target::Chat => "The bundle is the attached ZIP. Unzip it and start with bundle.md.".to_string(),
        });
    }
    out
}

/// The instruction handed to an agent alongside the folder path or ZIP.
fn agent_prompt(
    root: &str,
    name: &str,
    purpose: Purpose,
    doc_format: DocFormat,
    custom: &str,
    brand: bool,
    target: Target,
) -> String {
    let opening = |verb: &str| match target {
        Target::Cli => format!("{verb} the QA bundle at {root}. "),
        Target::Chat => format!("{verb} the QA bundle in the attached ZIP. Unzip it first. "),
    };
    let base = match purpose {
        Purpose::Custom => return fill_custom_prompt(custom, root, name, target),
        Purpose::Fix => [
            &opening("Work through"),
            "Start with bundle.md: each group is a page or area, its quoted master note ",
            "applies to every screenshot under it, and each screenshot's note says what is ",
            "wrong. Open each screenshot, and the key frames of any recording, before ",
            "changing anything.",
        ]
        .concat(),
        Purpose::Document => {
            let deliverable = match doc_format {
                DocFormat::Markdown => [
                    "Write the steps in second person and embed each image and GIF where it ",
                    "belongs using its relative path. Save the result as process.md inside the ",
                    "bundle folder and keep the file names, so the document works next to its ",
                    "images.",
                ]
                .concat(),
                DocFormat::Html => [
                    "Deliver one self-contained web page, process.html, saved inside the bundle ",
                    "folder: inline CSS, images by relative path, and each recording embedded ",
                    "with a <video> tag pointing at its MP4 (controls, muted, loop, playsinline; ",
                    "fall back to the GIF as an <img> if there is no MP4). Let the clips carry ",
                    "the steps they show; write only what a reader needs between them, so five ",
                    "steps on one screen become one clip and a sentence, not five screenshots. ",
                    "Write in second person and keep the file names.",
                ]
                .concat(),
            };
            [
                &opening("Using"),
                "write a step-by-step process document ",
                "for the workflow it shows. Read bundle.md first: each group is a stage, the ",
                "quoted note under it describes that stage, and each screenshot or recording ",
                "is one step with the reviewer's note saying what is happening. A recording's ",
                "key frames are stills taken at each click, with where the click landed, so ",
                "each click frame is one action to describe. Open every screenshot and every ",
                "key frame before writing. ",
                &deliverable,
            ]
            .concat()
        }
    };
    if brand {
        base + BRAND_LINE
    } else {
        base
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

/// Toggles: starts a recording via the overlay, cancels a countdown, or
/// stops the recording that is running.
fn trigger_record(app: &AppHandle) {
    let state: State<Shared> = app.state();
    let (countdown, active) = {
        let inner = state.lock().unwrap();
        (inner.countdown.clone(), inner.recording.is_some())
    };
    if let Some(flag) = countdown {
        flag.store(true, Ordering::SeqCst);
    } else if active {
        finish_recording(app);
    } else {
        open_overlay(app, "record");
    }
}

/// Toggles a studio recording: opens the overlay, cancels a countdown, or
/// stops the recording that is running.
fn trigger_studio(app: &AppHandle) {
    let state: State<Shared> = app.state();
    let (countdown, active) = {
        let inner = state.lock().unwrap();
        (inner.countdown.clone(), inner.studio.is_some())
    };
    if let Some(flag) = countdown {
        flag.store(true, Ordering::SeqCst);
    } else if active {
        finish_studio(app);
    } else {
        open_overlay(app, "studio");
    }
}

/// Stops the studio recording's screen side and asks the overlay to flush
/// the camera track; `finalize_studio` completes it when that is done, or
/// after a grace period if the overlay never answers.
fn finish_studio(app: &AppHandle) {
    let state: State<Shared> = app.state();
    let Some(active) = state.lock().unwrap().studio.take() else {
        return;
    };
    match active.stop() {
        Ok(finishing) => {
            state.lock().unwrap().studio_finishing = Some(finishing);
            let _ = app.emit("recording-stop", ());
            let app2 = app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(5));
                finalize_studio(&app2);
            });
        }
        Err(e) => {
            overlay::close_rec_badge(app);
            eprintln!("qacut: studio recording failed: {e}");
        }
    }
}

fn finalize_studio(app: &AppHandle) {
    let state: State<Shared> = app.state();
    let Some(finishing) = state.lock().unwrap().studio_finishing.take() else {
        return;
    };
    overlay::close_rec_badge(app);
    match finishing.finalize() {
        Ok(()) => {
            eprintln!("qacut: studio recording saved to {}", finishing.dir.display());
            let _ = app.opener().reveal_item_in_dir(finishing.dir.join("source.mp4"));
        }
        Err(e) => eprintln!("qacut: studio recording could not be finalised: {e}"),
    }
}

/// Freezes every monitor and opens the selection overlay in `mode`.
fn open_overlay(app: &AppHandle, mode: &str) {
    let state: State<Shared> = app.state();

    // Don't stack overlays if one is already up or on its way, and don't
    // start a still capture while a recording is running.
    {
        let mut inner = state.lock().unwrap();
        if inner.capturing
            || !inner.frames.is_empty()
            || inner.recording.is_some()
            || inner.studio.is_some()
        {
            return;
        }
        inner.capturing = true;
    }
    // Get out of the way: the user's screenshots should not have QACut in
    // them. Anything open comes back when they ask for it.
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
                    shot.video = done.video.then(|| {
                        std::path::Path::new(&shot.file)
                            .with_extension("mp4")
                            .to_string_lossy()
                            .to_string()
                    });
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

/// Hotkey and tray: start a fresh bundle and open the window on its name.
fn trigger_new_bundle(app: &AppHandle) {
    match do_new_bundle(app) {
        Ok(()) => {
            if let Err(e) = overlay::open_peek(app, Some("name")) {
                eprintln!("qacut: could not open bundle window: {e}");
            }
        }
        Err(e) => eprintln!("qacut: new bundle failed: {e}"),
    }
}

fn trigger_open_bundle(app: &AppHandle) {
    if let Err(e) = overlay::open_peek(app, Some("open")) {
        eprintln!("qacut: could not open bundle window: {e}");
    }
}

/// Puts the current session away: unsaved work is written out, a session
/// with no shots is deleted rather than left as an empty folder.
fn stash_session(app: &AppHandle, inner: &mut Inner) -> Result<(), String> {
    let dirty = inner.dirty;
    if let Some(session) = inner.session.as_mut() {
        if session.shot_count() == 0 {
            let _ = std::fs::remove_dir_all(&session.root);
        } else if dirty {
            export::write_bundle(session, &brand_dir(app)).map_err(|e| e.to_string())?;
        }
    }
    inner.session = None;
    inner.pending = None;
    inner.last_export = None;
    inner.dirty = false;
    Ok(())
}

/// Closes the current session and starts a fresh one, so the bundle window
/// has something to name straight away.
fn do_new_bundle(app: &AppHandle) -> Result<(), String> {
    overlay::close_note(app);
    let state: State<Shared> = app.state();
    {
        let mut inner = state.lock().unwrap();
        stash_session(app, &mut inner)?;
        ensure_session(app, &mut inner)?;
    }
    let _ = app.emit("session-changed", ());
    Ok(())
}

/// Reopens a bundle from disk as the live session.
fn do_open_bundle(app: &AppHandle, path: &str) -> Result<(), String> {
    overlay::close_note(app);
    let loaded = Session::load(std::path::Path::new(path)).map_err(|e| e.to_string())?;
    let state: State<Shared> = app.state();
    {
        let mut inner = state.lock().unwrap();
        if inner.session.as_ref().map(|s| s.root == loaded.root).unwrap_or(false) {
            return Ok(());
        }
        stash_session(app, &mut inner)?;
        inner.session = Some(loaded);
    }
    let _ = app.emit("session-changed", ());
    Ok(())
}

fn do_finish(app: &AppHandle, action: &str) -> Result<Export, String> {
    let state: State<Shared> = app.state();
    let mut result = {
        let mut inner = state.lock().unwrap();
        let session = inner
            .session
            .as_mut()
            .ok_or_else(|| "nothing captured yet".to_string())?;
        let mut ex = export::write_bundle(session, &brand_dir(app)).map_err(|e| e.to_string())?;
        if action == "zip" {
            let zp = export::write_zip(session).map_err(|e| e.to_string())?;
            ex.zip_path = Some(zp.to_string_lossy().to_string());
        }
        inner.dirty = false;
        inner.last_export = Some(ex.clone());
        ex
    };

    let prompt_for = |target: Target| -> Result<String, String> {
        let (purpose, doc_format, name, include_brand) = state
            .lock()
            .unwrap()
            .session
            .as_ref()
            .map(|s| (s.purpose, s.doc_format, s.title(), s.include_brand))
            .unwrap_or((Purpose::Fix, DocFormat::Markdown, String::new(), false));
        let custom = load_custom_prompt(app);
        if purpose == Purpose::Custom && custom.trim().is_empty() {
            return Err("write a custom prompt first".into());
        }
        let brand = include_brand && !BrandKit::load(&brand_dir(app)).is_empty();
        Ok(agent_prompt(&result.root, &name, purpose, doc_format, &custom, brand, target))
    };

    match action {
        // Chat hand-off: reveal the archive ready to drag in, and put the
        // matching prompt on the clipboard.
        "zip" => {
            let zp = result.zip_path.clone().unwrap_or_default();
            let prompt = prompt_for(Target::Chat)?;
            app.clipboard().write_text(prompt).map_err(|e| e.to_string())?;
            let _ = app.opener().reveal_item_in_dir(&zp);
            result.zip_path = Some(zp);
        }
        "chatprompt" => {
            let prompt = prompt_for(Target::Chat)?;
            app.clipboard().write_text(prompt).map_err(|e| e.to_string())?;
        }
        "markdown" => {
            app.clipboard()
                .write_text(result.markdown.clone())
                .map_err(|e| e.to_string())?;
        }
        "prompt" => {
            let prompt = prompt_for(Target::Cli)?;
            app.clipboard().write_text(prompt).map_err(|e| e.to_string())?;
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
            video: None,
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

/// The overlay, in record mode, hands over the region to record. A short
/// countdown runs first (the badge shows it, the record hotkey cancels it),
/// then the shot is filed with placeholder size and filled in on stop.
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

    let (frame, cancel) = {
        let mut inner = state.lock().unwrap();
        if inner.recording.is_some() || inner.countdown.is_some() {
            return Err("already recording".into());
        }
        let frame = inner
            .frames
            .iter()
            .find(|(f, _)| f.monitor_id == monitor)
            .map(|(f, _)| f.clone())
            .ok_or_else(|| "that monitor is no longer frozen".to_string())?;
        inner.frames.clear();
        let cancel = Arc::new(AtomicBool::new(false));
        inner.countdown = Some(cancel.clone());
        (frame, cancel)
    };
    capture::clear_scratch();

    if let Err(e) =
        overlay::open_rec_badge(&app, &frame, x, y, width, height, RECORD_COUNTDOWN_MS, false)
    {
        eprintln!("qacut: could not show recording overlay: {e}");
    }

    let started = std::time::Instant::now();
    while started.elapsed().as_millis() < RECORD_COUNTDOWN_MS as u128 {
        if cancel.load(Ordering::SeqCst) {
            state.lock().unwrap().countdown = None;
            overlay::close_rec_badge(&app);
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    {
        let mut inner = state.lock().unwrap();
        inner.countdown = None;
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
            video: None,
        });
        inner.recording = Some((rec, group, id));
        inner.dirty = true;
    }

    let _ = app.emit("session-changed", ());
    let _ = app.emit("recording-started", ());
    Ok(())
}

#[tauri::command]
async fn stop_recording(app: AppHandle) {
    finish_recording(&app);
}

/// The overlay, in studio mode, hands over the region. Same countdown as a
/// GIF recording, then the studio capture and event recorders start.
#[tauri::command]
async fn start_studio(
    app: AppHandle,
    state: State<'_, Shared>,
    monitor: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    overlay::close_capture(&app);

    let (frame, cancel) = {
        let mut inner = state.lock().unwrap();
        if inner.recording.is_some() || inner.studio.is_some() || inner.countdown.is_some() {
            return Err("already recording".into());
        }
        let frame = inner
            .frames
            .iter()
            .find(|(f, _)| f.monitor_id == monitor)
            .map(|(f, _)| f.clone())
            .ok_or_else(|| "that monitor is no longer frozen".to_string())?;
        inner.frames.clear();
        let cancel = Arc::new(AtomicBool::new(false));
        inner.countdown = Some(cancel.clone());
        (frame, cancel)
    };
    capture::clear_scratch();

    if let Err(e) =
        overlay::open_rec_badge(&app, &frame, x, y, width, height, RECORD_COUNTDOWN_MS, true)
    {
        eprintln!("qacut: could not show recording overlay: {e}");
    }

    let started = std::time::Instant::now();
    while started.elapsed().as_millis() < RECORD_COUNTDOWN_MS as u128 {
        if cancel.load(Ordering::SeqCst) {
            state.lock().unwrap().countdown = None;
            overlay::close_rec_badge(&app);
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let settings = studio::settings::Settings::load(&base_dir(&app));
    let active = studio::begin(&base_dir(&app), &frame, x, y, width, height, settings.keystrokes);
    {
        let mut inner = state.lock().unwrap();
        inner.countdown = None;
        match active {
            Ok(a) => inner.studio = Some(a),
            Err(e) => {
                overlay::close_rec_badge(&app);
                eprintln!("qacut: studio recording could not start: {e}");
                return Err(e.to_string());
            }
        }
    }
    let _ = app.emit("recording-started", ());
    Ok(())
}

#[tauri::command]
async fn start_studio_from_menu(app: AppHandle) {
    trigger_studio(&app);
}

/// The overlay's microphone/camera recorder has started. Its wall-clock
/// time is mapped onto the frames' clock through the two clocks read
/// together here; the error is the IPC latency, a few milliseconds.
#[tauri::command]
fn camera_started(state: State<Shared>, at_unix_ms: f64, has_video: bool, has_audio: bool) {
    let now_q = studio::events::now_100ns();
    let now_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64() * 1000.0)
        .unwrap_or(at_unix_ms);
    let ts = now_q + ((at_unix_ms - now_unix) * 10_000.0) as i64;
    let mut inner = state.lock().unwrap();
    if let Some(active) = inner.studio.as_mut() {
        active.camera = Some(studio::CameraStart { ts, has_video, has_audio });
    } else if let Some(f) = inner.studio_finishing.as_mut() {
        f.camera = Some(studio::CameraStart { ts, has_video, has_audio });
    }
}

/// One MediaRecorder chunk, appended in order to camera.webm.
#[tauri::command]
fn append_camera(state: State<Shared>, request: tauri::ipc::Request<'_>) -> Result<(), String> {
    use std::io::Write as _;
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err("expected raw bytes".into());
    };
    let dir = {
        let inner = state.lock().unwrap();
        inner
            .studio
            .as_ref()
            .map(|a| a.dir.clone())
            .or_else(|| inner.studio_finishing.as_ref().map(|f| f.dir.clone()))
            .ok_or("no studio recording")?
    };
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("camera.webm"))
        .map_err(|e| e.to_string())?;
    f.write_all(bytes).map_err(|e| e.to_string())
}

/// The overlay has flushed its last chunk (or had nothing to record).
#[tauri::command]
async fn camera_stopped(app: AppHandle) {
    finalize_studio(&app);
}

#[tauri::command]
fn get_studio_settings(app: AppHandle) -> studio::settings::Settings {
    studio::settings::Settings::load(&base_dir(&app))
}

#[tauri::command]
fn set_studio_settings(app: AppHandle, settings: studio::settings::Settings) -> Result<(), String> {
    settings.save(&base_dir(&app)).map_err(|e| e.to_string())
}

/// Lets a window that is about to close report why something failed.
#[tauri::command]
fn log_error(message: String) {
    eprintln!("qacut: {message}");
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
        brand: BrandKit::load(&brand_dir(&app)),
    }
}

/// Saves the voice notes that get inlined into bundle.md.
#[tauri::command]
fn set_brand_notes(app: AppHandle, text: String) -> Result<(), String> {
    let dir = brand_dir(&app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(export::BRAND_NOTES), text).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_include_brand(app: AppHandle, state: State<Shared>, include: bool) -> Result<(), String> {
    {
        let mut inner = state.lock().unwrap();
        let session = ensure_session(&app, &mut inner)?;
        session.include_brand = include;
        inner.dirty = true;
    }
    let _ = app.emit("session-changed", ());
    Ok(())
}

/// Creates the brand folder if needed and opens it, so files can be dropped in.
#[tauri::command]
fn open_brand_folder(app: AppHandle) -> Result<(), String> {
    let dir = brand_dir(&app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
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
        let session = ensure_session(&app, &mut inner)?;
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
fn list_bundles(app: AppHandle) -> Vec<BundleInfo> {
    Session::list(&base_dir(&app))
}

#[tauri::command]
async fn open_bundle(app: AppHandle, path: String) -> Result<(), String> {
    do_open_bundle(&app, &path)
}

#[tauri::command]
fn set_doc_format(app: AppHandle, state: State<Shared>, format: String) -> Result<(), String> {
    let f = DocFormat::parse(&format).ok_or("unknown format")?;
    {
        let mut inner = state.lock().unwrap();
        let session = ensure_session(&app, &mut inner)?;
        session.doc_format = f;
        inner.dirty = true;
    }
    let _ = app.emit("session-changed", ());
    Ok(())
}

#[tauri::command]
fn set_purpose(app: AppHandle, state: State<Shared>, purpose: String) -> Result<(), String> {
    let p = Purpose::parse(&purpose).ok_or("unknown purpose")?;
    {
        let mut inner = state.lock().unwrap();
        let session = ensure_session(&app, &mut inner)?;
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

/// Reorders within a group or moves to another; the folder is laid out to
/// match on the next finish.
#[tauri::command]
fn move_shot(
    app: AppHandle,
    state: State<Shared>,
    group: usize,
    shot: String,
    to_group: usize,
    to_index: usize,
) -> Result<(), String> {
    {
        let mut inner = state.lock().unwrap();
        let session = inner.session.as_mut().ok_or("nothing captured yet")?;
        if !session.move_shot(group, &shot, to_group, to_index) {
            return Err("no such shot or group".into());
        }
        inner.dirty = true;
    }
    let _ = app.emit("session-changed", ());
    Ok(())
}

// ------------------------------------------------------------- markup

#[derive(Clone, Serialize)]
struct Markup {
    /// The untouched image to draw over: the .orig.png if an edit was saved
    /// before, else the file itself.
    original: String,
    marks: serde_json::Value,
}

/// Opens the markup editor on a PNG (a shot or a recording's still).
#[tauri::command]
async fn edit_shot(app: AppHandle, path: String, label: String) -> Result<(), String> {
    let (w, h) = image::image_dimensions(&path).map_err(|e| e.to_string())?;
    overlay::open_editor(&app, &path, &label, w, h).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_markup(path: String) -> Result<Markup, String> {
    let png = std::path::Path::new(&path);
    if !png.exists() {
        return Err("that image is gone".into());
    }
    let [orig, marks, _] = model::sidecars(png);
    let original = if orig.exists() { orig } else { png.to_path_buf() };
    let marks = std::fs::read_to_string(marks)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| serde_json::Value::Array(Vec::new()));
    Ok(Markup {
        original: original.to_string_lossy().to_string(),
        marks,
    })
}

/// Writes the annotated PNG over the file, keeping the untouched original
/// and the marks beside it so the edit can be reopened.
#[tauri::command]
async fn save_markup(
    app: AppHandle,
    path: String,
    png_base64: String,
    marks: String,
) -> Result<(), String> {
    use base64::Engine as _;
    let png = std::path::Path::new(&path);
    let [orig, marks_path, _] = model::sidecars(png);
    if !orig.exists() {
        std::fs::copy(png, &orig).map_err(|e| e.to_string())?;
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(png_base64.as_bytes())
        .map_err(|e| e.to_string())?;
    std::fs::write(png, bytes).map_err(|e| e.to_string())?;
    std::fs::write(&marks_path, &marks).map_err(|e| e.to_string())?;

    // If this is a recording's still and its click mark was moved (or
    // removed), the "click at x,y" in bundle.md follows.
    let click = serde_json::from_str::<serde_json::Value>(&marks)
        .ok()
        .and_then(|v| v.as_array().cloned())
        .and_then(|items| {
            items
                .iter()
                .find(|m| m.get("kind").and_then(|k| k.as_str()) == Some("click"))
                .and_then(|m| {
                    Some((
                        m.get("x")?.as_f64()?.round() as u32,
                        m.get("y")?.as_f64()?.round() as u32,
                    ))
                })
        });
    {
        let state: State<Shared> = app.state();
        let mut inner = state.lock().unwrap();
        if let Some(session) = inner.session.as_mut() {
            'find: for g in &mut session.groups {
                for shot in &mut g.shots {
                    let dir = std::path::Path::new(&shot.abs_path)
                        .parent()
                        .map(std::path::Path::to_path_buf)
                        .unwrap_or_default();
                    for f in &mut shot.frames {
                        if dir.join(&f.file) == png {
                            f.x = click.map(|c| c.0);
                            f.y = click.map(|c| c.1);
                            break 'find;
                        }
                    }
                }
            }
        }
        inner.dirty = true;
    }
    let _ = app.emit("session-changed", ());
    Ok(())
}

/// Drops one still from a recording so a bad frame never reaches the agent.
#[tauri::command]
fn remove_frame(
    app: AppHandle,
    state: State<Shared>,
    group: usize,
    shot: String,
    file: String,
) -> Result<(), String> {
    {
        let mut inner = state.lock().unwrap();
        let session = inner.session.as_mut().ok_or("nothing captured yet")?;
        if !session.remove_frame(group, &shot, &file) {
            return Err("no such frame".into());
        }
        inner.dirty = true;
    }
    let _ = app.emit("session-changed", ());
    Ok(())
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

#[tauri::command]
async fn start_record(app: AppHandle) {
    trigger_record(&app);
}

#[tauri::command]
fn open_base_folder(app: AppHandle) -> Result<(), String> {
    let dir = base_dir(&app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Same as the tray's Quit: let a running recording finish its file first.
#[tauri::command]
async fn quit(app: AppHandle) {
    let state: State<Shared> = app.state();
    let (rec, st) = {
        let mut inner = state.lock().unwrap();
        (inner.recording.take(), inner.studio.take())
    };
    if let Some((rec, _, _)) = rec {
        let _ = rec.stop();
    }
    if let Some(active) = st {
        if let Ok(f) = active.stop() {
            let _ = f.finalize();
        }
    }
    capture::clear_scratch();
    app.exit(0);
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
                    } else if matches(HK_NEW) {
                        off_main(&app, trigger_new_bundle);
                    } else if matches(HK_STUDIO) {
                        off_main(&app, trigger_studio);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            frame_for,
            commit_selection,
            start_recording,
            stop_recording,
            start_studio,
            start_studio_from_menu,
            camera_started,
            append_camera,
            camera_stopped,
            get_studio_settings,
            set_studio_settings,
            log_error,
            cancel_capture,
            save_note,
            discard_pending,
            close_group,
            get_session,
            get_state,
            set_current_group,
            rename_bundle,
            new_bundle,
            list_bundles,
            open_bundle,
            set_purpose,
            set_doc_format,
            set_custom_prompt,
            set_brand_notes,
            set_include_brand,
            open_brand_folder,
            set_shot_note,
            set_group_note,
            delete_shot,
            move_shot,
            edit_shot,
            load_markup,
            save_markup,
            remove_frame,
            finish,
            copy_text,
            open_path,
            start_capture,
            start_group,
            start_record,
            open_base_folder,
            quit,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            for spec in [HK_CAPTURE, HK_RECORD, HK_STUDIO, HK_GROUP, HK_PEEK, HK_FINISH, HK_NEW] {
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
            let studio_i = MenuItem::with_id(app, "studio", "Record for people / stop", true, Some(HK_STUDIO))?;
            let st = studio::settings::Settings::load(&base_dir(&handle));
            let keys_i = CheckMenuItem::with_id(app, "st_keys", "Studio: capture keystrokes", true, st.keystrokes, None::<&str>)?;
            let mic_i = CheckMenuItem::with_id(app, "st_mic", "Studio: record microphone", true, st.mic, None::<&str>)?;
            let cam_i = CheckMenuItem::with_id(app, "st_cam", "Studio: record camera", true, st.camera, None::<&str>)?;
            let toggles = (keys_i.clone(), mic_i.clone(), cam_i.clone());
            let group_i = MenuItem::with_id(app, "group", "Wrap up group", true, Some(HK_GROUP))?;
            let peek_i = MenuItem::with_id(app, "peek", "Show bundle", true, Some(HK_PEEK))?;
            let finish_i = MenuItem::with_id(app, "finish", "Finish and copy path", true, Some(HK_FINISH))?;
            let new_i = MenuItem::with_id(app, "new", "New bundle", true, Some(HK_NEW))?;
            let open_i = MenuItem::with_id(app, "open", "Open bundle...", true, None::<&str>)?;
            let folder_i = MenuItem::with_id(app, "folder", "Open QACut folder", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit QACut", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[
                    &capture_i, &record_i, &studio_i, &group_i, &peek_i, &finish_i, &new_i,
                    &open_i, &folder_i, &keys_i, &mic_i, &cam_i, &quit_i,
                ],
            )?;

            // A trimmed copy of the mark rather than the app icon, whose
            // margins cost a third of the glyph at menubar size.
            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;

            TrayIconBuilder::with_id("main")
                .icon(tray_icon)
                .tooltip("QACut")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "capture" => off_main(app, trigger_capture),
                    "record" => off_main(app, trigger_record),
                    "studio" => off_main(app, trigger_studio),
                    "st_keys" | "st_mic" | "st_cam" => {
                        // The item toggled itself; persist what it shows.
                        let s = studio::settings::Settings {
                            keystrokes: toggles.0.is_checked().unwrap_or(false),
                            mic: toggles.1.is_checked().unwrap_or(false),
                            camera: toggles.2.is_checked().unwrap_or(false),
                        };
                        if let Err(e) = s.save(&base_dir(app)) {
                            eprintln!("qacut: could not save settings: {e}");
                        }
                    }
                    "group" => off_main(app, trigger_group),
                    "peek" => off_main(app, trigger_peek),
                    "finish" => off_main(app, trigger_finish),
                    "new" => off_main(app, trigger_new_bundle),
                    "open" => off_main(app, trigger_open_bundle),
                    "folder" => {
                        let dir = base_dir(app);
                        let _ = std::fs::create_dir_all(&dir);
                        let _ = app.opener().open_path(dir.to_string_lossy().to_string(), None::<&str>);
                    }
                    "quit" => {
                        // Let a running recording write its trailer first.
                        let state: State<Shared> = app.state();
                        let (rec, st) = {
                            let mut inner = state.lock().unwrap();
                            (inner.recording.take(), inner.studio.take())
                        };
                        if let Some((rec, _, _)) = rec {
                            let _ = rec.stop();
                        }
                        if let Some(active) = st {
                            if let Ok(f) = active.stop() {
                                let _ = f.finalize();
                            }
                        }
                        capture::clear_scratch();
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // The brand folder exists from the start so there is somewhere
            // obvious to drop files before the first bundle window opens.
            let _ = std::fs::create_dir_all(brand_dir(&handle));

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
        let filled = fill_custom_prompt("Review {name} at {root}.", "C:/b", "Sprint 4", Target::Cli);
        assert_eq!(filled, "Review Sprint 4 at C:/b.");

        let appended = fill_custom_prompt("Fix everything you see.", "C:/b", "x", Target::Cli);
        assert!(appended.starts_with("Fix everything you see."));
        assert!(appended.ends_with("The bundle is at C:/b. Start with bundle.md."));

        assert_eq!(
            fill_custom_prompt("", "C:/b", "x", Target::Cli),
            "The bundle is at C:/b. Start with bundle.md."
        );

        // For chat the path is never mentioned; the upload is.
        let chat = fill_custom_prompt("Look at {root}.", "C:/b", "x", Target::Chat);
        assert_eq!(chat, "Look at the attached ZIP.");
        let chat = fill_custom_prompt("Go.", "C:/b", "x", Target::Chat);
        assert!(chat.ends_with("The bundle is the attached ZIP. Unzip it and start with bundle.md."));
    }

    #[test]
    fn built_in_prompts_switch_between_folder_and_zip() {
        let cli = agent_prompt("C:/b", "n", Purpose::Fix, DocFormat::Markdown, "", false, Target::Cli);
        assert!(cli.starts_with("Work through the QA bundle at C:/b. "));
        let chat = agent_prompt("C:/b", "n", Purpose::Fix, DocFormat::Markdown, "", true, Target::Chat);
        assert!(chat.starts_with("Work through the QA bundle in the attached ZIP. Unzip it first. "));
        assert!(!chat.contains("C:/b"));
        assert!(chat.ends_with("match them in anything you produce."));
        let doc = agent_prompt("C:/b", "n", Purpose::Document, DocFormat::Markdown, "", false, Target::Chat);
        assert!(doc.starts_with("Using the QA bundle in the attached ZIP. Unzip it first. write a step-by-step"));
        assert!(doc.contains("process.md"));
        let page = agent_prompt("C:/b", "n", Purpose::Document, DocFormat::Html, "", false, Target::Cli);
        assert!(page.contains("process.html"));
        assert!(page.contains("<video>"));
    }
}
