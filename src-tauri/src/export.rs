use crate::model::{sidecars, slug, Session, ShotKind};
use anyhow::Result;
use serde::Serialize;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

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
    /// Set when the bundle was also zipped; the path of the archive.
    pub zip_path: Option<String>,
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

        // Files that lived in the old directory now live in the new one.
        // A shot moved here from another group still points at that
        // group's directory; the layout pass below brings it over.
        for g in &mut session.groups {
            for shot in &mut g.shots {
                if let Ok(rest) = Path::new(&shot.abs_path).strip_prefix(&from) {
                    shot.abs_path = to.join(rest).to_string_lossy().to_string();
                }
            }
        }
        session.groups[i].dir = desired;
    }

    // 2. Files follow reading order.
    normalize_layout(session)?;

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
        zip_path: None,
    })
}

/// Zips the (already written) bundle folder to `<folder>.zip` beside it,
/// for hand-off to chat agents that only take uploads. Entries are prefixed
/// with the folder name so unzipping yields one folder, and any earlier
/// archive is replaced.
pub fn write_zip(session: &Session) -> Result<PathBuf> {
    use std::io::Write as _;
    use zip::write::SimpleFileOptions;

    let root = &session.root;
    let folder = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| anyhow::anyhow!("bundle folder has no name"))?;
    let parent = root
        .parent()
        .ok_or_else(|| anyhow::anyhow!("bundle folder has no parent"))?;
    let zip_path = parent.join(format!("{folder}.zip"));

    let file = std::fs::File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(std::io::BufWriter::new(file));
    let opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let mut files = Vec::new();
    list_files(root, root, &mut files);
    files.sort();
    for rel in files {
        let mut src = std::fs::File::open(root.join(&rel))?;
        zip.start_file(format!("{folder}/{rel}"), opts)?;
        std::io::copy(&mut src, &mut zip)?;
    }
    zip.finish()?.flush()?;
    Ok(zip_path)
}

fn ext_of(file: &str) -> String {
    Path::new(file)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "png".into())
}

/// Lays the folder out to match the session after reorders and moves:
/// every shot ends up in its group's directory as `NN.ext` in reading
/// order, with a recording's stills beside it as `NN-frames/`. Two passes
/// through temporary names, so shots swapping numbers never overwrite each
/// other. Paths in the session are updated to match.
fn normalize_layout(session: &mut Session) -> std::io::Result<()> {
    let root = session.root.clone();

    // Pass 1: everything to a temporary, collision-free name in its group.
    for g in &mut session.groups {
        let dir = root.join(&g.dir);
        std::fs::create_dir_all(&dir)?;
        for shot in &mut g.shots {
            let tmp = dir.join(format!("tmp-{}.{}", shot.id, ext_of(&shot.file)));
            move_shot_files(shot, &tmp)?;
        }
    }

    // Pass 2: final names in reading order.
    for g in &mut session.groups {
        let dir = root.join(&g.dir);
        for (i, shot) in g.shots.iter_mut().enumerate() {
            let name = format!("{:02}.{}", i + 1, ext_of(&shot.file));
            move_shot_files(shot, &dir.join(&name))?;
            shot.file = name;
            let stem = format!("{:02}-frames", i + 1);
            for f in &mut shot.frames {
                let base = Path::new(&f.file)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                f.file = format!("{stem}/{base}");
            }
        }
    }
    Ok(())
}

/// Renames a shot's file, its markup sidecars, and its key-frame directory
/// if it has one, to `dest`, then points the shot at it.
fn move_shot_files(shot: &mut crate::model::Shot, dest: &Path) -> std::io::Result<()> {
    let cur = PathBuf::from(&shot.abs_path);
    if cur == dest {
        return Ok(());
    }
    let old_frames = shot.frames_dir();
    if cur.exists() {
        std::fs::rename(&cur, dest)?;
    }
    for (from, to) in sidecars(&cur).iter().zip(sidecars(dest).iter()) {
        if from.exists() {
            std::fs::rename(from, to)?;
        }
    }
    shot.abs_path = dest.to_string_lossy().to_string();
    if let (Some(from), Some(to)) = (old_frames, shot.frames_dir()) {
        if from.exists() && from != to {
            std::fs::rename(&from, &to)?;
        }
    }
    Ok(())
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
        "A screenshot marked auto-captured is one of a sequence taken while the reviewer ",
        "did something: at the start, at each click or Enter (with where the click landed, ",
        "ringed in the image), and at the end. Read them in order; each is one action. ",
        "A file ending .orig.png is the unedited original behind an annotated image; ",
        "use the annotated one.\n"
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
                if let Some(v) = &shot.video {
                    let _ = write!(md, " Video: [{}/{v}]({}/{v}) (H.264 MP4, same clip).", g.dir, g.dir);
                }
                let _ = writeln!(md);
                if !shot.frames.is_empty() {
                    let _ = writeln!(md);
                    let _ = writeln!(md, "Key frames:");
                    let _ = writeln!(md);
                    for f in &shot.frames {
                        let _ = writeln!(md, "- [{}]({}/{})", f.label(), g.dir, f.file);
                    }
                }
            } else if let Some(m) = &shot.moment {
                let _ = writeln!(
                    md,
                    "_{} × {} px, auto-captured at {}, captured {when}_",
                    shot.width,
                    shot.height,
                    m.label()
                );
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
            video: None,
            moment: None,
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
                KeyFrame { file: "02-frames/01.png".into(), at_ms: 0, event: "start".into(), x: None, y: None },
                KeyFrame { file: "02-frames/02.png".into(), at_ms: 3140, event: "click".into(), x: Some(412), y: Some(188) },
                KeyFrame { file: "02-frames/03.png".into(), at_ms: 12400, event: "end".into(), x: None, y: None },
            ],
            video: Some("02.mp4".into()),
            moment: None,
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
        assert!(md.contains("_Recording, 12 s, 720 × 400 px, captured 14:57:10._ Video: [01/02.mp4](01/02.mp4)"));
        assert!(md.contains("- [0 s, start](01/02-frames/01.png)"));
        assert!(md.contains("- [3 s, click at 412,188](01/02-frames/02.png)"));
        assert!(md.contains("- [12 s, end](01/02-frames/03.png)"));
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

#[cfg(test)]
mod layout_tests {
    use super::*;
    use crate::model::{KeyFrame, Shot, ShotKind};

    fn file_shot(s: &mut Session, id: &str, ext: &str, kind: ShotKind) {
        let (_, file, abs) = s.reserve_shot(ext);
        std::fs::write(&abs, id.as_bytes()).unwrap();
        let mut sh = Shot {
            id: id.into(),
            file,
            abs_path: abs.to_string_lossy().to_string(),
            title: String::new(),
            note: String::new(),
            width: 1,
            height: 1,
            captured_at: String::new(),
            kind,
            duration_ms: 0,
            frames: Vec::new(),
            video: None,
            moment: None,
        };
        if kind == ShotKind::Recording {
            let dir = sh.frames_dir().unwrap();
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("01.png"), b"f").unwrap();
            let rel = format!(
                "{}/01.png",
                dir.file_name().unwrap().to_string_lossy()
            );
            sh.frames.push(KeyFrame { file: rel, at_ms: 0, event: "start".into(), x: None, y: None });
        }
        s.current().shots.push(sh);
    }

    #[test]
    fn export_lays_files_out_in_reading_order_after_moves() {
        let base = std::env::temp_dir().join(format!(
            "qacut-test-layout-{}",
            chrono::Local::now().timestamp_micros()
        ));
        std::fs::create_dir_all(&base).unwrap();
        let mut s = Session::start(&base.join("b")).unwrap();
        file_shot(&mut s, "a", "png", ShotKind::Image);
        file_shot(&mut s, "b", "png", ShotKind::Image);
        file_shot(&mut s, "rec", "gif", ShotKind::Recording);
        s.close_group("First", "").unwrap();
        file_shot(&mut s, "d", "png", ShotKind::Image);

        // Reverse group 1 and pull d in front of everything there.
        assert!(s.move_shot(1, "rec", 1, 0));
        assert!(s.move_shot(1, "b", 1, 0));
        assert!(s.move_shot(2, "d", 1, 0));

        let ex = write_bundle(&mut s, &base.join("nobrand")).unwrap();

        // Reading order is d, b, rec, a; files say so and hold the right bytes.
        let g = &s.groups[0];
        let names: Vec<_> = g.shots.iter().map(|x| x.file.as_str()).collect();
        assert_eq!(names, ["01.png", "02.png", "03.gif", "04.png"]);
        let dir = s.root.join("01-first");
        assert_eq!(std::fs::read(dir.join("01.png")).unwrap(), b"d");
        assert_eq!(std::fs::read(dir.join("02.png")).unwrap(), b"b");
        assert_eq!(std::fs::read(dir.join("03.gif")).unwrap(), b"rec");
        assert_eq!(std::fs::read(dir.join("04.png")).unwrap(), b"a");
        assert!(dir.join("03-frames/01.png").exists());
        assert_eq!(g.shots[2].frames[0].file, "03-frames/01.png");
        assert!(ex.markdown.contains("![1.3](01-first/03.gif)"));
        assert!(ex.markdown.contains("- [0 s, start](01-first/03-frames/01.png)"));

        // The old group-2 file is gone from its old home and nothing is left over.
        let leftovers: Vec<_> = std::fs::read_dir(s.root.join("02"))
            .unwrap()
            .flatten()
            .collect();
        assert!(leftovers.is_empty());
        assert!(std::fs::read_dir(&dir).unwrap().flatten().all(|e| {
            !e.file_name().to_string_lossy().starts_with("tmp-")
        }));

        // Exporting again is a no-op on disk.
        let before: Vec<_> = s.groups[0].shots.iter().map(|x| x.abs_path.clone()).collect();
        write_bundle(&mut s, &base.join("nobrand")).unwrap();
        let after: Vec<_> = s.groups[0].shots.iter().map(|x| x.abs_path.clone()).collect();
        assert_eq!(before, after);

        std::fs::remove_dir_all(base).unwrap();
    }
}

#[cfg(test)]
mod zip_tests {
    use super::*;

    #[test]
    fn zip_holds_the_whole_folder_under_one_directory() {
        let base = std::env::temp_dir().join(format!(
            "qacut-test-zip-{}",
            chrono::Local::now().timestamp_micros()
        ));
        std::fs::create_dir_all(&base).unwrap();
        let mut s = Session::start(&base).unwrap();
        s.rename("Zipped").unwrap();
        let (_, file, abs) = s.reserve_shot("png");
        std::fs::write(&abs, b"png").unwrap();
        s.current().shots.push(crate::model::Shot {
            id: "a".into(),
            file,
            abs_path: abs.to_string_lossy().to_string(),
            title: String::new(),
            note: "n".into(),
            width: 1,
            height: 1,
            captured_at: String::new(),
            kind: ShotKind::Image,
            duration_ms: 0,
            frames: Vec::new(),
            video: None,
            moment: None,
        });
        write_bundle(&mut s, &base.join("nobrand")).unwrap();
        let zip_path = write_zip(&s).unwrap();
        assert!(zip_path.ends_with(format!("{}-zipped.zip", s.id)));

        let mut archive = zip::ZipArchive::new(std::fs::File::open(&zip_path).unwrap()).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        let folder = format!("{}-zipped", s.id);
        assert!(names.contains(&format!("{folder}/bundle.md")));
        assert!(names.contains(&format!("{folder}/manifest.json")));
        assert!(names.contains(&format!("{folder}/01/01.png")));
        assert!(names.iter().all(|n| n.starts_with(&folder)));
        std::fs::remove_dir_all(base).unwrap();
    }
}
