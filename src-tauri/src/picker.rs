//! Choosing a region, the Omarchy way.
//!
//! On Windows the app draws its own selection overlay: one borderless
//! always-on-top window per monitor, positioned to cover it exactly. Wayland
//! has no way for a client to position a window, so that overlay cannot work
//! here and is not worth rebuilding -- Omarchy already ships a better picker
//! than the one being replaced.
//!
//! `omarchy-capture-region` freezes the screen with hyprpicker, runs slurp
//! over it with every window and monitor as a snap target, and supports the
//! keyboard navigation the rest of the desktop uses. Calling it means Omashot's
//! selection behaves exactly like a screenshot, because it is the same code.
//!
//! The pixels still come from the grab Omashot took when the key was pressed,
//! not from a second one taken afterwards, so the picker's own overlay can
//! never end up in the shot.

use std::process::Command;

use crate::capture::Frame;

/// A picked region, in the monitor-local logical coordinates the capture
/// commands expect.
pub struct Selection {
    pub monitor: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// slurp's format, which `omarchy-capture-region` also prints: "X,Y WxH",
/// in the compositor's logical coordinate space.
fn parse(out: &str) -> Option<(i32, i32, u32, u32)> {
    let line = out.lines().find(|l| !l.trim().is_empty())?.trim();
    let (pos, size) = line.split_once(char::is_whitespace)?;
    let (x, y) = pos.split_once(',')?;
    let (w, h) = size.split_once('x')?;
    Some((
        x.trim().parse().ok()?,
        y.trim().parse().ok()?,
        w.trim().parse().ok()?,
        h.trim().parse().ok()?,
    ))
}

/// Which monitor holds a point, by the frames' own logical geometry. Falls
/// back to the first frame so a selection is never silently dropped because
/// of an off-by-one at a monitor edge.
fn frame_at<'a>(frames: &'a [Frame], x: i32, y: i32) -> Option<&'a Frame> {
    frames
        .iter()
        .find(|f| {
            x >= f.x && y >= f.y && x < f.x + f.width as i32 && y < f.y + f.height as i32
        })
        .or_else(|| frames.first())
}

fn run(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    if !out.status.success() {
        // A non-zero exit is how both tools say "cancelled", which is not
        // an error worth reporting.
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Asks the user for a region and maps it onto one of the frozen frames.
///
/// Returns `None` when the pick was cancelled, which is the common case and
/// not a failure.
pub fn pick(frames: &[Frame]) -> Option<Selection> {
    // Omarchy's picker first, so the selection matches every other capture on
    // the system. Plain slurp is the fallback for a Wayland session without
    // Omarchy, where the app still works, just without the snapping.
    let out = run("omarchy-capture-region", &["smart"])
        .or_else(|| run("slurp", &["-d"]))?;

    let (sx, sy, w, h) = parse(&out)?;
    if w == 0 || h == 0 {
        return None;
    }

    let frame = frame_at(frames, sx, sy)?;
    Some(Selection {
        monitor: frame.monitor_id.clone(),
        // The commands crop relative to the monitor, and scale to physical
        // pixels themselves.
        x: (sx - frame.x) as f64,
        y: (sy - frame.y) as f64,
        width: w as f64,
        height: h as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_slurp_output() {
        assert!(matches!(parse("100,200 640x480\n"), Some((100, 200, 640, 480))));
    }

    #[test]
    fn parses_negative_origin() {
        // A monitor left of the primary one has negative coordinates.
        assert!(matches!(parse("-1920,0 800x600"), Some((-1920, 0, 800, 600))));
    }

    #[test]
    fn rejects_rubbish() {
        assert!(parse("").is_none());
        assert!(parse("cancelled").is_none());
        assert!(parse("100,200").is_none());
    }

    fn frame(id: &str, x: i32, y: i32, w: u32, h: u32) -> Frame {
        Frame {
            monitor_id: id.into(),
            x,
            y,
            width: w,
            height: h,
            scale: 1.0,
            png_path: String::new(),
        }
    }

    #[test]
    fn picks_the_monitor_the_point_is_on() {
        let frames = vec![
            frame("left", -1920, 0, 1920, 1080),
            frame("right", 0, 0, 2560, 1440),
        ];
        assert_eq!(frame_at(&frames, 10, 10).unwrap().monitor_id, "right");
        assert_eq!(frame_at(&frames, -5, 10).unwrap().monitor_id, "left");
        // The right edge belongs to the next monitor, not this one.
        assert_eq!(frame_at(&frames, -1920, 0).unwrap().monitor_id, "left");
    }
}
