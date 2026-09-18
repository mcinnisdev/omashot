//! Studio recording options, persisted in `~/QACut/settings.json`. All off
//! by default; each is a toggle in the tray.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Record key presses through a low-level hook so shortcuts can be shown.
    pub keystrokes: bool,
    /// Record narration from the default microphone.
    pub mic: bool,
    /// Record the webcam for a picture-in-picture bubble.
    pub camera: bool,
}

impl Settings {
    pub fn load(base: &Path) -> Settings {
        std::fs::read_to_string(base.join("settings.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, base: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(base)?;
        std::fs::write(
            base.join("settings.json"),
            serde_json::to_string_pretty(self).unwrap_or_default(),
        )
    }
}
