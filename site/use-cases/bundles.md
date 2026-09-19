# QACut Bundles: bigger jobs for your agent

**For:** work that spans pages. A round of UI fixes across a whole app, a process to turn into documentation, a walkthrough someone else will follow.

A bundle is a folder: screenshots and clips in groups, a note on each, and a markdown file that reads top to bottom. An agent reads the file, opens the images, and knows which page each note belongs to. That structure is what keeps a twenty-screenshot job from turning into a guessing game.

## Three things it is good at

**A fix list across an app.** Capture as you go through each page. `Ctrl+Shift+G` wraps up a page as a group with a master note ("everything on the settings page is a bit off") and the shot notes carry the specifics. Finish, copy the agent prompt, paste. The agent works through it in order.

**A process document.** Turn on auto-capture with `Ctrl+Shift+3` and do the process. QACut records a clip and takes a still at every click, labelled with where the click landed. Afterwards, drop the stills that do not help, add a note to the ones that do, and set the bundle's purpose to **Write process doc**. The agent gets a numbered sequence of actions and produces the document, as Markdown or as a single web page with the clips playing inline.

**A guided tutorial.** Same capture, different prompt. Your brand kit rides along in the bundle, so the output sounds like you and carries your logo.

## The flow

1. `Ctrl+Shift+2` for a screenshot, or `Ctrl+Shift+3` to auto-capture a process. Drag the region, type a note.
2. `Ctrl+Shift+G` when you move to a new page or area. Give the group you just finished a master note.
3. `Ctrl+Shift+Q` to open the bundle window whenever you want to reorder, rename, mark up a screenshot, or drop a bad still.
4. `Ctrl+Shift+Enter` to finish. The folder path is on your clipboard. **Copy agent prompt** gives you the instruction with the path filled in; **Save ZIP for chat** packages it for an agent that can only take uploads.

## Marking up

Open any screenshot from the bundle window to add arrows, highlights and step counters, or to blur something that should not leave the building. The blur actually removes the pixels. The original is kept beside the annotated copy.

## When a bundle is too much

One tweak? Use a [quick shot](/use-cases/quick-shots). Want a video rather than a document? That is [QACut Studio](/use-cases/studio).

[How bundles work in detail](/docs/qacut)
