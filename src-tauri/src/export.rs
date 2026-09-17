use crate::model::Session;
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

/// Turns a title into a directory-safe suffix: "Settings page" -> "settings-page".
fn slug(title: &str) -> String {
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

    let _ = writeln!(md, "# QA bundle {}", session.id);
    let _ = writeln!(md);
    let groups: Vec<_> = session.groups.iter().filter(|g| !g.is_empty()).collect();
    let _ = writeln!(
        md,
        "{} screenshot{} across {} group{}. Image paths are relative to this file.",
        session.shot_count(),
        if session.shot_count() == 1 { "" } else { "s" },
        groups.len(),
        if groups.len() == 1 { "" } else { "s" }
    );

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
            let _ = writeln!(md, "### {label}");
            let _ = writeln!(md);
            let _ = writeln!(md, "![{label}]({rel})");
            let _ = writeln!(md);
            if shot.note.trim().is_empty() {
                let _ = writeln!(md, "_No note._");
            } else {
                let _ = writeln!(md, "{}", shot.note.trim());
            }
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_directory_safe() {
        assert_eq!(slug("Settings page"), "settings-page");
        assert_eq!(slug("  Billing / Invoices!! "), "billing-invoices");
        assert_eq!(slug(""), "");
        assert_eq!(slug("---"), "");
    }
}
