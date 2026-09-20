---
description: "Loose shots in Omashot: one screenshot with a note, marked up, copied as a path for an agent or as an image for a person."
---

# Shots

A **shot** is one screenshot with a note on it. Taken outside a brief, it is a
**loose shot**: the fastest way to tell an agent, or a person, exactly what you
are looking at.

For a bigger job — several pages, a whole flow, something worth sections — see
[briefs](/docs/briefs).

## Take one

<kbd>Super</kbd><kbd>Alt</kbd><kbd>Q</kbd> freezes the screen. Drag a region.
The shot opens large with the markup tools and a note box beside it.

<figure class="qc-shot">
<img src="/media/docs/shot-window.png" alt="A shot open in Omashot: the image with markup tools along the bottom, and the note box on the right." loading="lazy" />
<figcaption>A shot, its markup tools and its note. Omashot is wearing the Solitude theme here, because that is what the desktop was set to.</figcaption>
</figure>

Type what is wrong and press <kbd>Enter</kbd>. Take another. Keep going.

## Hand them off

**Finish and hand off** (<kbd>Ctrl</kbd><kbd>Enter</kbd>) copies every loose
shot with its note, ready to paste:

```
/home/you/Omashot/Quick/2026-09-19_101512/01.png
The save button is clipped at 125% scaling.

/home/you/Omashot/Quick/2026-09-19_101512/02.png
Same at 150%. The footer overlaps the form.
```

That clears them. The next shot starts a new folder, so an agent is never
pointed at work you have already dealt with. From a terminal, `omashot copy`
does the same thing.

## Or hand one to a person

**Copy image** (<kbd>Ctrl</kbd><kbd>Shift</kbd><kbd>C</kbd>) puts the
marked-up PNG on the clipboard as an image rather than a path. That is the
fastest answer to "which button do you mean?" — paste it straight into a
chat, an email or a ticket.

## Marking up

| Tool | Key | For |
| --- | --- | --- |
| Move | <kbd>M</kbd> | Select and reposition a mark |
| Arrow | <kbd>A</kbd> | Point at the thing |
| Highlight | <kbd>H</kbd> | Box the area |
| Blur | <kbd>B</kbd> | Remove something |
| Step | <kbd>S</kbd> | Numbered counters, in order |

Marks are kept as data in a `.marks.json` beside the shot, with the untouched
original as `.orig.png`. That means an edit can be reopened and adjusted
later rather than painted over a flattened image.

Blur genuinely removes the pixels underneath — it is not a grey rectangle laid
on top — so a blurred shot is safe to send.

## Marking up something you already have

`omashot edit some-screenshot.png` adopts an image that already exists, files
it as a loose shot and opens it for markup.

That is also how Omashot can take over Omarchy's own screenshot key. Put this
in `~/.config/environment.d/omashot.conf` and log back in:

```
OMARCHY_SCREENSHOT_EDITOR=omashot-edit
```

Now every <kbd>Super</kbd><kbd>Shift</kbd><kbd>S</kbd> can become a noted shot
with one click on the notification.

::: tip Draw in omasnap instead
Add `OMASHOT_MARKUP=omasnap` beside it and the drawing goes to
[omasnap](https://github.com/tobi/omasnap) first, with Omashot picking the
result back up. You gain omasnap's tools and lose re-editable marks, since it
flattens what it saves.
:::

## Where they go

```
~/Omashot/Quick/2026-09-19_101512/
  01.png
  01.orig.png        the untouched original, once you mark it up
  01.marks.json      the marks, so the edit can be reopened
  02.png
```

Set `$OMASHOT_DIR` to put `~/Omashot` somewhere else.
