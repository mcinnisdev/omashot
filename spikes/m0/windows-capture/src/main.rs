// Spike: Windows.Graphics.Capture through the windows-capture crate, frames
// kept GPU-side and handed straight to its Media Foundation H.264 encoder.
// Measures delivered frame rate over 5 s on the primary monitor with the
// cursor hidden, and writes spike.mp4 beside the binary's cwd.
use std::time::{Duration, Instant};

use windows_capture::capture::{Context, GraphicsCaptureApiHandler};
use windows_capture::encoder::{
    AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder,
};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::monitor::Monitor;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};

const SECONDS: u64 = 5;

struct Spike {
    encoder: Option<VideoEncoder>,
    start: Instant,
    times: Vec<Instant>,
    size: (u32, u32),
}

impl GraphicsCaptureApiHandler for Spike {
    type Flags = (u32, u32);
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let (w, h) = ctx.flags;
        let encoder = VideoEncoder::new(
            VideoSettingsBuilder::new(w, h).frame_rate(60).bitrate(12_000_000),
            AudioSettingsBuilder::default().disabled(true),
            ContainerSettingsBuilder::default(),
            "spike.mp4",
        )?;
        Ok(Self { encoder: Some(encoder), start: Instant::now(), times: Vec::new(), size: (w, h) })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        self.times.push(Instant::now());
        self.size = (frame.width(), frame.height());
        if let Some(enc) = self.encoder.as_mut() {
            enc.send_frame(frame)?;
        }
        if self.start.elapsed() >= Duration::from_secs(SECONDS) {
            if let Some(enc) = self.encoder.take() {
                enc.finish()?;
            }
            let n = self.times.len();
            let mut gaps: Vec<f64> = self
                .times
                .windows(2)
                .map(|w| (w[1] - w[0]).as_secs_f64() * 1000.0)
                .collect();
            gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let fps = n as f64 / self.start.elapsed().as_secs_f64();
            let p50 = gaps.get(gaps.len() / 2).copied().unwrap_or(0.0);
            let p95 = gaps.get(gaps.len() * 95 / 100).copied().unwrap_or(0.0);
            let max = gaps.last().copied().unwrap_or(0.0);
            let bytes = std::fs::metadata("spike.mp4").map(|m| m.len()).unwrap_or(0);
            println!(
                "WC frames={n} fps={fps:.1} size={}x{} gap_ms p50={p50:.1} p95={p95:.1} max={max:.1} mp4_bytes={bytes} ({:.0} KB/s)",
                self.size.0,
                self.size.1,
                bytes as f64 / 1024.0 / SECONDS as f64
            );
            control.stop();
        }
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {
    let monitor = Monitor::primary().expect("primary monitor");
    let w = monitor.width().expect("width");
    let h = monitor.height().expect("height");
    let settings = Settings::new(
        monitor,
        CursorCaptureSettings::WithoutCursor,
        DrawBorderSettings::WithoutBorder,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Bgra8,
        (w, h),
    );
    Spike::start(settings).expect("capture");
}
