# QACut: quick shots for your agent

**For:** the moment you spot a UI problem while an agent is already working with you, and you want it fixed without breaking your stride.

You are reviewing a page. The save button is clipped. You could describe it in words, or you could show it. `Ctrl+Shift+1`, drag the button, type "clipped at 125% scaling", `Ctrl+Enter`, paste. The agent gets a path it can open and the note that tells it what to do.

## The flow

1. `Ctrl+Shift+1`. The screen freezes.
2. Drag the region that matters.
3. Type what is wrong, or what you want.
4. `Ctrl+Enter`. Paste into your agent.

Ten seconds, no window to manage, nothing to name.

## A few at once

Three small things on the same page? Take them one after another, pressing `Enter` on each note instead of `Ctrl+Enter`. The note box counts the batch. On the last one, `Ctrl+Enter` copies all of them with their notes, so the agent gets one message with three screenshots and three instructions.

The batch then closes. Your next quick shot starts a fresh folder, and an agent you talk to later never sees the shots you already had fixed.

## What the agent gets

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

Plain paths and plain text. It works with a CLI agent that can open files, and the folder is small enough to zip and drop into a chat.

## When to step up

When the list grows past a handful, or spans several pages, or you want the agent to write documentation rather than fix things, switch to [QACut Bundles](/use-cases/bundles). Groups, master notes and a purpose-specific prompt keep a big job legible.

[How quick shots work in detail](/docs/quick)
