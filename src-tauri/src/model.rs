use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Turns a title into a directory-safe suffix: "Settings page" -> "settings-page".
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    let mut last_dash = true;
    for ch in title.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
        if out.len() >= 40 {
            break;
        }
    }
    out.trim_matches('-').to_string()
}

/// A single captured region plus the note the user typed for it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shot {
    pub id: String,
    /// File name relative to the group directory, e.g. "03.png".
    pub file: String,
    /// Absolute path, handed to the webview via convertFileSrc for thumbnails.
    pub abs_path: String,
    pub note: String,
    pub width: u32,
    pub height: u32,
    pub captured_at: String,
}

/// A run of shots that share a heading and a master note.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Group {
    /// 1-based, stable for the life of the session.
    pub index: usize,
    pub title: String,
    pub master_note: String,
    /// Directory name under the session root. "01" while capturing,
    /// renamed to "01-settings-page" at export.
    pub dir: String,
    pub shots: Vec<Shot>,
}

impl Group {
    fn new(index: usize) -> Self {
        Group {
            index,
            title: String::new(),
            master_note: String::new(),
            dir: format!("{index:02}"),
            shots: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.shots.is_empty() && self.title.is_empty() && self.master_note.is_empty()
    }

    pub fn heading(&self) -> String {
        if self.title.trim().is_empty() {
            format!("Group {}", self.index)
        } else {
            self.title.trim().to_string()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    /// Timestamp the session started, also the folder name's prefix.
    pub id: String,
    /// Optional label the user gave the bundle. Appended to the folder name
    /// as a slug and used as the bundle title.
    pub name: String,
    pub started_at: String,
    pub root: PathBuf,
    /// Index of the group new shots go into. Usually the newest, but the
    /// user can point it back at an earlier group.
    pub current: usize,
    pub groups: Vec<Group>,
}

impl Session {
    /// Creates `<base>/<timestamp>/01/` on disk and returns the session.
    pub fn start(base: &Path) -> std::io::Result<Self> {
        let now = chrono::Local::now();
        let id = now.format("%Y-%m-%d_%H%M%S").to_string();
        let root = base.join(&id);
        std::fs::create_dir_all(&root)?;

        let first = Group::new(1);
        std::fs::create_dir_all(root.join(&first.dir))?;

        Ok(Session {
            id,
            name: String::new(),
            started_at: now.to_rfc3339(),
            root,
            current: 1,
            groups: vec![first],
        })
    }

    /// What the bundle is called in headings: the user's label, else the id.
    pub fn title(&self) -> String {
        if self.name.trim().is_empty() {
            self.id.clone()
        } else {
            self.name.trim().to_string()
        }
    }

    /// The folder name the session should have for its current label.
    fn dir_name(&self) -> String {
        let s = slug(&self.name);
        if s.is_empty() {
            self.id.clone()
        } else {
            format!("{}-{s}", self.id)
        }
    }

    /// Relabels the bundle and renames its folder on disk to match.
    pub fn rename(&mut self, name: &str) -> std::io::Result<()> {
        self.name = name.trim().to_string();
        let desired = self.dir_name();
        let Some(parent) = self.root.parent().map(Path::to_path_buf) else {
            return Ok(());
        };
        let to = parent.join(&desired);
        if to == self.root {
            return Ok(());
        }
        std::fs::rename(&self.root, &to)?;
        self.root = to;
        for g in &mut self.groups {
            for shot in &mut g.shots {
                shot.abs_path = self
                    .root
                    .join(&g.dir)
                    .join(&shot.file)
                    .to_string_lossy()
                    .to_string();
            }
        }
        Ok(())
    }

    pub fn current(&mut self) -> &mut Group {
        let idx = self.current;
        let pos = self
            .groups
            .iter()
            .position(|g| g.index == idx)
            .unwrap_or(self.groups.len() - 1);
        &mut self.groups[pos]
    }

    /// Points new captures at an existing group. Returns false if there is
    /// no such group.
    pub fn set_current(&mut self, index: usize) -> bool {
        if self.groups.iter().any(|g| g.index == index) {
            self.current = index;
            true
        } else {
            false
        }
    }

    pub fn shot_count(&self) -> usize {
        self.groups.iter().map(|g| g.shots.len()).sum()
    }

    /// Starts a new group. If the current group was never used, this just
    /// titles it in place instead of leaving an empty group behind.
    pub fn begin_group(&mut self, title: &str, master_note: &str) -> std::io::Result<usize> {
        if self.current().is_empty() {
            let g = self.current();
            g.title = title.trim().to_string();
            g.master_note = master_note.trim().to_string();
            return Ok(g.index);
        }

        let next = Group::new(self.groups.len() + 1);
        std::fs::create_dir_all(self.root.join(&next.dir))?;
        let index = next.index;
        self.groups.push(Group {
            title: title.trim().to_string(),
            master_note: master_note.trim().to_string(),
            ..next
        });
        self.current = index;
        Ok(index)
    }

    /// Reserves the next `NN.png` in the current group and returns
    /// (group index, file name, absolute path). Numbers only ever go up, so
    /// deleting a shot never lets a later one overwrite an existing file.
    pub fn reserve_shot(&mut self) -> (usize, String, PathBuf) {
        let root = self.root.clone();
        let g = self.current();
        let highest = g
            .shots
            .iter()
            .filter_map(|s| s.file.trim_end_matches(".png").parse::<usize>().ok())
            .max()
            .unwrap_or(0);
        let file = format!("{:02}.png", highest + 1);
        let abs = root.join(&g.dir).join(&file);
        (g.index, file, abs)
    }

    pub fn group_mut(&mut self, index: usize) -> Option<&mut Group> {
        self.groups.iter_mut().find(|g| g.index == index)
    }

    pub fn shot_mut(&mut self, group: usize, shot_id: &str) -> Option<&mut Shot> {
        self.group_mut(group)?
            .shots
            .iter_mut()
            .find(|s| s.id == shot_id)
    }

    /// Removes a shot and deletes its file. Remaining files keep their
    /// original names; numbering gaps are harmless and renaming mid-session
    /// would invalidate paths the peek window is already showing.
    pub fn remove_shot(&mut self, group: usize, shot_id: &str) {
        let Some(g) = self.group_mut(group) else { return };
        if let Some(pos) = g.shots.iter().position(|s| s.id == shot_id) {
            let shot = g.shots.remove(pos);
            let _ = std::fs::remove_file(&shot.abs_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shot(file: &str, abs: &Path) -> Shot {
        Shot {
            id: file.to_string(),
            file: file.to_string(),
            abs_path: abs.to_string_lossy().to_string(),
            note: String::new(),
            width: 1,
            height: 1,
            captured_at: String::new(),
        }
    }

    fn temp_base(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "qacut-test-{tag}-{}",
            chrono::Local::now().timestamp_micros()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn slugs_are_directory_safe() {
        assert_eq!(slug("Settings page"), "settings-page");
        assert_eq!(slug("  Billing / Invoices!! "), "billing-invoices");
        assert_eq!(slug(""), "");
        assert_eq!(slug("---"), "");
    }

    #[test]
    fn shot_numbers_never_reuse_a_file() {
        let base = temp_base("numbers");
        let mut s = Session::start(&base).unwrap();
        for _ in 0..3 {
            let (_, file, abs) = s.reserve_shot();
            std::fs::write(&abs, b"png").unwrap();
            s.current().shots.push(shot(&file, &abs));
        }
        s.remove_shot(1, "02.png");
        let (_, next, _) = s.reserve_shot();
        assert_eq!(next, "04.png");
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn rename_moves_the_folder_and_updates_paths() {
        let base = temp_base("rename");
        let mut s = Session::start(&base).unwrap();
        let (_, file, abs) = s.reserve_shot();
        std::fs::write(&abs, b"png").unwrap();
        s.current().shots.push(shot(&file, &abs));

        s.rename("Settings review").unwrap();
        assert!(s.root.ends_with(format!("{}-settings-review", s.id)));
        assert!(Path::new(&s.groups[0].shots[0].abs_path).exists());

        // Clearing the label moves it back.
        s.rename("").unwrap();
        assert!(s.root.ends_with(&s.id));
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn current_group_can_point_backwards() {
        let base = temp_base("current");
        let mut s = Session::start(&base).unwrap();
        s.current().shots.push(shot("01.png", Path::new("")));
        let second = s.begin_group("Billing", "").unwrap();
        assert_eq!(second, 2);
        assert_eq!(s.current().index, 2);
        assert!(s.set_current(1));
        assert_eq!(s.reserve_shot().0, 1);
        assert!(!s.set_current(9));
        std::fs::remove_dir_all(base).unwrap();
    }
}
