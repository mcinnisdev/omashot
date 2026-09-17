use crate::model::{slug, Session};
use anyhow::Result;
use serde::Serialize;
use std::fmt::Write as _;

#[derive(Clone, Debug, Serialize)]
pub struct Export {
    pub root: String,
    pub markdown: String,
    pub groups: usize,
    pub shots: usize,
}

/// Renames group directories to include their titles, then writes
/// `bundle.md` and `manifest.json`. Safe to call more than once: a group
/// whose directory already carries its slug is left alone.
pub fn write_bundle(session: &mut Session) -> Result<Export> {
    // 1. Give each group directory a readable name.
    for i in 0..session.groups.len() {
        let (index, title, current) = {
            let g = &session.groups[i];
            (g.index, g.title.clone(), g.dir.clone())
        };
        let s = slug(&title);
        let desired = if s.is_empty() {
            format!("{index:02}")
        } else {
            format!("{index:02}-{s}")
        };
        if desired == current {
            continue;
        }

        let from = session.root.join(&current);
        let to = session.root.join(&desired);
        if from.exists() && !to.exists() {
            std::fs::rename(&from, &to)?;
        } else if !to.exists() {
            std::fs::create_dir_all(&to)?;
        }

        let root = session.root.clone();
        let g = &mut session.groups[i];
        g.dir = desired.clone();
        for shot in &mut g.shots {
            shot.abs_path = root
                .join(&desired)
                .join(&shot.file)
                .to_string_lossy()
                .to_string();
        }
    }

    let markdown = render_markdown(session);
    std::fs::write(session.root.join("bundle.md"), &markdown)?;
    std::fs::write(
        session.root.join("manifest.json"),
        serde_json::to_string_pretty(session)?,
    )?;

    Ok(Export {
        root: session.root.to_string_lossy().to_string(),
        markdown,
        groups: session.groups.iter().filter(|g| !g.is_empty()).count(),
        shots: session.shot_count(),
    })
}

/// The agent-facing view of the bundle. Image links are relative to the
/// session root so the folder can be moved or handed to a CLI as-is.
pub fn render_markdown(session: &Session) -> String {
    let mut md = String::new();

    let _ = writeln!(md, "# QA bundle: {}", session.title());
    let _ = writeln!(md);
    let groups: Vec<_> = session.groups.iter().filter(|g| !g.is_empty()).collect();
    let _ = writeln!(
        md,
        "{} screenshot{} across {} group{}, captured {}. Image paths are relative to this file.",
        session.shot_count(),
        if session.shot_count() == 1 { "" } else { "s" },
        groups.len(),
        if groups.len() == 1 { "" } else { "s" },
        session.started_at.get(..10).unwrap_or(&session.started_at)
    );
    let _ = writeln!(md);
    md.push_str(concat!(
        "How to read this: each group is one page or area of the product. ",
        "The quoted text under a group heading is the reviewer's note for the whole group. ",
        "Each numbered item is a screenshot of one region, followed by the reviewer's note ",
        "on what is wrong there. Open the image before acting on the note.
"
    ));

    for g in groups {
        let _ = writeln!(md);
        let _ = writeln!(md, "## {}. {}", g.index, g.heading());
        if !g.master_note.trim().is_empty() {
            let _ = writeln!(md);
            for line in g.master_note.trim().lines() {
                let _ = writeln!(md, "> {line}");
            }
        }

        if g.shots.is_empty() {
            let _ = writeln!(md);
            let _ = writeln!(md, "_No screenshots in this group._");
            continue;
        }

        for (i, shot) in g.shots.iter().enumerate() {
            let rel = format!("{}/{}", g.dir, shot.file);
            let label = format!("{}.{}", g.index, i + 1);
            let _ = writeln!(md);
            if shot.title.trim().is_empty() {
                let _ = writeln!(md, "### {label}");
            } else {
                let _ = writeln!(md, "### {label} {}", shot.title.trim());
            }
            let _ = writeln!(md);
            let _ = writeln!(md, "![{label}]({rel})");
            let _ = writeln!(md);
            if shot.note.trim().is_empty() {
                let _ = writeln!(md, "_No note._");
            } else {
                let _ = writeln!(md, "{}", shot.note.trim());
            }
            let _ = writeln!(md);
            let _ = writeln!(
                md,
                "_{} × {} px, captured {}_",
                shot.width,
                shot.height,
                shot.captured_at.get(11..19).unwrap_or(&shot.captured_at)
            );
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Shot;

    #[test]
    fn markdown_has_the_agent_facing_shape() {
        let base = std::env::temp_dir().join(format!(
            "qacut-test-md-{}",
            chrono::Local::now().timestamp_micros()
        ));
        let mut s = Session::start(&base).unwrap();
        s.name = "Settings review".into();
        s.close_group("Settings page", "Everything on this page").unwrap();
        s.current().shots.push(Shot {
            id: "a".into(),
            file: "01.png".into(),
            abs_path: String::new(),
            title: "Save button".into(),
            note: "Save button is clipped".into(),
            width: 640,
            height: 200,
            captured_at: "2026-09-17T14:56:50+00:00".into(),
        });

        let md = render_markdown(&s);
        assert!(md.starts_with("# QA bundle: Settings review
"));
        assert!(md.contains("How to read this: each group is one page or area of the product. The quoted"));
        assert!(!md.contains("  "), "no double spaces from string continuation");
        assert!(md.contains("## 1. Settings page"));
        assert!(md.contains("### 1.1 Save button"));
        assert!(md.contains("> Everything on this page"));
        assert!(md.contains("![1.1](01/01.png)"));
        assert!(md.contains("Save button is clipped"));
        assert!(md.contains("_640 × 200 px, captured 14:56:50_"));
        std::fs::remove_dir_all(base).unwrap();
    }
}
