//! User settings, persisted in `~/QACut/settings.json`: the studio's
//! recording toggles (all off by default, each a toggle in the tray) and
//! the global shortcuts.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// One global shortcut per action, in the global-shortcut parser's
/// spelling ("CommandOrControl+Shift+2"). Empty disables the action's key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkeys {
    pub capture: String,
    pub record: String,
    pub studio: String,
    pub zoom: String,
    pub group: String,
    pub peek: String,
    pub finish: String,
    pub new: String,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Hotkeys {
            capture: "CommandOrControl+Shift+2".into(),
            record: "CommandOrControl+Shift+R".into(),
            studio: "CommandOrControl+Shift+3".into(),
            zoom: "CommandOrControl+Shift+Z".into(),
            group: "CommandOrControl+Shift+G".into(),
            peek: "CommandOrControl+Shift+Q".into(),
            finish: "CommandOrControl+Shift+Enter".into(),
            new: "CommandOrControl+Shift+N".into(),
        }
    }
}

impl Hotkeys {
    /// (action id, spec) pairs, for registration and the menu.
    pub fn entries(&self) -> [(&'static str, &str); 8] {
        [
            ("capture", &self.capture),
            ("record", &self.record),
            ("studio", &self.studio),
            ("zoom", &self.zoom),
            ("group", &self.group),
            ("peek", &self.peek),
            ("finish", &self.finish),
            ("new", &self.new),
        ]
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Record key presses through a low-level hook so shortcuts can be shown.
    pub keystrokes: bool,
    /// Record narration from the default microphone.
    pub mic: bool,
    /// Record the webcam for a picture-in-picture bubble.
    pub camera: bool,
    pub hotkeys: Hotkeys,
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
