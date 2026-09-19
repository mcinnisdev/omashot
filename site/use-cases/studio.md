# QACut Studio: recordings people will actually watch

Raw screen recordings are hard to follow: the cursor is tiny, nothing marks a click, and the interesting part is a small corner of a big screen. Studio records the cursor, clicks and keystrokes as data alongside the video, then draws them back in properly, so everything about the finished video is still editable after you stop recording.

## Create polished screen recordings

**For:** a tutorial for a client, a walkthrough for a colleague, a demo of the thing you just built.

1. `Ctrl+Shift+R`. Drag the region, adjust its edges, press **Record**. A three-second countdown lets you get in place.
2. Do the thing. `Ctrl+Space` zooms in where the cursor is; again to zoom out. The key is only taken over while you record.
3. `Ctrl+Shift+R` to stop. The studio opens on the recording.
4. Tidy it, then **Export…** writes an H.264 MP4, exactly as previewed, into the recording's folder.

In the studio, everything the recording captured is now a control rather than a fact:

<div class="qc qc-embed">
<div class="qc-grid">
<div class="qc-feature"><div class="ico"><svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="7"/><path d="M16.5 16.5 21 21M8 11h6M11 8v6"/></svg></div><h3>Zoom that follows</h3><p>Zooms marked while recording become blocks on the timeline. Drag them, resize them, add more. New zooms follow the cursor, with a dead zone so they never twitch and a tightness slider.</p></div>
<div class="qc-feature"><div class="ico"><svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"/><circle cx="12" cy="12" r="8.5"/></svg></div><h3>Clicks and keys</h3><p>A smoothed, enlarged cursor with a ripple where every click landed. Keystroke badges for shortcuts only, or every key. Hide any single badge.</p></div>
<div class="qc-feature"><div class="ico"><svg viewBox="0 0 24 24"><circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M20 4 8.4 15.6M8.4 8.4 20 20"/></svg></div><h3>Trim and cut</h3><p>Start and end handles you can actually grab. Cut a stretch out of the middle, then drag it, resize it, or put it back. The preview follows every move.</p></div>
<div class="qc-feature"><div class="ico"><svg viewBox="0 0 24 24"><path d="M12 15V3M7 8l5-5 5 5M4 14v6h16v-6"/></svg></div><h3>Export that matches</h3><p>The preview and the export share one renderer. Hardware H.264 where the machine has it, AAC narration, 720p to 1440p, 30 or 60 fps.</p></div>
</div>
</div>

## Include your microphone

**For:** a walkthrough that explains itself.

Turn on **Microphone** under **Enable inputs** in the tray's Studio section, once. It stays on until you turn it off. Narration is recorded on the same clock as the frames, so it stays in sync through every trim and cut, and it's encoded as AAC in the export.

## Include your camera

**For:** a demo with a face on it.

Turn on **Camera** under **Enable inputs**. While you record, a live preview sits in a corner the region doesn't cover, so you know you're in frame. In the studio your camera becomes a bubble: circle or rounded, any corner, any size, on top of the composited video.

## Include your brand

**For:** a video that looks like it came from your company, not from a screen recorder.

Put your logo in `~/QACut/brand/`. In the studio's inspector, choose a background, set the padding and corner radius of the frame, add a title and subtitle in the space above or below it, and pick the logo for a corner. Everything is baked into the export, at the export's resolution.

## Filming yourself using QACut

Studio recordings are how the walkthroughs on this site get made. Record the screen you are working on, and the tool disappears from the footage: its overlays are excluded from capture.

Need the same steps as a document rather than a video? That is [QACut Bundles](/use-cases/bundles) with auto-capture.

[How the studio works in detail](/docs/studio)
