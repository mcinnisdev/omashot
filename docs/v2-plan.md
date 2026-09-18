# QACut v2: polished recordings for people

v1 (tag `v1.0.0`, branch `v1`) is the lightweight bundle tool: captures,
recordings, notes and marks handed to an agent as a folder or ZIP. It stays
as it is and remains the rollback.

v2 keeps everything v1 does and adds a second mode whose output is a video a
person watches: smooth cursor, automatic zoom that follows the work, click
and keystroke effects, trimming, several clips joined with transitions and
title cards. An agent may still embed the result, but the viewer is the
customer.

## What "high end" means here

Screen Studio, Cursorful and Loom's editor set the bar. What they share:

- The real cursor is hidden at capture time and drawn back in post, so it can
  be smoothed, enlarged, and given a click effect.
- The camera zooms to where the work is, automatically, and the operator can
  adjust every zoom afterwards.
- The frame sits on a padded background with rounded corners and a shadow,
  which is most of what reads as "polished".
- Export is a real MP4 at 30 or 60 fps.

None of that can be done while recording. v2 therefore records raw material
and renders the finished video afterwards. That is the single biggest change
from v1, where the GIF and MP4 are finished the moment recording stops.

## Architecture

Three layers. The first is Rust, the other two live in the webview.

### 1. Capture (Rust)

Records a *source*, not a result.

- **Frames.** Windows.Graphics.Capture at 30 or 60 fps into a high-bitrate
  H.264 MP4 through the Media Foundation writer v1 already has. WGC is
  GPU-side and cheap; the GDI polling v1 uses is fine at 10 fps and not at
  60. Cursor capture is switched off so the real pointer never appears.
- **Events**, timestamped against the same clock, written to `events.json`:
  - cursor position at 120 Hz (GetCursorPos on a thread; trivial cost),
  - cursor shape changes (arrow, hand, I-beam) via GetCursorInfo,
  - button down and up with position,
  - key presses through a low-level keyboard hook (`WH_KEYBOARD_LL`, no
    admin needed) so shortcuts can be shown as they are used,
  - foreground window title changes, which are natural chapter and zoom hints.
- **Narration and camera.** The microphone and, optionally, a webcam are
  recorded alongside the screen from the first milestone, so the viewer can
  hear the operator and see a small window of them driving. Both are
  captured in the webview (getUserMedia into a WebM with an audio track and
  a video track) by the recording overlay window that already exists while
  a recording runs, which also gives a live preview bubble for free. Sync is
  by a shared start signal: Rust stamps the moment the first screen frame is
  grabbed, the overlay stamps when its recorder starts, and the offset goes
  in `project.json`. MediaRecorder timestamps are monotonic, so drift over a
  walkthrough-length clip is well under a frame.
- **Project folder** per recording: `source.mp4`, `events.json`,
  `camera.webm` (audio, and video when the camera was on), `project.json`
  (every editing decision), and later `export.mp4`.

v1's quick recording (GIF plus stills for agents) stays exactly as it is.
The studio recording is a separate mode: "Record for people".

### 2. Studio: preview and editing (TypeScript in the webview)

A new window. It decodes `source.mp4` with WebCodecs, composites each frame
on a canvas, and plays it back. The same compositor draws the preview and
renders the export, so what you see is what you get.

The compositor applies, in order: trim, background and padding, zoom
transform, the frame, the synthetic cursor, click ripples, keystroke badges,
the camera bubble (corner, size, circle or rounded, shown or hidden per
segment, always above the zoomed frame), titles and transitions. Narration
plays in sync with the preview and follows every trim and cut. Every one of those is a function of `project.json`
and the events, never a change to the source.

The timeline has tracks for clips, zoom blocks, clicks and keys. Zoom blocks
are what the operator edits most: drag an edge to change when the zoom
starts or ends, drag the block to move it, drag the box in the preview to
change where it looks and how close.

### 3. Export (TypeScript, Rust fallback)

The compositor renders every output frame off-screen; WebCodecs
`VideoEncoder` produces H.264, `AudioEncoder` produces AAC from the
narration (Opus in WebM where AAC is unavailable), and a small muxer writes
the MP4. WebView2 has
had WebCodecs since Chromium 94 and uses the same Media Foundation encoder
underneath. If encoding is unavailable on a machine, frames go over IPC to
the Rust writer instead, slower but identical output. GIF and WebM come from
the same frames.

### Why the editor is in the webview and not Rust

The editing loop is UI-heavy and will change constantly. TypeScript on a
canvas is the fastest place to iterate, has GPU compositing for free, and is
the same runtime the rest of QACut's windows already use. Rust keeps what it
is good at: capture, hooks, files, encoders. If export performance on
integrated GPUs turns out to be the bottleneck, the compositor's per-frame
math is small enough to port to a Rust/wgpu renderer later without
changing the project format.

## The two algorithms that carry the feel

**Cursor smoothing.** Resample the 120 Hz path with a critically damped
spring (no overshoot, no wobble) and snap the endpoint to the click target so
a click always lands exactly where it happened. A click grows a ring for
about 400 ms. Cursor scale is a project setting; 1.5x to 2x reads well after
a zoom.

**Auto zoom.** From the events, find attention: a click, typing, or the
cursor dwelling in one area for more than about a second. Open a zoom block
there: ease in over 600 ms to a window about half the frame, centred on the
attention point, clamped to the frame. Extend the block while attention
stays inside it. Close it, easing out, when the cursor leaves for more than
a second, before a large jump, or at a window change. Two blocks closer than
about 1.5 s merge. The result is a list of blocks the operator can see and
change; the algorithm only ever proposes.

## Milestones

Each one ships something usable and can be the point where v2 stops if it
has to.

| # | Milestone | Delivers | Size |
| --- | --- | --- | --- |
| 0 | Spike | WebCodecs decode, H.264 and AAC encode confirmed inside WebView2; getUserMedia (mic and camera) confirmed in a Tauri window with the permission prompt handled; WGC capture at 60 fps confirmed; keyboard hook confirmed against the EDR your clients run | days |
| 1 | Source capture | "Record for people": WGC to high-bitrate MP4 plus events.json, cursor hidden, mic and camera to camera.webm with the sync offset, project folder | 2 weeks |
| 2 | Studio preview | Studio window: decode, composite, play, scrub with narration; padding and background; smoothed cursor; click ripple; keystroke badges; camera bubble | 2 weeks |
| 3 | Zoom and trim | Auto zoom proposals; zoom blocks editable on the timeline and in the preview; trim in and out; split, with audio following | 2–3 weeks |
| 4 | Export | MP4 at 30/60 fps with AAC narration via WebCodecs and the Rust fallback; GIF and WebM; export presets | 1–2 weeks |
| 5 | Sequences | Several clips in one project, cut and cross-fade, title and transition cards, per-clip settings including camera bubble on or off | 2 weeks |
| 6 | Polish | Brand kit applied to backgrounds, titles and watermark; presets; shortcuts; performance on integrated GPUs | ongoing |

Roughly ten to twelve focused weeks to milestone 5, working the way v1 was
built: one feature at a time, tried on a real recording each step.

## Risks, and the spike that retires each

- **WebCodecs H.264 encode inside WebView2.** Usually present on Windows 10
  and 11; verify in milestone 0 on the machines that matter. The Rust
  fallback exists either way.
- **WGC availability.** Needs Windows 10 1903 or later. Fall back to v1's
  GDI capture at 30 fps with a warning if absent.
- **Keyboard hook and endpoint security.** A low-level hook is standard
  (every screen recorder with keystroke display uses one) but some EDR
  products flag it. Test against what Castle Rock Sky's clients run. Make
  keystroke capture opt-in per recording.
- **Performance.** 1080p60 compositing on a canvas is fine on any recent
  discrete or Apple-class GPU and borderline on old integrated ones. Preview
  can drop to 30 fps; export is offline and can take as long as it takes.
- **Scope.** The full suite is large. The milestones are ordered so that
  stopping after 3 still gives a product people would pay for.

## Decisions (made 2026-09-18)

1. **Audio and camera from milestone 1.** Narration and an optional camera
   window are captured with the screen from the start, so the clock, the
   project format and the muxer are designed around them rather than
   retrofitted. Both are off by default and one click to enable per
   recording.
2. **Region and full screen first.** Window capture that follows a moving
   window comes in milestone 5.
3. **Keystroke capture is opt-in** per recording, and the hook is tested
   against the endpoint security Castle Rock Sky's clients run before it is
   on by default anywhere.
4. **Same repository.** `src/studio/` and `src-tauri/src/studio/`, opened
   from a recording's "Open in Studio" and from a new hotkey, with v1's
   windows untouched. The `v1` branch is the rollback.

## Milestone 0 results (2026-09-18, Nick's machine)

Machine: Windows 11 25H2, Intel Arc 140V, 1920 x 1200 at 60 Hz, WebView2
runtime 153 (same build as Edge 153). Spike sources are in `spikes/m0/`.

**WebCodecs in the WebView2 engine: yes.** In a headed Edge window (the
headless mode has no GPU and is not representative):

- H.264 encode, High profile (`avc1.640028`), hardware and software: supported.
  Baseline and Main at levels 3.0/3.1 report unsupported only because those
  levels cannot hold 1080p60 by spec; ask for High or raise the level.
- H.264, VP9 and AV1 decode: supported. VP9 and AV1 encode: supported.
- AAC (`mp4a.40.2`) and Opus audio encode: supported.
- MediaRecorder with `video/mp4;codecs=avc1,mp4a.40.2` and WebM VP9/Opus:
  supported. A microphone and a camera were enumerated.

So the export path (canvas compositor, `VideoEncoder` H.264, `AudioEncoder`
AAC, small muxer) and the mic and camera path (getUserMedia into
MediaRecorder) both stand. The permission prompt inside a Tauri window is
the one thing still to confirm, in milestone 1.

**Windows.Graphics.Capture: yes, but not through xcap.** xcap's WGC recorder
copies every frame through the CPU; it managed 27 fps on a static screen and
fell to 10 fps with a spinning animation on screen. The `windows-capture`
crate keeps frames on the GPU and hands them to Media Foundation directly:
with the same animation it delivered 199 frames in 5 s (median gap 20 ms,
95th percentile 36 ms) at 1920 x 1200 with the cursor hidden, and wrote a
valid 60 fps, 9 Mbit/s MP4 in real time on the built-in H.264 encoder. WGC
only produces a frame when the screen changes, so a still screen yields
fewer frames and the recorder repeats the last one at the output rate.
Decision: milestone 1 builds on `windows-capture` (its encoder for the
source MP4; v1's Media Foundation writer stays for the GIF-mode MP4).

**Low-level keyboard hook: yes.** `WH_KEYBOARD_LL` installs without
elevation and saw every synthetic and real key press during the test, with
the injected flag set on synthetic ones so they can be filtered. The
endpoint-security check on client machines is still to do before it is on
by default.

## Milestone 1 result (2026-09-18)

Shipped as "Studio recording" (`Ctrl+Shift+3`) in the QACut Studio section
of the tray. Nick's first real recording, 12 s at 1920 x 1200 with keystrokes,
microphone and camera on, produced:

- `source.mp4`: 60 fps declared, 11.8 Mbit/s, 465 frames delivered (WGC only
  produces a frame when the screen changes), cursor hidden.
- `events.json`: 781 cursor samples, 60 cursor-shape changes, the foreground
  window title, and the key presses of the stop chord.
- `camera.webm`: video and audio, with a 252 ms offset to the first source
  frame recorded in `project.json`. No permission prompt appeared; the
  preview showed in a free corner.

Two notes for milestone 2. MediaRecorder writes a WebM with no duration or
cues, so `<video>` seeking in it is poor; the studio should decode it with
WebCodecs or remux it once. And the key that triggers the stop hotkey is
not in the events, because the hook is stopped before it delivers, which is
harmless.

## Status (2026-09-18)

Milestones 1 to 4 are in: source capture with narration and camera, the
studio preview, zoom blocks (marked live or added after, following the
cursor or fixed), trim and cuts, keystroke and click effects, title and
logo, and MP4 export, all verified on real recordings. Configurable
shortcuts and the two-section tray came along the way. What remains from
the plan is milestone 5, sequences and title cards, and the polish
column, which now waits on feedback from use rather than on a plan.

## Non-goals for v2

Live streaming, cloud upload, collaborative editing, a timeline for audio
editing beyond trim and gain. Each is a product on its own and none is
needed for a polished walkthrough.
