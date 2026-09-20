---
description: "Recordings in Omashot: cursor smoothing, click ripples, follow zooms, camera and narration, trim and cut, exported to MP4. Capturing a source on Linux is not there yet."
---

# Recordings

A brief is for an agent. A **recording** is for a person: a walkthrough
someone will actually watch.

::: danger Not working on Linux yet
Omashot can edit, composite and export a recording on Linux, but **capturing
a source is still Windows-only**. The Linux path will go through
`gpu-screen-recorder`, which Omarchy already ships and which is a better
recorder than the one being replaced.

Everything below describes what the editor does with a recording once it
exists. Until the capture side lands, the honest answer for a walkthrough on
Omarchy is `omarchy capture screenrecording`.
:::

## The idea

The screen is recorded as video, and everything else — where the cursor was,
what it looked like, every click, every keystroke, which window was in front —
is recorded beside it as data on the same clock.

Nothing is burned in. The cursor you see in the export is drawn from the data,
which is why it can be smoothed, enlarged and given a ripple after the fact,
and why a zoom can be added to a moment you did not mark while recording.

## What the editor does

- **Zooms.** Marked live with <kbd>Super</kbd><kbd>Alt</kbd><kbd>Z</kbd>, or
  added later. A zoom follows the cursor with a dead zone, so it never
  twitches, and a tightness slider decides how closely.
- **Trim and cut.** Grab the handles to trim; cut a stretch out of the middle
  and put it back if you change your mind. The preview follows every move.
- **Cursor, clicks and keys.** Smoothed and enlarged cursor, a ripple on every
  click, keystroke badges for shortcuts only or for every key, any badge
  hideable.
- **Camera and narration.** A camera bubble in any corner at any size, and
  microphone narration, both in sync.
- **Frame.** Padding, corner radius, background, a title above the frame, your
  logo in a corner — see [brand kit](/docs/brand-kit).
- **Export.** H.264 MP4 with AAC narration, 720p to 1440p, 30 or 60 fps,
  exactly as previewed, written into the recording's own folder.

The preview *is* the export: both go through the same compositor, so what you
see while editing is what lands in the file.

## Keystroke badges

Also not available on Linux. Recording what you typed needs `/dev/input`
access that Wayland does not hand out, so the badges are Windows-only for the
same reason a [trail](/docs/briefs#trail-a-process) falls back to a timer.
