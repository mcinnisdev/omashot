//! Driving Omashot from the outside.
//!
//! Wayland has no global hotkey API, and the X11 grab the global-shortcut
//! plugin falls back to registers without error but never fires for a
//! Wayland client. So on Omarchy the compositor owns the keys, the way it
//! owns every other key on the system: `bindings.lua` runs `omashot quick`,
//! and that reaches the running app over a socket in `$XDG_RUNTIME_DIR`.
//!
//! The same binary is both sides. With no arguments it is the app; with a
//! verb it is a one-shot client that talks to the app and exits. That keeps
//! `omashot` a single thing to install, package and put on a key.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use tauri::AppHandle;

/// How long `omashot <verb>` waits for an app it had to start itself.
const START_TIMEOUT: Duration = Duration::from_secs(10);

/// The command line, as verbs rather than as the names the code grew up
/// with. `(verb, action id, what it does)`.
///
/// The verb is the word; the action id is what `action_for` dispatches on and
/// what the settings file has always called it. Keeping them apart means the
/// vocabulary can be the user's without renaming the whole codebase under it.
///
/// Note that `record` means recording a video and `trail` means collecting
/// stills. The Windows build called those `studio` and `record`, which put
/// the word "record" on the one of the two that does not make a recording.
pub const VERBS: &[(&str, &str, &str)] = &[
    ("shot", "quick", "take one shot and note it"),
    ("copy", "quick_finish", "copy the loose shots and clear them"),
    ("add", "capture", "add a shot to the open brief"),
    ("trail", "record", "start or stop trailing what you do, as shots"),
    ("record", "studio", "start or stop a screen recording"),
    ("zoom", "zoom", "mark a zoom, while recording"),
    ("section", "group", "close this section and start the next"),
    ("brief", "peek", "open the brief"),
    ("done", "finish", "finish the brief and copy its path"),
];

/// The one verb that takes an argument: a PNG already on disk, adopted as a
/// loose shot and opened for markup. This is what `omashot-edit` calls, and
/// so what Omarchy's screenshot key reaches when it is pointed at Omashot.
pub const EDIT: (&str, &str, &str) = ("edit", "adopt", "mark up an image and hand it off");

fn action_id(verb: &str) -> Option<&'static str> {
    if verb == EDIT.0 {
        return Some(EDIT.1);
    }
    VERBS
        .iter()
        .find(|(v, _, _)| *v == verb)
        .map(|(_, id, _)| *id)
}

pub fn socket_path() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    // One socket per Wayland session, so two compositors on one login do not
    // fight over it.
    let session = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "0".into());
    dir.join(format!("omashot-{session}.sock"))
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
        println!("omashot {}", env!("CARGO_PKG_VERSION"));
        return Some(0);
    }
    let Some(action) = action_id(verb) else {
        eprintln!("omashot: no such command: {verb}");
        print_help();
        return Some(2);
    };

    let arg = if verb == EDIT.0 {
        let Some(path) = args.get(1) else {
            eprintln!("omashot: {} needs a file", EDIT.0);
            return Some(2);
        };
        // An absolute path, because the app's working directory is not ours.
        match std::fs::canonicalize(path) {
            Ok(p) => Some(p.to_string_lossy().into_owned()),
            Err(e) => {
                eprintln!("omashot: {path}: {e}");
                return Some(2);
            }
        }
    } else {
        None
    };

    Some(match send(verb, action, arg.as_deref()) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("omashot: {e}");
            1
        }
    })
}

fn print_help() {
    println!("omashot — capture your screen, hand off the work\n");
    println!("  omashot{:11}run the app; it sits in the bar", "");
    for (verb, _, what) in VERBS {
        println!("  omashot {verb:<10}{what}");
    }
    println!("  omashot {:<10}{}", format!("{} FILE", EDIT.0), EDIT.2);
    println!("\nBind these in ~/.config/hypr/bindings.lua, or use the Omarchy menu.");
}

fn connect() -> Option<UnixStream> {
    UnixStream::connect(socket_path()).ok()
}

/// Sends one verb, starting the app first if it is not up yet. Starting it
/// here is what makes a key binding work on a fresh login without anything
/// in autostart: the first press brings the app up and then does the thing.
fn send(verb: &str, action: &str, arg: Option<&str>) -> Result<(), String> {
    let stream = match connect() {
        Some(s) => s,
        None => {
            spawn_app()?;
            wait_for_app().ok_or("the app did not come up in time")?
        }
    };
    let mut stream = stream;
    // One line: the action, then a tab and its argument when it has one.
    // A tab cannot appear in the action names and is stripped from paths by
    // canonicalize, so nothing needs escaping.
    let line = match arg {
        Some(a) => format!("{action}\t{a}\n"),
        None => format!("{action}\n"),
    };
    stream
        .write_all(line.as_bytes())
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

/// True when another Omashot already holds the socket. The second copy of a
/// tray app should get out of the way rather than put up a second tray icon.
pub fn already_running() -> bool {
    connect().is_some()
}

/// Listens for verbs and runs them. Each action goes through the same
/// `off_main` worker the tray and the hotkeys use, because creating a window
/// from inside the socket thread would queue behind the event loop.
pub fn serve(app: AppHandle, dispatch: fn(&AppHandle, &str, Option<&str>) -> bool) {
    let path = socket_path();
    // A socket left behind by a crash: nothing is listening on it, or
    // `already_running` would have caught it, so it is safe to clear.
    let _ = std::fs::remove_file(&path);

    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("omashot: no command socket at {} ({e}); keys bound through `omashot <verb>` will not work", path.display());
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
            let (action, arg) = match line.trim_end_matches('\n').split_once('\t') {
                Some((a, v)) => (a, Some(v)),
                None => (line.trim(), None),
            };
            let reply = if dispatch(&app, action, arg) {
                "ok\n".to_string()
            } else {
                format!("no such action: {action}\n")
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
