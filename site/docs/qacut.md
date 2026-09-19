# Bundles for agents

A bundle is a folder: screenshots and clips in groups, a note on each, and a markdown file that reads in order. It is built for handing to an AI agent, whether that is a bug list to fix or a set of steps to turn into documentation.

## Quick shots

Not everything needs a bundle. `Ctrl+Shift+1` freezes the screen, you drag a region and type a note, and `Enter` saves it. `Ctrl+Enter` instead puts the screenshot's path and your note on the clipboard, ready to paste into an agent with whatever else you want to say. The file lands in a batch folder under `~/QACut/Quick/`, named for when the batch started, with the note beside it as a small markdown file and a `notes.md` listing every shot in the batch.

To send a few together, just keep going: `Ctrl+Shift+1`, note, `Enter`, again. `Ctrl+Enter` on the last shot (or **Finish batch**, or the tray's **Finish quick batch and copy paths**) copies every shot in the batch with its note, plus the folder path, in one paste, and closes the batch. The next quick shot starts a fresh folder, so an agent is never pointed at shots you have already dealt with. **New batch** moves the shot you are noting into a fresh folder without closing the old one. Use quick shots for one-off fixes and short sessions; use a bundle when there are enough shots across enough pages that an agent needs the structure to keep them straight.

## Capturing

`Ctrl+Shift+2` freezes the screen. Drag a region and a note box appears under it. The header reads **Group 1 / Shot 1**; click either name and type to rename it. `Enter` saves, `Shift+Enter` adds a line, `Esc` keeps the screenshot with no note.

`Ctrl+Shift+3` auto-captures a region instead: a GIF and an MP4 of the region, plus a still at every click labelled with where the click landed. Press it again to stop. Use this for a process; use screenshots for faults.

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

Finishing closes the bundle: the next capture starts a new one. To add to a finished bundle, press **Capture here** on one of its groups in the bundle window, or reopen it later from **Bundle → Open bundle…**.

**Save ZIP for chat** zips the folder, shows the archive in Explorer ready to drag into a chat, and copies a prompt that says "the attached ZIP".

The markdown file explains itself at the top: each group is a page or area, the quoted note under a heading applies to everything in the group, and each numbered item is a screenshot with its note. Recordings list their stills as actions: "3 s, click at 412,188".

## Purposes

- **Fix issues** asks the agent to work through the screenshots and fix what the notes describe.
- **Write process doc** asks for a step-by-step document, as Markdown or as one self-contained web page with the clips playing inline.
- **Custom prompt** is yours. `{root}` becomes the folder path and `{name}` the bundle name.
