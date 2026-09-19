//! What Omashot is doing right now, published for the desktop to read.
//!
//! The Omarchy bar is a Quickshell process, not something an app can push
//! into, so state goes where a bar widget can watch it: one small JSON file
//! in `$XDG_RUNTIME_DIR`, rewritten whenever anything changes. Quickshell's
//! `FileView` watches it and the widget redraws.
//!
//! Runtime dir rather than config: this is state about a running process and
//! should not outlive the login session. A stale file from a crash is
//! cleared on the next start and removed on a clean exit.

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::Shared;

#[derive(Serialize, Default, PartialEq)]
pub struct Status {
    /// The open brief's name, or null when there is none. An open brief
    /// with no name yet is `Some("")`, which is still an open brief.
    pub brief: Option<String>,
    /// Shots in the open brief, across every section.
    pub shots: usize,
    pub sections: usize,
    /// Loose shots: taken outside a brief and not yet handed off.
    pub quick: usize,
    /// A trail is running, collecting a shot at every step.
    pub trailing: bool,
    /// A screen recording is running.
    pub recording: bool,
}

pub fn path() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let session = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "0".into());
    dir.join(format!("omashot-{session}.state"))
}

pub fn read(app: &AppHandle) -> Status {
    let state: tauri::State<Shared> = app.state();
    let inner = state.lock().unwrap();

    let (brief, shots, sections) = match inner.session.as_ref() {
        // An unnamed brief is still an open brief; the widget shows the
        // count and leaves the name blank rather than pretending to be idle.
        Some(s) => (
            Some(s.name.clone()),
            s.groups.iter().map(|g| g.shots.len()).sum(),
            s.groups.len(),
        ),
        None => (None, 0, 0),
    };

    Status {
        brief,
        shots,
        sections,
        quick: inner
            .quick_batch
            .as_ref()
            .map(|d| count_shots(d))
            .unwrap_or(0),
        trailing: inner.recording.is_some(),
        recording: inner.studio.is_some(),
    }
}

/// Loose shots are files on disk rather than a list in memory, so they are
/// counted rather than tracked.
fn count_shots(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .filter(|e| {
                    let p = e.path();
                    p.extension().is_some_and(|x| x == "png")
                        // .orig.png sits beside an edited shot; it is the
                        // same shot, not another one.
                        && !p.to_string_lossy().ends_with(".orig.png")
                })
                .count()
        })
        .unwrap_or(0)
}

/// Rewrites the state file. Written to a temporary file and renamed, so a
/// watcher never reads a half-written one.
fn write(status: &Status) {
    let Ok(json) = serde_json::to_string(status) else {
        return;
    };
    let dest = path();
    let tmp = dest.with_extension("state.tmp");
    if std::fs::write(&tmp, json).is_ok() {
        let _ = std::fs::rename(&tmp, &dest);
    }
}

/// Publishes the state, and keeps publishing it whenever it changes.
///
/// A timer rather than a call at every mutation site: shots, sections, loose
/// shots, trails and recordings all move this state from a dozen places, and
/// one missed call is a bar widget that lies. Comparing first means the
/// file only moves when something really changed, so the widget is not woken
/// once a second for nothing.
pub fn watch(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last: Option<Status> = None;
        loop {
            let now = read(&app);
            if last.as_ref() != Some(&now) {
                write(&now);
                last = Some(now);
            }
            std::thread::sleep(std::time::Duration::from_millis(700));
        }
    });
}

pub fn clear() {
    let _ = std::fs::remove_file(path());
}
