---
description: "Briefs in Omashot: shots grouped into sections with notes, a brief.md an agent reads top to bottom, trails that capture a process hands-free, and exported process docs."
---

# Briefs

A **brief** is the thing you hand over: a folder of shots in reading order,
grouped into sections, with a `brief.md` written for whoever gets it — an
agent, or a person.

For a single fix, a [loose shot](/docs/shots) is quicker. A brief is for a
list of problems across a whole app, or the raw material for a process
document.

## Build one

<kbd>Super</kbd><kbd>Alt</kbd><kbd>A</kbd> adds a shot to the open brief. Drag
a region, type what is wrong, press <kbd>Enter</kbd>. The brief opens itself
if there isn't one.

<kbd>Super</kbd><kbd>Alt</kbd><kbd>N</kbd> closes the current **section** and
starts the next, asking for a name and a note for the section as a whole. A
section is one page, one screen, one step of a process.

<kbd>Super</kbd><kbd>Alt</kbd><kbd>B</kbd> opens the brief. From there you can
rename it and its sections, re-note any shot, reorder shots within a section
or move them between sections, open the markup editor on any of them, and
delete the ones that turned out not to matter.

<kbd>Super</kbd><kbd>Alt</kbd><kbd>D</kbd> is done: it writes the brief out and
copies its path. The brief then closes, so the next shot starts a fresh one.

<figure class="qc-shot">
<img src="/media/docs/brief-window.png" alt="The brief window: a menubar reading Brief, Shoot, Hand off, Help, a name field, the purpose menu and the hand-off buttons." loading="lazy" />
<figcaption>An empty brief, waiting for its first shot.</figcaption>
</figure>

## Trail a process

<kbd>Super</kbd><kbd>Alt</kbd><kbd>T</kbd> starts a **trail**: Omashot captures
while you do the thing, hands free, and stops when you press it again. The
brief opens on what it collected so you can drop the noise, note the keepers
and fix the order.

Every frame is an ordinary shot — markup, notes, reordering, all of it. No
video is made, because an agent cannot read one.

::: warning On Wayland, a trail is on a timer
Triggering a shot on every click needs `/dev/input` access that Wayland does
not hand out, so a trail currently takes shots at an interval instead. It is
the one place Omashot is worse on Linux than on Windows.
:::

## What lands on disk

```
~/Omashot/2026-09-19_143022-settings-review/
  brief.md               everything in reading order, images linked relatively
  manifest.json          the same data, structured
  01-settings-page/      01.png 02.png 03.png
  02-billing/            01.png
  brand/                 a copy of ~/Omashot/brand, if you included it
```

And `brief.md` itself:

```markdown
# Brief: Settings review

3 shots across 2 sections, captured 19 Sep 2026.
Image paths are relative to this file.

## 01 · Settings page

> The whole page is the wrong width at 125% scaling.

![01](01-settings-page/01.png)
The save button is clipped on the right.
```

Relative image paths mean the folder can be moved, zipped or handed to
something that only takes uploads, and the links still resolve.

## Handing it off

From the brief window, under **Hand off**:

- **Copy agent prompt** — the path wrapped in an instruction written for the
  job. What that says depends on the **purpose** you picked; see
  [prompts](/docs/prompts).
- **Copy folder path** — just the path, for when you have your own wording.
- **Save ZIP for chat** — for an agent that only takes uploads.
- **Export document** — a finished process document straight from the brief:
  sections become headings, notes become numbered steps, images sit under
  each. One self-contained web page you can send, or Markdown beside the
  images. No agent needed.

## Reopening one

**Brief → Open a brief…** lists what is in `~/Omashot` and reopens any of
them from its `manifest.json`, so you can add to a brief you finished
yesterday. A brief that has been moved still opens: the manifest is read
relative to where the folder actually is.
