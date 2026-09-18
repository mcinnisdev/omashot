# Bundles for agents

A bundle is a folder: screenshots and clips in groups, a note on each, and a markdown file that reads in order. It is built for handing to an AI agent, whether that is a bug list to fix or a set of steps to turn into documentation.

## Capturing

`Ctrl+Shift+2` freezes the screen. Drag a region and a note box appears under it. The header reads **Group 1 / Shot 1**; click either name and type to rename it. `Enter` saves, `Shift+Enter` adds a line, `Esc` keeps the screenshot with no note.

`Ctrl+Shift+R` auto-captures a region instead: a GIF and an MP4 of the region, plus a still at every click labelled with where the click landed. Press it again to stop. Use this for a process; use screenshots for faults.

`Ctrl+Shift+G` wraps up the group you just captured with a master note and opens the next group. Master notes describe the page or area; shot notes describe what is wrong or what is happening.

## The bundle window

`Ctrl+Shift+Q` shows the bundle: every group and shot, editable in place. From here you can

- rename the bundle, which renames its folder,
- drag shots to reorder them or move them between groups,
- point new captures at an earlier group with **Capture here**,
- open a screenshot in the editor to add arrows, highlights, step counters, or blur something sensitive,
- drop a bad still from a recording before it reaches the agent,
- choose the bundle's purpose: fix issues, write a process doc, or your own prompt.

## Handing off

**Copy agent prompt** writes the bundle and copies an instruction with the folder path filled in. Paste it into a CLI agent.

**Save ZIP for chat** zips the folder, shows the archive in Explorer ready to drag into a chat, and copies a prompt that says "the attached ZIP".

The markdown file explains itself at the top: each group is a page or area, the quoted note under a heading applies to everything in the group, and each numbered item is a screenshot with its note. Recordings list their stills as actions: "3 s, click at 412,188".

## Purposes

- **Fix issues** asks the agent to work through the screenshots and fix what the notes describe.
- **Write process doc** asks for a step-by-step document, as Markdown or as one self-contained web page with the clips playing inline.
- **Custom prompt** is yours. `{root}` becomes the folder path and `{name}` the bundle name.
