use anyhow::{anyhow, Result};
use image::RgbaImage;
use serde::Serialize;
use std::path::{Path, PathBuf};

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
