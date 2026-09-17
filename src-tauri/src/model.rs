use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    pub id: String,
    pub started_at: String,
    pub root: PathBuf,
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
            started_at: now.to_rfc3339(),
            root,
            groups: vec![first],
        })
    }

    pub fn current(&mut self) -> &mut Group {
        self.groups.last_mut().expect("session always has a group")
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
        Ok(index)
    }

    /// Reserves the next `NN.png` in the current group and returns
    /// (group index, file name, absolute path).
    pub fn reserve_shot(&mut self) -> (usize, String, PathBuf) {
        let root = self.root.clone();
        let g = self.current();
        let file = format!("{:02}.png", g.shots.len() + 1);
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
