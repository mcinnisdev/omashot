//! H.264 MP4 output for recordings, beside the GIF. Uses the encoder that
//! ships with Windows through Media Foundation, so there is nothing to
//! install. Elsewhere the writer reports unsupported and the recording
//! keeps its GIF and stills.

use anyhow::Result;
use image::RgbaImage;
use std::path::Path;

/// Frames a second the file is declared at. Samples carry their real
/// timestamps, so uneven capture still plays at wall-clock pace.
pub const FPS: u32 = 10;
/// Bits a second at 720 px wide; scaled by area for other sizes.
const BITRATE_AT_720: u32 = 2_500_000;

/// H.264 wants even dimensions.
pub fn even(n: u32) -> u32 {
    (n / 2 * 2).max(2)
}

#[cfg(windows)]
pub use win::Mp4Writer;

#[cfg(windows)]
mod win {
    use super::*;
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::Media::MediaFoundation::*;
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

    pub struct Mp4Writer {
        writer: IMFSinkWriter,
        stream: u32,
        width: u32,
        height: u32,
        last_ms: u64,
        finished: bool,
    }

    // The sink writer is used from the one recorder thread only.
    unsafe impl Send for Mp4Writer {}

    impl Mp4Writer {
        pub fn new(path: &Path, width: u32, height: u32) -> Result<Self> {
            let (width, height) = (even(width), even(height));
            let bitrate = ((BITRATE_AT_720 as f64) * (width as f64 * height as f64)
                / (720.0 * 405.0))
                .clamp(600_000.0, 8_000_000.0) as u32;

            unsafe {
                // Both calls are idempotent per thread/process; a failure to
                // initialise COM here just means it already was.
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                MFStartup(MF_VERSION, MFSTARTUP_FULL)?;

                let url = HSTRING::from(path.to_string_lossy().as_ref());
                let writer = MFCreateSinkWriterFromURL(PCWSTR(url.as_ptr()), None, None)?;

                let out = MFCreateMediaType()?;
                out.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
                out.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)?;
                out.SetUINT32(&MF_MT_AVG_BITRATE, bitrate)?;
                out.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
                out.SetUINT64(&MF_MT_FRAME_SIZE, ((width as u64) << 32) | height as u64)?;
                out.SetUINT64(&MF_MT_FRAME_RATE, ((FPS as u64) << 32) | 1)?;
                out.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1)?;
                let stream = writer.AddStream(&out)?;

                let inp = MFCreateMediaType()?;
                inp.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
                inp.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
                inp.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
                inp.SetUINT64(&MF_MT_FRAME_SIZE, ((width as u64) << 32) | height as u64)?;
                inp.SetUINT64(&MF_MT_FRAME_RATE, ((FPS as u64) << 32) | 1)?;
                inp.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1)?;
                // A negative stride tells MF the rows are top-down, which is
                // how the image crate hands them over.
                inp.SetUINT32(&MF_MT_DEFAULT_STRIDE, (-(width as i32 * 4)) as u32)?;
                writer.SetInputMediaType(stream, &inp, None)?;
                writer.BeginWriting()?;

                Ok(Mp4Writer {
                    writer,
                    stream,
                    width,
                    height,
                    last_ms: 0,
                    finished: false,
                })
            }
        }

        /// Appends one frame shown from `at_ms` for `duration_ms`. The
        /// image may be a pixel wider or taller than the stream; the edge
        /// is cropped.
        pub fn write(&mut self, img: &RgbaImage, at_ms: u64, duration_ms: u64) -> Result<()> {
            let (w, h) = (self.width, self.height);
            let len = w * h * 4;
            unsafe {
                let buffer = MFCreateMemoryBuffer(len)?;
                let mut ptr: *mut u8 = std::ptr::null_mut();
                buffer.Lock(&mut ptr, None, None)?;
                let dst = std::slice::from_raw_parts_mut(ptr, len as usize);
                for y in 0..h {
                    for x in 0..w {
                        let p = img.get_pixel(x.min(img.width() - 1), y.min(img.height() - 1)).0;
                        let i = ((y * w + x) * 4) as usize;
                        // RGB32 is BGRA in memory.
                        dst[i] = p[2];
                        dst[i + 1] = p[1];
                        dst[i + 2] = p[0];
                        dst[i + 3] = 255;
                    }
                }
                buffer.Unlock()?;
                buffer.SetCurrentLength(len)?;

                let sample = MFCreateSample()?;
                sample.AddBuffer(&buffer)?;
                sample.SetSampleTime((at_ms * 10_000) as i64)?;
                sample.SetSampleDuration((duration_ms.max(1) * 10_000) as i64)?;
                self.writer.WriteSample(self.stream, &sample)?;
            }
            self.last_ms = at_ms;
            Ok(())
        }

        pub fn finish(mut self) -> Result<()> {
            self.finished = true;
            unsafe { self.writer.Finalize()? };
            Ok(())
        }
    }

    impl Drop for Mp4Writer {
        fn drop(&mut self) {
            if !self.finished {
                unsafe {
                    let _ = self.writer.Finalize();
                }
            }
        }
    }
}

#[cfg(not(windows))]
pub struct Mp4Writer;

#[cfg(not(windows))]
impl Mp4Writer {
    pub fn new(_path: &Path, _width: u32, _height: u32) -> Result<Self> {
        Err(anyhow::anyhow!("MP4 output is Windows only for now"))
    }
    pub fn write(&mut self, _img: &RgbaImage, _at_ms: u64, _duration_ms: u64) -> Result<()> {
        Ok(())
    }
    pub fn finish(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn encodes_a_short_clip_with_the_system_encoder() {
        let path = std::env::temp_dir().join(format!(
            "qacut-test-{}.mp4",
            chrono::Local::now().timestamp_micros()
        ));
        let (w, h) = (322, 181); // odd on purpose: the writer must round down
        let mut writer = Mp4Writer::new(&path, w, h).expect("encoder available");
        for i in 0..20u64 {
            let mut img = RgbaImage::new(w, h);
            for (x, y, p) in img.enumerate_pixels_mut() {
                let v = ((x + y + i as u32 * 7) % 256) as u8;
                p.0 = [v, 255 - v, (x % 256) as u8, 255];
            }
            writer.write(&img, i * 100, 100).unwrap();
        }
        writer.finish().unwrap();
        let len = std::fs::metadata(&path).unwrap().len();
        assert!(len > 2_000, "mp4 is only {len} bytes");
        let head = std::fs::read(&path).unwrap();
        assert_eq!(&head[4..8], b"ftyp", "not an MP4 container");
        std::fs::remove_file(path).unwrap();
    }
}
