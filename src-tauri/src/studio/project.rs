//! The project folder: `source.mp4`, `events.json`, `project.json`, later
//! `camera.webm` and `export.mp4`. Everything the studio decides lives in
//! `project.json`; the source and events are never modified.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// The narration and camera track, when one was recorded.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Camera {
    pub file: String,
    /// Milliseconds from the first source frame to the first camera sample.
    /// Negative when the camera started first.
    pub offset_ms: i64,
    pub has_video: bool,
    pub has_audio: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub created_at: String,
    pub source: String,
    pub events: String,
    #[serde(default)]
    pub camera: Option<Camera>,
    /// The captured monitor, physical pixels, in screen coordinates.
    pub monitor: Rect,
    pub scale: f64,
    /// The region the user drew, physical pixels relative to the monitor.
    /// The source holds the whole monitor; the studio crops to this.
    pub region: Rect,
    pub fps: u32,
    /// QPC-based timestamp (100 ns units) of the first source frame; the
    /// zero of every time in `events.json`.
    #[serde(default)]
    pub first_frame_ts: i64,
    #[serde(default)]
    pub frames: u64,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub keystrokes: bool,
    /// The studio's editing decisions. Empty until the studio exists.
    #[serde(default)]
    pub edits: serde_json::Value,
}

impl Project {
    pub fn new(dir: &Path, monitor: Rect, scale: f64, region: Rect, keystrokes: bool) -> Self {
        let id = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        Project {
            version: VERSION,
            id,
            name: String::new(),
            created_at: chrono::Local::now().to_rfc3339(),
            source: "source.mp4".into(),
            events: "events.json".into(),
            camera: None,
            monitor,
            scale,
            region,
            fps: 60,
            first_frame_ts: 0,
            frames: 0,
            duration_ms: 0,
            keystrokes,
            edits: serde_json::Value::Object(Default::default()),
        }
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        std::fs::write(dir.join("project.json"), serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn load(dir: &Path) -> Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(dir.join("project.json"))?)?)
    }
}

/// Studio projects live beside the bundles, under `~/Omashot/Studio/`.
pub fn studio_dir(base: &Path) -> PathBuf {
    base.join("Studio")
}

/// A recording as listed in the studio's Recordings panel.
#[derive(Clone, Debug, Serialize)]
pub struct StudioInfo {
    pub dir: String,
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub duration_ms: u64,
    pub frames: u64,
    pub has_camera: bool,
}

/// Every finished recording under the studio folder, newest first.
pub fn list(base: &Path) -> Vec<StudioInfo> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(studio_dir(base)) else { return out };
    for entry in entries.flatten() {
        let dir = entry.path();
        let Ok(p) = Project::load(&dir) else { continue };
        if p.frames == 0 {
            continue;
        }
        out.push(StudioInfo {
            dir: dir.to_string_lossy().to_string(),
            id: p.id,
            name: p.name,
            created_at: p.created_at,
            duration_ms: p.duration_ms,
            frames: p.frames,
            has_camera: p.camera.is_some(),
        });
    }
    out.sort_by(|a, b| b.id.cmp(&a.id));
    out
}

/// Creates `~/Omashot/Studio/<timestamp>/`.
pub fn new_project_dir(base: &Path) -> Result<PathBuf> {
    let id = chrono::Local::now().format("%Y-%m-%d_%H%M%S").to_string();
    let dir = studio_dir(base).join(id);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
