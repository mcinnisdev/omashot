use anyhow::{anyhow, Result};
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame as GifFrame, RgbaImage};
use serde::Serialize;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tauri::AppHandle;

use crate::model::KeyFrame;

/// One monitor's frozen frame. Geometry is in logical points (what Tauri
/// uses to place windows); `scale` converts to the physical pixels the
/// captured image is actually in.
#[derive(Clone, Debug, Serialize)]
pub struct Frame {
    pub monitor_id: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    /// PNG on disk so the overlay webview can display it.
    pub png_path: String,
}

/// Captures every monitor and writes each frame to `tmp`. Returns the frames
/// alongside the in-memory images, which are kept so cropping does not have to
/// decode the PNG again.
pub fn freeze_all(tmp: &Path) -> Result<Vec<(Frame, RgbaImage)>> {
    std::fs::create_dir_all(tmp)?;

    let monitors = xcap::Monitor::all().map_err(|e| anyhow!("monitor enumeration: {e}"))?;
    if monitors.is_empty() {
        return Err(anyhow!("no monitors found"));
    }

    let mut out = Vec::new();
    for m in monitors {
        let image = match m.capture_image() {
            Ok(img) => img,
            // A monitor can refuse capture (locked, DRM-protected output).
            // Skip it rather than aborting the whole capture.
            Err(_) => continue,
        };

        let id = m.id().map(|v| v.to_string()).unwrap_or_else(|_| "0".into());
        let scale = m.scale_factor().unwrap_or(1.0) as f64;
        let phys_w = image.width();
        let phys_h = image.height();

        let png_path = tmp.join(format!("frame-{id}.png"));
        image.save(&png_path)?;

        out.push((
            Frame {
                monitor_id: id,
                x: m.x().unwrap_or(0),
                y: m.y().unwrap_or(0),
                // Logical size, so the overlay window covers the monitor exactly.
                width: (phys_w as f64 / scale).round() as u32,
                height: (phys_h as f64 / scale).round() as u32,
                scale,
                png_path: png_path.to_string_lossy().to_string(),
            },
            image,
        ));
    }

    if out.is_empty() {
        return Err(anyhow!("every monitor refused capture"));
    }
    Ok(out)
}

/// Crops a selection given in CSS pixels relative to the overlay window,
/// writes it to `dest`, and returns the physical pixel size written.
pub fn crop_selection(
    frame: &Frame,
    src: &RgbaImage,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    dest: &Path,
) -> Result<(u32, u32)> {
    let s = frame.scale;
    let px = (x * s).round().max(0.0) as u32;
    let py = (y * s).round().max(0.0) as u32;
    let mut pw = (w * s).round() as u32;
    let mut ph = (h * s).round() as u32;

    if pw == 0 || ph == 0 {
        return Err(anyhow!("selection was empty"));
    }
    // Clamp to the frame so a drag that ran off the edge still works.
    pw = pw.min(src.width().saturating_sub(px));
    ph = ph.min(src.height().saturating_sub(py));
    if pw == 0 || ph == 0 {
        return Err(anyhow!("selection fell outside the monitor"));
    }

    let cropped = image::imageops::crop_imm(src, px, py, pw, ph).to_image();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    cropped.save(dest)?;
    Ok((pw, ph))
}

/// Per-run scratch directory for frozen frames.
pub fn scratch_dir() -> PathBuf {
    std::env::temp_dir().join("qacut-frames")
}

pub fn clear_scratch() {
    let _ = std::fs::remove_dir_all(scratch_dir());
}

// ------------------------------------------------------------- recording

/// Target frame interval. Ten frames a second reads fine for a process
/// demonstration and keeps the GIF small.
const FRAME_INTERVAL: Duration = Duration::from_millis(100);
/// Frames wider than this are scaled down before encoding.
const MAX_WIDTH: u32 = 720;
/// How often a still is saved beside the GIF.
const KEYFRAME_EVERY: Duration = Duration::from_secs(2);
/// Stills are thinned to at most this many when the recording stops.
const MAX_KEYFRAMES: usize = 12;
/// A recording nobody stopped ends itself here.
const MAX_DURATION: Duration = Duration::from_secs(10 * 60);

/// What a finished recording produced.
pub struct Recorded {
    pub width: u32,
    pub height: u32,
    pub duration_ms: u64,
    pub frames: Vec<KeyFrame>,
}

/// A recording in progress. Frames are captured, cursor-marked, scaled and
/// streamed into the GIF on a worker thread, so stopping is immediate.
pub struct Recording {
    stop: Arc<AtomicBool>,
    handle: JoinHandle<Result<Recorded>>,
}

impl Recording {
    pub fn stop(self) -> Result<Recorded> {
        self.stop.store(true, Ordering::SeqCst);
        self.handle
            .join()
            .map_err(|_| anyhow!("recorder thread panicked"))?
    }
}

/// Starts recording a region of `frame`'s monitor. The selection is in CSS
/// pixels relative to the overlay window, like `crop_selection`. `gif` is
/// the output file; stills go in `frames_dir`, named relative to `rel_base`
/// (the group directory) for the manifest.
#[allow(clippy::too_many_arguments)]
pub fn start_recording(
    app: AppHandle,
    frame: &Frame,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    gif: PathBuf,
    frames_dir: PathBuf,
) -> Result<Recording> {
    let s = frame.scale;
    let px = (x * s).round().max(0.0) as u32;
    let py = (y * s).round().max(0.0) as u32;
    let phys_w = (frame.width as f64 * s).round() as u32;
    let phys_h = (frame.height as f64 * s).round() as u32;
    let pw = ((w * s).round() as u32).min(phys_w.saturating_sub(px));
    let ph = ((h * s).round() as u32).min(phys_h.saturating_sub(py));
    if pw == 0 || ph == 0 {
        return Err(anyhow!("selection was empty"));
    }

    let monitor_id = frame.monitor_id.clone();
    let (mon_x, mon_y) = (frame.x, frame.y);
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();

    let handle = std::thread::spawn(move || -> Result<Recorded> {
        let monitor = xcap::Monitor::all()
            .map_err(|e| anyhow!("monitor enumeration: {e}"))?
            .into_iter()
            .find(|m| m.id().map(|v| v.to_string()).unwrap_or_default() == monitor_id)
            .ok_or_else(|| anyhow!("that monitor is gone"))?;

        let (out_w, out_h) = if pw > MAX_WIDTH {
            (MAX_WIDTH, (ph as f64 * MAX_WIDTH as f64 / pw as f64).round().max(1.0) as u32)
        } else {
            (pw, ph)
        };
        // Drawn before downscaling, so size it to land at 12 px afterwards.
        let ring = (12.0 * pw as f64 / out_w as f64).round().max(6.0) as i64;

        std::fs::create_dir_all(&frames_dir)?;
        let file = BufWriter::new(std::fs::File::create(&gif)?);
        let mut enc = GifEncoder::new_with_speed(file, 30);
        enc.set_repeat(Repeat::Infinite)?;

        let rel_dir = frames_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let start = Instant::now();
        let mut last = start;
        let mut last_key: Option<Instant> = None;
        let mut frames: Vec<KeyFrame> = Vec::new();

        loop {
            let tick = Instant::now();
            let mut img = match monitor.capture_region(px, py, pw, ph) {
                Ok(i) => i,
                // A transient failure (screen lock, resolution change)
                // should not end the recording; just skip the frame.
                Err(_) => {
                    if flag.load(Ordering::SeqCst) {
                        break;
                    }
                    std::thread::sleep(FRAME_INTERVAL);
                    continue;
                }
            };

            if let Ok(pos) = app.cursor_position() {
                let cx = pos.x.round() as i64 - mon_x as i64 - px as i64;
                let cy = pos.y.round() as i64 - mon_y as i64 - py as i64;
                mark_cursor(&mut img, cx, cy, ring);
            }

            let img = if out_w != pw {
                image::imageops::resize(&img, out_w, out_h, image::imageops::FilterType::Triangle)
            } else {
                img
            };

            let now = Instant::now();
            if last_key.map_or(true, |t| now - t >= KEYFRAME_EVERY) {
                let name = format!("{:02}.png", frames.len() + 1);
                if img.save(frames_dir.join(&name)).is_ok() {
                    frames.push(KeyFrame {
                        file: format!("{rel_dir}/{name}"),
                        at_ms: (now - start).as_millis() as u64,
                    });
                }
                last_key = Some(now);
            }

            // Each frame is shown for as long as the previous one really
            // took, so the GIF keeps wall-clock time even if capture lags.
            let delay_ms = ((now - last).as_millis() as u32).max(20);
            last = now;
            enc.encode_frame(GifFrame::from_parts(
                img,
                0,
                0,
                Delay::from_numer_denom_ms(delay_ms, 1),
            ))?;

            if flag.load(Ordering::SeqCst) || start.elapsed() > MAX_DURATION {
                break;
            }
            let spent = tick.elapsed();
            if spent < FRAME_INTERVAL {
                std::thread::sleep(FRAME_INTERVAL - spent);
            }
        }
        drop(enc);

        let frames = thin_keyframes(frames, &frames_dir);

        Ok(Recorded {
            width: out_w,
            height: out_h,
            duration_ms: start.elapsed().as_millis() as u64,
            frames,
        })
    });

    Ok(Recording { stop, handle })
}

/// Draws a translucent ring where the cursor is, since screen grabs do not
/// include it and a demonstration without a pointer is hard to follow.
fn mark_cursor(img: &mut RgbaImage, cx: i64, cy: i64, r: i64) {
    let (w, h) = (img.width() as i64, img.height() as i64);
    if cx < -r || cy < -r || cx > w + r || cy > h + r {
        return;
    }
    for dy in -r - 2..=r + 2 {
        for dx in -r - 2..=r + 2 {
            let (x, y) = (cx + dx, cy + dy);
            if x < 0 || y < 0 || x >= w || y >= h {
                continue;
            }
            let d = ((dx * dx + dy * dy) as f64).sqrt();
            let rf = r as f64;
            let alpha = if d > rf + 1.5 {
                0.0
            } else if d >= rf - 2.5 {
                0.9
            } else {
                0.25
            };
            if alpha == 0.0 {
                continue;
            }
            let p = img.get_pixel_mut(x as u32, y as u32);
            let blend = |c: u8, t: u8| (c as f64 * (1.0 - alpha) + t as f64 * alpha).round() as u8;
            p.0 = [blend(p.0[0], 255), blend(p.0[1], 196), blend(p.0[2], 0), 255];
        }
    }
}

/// Keeps at most `MAX_KEYFRAMES` stills, evenly spaced, first and last
/// included, and deletes the rest from disk.
fn thin_keyframes(frames: Vec<KeyFrame>, dir: &Path) -> Vec<KeyFrame> {
    if frames.len() <= MAX_KEYFRAMES {
        return frames;
    }
    let n = frames.len();
    let keep: std::collections::BTreeSet<usize> = (0..MAX_KEYFRAMES)
        .map(|i| (i * (n - 1)) / (MAX_KEYFRAMES - 1))
        .collect();
    let mut kept = Vec::new();
    for (i, f) in frames.into_iter().enumerate() {
        if keep.contains(&i) {
            kept.push(f);
        } else if let Some(name) = Path::new(&f.file).file_name() {
            let _ = std::fs::remove_file(dir.join(name));
        }
    }
    kept
}
