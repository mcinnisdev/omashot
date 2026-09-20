---
title: A walkthrough
description: "When reading it is worse than watching it: a screen recording with cursor smoothing, click ripples, follow zooms and narration. Capturing on Linux is not there yet."
---

# A walkthrough

Some things are worse to read than to watch. A drag, a timing, a thing that
only makes sense in motion.

::: danger Not working on Linux yet
Omashot can edit, composite and export a recording on Linux, but **capturing a
source is still Windows-only**. Until that lands, use
`omarchy capture screenrecording` for a walkthrough on Omarchy.

The rest of this page describes what the editor does with a recording once it
exists.
:::

## Why it is not just a screen recording

The video is only half of what gets recorded. Where the cursor was, what it
looked like, every click and keystroke, which window was in front — all of it
is captured beside the video as data on the same clock, and nothing is burned
into the frames.

So the cursor in the export is drawn, not filmed. That is why it can be
smoothed and enlarged afterwards, why every click gets a ripple, and why you
can add a zoom to a moment you did not think to mark at the time.

## What you do

Record. Then, in the editor: trim the dead air off both ends, cut the stretch
where you went to find the other window, drop a zoom on the part that matters,
turn on a camera bubble if it should be you explaining it, and export.

The preview is the export — both go through the same compositor — so there is
no render-and-discover step.

## What it is not for

An agent. A video is the wrong artifact for something that reads: it cannot
open frame 412, and it cannot be diffed, searched or quoted. If the audience is
an agent, that is a [brief](/use-cases/fix-list) or
[a trail](/use-cases/trail-a-process).

- [Recordings in full](/docs/recordings)
