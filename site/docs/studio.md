---
description: "Omashot Studio records the cursor, clicks and keys as data, then renders zooms, click ripples, keystroke badges, a camera bubble and narration to an MP4."
---

# Polished screen recordings

Omashot Studio records a source, then renders a finished video from it. The cursor is captured as data, not pixels, so it can be smoothed and enlarged. Every click and key press is on the same clock as the frames. The camera zooms to where the work is. What you see in the preview is what the export produces.

## Recording

Turn on what you want in the tray's Studio section: **Capture keystrokes**, **Record microphone**, **Record camera**. They stay on until you turn them off.

`Ctrl+Shift+R` opens the overlay. Drag the region, pull its edges to adjust, press **Record** or `Enter`. The badge counts down from three, then shows REC and the time. Everything outside the region stays tinted so you can tell when something has drifted out of shot. Your camera preview sits in a corner.

`Ctrl+Space` marks a zoom: the view will push in on wherever the cursor is at that moment. Press it again to zoom out. Marks become editable blocks in the studio. The key is only taken over while a recording runs, so your editor keeps it otherwise.

`Ctrl+Shift+R` stops. The studio opens on the recording.

## The studio

The preview plays the composited video: a padded background, the frame with rounded corners and a shadow, the cursor, ripples on clicks, keystroke badges, the camera bubble, and narration.

The inspector on the right sets the look. Padding, corners, background and shadow. Cursor size and smoothing. Which keystrokes show: shortcuts only, or every key. The camera bubble's size, corner and shape. A title and subtitle in the padding, and a logo from your brand folder.

The timeline under the transport shows a filmstrip of the recording with zoom blocks above and clicks and key presses as markers. Click a key marker to hide that badge.

## Zooms

Each zoom block eases in over 600 ms, holds, and eases out. Drag a block to move it, drag its edges to retime it, drag in the preview to change where it looks, and set how close with the slider. **Follow the cursor** keeps the camera on the work as the cursor moves, with a dead zone so it does not twitch; **Follow tightness** sets how eager it is. **Add zoom here** makes one at the playhead for the times you did not mark one live.

## Trim and cuts

**Start here** and **End here** set where the video begins and ends, or drag the white handles. **Cut** twice removes a stretch from the middle: once at its start, once where it should resume. Cuts are blocks like zooms: drag them, resize them, click one and Remove it. Playback skips removed material and the preview follows every drag.

## Export

**Export…** renders every kept frame at 720p, 1080p or 1440p and 30 or 60 fps, encodes H.264 with your narration as AAC, and writes an MP4 into the recording's folder. Naming the recording names the file and the folder.

Export uses the machine's hardware encoder where it has one. Expect a few times real time at 1080p30; the camera bubble is the slow part.
