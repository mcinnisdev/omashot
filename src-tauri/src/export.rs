use crate::model::{slug, Session, ShotKind};
use anyhow::Result;
use serde::Serialize;
use std::fmt::Write as _;
use std::path::Path;

/// Notes file inside the brand folder that is inlined into bundle.md.
pub const BRAND_NOTES: &str = "brand.md";

/// What the user keeps in `~/QACut/brand/`: voice notes plus any files.
#[derive(Clone, Debug, Default, Serialize)]
pub struct BrandKit {
    pub notes: String,
    /// Paths relative to the brand folder, sorted, `brand.md` excluded.
    pub files: Vec<String>,
}

impl BrandKit {
    pub fn load(dir: &Path) -> BrandKit {
        let notes = std::fs::read_to_string(dir.join(BRAND_NOTES)).unwrap_or_default();
        let mut files = Vec::new();
        list_files(dir, dir, &mut files);
        files.retain(|f| f != BRAND_NOTES);
        files.sort();
        BrandKit { notes, files }
    }

    pub fn is_empty(&self) -> bool {
        self.notes.trim().is_empty() && self.files.is_empty()
    }
}

fn list_files(base: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            list_files(base, &path, out);
        } else if let Ok(rel) = path.strip_prefix(base) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)?.flatten() {
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if src.is_dir() {
            copy_dir(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize)]
pub struct Export {
    pub root: String,
    pub markdown: String,
    pub groups: usize,
    pub shots: usize,
}

/// Renames group directories to include their titles, copies the brand kit
/// in (or out) as the session asks, then writes `bundle.md` and
/// `manifest.json`. Safe to call more than once: a group whose directory
/// already carries its slug is left alone.
pub fn write_bundle(session: &mut Session, brand_src: &Path) -> Result<Export> {
    // 0. Brand kit: a fresh copy every time so removed files do not linger.
    let brand_dst = session.root.join("brand");
    let kit = BrandKit::load(brand_src);
    let brand = if session.include_brand && !kit.is_empty() {
        let _ = std::fs::remove_dir_all(&brand_dst);
        copy_dir(brand_src, &brand_dst)?;
        Some(kit)
    } else {
        let _ = std::fs::remove_dir_all(&brand_dst);
        None
    };

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

    let markdown = render_markdown(session, brand.as_ref());
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
pub fn render_markdown(session: &Session, brand: Option<&BrandKit>) -> String {
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
        "on what is wrong there. Open the image before acting on the note. ",
        "A recording is an animated GIF; the key frames listed under it are stills at ",
        "regular intervals, so read those in order if you cannot play it.\n"
    ));

    if let Some(kit) = brand {
        let _ = writeln!(md);
        let _ = writeln!(md, "## Brand kit");
        let _ = writeln!(md);
        let _ = writeln!(
            md,
            "The `brand/` folder describes the business this bundle is for. Match its \
             voice and visual identity in anything you produce."
        );
        if !kit.notes.trim().is_empty() {
            let _ = writeln!(md);
            let _ = writeln!(md, "{}", kit.notes.trim());
        }
        if !kit.files.is_empty() {
            let _ = writeln!(md);
            let _ = writeln!(md, "Files:");
            let _ = writeln!(md);
            for f in &kit.files {
                let _ = writeln!(md, "- `brand/{f}`");
            }
        }
    }

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
            let when = shot.captured_at.get(11..19).unwrap_or(&shot.captured_at);
            if shot.kind == ShotKind::Recording {
                let secs = (shot.duration_ms as f64 / 1000.0).round() as u64;
                let _ = write!(
                    md,
                    "_Recording, {secs} s, {} × {} px, captured {when}._",
                    shot.width, shot.height
                );
                if !shot.frames.is_empty() {
                    let _ = write!(md, " Key frames:");
                    for (k, f) in shot.frames.iter().enumerate() {
                        let sep = if k == 0 { " " } else { ", " };
                        let _ = write!(
                            md,
                            "{sep}[{} s]({}/{})",
                            (f.at_ms as f64 / 1000.0).round() as u64,
                            g.dir,
                            f.file
                        );
                    }
                }
                let _ = writeln!(md);
            } else {
                let _ = writeln!(
                    md,
                    "_{} × {} px, captured {when}_",
                    shot.width, shot.height
                );
            }
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{KeyFrame, Shot};

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
            kind: ShotKind::Image,
            duration_ms: 0,
            frames: Vec::new(),
        });
        s.current().shots.push(Shot {
            id: "b".into(),
            file: "02.gif".into(),
            abs_path: String::new(),
            title: String::new(),
            note: "Open the menu, pick Export".into(),
            width: 720,
            height: 400,
            captured_at: "2026-09-17T14:57:10+00:00".into(),
            kind: ShotKind::Recording,
            duration_ms: 12400,
            frames: vec![
                KeyFrame { file: "02-frames/01.png".into(), at_ms: 0 },
                KeyFrame { file: "02-frames/02.png".into(), at_ms: 2010 },
            ],
        });

        let md = render_markdown(&s, None);
        assert!(md.starts_with("# QA bundle: Settings review\n"));
        assert!(!md.contains("## Brand kit"));
        assert!(md.contains("How to read this: each group is one page or area of the product. The quoted"));
        assert!(!md.contains("  "), "no double spaces from string continuation");
        assert!(md.contains("## 1. Settings page"));
        assert!(md.contains("### 1.1 Save button"));
        assert!(md.contains("> Everything on this page"));
        assert!(md.contains("![1.1](01/01.png)"));
        assert!(md.contains("Save button is clipped"));
        assert!(md.contains("_640 × 200 px, captured 14:56:50_"));
        assert!(md.contains("![1.2](01/02.gif)"));
        assert!(md.contains("_Recording, 12 s, 720 × 400 px, captured 14:57:10._ Key frames: [0 s](01/02-frames/01.png), [2 s](01/02-frames/02.png)"));
        std::fs::remove_dir_all(base).unwrap();
    }
}

#[cfg(test)]
mod brand_tests {
    use super::*;

    #[test]
    fn brand_kit_is_copied_in_and_described() {
        let base = std::env::temp_dir().join(format!(
            "qacut-test-brand-{}",
            chrono::Local::now().timestamp_micros()
        ));
        let brand = base.join("brand");
        std::fs::create_dir_all(brand.join("fonts")).unwrap();
        std::fs::write(brand.join(BRAND_NOTES), "Plain, warm, never salesy.").unwrap();
        std::fs::write(brand.join("logo.png"), b"png").unwrap();
        std::fs::write(brand.join("fonts/Inter.ttf"), b"ttf").unwrap();

        let mut s = Session::start(&base.join("bundles")).unwrap();
        let ex = write_bundle(&mut s, &brand).unwrap();
        assert!(s.root.join("brand/logo.png").exists());
        assert!(s.root.join("brand/fonts/Inter.ttf").exists());
        assert!(ex.markdown.contains("## Brand kit"));
        assert!(ex.markdown.contains("Plain, warm, never salesy."));
        assert!(ex.markdown.contains("- `brand/fonts/Inter.ttf`"));
        assert!(ex.markdown.contains("- `brand/logo.png`"));
        assert!(!ex.markdown.contains("brand/brand.md"));

        // Opting out removes the copy again.
        s.include_brand = false;
        let ex = write_bundle(&mut s, &brand).unwrap();
        assert!(!s.root.join("brand").exists());
        assert!(!ex.markdown.contains("## Brand kit"));

        // An empty kit is never copied even when included.
        s.include_brand = true;
        let ex = write_bundle(&mut s, &base.join("nowhere")).unwrap();
        assert!(!s.root.join("brand").exists());
        assert!(!ex.markdown.contains("## Brand kit"));

        std::fs::remove_dir_all(base).unwrap();
    }
}
