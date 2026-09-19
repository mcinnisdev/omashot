# QACut Bundles: bigger jobs

A bundle is a folder: screenshots in groups, a note on each, and a markdown file that reads top to bottom. That structure is what turns a pile of captures into a document, or a task list an agent can work through without guessing.

## Organize, mark up, notate, export

**For:** turning what you captured into a finished document, with nobody else in the loop.

1. `Ctrl+Shift+2` for each screenshot. Type a note under it: what's happening, or what to do.
2. `Ctrl+Shift+G` when you move to a new page or area. Give the group you just finished a master note.
3. `Ctrl+Shift+Q` opens the bundle window. Rename, reorder by dragging, and **Review** any shot to add arrows, highlights, step counters, or blur something that shouldn't leave the building.
4. **Export doc.** The document is done: sections from your groups, numbered steps from your notes, images under each. One self-contained web page you can send, or Markdown for a docs platform.

No agent is involved. The bundle already has everything a process document needs, so the export just lays it out.

## Automated process capture, edit, export

**For:** a process you'd rather do than describe. Auto-capture takes the screenshots for you.

1. `Ctrl+Shift+3`, drag the region, adjust its edges, press **Record**. A three-second countdown lets you get in place.
2. Do the process. QACut takes a full-resolution still at the start, at every click or Enter with a ring where the click landed, and at the end.
3. `Ctrl+Shift+3` to stop. The bundle window opens on the sequence.
4. Click the first still to open the review view, then walk the run with the arrow keys: delete the noise, type a note for each keeper, move the click ring if the pointer was somewhere unhelpful, blur what needs it.
5. **Export doc.**

No video is made. Agents can't do anything with one, and recordings for people are the [Studio's](/use-cases/studio) job. What you get is a timeline of stills you can clean up, structure and annotate.

## Send to an agent with prompt and brand kit for polish

**For:** the same document, but with the prose written by an agent in your voice.

1. Put your logo, colours and voice notes in `~/QACut/brand/`, once. Open **Hand off → Brand kit…** in the bundle window to write the notes: tone, audience, terminology, things never to say.
2. Set the purpose to **Write process doc**, and pick Markdown or a web page.
3. **Copy agent prompt**. The instruction has the folder path filled in and tells the agent to match the brand kit, which rides along inside the bundle.
4. Paste into a CLI agent. For a chat agent that only takes uploads, **Save ZIP for chat** packages the folder and copies a prompt that says "the attached ZIP".

The agent reads `bundle.md`, opens the images in order, and writes the document your notes describe. The prompt itself is yours to edit, and you can save your own under other names in the [Prompt library](/docs/prompts); they show up in the purpose menu.

## Organize, mark up, notate a task list for agents

**For:** a round of UI fixes across a whole app, or any list of changes that spans pages.

1. Capture as you go through each page. Each shot's note says what's wrong: "clipped at 125% scaling", "this toggle never saves".
2. Group by page with `Ctrl+Shift+G`, and let the master note carry what's true for the whole page.
3. Mark up where words are slow: an arrow at the misaligned label, a highlight on the wrong colour.
4. `Ctrl+Shift+Enter` to finish. Purpose **Fix issues**, **Copy agent prompt**, paste.

The agent gets a folder it can work from, in order, with each note tied to the screenshot it's about. Twenty screenshots across six pages stay legible because the structure does the explaining.

[How bundles work in detail](/docs/qacut) · [Brand kit](/docs/brand-kit)
