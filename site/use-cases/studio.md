# QACut Studio: recordings people will actually watch

**For:** showing a person how something is done. A tutorial for a client, a walkthrough for a colleague, a demo of the thing you just built.

Raw screen recordings are hard to follow: the cursor is tiny, nothing marks a click, and the interesting part is a small corner of a big screen. Studio records the cursor, clicks and keystrokes as data alongside the video, then draws them back in properly: a smoothed, enlarged cursor, a ripple on every click, keystroke badges, and zooms that follow the work.

## The flow

1. In the tray, turn on what you want under **Enable inputs**: keystrokes, microphone, camera.
2. `Ctrl+Shift+R`. Drag the region, adjust its edges, press **Record**. A three-second countdown lets you get in place.
3. Do the thing. `Ctrl+Space` zooms in where the cursor is; again to zoom out. The key is only taken over while you record.
4. `Ctrl+Shift+R` to stop. The studio opens on the recording.

## In the studio

Everything the recording captured is now editable, after the fact. Everything Screen Studio and Loom charge for, drawn from data you already recorded.

<div class="qc qc-embed">
<div class="qc-grid">
<div class="qc-feature"><div class="ico"><svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="7"/><path d="M16.5 16.5 21 21M8 11h6M11 8v6"/></svg></div><h3>Zoom that follows</h3><p>A zoom block pushes in with an eased 600 ms move, holds still while the cursor moves inside the middle of the view, and eases after it near the edges. Tightness is a slider.</p></div>
<div class="qc-feature"><div class="ico"><svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"/><circle cx="12" cy="12" r="8.5"/></svg></div><h3>Clicks and keys</h3><p>A ripple where every click landed. Keystroke badges for shortcuts only, or every key. Hide any single badge from the timeline.</p></div>
<div class="qc-feature"><div class="ico"><svg viewBox="0 0 24 24"><path d="M4 7h3l2-2.5h6L17 7h3v11H4z"/><circle cx="12" cy="12.5" r="3.5"/></svg></div><h3>Camera and voice</h3><p>Your camera in a circle or rounded bubble, any corner, any size. Narration recorded on the same clock and cut with the video.</p></div>
<div class="qc-feature"><div class="ico"><svg viewBox="0 0 24 24"><circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M20 4 8.4 15.6M8.4 8.4 20 20"/></svg></div><h3>Trim and cut</h3><p>Start and end handles you can actually grab. Cut a stretch out of the middle, then drag it, resize it, or put it back. A filmstrip shows where you are.</p></div>
<div class="qc-feature"><div class="ico"><svg viewBox="0 0 24 24"><path d="M4 5h13M10.5 5v14"/><rect x="16" y="15" width="5" height="5" rx="1"/></svg></div><h3>Title and logo</h3><p>A title and subtitle in the padding above or below the frame, and a logo from your brand folder in a corner of it. Five backgrounds, or plain.</p></div>
<div class="qc-feature"><div class="ico"><svg viewBox="0 0 24 24"><path d="M12 15V3M7 8l5-5 5 5M4 14v6h16v-6"/></svg></div><h3>Export that matches</h3><p>The preview and the export share one renderer. Hardware H.264 where the machine has it, AAC narration, 30 or 60 fps, written straight into the recording's folder.</p></div>
</div>
</div>

## Export

**Export…** writes an H.264 MP4 with AAC narration, 720p to 1440p, 30 or 60 fps, exactly as previewed. Hardware encoding where the machine has it. The file lands in the recording's folder, ready to send.

## Filming yourself using QACut

Studio recordings are how the walkthroughs on this site get made. Record the screen you are working on, and the tool disappears from the footage: its overlays are excluded from capture.

Need the same steps as a document rather than a video? That is [QACut Bundles](/use-cases/bundles) with auto-capture.

[How the studio works in detail](/docs/studio)
