//! User settings, persisted in `~/Omashot/settings.json`: the studio's
//! recording toggles (all off by default, each a toggle in the tray) and
//! the global shortcuts.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// One global shortcut per action, in the global-shortcut parser's
/// spelling ("CommandOrControl+Shift+2"). Empty disables the action's key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkeys {
    pub quick: String,
    /// Finish the quick batch from anywhere. Empty by default: any chord
    /// worth having here (Ctrl+Enter) is one other apps rely on.
    pub quick_finish: String,
    pub capture: String,
    pub record: String,
    pub studio: String,
    pub zoom: String,
    pub group: String,
    pub peek: String,
    pub finish: String,
}

impl Default for Hotkeys {
    /// On Linux these are labels rather than registrations: the compositor
    /// owns the keys and runs `omashot <verb>`, so what is stored here is
    /// only what the windows print in their hints. They are kept in step
    /// with `omarchy/bindings.lua` by hand -- the app has no safe way to
    /// read a Lua config, and guessing would print a key that does nothing.
    #[cfg(target_os = "linux")]
    fn default() -> Self {
        Hotkeys {
            // The field names are the old action ids; the keys are the new
            // verbs. `record` is the trail (stills) and `studio` is the
            // screen recording, which is why those two look swapped.
            quick: "Super+Alt+Q".into(),
            quick_finish: String::new(),
            capture: "Super+Alt+A".into(),
            record: "Super+Alt+T".into(),
            studio: "Super+Alt+R".into(),
            zoom: "Super+Alt+Z".into(),
            group: "Super+Alt+N".into(),
            peek: "Super+Alt+B".into(),
            finish: "Super+Alt+D".into(),
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn default() -> Self {
        Hotkeys {
            quick: "CommandOrControl+Shift+1".into(),
            quick_finish: String::new(),
            capture: "CommandOrControl+Shift+2".into(),
            record: "CommandOrControl+Shift+3".into(),
            studio: "CommandOrControl+Shift+R".into(),
            // Only registered while a Studio recording runs, so it does not
            // take Ctrl+Space away from editors the rest of the time.
            zoom: "CommandOrControl+Space".into(),
            group: "CommandOrControl+Shift+G".into(),
            peek: "CommandOrControl+Shift+Q".into(),
            finish: "CommandOrControl+Shift+Enter".into(),
        }
    }
}

impl Hotkeys {
    /// The defaults before 2.1 moved auto-capture to 3, Studio to R and zoom
    /// to Ctrl+Space. A settings file still on exactly these is upgraded.
    pub fn legacy() -> Self {
        Hotkeys {
            quick: "CommandOrControl+Shift+1".into(),
            quick_finish: String::new(),
            capture: "CommandOrControl+Shift+2".into(),
            record: "CommandOrControl+Shift+R".into(),
            studio: "CommandOrControl+Shift+3".into(),
            zoom: "CommandOrControl+Shift+Z".into(),
            group: "CommandOrControl+Shift+G".into(),
            peek: "CommandOrControl+Shift+Q".into(),
            finish: "CommandOrControl+Shift+Enter".into(),
        }
    }

    /// (action id, spec) pairs, for registration and the menu.
    pub fn entries(&self) -> [(&'static str, &str); 9] {
        [
            ("quick", &self.quick),
            ("quick_finish", &self.quick_finish),
            ("capture", &self.capture),
            ("record", &self.record),
            ("studio", &self.studio),
            ("zoom", &self.zoom),
            ("group", &self.group),
            ("peek", &self.peek),
            ("finish", &self.finish),
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
        let mut s: Settings = std::fs::read_to_string(base.join("settings.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        if s.hotkeys == Hotkeys::legacy() {
            s.hotkeys = Hotkeys::default();
        }
        s
    }

    pub fn save(&self, base: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(base)?;
        std::fs::write(
            base.join("settings.json"),
            serde_json::to_string_pretty(self).unwrap_or_default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_settings_file_on_the_old_defaults_moves_to_the_new_ones() {
        let dir = std::env::temp_dir().join(format!("omashot-settings-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let old = Settings { hotkeys: Hotkeys::legacy(), ..Default::default() };
        old.save(&dir).unwrap();
        assert_eq!(Settings::load(&dir).hotkeys, Hotkeys::default());

        let mut custom = Hotkeys::legacy();
        custom.capture = "CommandOrControl+Alt+2".into();
        Settings { hotkeys: custom.clone(), ..Default::default() }.save(&dir).unwrap();
        assert_eq!(Settings::load(&dir).hotkeys, custom);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
