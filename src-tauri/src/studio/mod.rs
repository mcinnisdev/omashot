//! v2 "Record for people": records a source, not a result. A high-bitrate
//! MP4 of the monitor with the cursor hidden, plus every input and context
//! event on the same clock, into a project folder the studio edits later.
//! Nothing here touches v1's GIF recordings.

pub mod capture;
pub mod events;
pub mod project;
pub mod settings;

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::capture::Frame;
use project::{Project, Rect};

/// What the overlay reported when its microphone/camera recorder started.
#[derive(Clone, Copy, Debug)]
pub struct CameraStart {
    /// On the frames' clock (100 ns units).
    pub ts: i64,
    pub has_video: bool,
    pub has_audio: bool,
}

/// A studio recording in progress.
pub struct Active {
    pub dir: PathBuf,
    capture: capture::SourceCapture,
    events: events::EventRecorder,
    project: Project,
    started: Instant,
    pub camera: Option<CameraStart>,
}

/// A recording whose screen side has stopped while the overlay flushes
/// the last camera chunks.
pub struct Finishing {
    pub dir: PathBuf,
    pub camera: Option<CameraStart>,
}

impl Finishing {
    /// Fills in the camera track from what landed in `camera.webm` and the
    /// start timestamp, then saves the project.
    pub fn finalize(&self) -> Result<()> {
        let mut project = Project::load(&self.dir)?;
        let file = self.dir.join("camera.webm");
        let has_file = std::fs::metadata(&file).map(|m| m.len() > 0).unwrap_or(false);
        project.camera = match (self.camera, has_file) {
            (Some(c), true) => Some(project::Camera {
                file: "camera.webm".into(),
                offset_ms: (c.ts - project.first_frame_ts) / 10_000,
                has_video: c.has_video,
                has_audio: c.has_audio,
            }),
            _ => {
                let _ = std::fs::remove_file(&file);
                None
            }
        };
        project.save(&self.dir)
    }
}

/// Starts capturing the monitor under the region. The region is in CSS
/// pixels relative to the overlay window, like everything else the overlay
/// hands over; it is stored in physical pixels relative to the monitor.
#[allow(clippy::too_many_arguments)]
pub fn begin(
    base: &Path,
    frame: &Frame,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    keystrokes: bool,
) -> Result<Active> {
    let s = frame.scale;
    let region = Rect {
        x: (x * s).round() as i32,
        y: (y * s).round() as i32,
        width: (w * s).round() as u32,
        height: (h * s).round() as u32,
    };
    let monitor = Rect {
        x: frame.x,
        y: frame.y,
        width: (frame.width as f64 * s).round() as u32,
        height: (frame.height as f64 * s).round() as u32,
    };

    let dir = project::new_project_dir(base)?;
    let source = dir.join("source.mp4");

    // Events first so nothing that happens during encoder start-up is lost;
    // timestamps are absolute on the QPC clock and made relative to the
    // first frame at the end.
    let events = events::EventRecorder::start(keystrokes)?;
    let capture = capture::start(monitor.x + region.x, monitor.y + region.y, &source)?;

    let project = Project::new(&dir, monitor, s, region, keystrokes);
    Ok(Active {
        dir,
        capture,
        events,
        project,
        started: Instant::now(),
        camera: None,
    })
}

impl Active {
    /// The zoom hotkey while recording. Returns whether a zoom is now on.
    pub fn mark_zoom(&mut self) -> bool {
        self.events.mark_zoom()
    }

    /// Stops both recorders, writes `events.json` and `project.json`, and
    /// returns what is left to do once the overlay has flushed the camera.
    pub fn stop(self) -> Result<Finishing> {
        let (first_ts, frames) = self.capture.stop()?;
        let events = self.events.stop();
        let duration_ms = self.started.elapsed().as_millis() as u64;

        let mut project = self.project;
        project.first_frame_ts = first_ts;
        project.frames = frames;
        project.duration_ms = duration_ms;

        let out = events.relative_to(first_ts, &project.monitor);
        std::fs::write(
            self.dir.join("events.json"),
            serde_json::to_string(&out)?,
        )?;
        project.save(&self.dir)?;
        Ok(Finishing {
            dir: self.dir,
            camera: self.camera,
        })
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    /// A real three-second recording of the primary monitor, without the
    /// keyboard hook. Needs a display; that is the point.
    #[test]
    fn records_a_source_and_events_end_to_end() {
        let base = std::env::temp_dir().join(format!(
            "omashot-test-studio-{}",
            chrono::Local::now().timestamp_micros()
        ));
        std::fs::create_dir_all(&base).unwrap();

        let m = xcap::Monitor::all().unwrap().into_iter().next().unwrap();
        let scale = m.scale_factor().unwrap_or(1.0) as f64;
        let (pw, ph) = (m.width().unwrap(), m.height().unwrap());
        let frame = Frame {
            monitor_id: "test".into(),
            x: m.x().unwrap_or(0),
            y: m.y().unwrap_or(0),
            width: (pw as f64 / scale).round() as u32,
            height: (ph as f64 / scale).round() as u32,
            scale,
            png_path: String::new(),
        };

        let active = begin(&base, &frame, 10.0, 10.0, 400.0, 300.0, false).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(3));
        let finishing = active.stop().unwrap();
        finishing.finalize().unwrap();
        let dir = finishing.dir.clone();

        let project = Project::load(&dir).unwrap();
        assert!(project.frames > 0, "no frames captured");
        assert!(project.camera.is_none());
        assert!(project.first_frame_ts > 0);
        assert_eq!(project.region.x, (10.0 * scale).round() as i32);
        assert_eq!(project.monitor.width, pw);
        let src = std::fs::metadata(dir.join("source.mp4")).unwrap().len();
        assert!(src > 10_000, "source.mp4 is only {src} bytes");

        let ev: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("events.json")).unwrap()).unwrap();
        let cursor = ev["cursor"].as_array().unwrap();
        assert!(cursor.len() >= 20, "only {} cursor samples in 3 s", cursor.len());
        assert!(!ev["shapes"].as_array().unwrap().is_empty());
        assert!(!ev["windows"].as_array().unwrap().is_empty());
        // Times are relative to the first frame, so they straddle zero at
        // most slightly and then run to about three seconds.
        let last_t = cursor.last().unwrap()[0].as_f64().unwrap();
        assert!(last_t > 2500.0 && last_t < 4000.0, "last cursor sample at {last_t} ms");

        std::fs::remove_dir_all(base).unwrap();
    }
}
