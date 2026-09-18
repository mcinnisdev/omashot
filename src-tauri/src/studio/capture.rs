//! Source capture: Windows.Graphics.Capture through the `windows-capture`
//! crate, frames kept on the GPU and handed straight to its Media
//! Foundation H.264 encoder. The cursor is not captured; the studio draws
//! its own from the events.

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(windows)]
pub use win::{start, SourceCapture};

/// Bits per second for a given frame size: about 5.5 Mbit/s per megapixel,
/// which keeps UI text crisp at 60 fps without ballooning the file.
pub fn bitrate_for(width: u32, height: u32) -> u32 {
    let mpx = width as f64 * height as f64 / 1_000_000.0;
    (mpx * 5_500_000.0).clamp(4_000_000.0, 30_000_000.0) as u32
}

/// Shared with the capture thread: the first frame's timestamp and a count.
#[derive(Clone)]
pub struct Shared {
    pub first_ts: Arc<Mutex<Option<i64>>>,
    pub frames: Arc<AtomicU64>,
}

#[derive(Clone)]
pub struct Flags {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub shared: Shared,
}

#[cfg(windows)]
mod win {
    use super::*;
    use windows_capture::capture::{CaptureControl, Context, GraphicsCaptureApiHandler};
    use windows_capture::encoder::{
        AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder,
        VideoSettingsSubType,
    };
    use windows_capture::frame::Frame;
    use windows_capture::graphics_capture_api::InternalCaptureControl;
    use windows_capture::monitor::Monitor;
    use windows_capture::settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    };

    type BoxError = Box<dyn std::error::Error + Send + Sync>;

    pub struct Handler {
        encoder: Option<VideoEncoder>,
        shared: Shared,
    }

    impl GraphicsCaptureApiHandler for Handler {
        type Flags = Flags;
        type Error = BoxError;

        fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
            let f = ctx.flags;
            // H.264 rather than the crate's HEVC default: it decodes on
            // every machine the studio might run on.
            let encoder = VideoEncoder::new(
                VideoSettingsBuilder::new(f.width, f.height)
                    .sub_type(VideoSettingsSubType::H264)
                    .frame_rate(60)
                    .bitrate(bitrate_for(f.width, f.height)),
                AudioSettingsBuilder::default().disabled(true),
                ContainerSettingsBuilder::default(),
                &f.path,
            )?;
            Ok(Handler {
                encoder: Some(encoder),
                shared: f.shared,
            })
        }

        fn on_frame_arrived(
            &mut self,
            frame: &mut Frame,
            _control: InternalCaptureControl,
        ) -> Result<(), Self::Error> {
            {
                let mut first = self.shared.first_ts.lock().unwrap();
                if first.is_none() {
                    *first = Some(frame.timestamp()?.Duration);
                }
            }
            self.shared.frames.fetch_add(1, Ordering::Relaxed);
            if let Some(enc) = self.encoder.as_mut() {
                enc.send_frame(frame)?;
            }
            Ok(())
        }

        fn on_closed(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    pub struct SourceCapture {
        control: CaptureControl<Handler, BoxError>,
        shared: Shared,
    }

    /// Starts capturing the monitor under the screen point `(sx, sy)`.
    pub fn start(sx: i32, sy: i32, path: &Path) -> Result<SourceCapture> {
        use windows_sys::Win32::Foundation::POINT;
        use windows_sys::Win32::Graphics::Gdi::{MonitorFromPoint, MONITOR_DEFAULTTONEAREST};

        let hmon = unsafe { MonitorFromPoint(POINT { x: sx, y: sy }, MONITOR_DEFAULTTONEAREST) };
        if hmon.is_null() {
            return Err(anyhow!("no monitor at that point"));
        }
        let monitor = Monitor::from_raw_hmonitor(hmon);
        let width = monitor.width().map_err(|e| anyhow!("monitor width: {e}"))?;
        let height = monitor.height().map_err(|e| anyhow!("monitor height: {e}"))?;

        let shared = Shared {
            first_ts: Arc::new(Mutex::new(None)),
            frames: Arc::new(AtomicU64::new(0)),
        };
        let settings = Settings::new(
            monitor,
            CursorCaptureSettings::WithoutCursor,
            DrawBorderSettings::WithoutBorder,
            SecondaryWindowSettings::Default,
            MinimumUpdateIntervalSettings::Default,
            DirtyRegionSettings::Default,
            ColorFormat::Bgra8,
            Flags {
                path: path.to_path_buf(),
                width,
                height,
                shared: shared.clone(),
            },
        );
        let control = Handler::start_free_threaded(settings)
            .map_err(|e| anyhow!("capture start: {e}"))?;
        Ok(SourceCapture { control, shared })
    }

    impl SourceCapture {
        /// Finishes the file and ends the session. Returns the first frame's
        /// timestamp (100 ns units on the QPC clock) and the frame count.
        pub fn stop(self) -> Result<(i64, u64)> {
            {
                let cb = self.control.callback();
                let mut handler = cb.lock();
                if let Some(enc) = handler.encoder.take() {
                    enc.finish().map_err(|e| anyhow!("encoder finish: {e}"))?;
                }
            }
            self.control
                .stop()
                .map_err(|e| anyhow!("capture stop: {e}"))?;
            let first = self.shared.first_ts.lock().unwrap().unwrap_or(0);
            Ok((first, self.shared.frames.load(Ordering::Relaxed)))
        }
    }
}

#[cfg(not(windows))]
pub struct SourceCapture;

#[cfg(not(windows))]
pub fn start(_sx: i32, _sy: i32, _path: &Path) -> Result<SourceCapture> {
    Err(anyhow!("studio capture is Windows only for now"))
}

#[cfg(not(windows))]
impl SourceCapture {
    pub fn stop(self) -> Result<(i64, u64)> {
        Ok((0, 0))
    }
}
