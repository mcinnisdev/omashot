//! Driving Omacut from the outside.
//!
//! Wayland has no global hotkey API, and the X11 grab the global-shortcut
//! plugin falls back to registers without error but never fires for a
//! Wayland client. So on Omarchy the compositor owns the keys, the way it
//! owns every other key on the system: `bindings.lua` runs `omacut quick`,
//! and that reaches the running app over a socket in `$XDG_RUNTIME_DIR`.
//!
//! The same binary is both sides. With no arguments it is the app; with a
//! verb it is a one-shot client that talks to the app and exits. That keeps
//! `omacut` a single thing to install, package and put on a key.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use tauri::AppHandle;

/// How long `omacut <verb>` waits for an app it had to start itself.
const START_TIMEOUT: Duration = Duration::from_secs(10);

/// Verbs that map straight onto an action. Kept beside the hotkey ids in
/// `action_for` so a key and a command can never mean different things.
pub const VERBS: &[(&str, &str)] = &[
    ("quick", "take a quick shot"),
    ("quick-finish", "finish the quick batch and copy it"),
    ("capture", "capture a region into the bundle"),
    ("record", "start or stop auto-capture"),
    ("studio", "start or stop a Studio recording"),
    ("zoom", "mark a zoom (only while recording)"),
    ("group", "close the current group and start the next"),
    ("peek", "open the bundle window"),
    ("finish", "finish the bundle and copy its path"),
];

/// Hyphens read better on a command line than the underscores the settings
/// file uses for the same actions.
fn action_id(verb: &str) -> String {
    verb.replace('-', "_")
}

pub fn socket_path() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    // One socket per Wayland session, so two compositors on one login do not
    // fight over it.
    let session = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "0".into());
    dir.join(format!("omacut-{session}.sock"))
}

// ---------------------------------------------------------------- the client

/// Runs the client half when the command line carries a verb.
///
/// Returns `None` when there is nothing to do as a client and the process
/// should go on to be the app; otherwise the exit code to leave with.
pub fn run_client() -> Option<i32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verb = args.first()?.as_str();

    if matches!(verb, "-h" | "--help" | "help") {
        print_help();
        return Some(0);
    }
    if matches!(verb, "-V" | "--version" | "version") {
        println!("omacut {}", env!("CARGO_PKG_VERSION"));
        return Some(0);
    }
    if !VERBS.iter().any(|(v, _)| *v == verb) {
        eprintln!("omacut: no such command: {verb}");
        print_help();
        return Some(2);
    }

    Some(match send(verb) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("omacut: {e}");
            1
        }
    })
}

fn print_help() {
    println!("omacut — capture your screen, hand off the work\n");
    println!("  omacut{:15}run the app (it lives in the tray)", "");
    for (verb, what) in VERBS {
        println!("  omacut {verb:<14}{what}");
    }
    println!("\nBind these in ~/.config/hypr/bindings.lua; see `omacut help` in the docs.");
}

fn connect() -> Option<UnixStream> {
    UnixStream::connect(socket_path()).ok()
}

/// Sends one verb, starting the app first if it is not up yet. Starting it
/// here is what makes a key binding work on a fresh login without anything
/// in autostart: the first press brings the app up and then does the thing.
fn send(verb: &str) -> Result<(), String> {
    let stream = match connect() {
        Some(s) => s,
        None => {
            spawn_app()?;
            wait_for_app().ok_or("the app did not come up in time")?
        }
    };
    let mut stream = stream;
    stream
        .write_all(format!("{}\n", action_id(verb)).as_bytes())
        .map_err(|e| format!("could not send {verb}: {e}"))?;
    stream.flush().ok();

    let mut reply = String::new();
    BufReader::new(&stream)
        .read_line(&mut reply)
        .map_err(|e| format!("no reply to {verb}: {e}"))?;
    match reply.trim() {
        "ok" => Ok(()),
        other if other.is_empty() => Err(format!("{verb}: the app closed the connection")),
        other => Err(other.to_string()),
    }
}

fn spawn_app() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot find myself: {e}"))?;
    std::process::Command::new(exe)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("could not start the app: {e}"))?;
    Ok(())
}

fn wait_for_app() -> Option<UnixStream> {
    let deadline = Instant::now() + START_TIMEOUT;
    while Instant::now() < deadline {
        if let Some(s) = connect() {
            return Some(s);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    None
}

// ---------------------------------------------------------------- the server

/// True when another Omacut already holds the socket. The second copy of a
/// tray app should get out of the way rather than put up a second tray icon.
pub fn already_running() -> bool {
    connect().is_some()
}

/// Listens for verbs and runs them. Each action goes through the same
/// `off_main` worker the tray and the hotkeys use, because creating a window
/// from inside the socket thread would queue behind the event loop.
pub fn serve(app: AppHandle, dispatch: fn(&AppHandle, &str) -> bool) {
    let path = socket_path();
    // A socket left behind by a crash: nothing is listening on it, or
    // `already_running` would have caught it, so it is safe to clear.
    let _ = std::fs::remove_file(&path);

    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("omacut: no command socket at {} ({e}); keys bound through `omacut <verb>` will not work", path.display());
            return;
        }
    };

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let mut line = String::new();
            if BufReader::new(&stream).read_line(&mut line).is_err() {
                continue;
            }
            let reply = if dispatch(&app, line.trim()) {
                "ok\n".to_string()
            } else {
                format!("no such action: {}\n", line.trim())
            };
            let mut stream = stream;
            let _ = stream.write_all(reply.as_bytes());
        }
    });
}

/// Takes the socket file away on the way out, so the next launch does not
/// have to reason about whether it is stale.
pub fn cleanup() {
    let _ = std::fs::remove_file(socket_path());
}
