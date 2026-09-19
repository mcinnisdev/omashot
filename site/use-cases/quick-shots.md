# QACut Basic: quick shots

One hotkey, one region, and the shot opens large with the markup tools and a note beside it. Where it goes next is up to you: the clipboard as an image, or an agent as a path and a note.

## Copy and paste

**For:** "Here's what I'm seeing." A screenshot into a chat, an email or a ticket, with nothing else in the way.

1. `Ctrl+Shift+1`. The screen freezes.
2. Drag the region.
3. **Copy image**, or `Ctrl+Shift+C`. Paste.

The shot is also saved in the batch folder under `~/QACut/Quick/`, so it's still there if you need it later. `Esc` closes the window and keeps it.

## Mark up, copy, paste

**For:** "Which button do you mean?" The same screenshot with an arrow on the button, the field highlighted, and the customer's name blurred.

1. `Ctrl+Shift+1`, drag the region.
2. Pick a tool along the top: **Arrow**, **Highlight**, **Blur**, **Step** counter. Draw. `Ctrl+Z` undoes.
3. **Copy image**. Paste.

Blur is pixelation, so the text underneath is really gone, not softened. The untouched original stays beside the file as `01.orig.png` in case you need it.

## Agent feedback loops

**For:** the moment you spot a UI problem while an agent is already working with you, and you want it fixed without breaking your stride.

1. `Ctrl+Shift+1`, drag the button that's wrong.
2. Type what's wrong in the note.
3. `Ctrl+Enter`. Paste into your agent.

The agent gets a path it can open and the note that tells it what to do:

```
C:\Users\nick\QACut\Quick\2026-09-19_101512\01.png
The save button is clipped at 125% scaling.
```

Three small things on the same page? Press `Enter` on each note instead of `Ctrl+Enter` and keep going. The window counts the batch. On the last one, `Ctrl+Enter` copies all of them with their notes, so the agent gets one message with three screenshots and three instructions:

```
3 quick shots in C:\Users\nick\QACut\Quick\2026-09-19_101512. Each PNG has
its note in the .md beside it; notes.md lists them all.

C:\Users\nick\QACut\Quick\2026-09-19_101512\01.png
The save button is clipped at 125% scaling.

C:\Users\nick\QACut\Quick\2026-09-19_101512\02.png
This toggle animates but the state never saves.

C:\Users\nick\QACut\Quick\2026-09-19_101512\03.png
Align this label with the field above it.
```

The batch then closes. Your next quick shot starts a fresh folder, and an agent you talk to later never sees the shots you already had fixed.

## When to step up

When the list grows past a handful, or spans several pages, or you want a document rather than a fix, switch to [QACut Bundles](/use-cases/bundles). Groups, master notes and a purpose-specific prompt keep a big job legible.

[How quick shots work in detail](/docs/quick)
