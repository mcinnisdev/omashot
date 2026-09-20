---
title: A fix list for an agent
description: "Six things wrong across three pages, shot and noted into a brief an agent reads top to bottom and works through without asking you what you meant."
---

# A fix list for an agent

You have been through the app and found six things wrong across three pages.
Describing them in prose means writing six paragraphs that each begin
"on the settings page, the thing under the heading…". Screenshots in a chat
lose their order and their notes the moment there are more than two.

A brief keeps the picture and the sentence together, in order, in one folder.

## Do it

1. <kbd>Super</kbd><kbd>Alt</kbd><kbd>A</kbd>, drag the thing that is wrong,
   type what is wrong with it, <kbd>Enter</kbd>. Repeat.
2. <kbd>Super</kbd><kbd>Alt</kbd><kbd>N</kbd> when you move to the next page.
   Name the section after the page and give it a note covering the page as a
   whole.
3. <kbd>Super</kbd><kbd>Alt</kbd><kbd>B</kbd> to look over what you have —
   drop the shots that turned out not to matter, fix the order, sharpen a
   note.
4. **Hand off → Copy agent prompt**, with the purpose left on *Fix issues*.

## What the agent gets

```
Fix the issues in the brief at /home/you/Omashot/2026-09-19_143022-settings-review.
Start with brief.md: each section is a page or area, its quoted section note
covers the whole page, and each screenshot has its own note underneath.
```

It opens `brief.md`, reads it top to bottom, and opens each image beside the
note that belongs to it. Relative paths mean the folder can be moved or
zipped and the links still resolve.

## Why it works

The agent never has to guess which screenshot a sentence refers to, because
the sentence is under the screenshot. It never has to guess what "the page"
means, because the section says. And it never sees the six things you already
fixed last week, because finishing a brief closes it.

- [Briefs in full](/docs/briefs)
- [Changing what gets copied](/docs/prompts)
