# QACut Studio: recordings people will actually watch

**For:** showing a person how something is done. A tutorial for a client, a walkthrough for a colleague, a demo of the thing you just built.

Raw screen recordings are hard to follow: the cursor is tiny, nothing marks a click, and the interesting part is a small corner of a big screen. Studio records the cursor, clicks and keystrokes as data alongside the video, then draws them back in properly: a smoothed, enlarged cursor, a ripple on every click, keystroke badges, and zooms that follow the work.

## The flow

1. In the tray, turn on what you want under **Enable inputs**: keystrokes, microphone, camera.
2. `Ctrl+Shift+R`. Drag the region, adjust its edges, press **Record**. A three-second countdown lets you get in place.
3. Do the thing. `Ctrl+Space` zooms in where the cursor is; again to zoom out. The key is only taken over while you record.
4. `Ctrl+Shift+R` to stop. The studio opens on the recording.

## In the studio

Everything the recording captured is now editable, after the fact:

- **Zooms** are blocks on the timeline. Drag them, resize them, add more. New zooms follow the cursor, with a dead zone so they never twitch and a tightness slider for how closely they track.
- **Trim and cut.** Drag the start and end handles, or cut a stretch out of the middle and put it back if you change your mind. The preview follows every move.
- **Cursor, clicks and keys.** Cursor size, ripple, badges for shortcuts only or every key. Hide any single badge.
- **Camera and narration.** Your camera in a bubble, any corner, any size. Narration recorded on the same clock and cut with the video.
- **Frame.** Padding, corner radius, a background, a title above the frame, your logo in a corner.

## Export

**Export…** writes an H.264 MP4 with AAC narration, 720p to 1440p, 30 or 60 fps, exactly as previewed. Hardware encoding where the machine has it. The file lands in the recording's folder, ready to send.

## Filming yourself using QACut

Studio recordings are how the walkthroughs on this site get made. Record the screen you are working on, and the tool disappears from the footage: its overlays are excluded from capture.

Need the same steps as a document rather than a video? That is [QACut Bundles](/use-cases/bundles) with auto-capture.

[How the studio works in detail](/docs/studio)
