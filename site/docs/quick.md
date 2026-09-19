# Quick shots

QACut Basic is the quick lane. A quick shot is one screenshot and one note, saved on its own and pasted straight into whatever agent you are talking to. Or, marked up and copied as an image, sent straight to a person.

## Taking one

`Ctrl+Shift+1` freezes the screen. Drag a region and the shot opens large, with the markup tools along the top and the note beside it. Type a note and press `Enter` to save it. `Ctrl+Enter` instead finishes and hands off: the screenshot's path and your note go on the clipboard, ready to paste with whatever else you want to say.

```
C:\Users\nick\QACut\Quick\2026-09-19_101512\01.png
The save button is clipped at 125% scaling.
```

The file lands in a batch folder under `~/QACut/Quick/`, named for when the batch started, with the note beside it as a small markdown file and a `notes.md` listing every shot in the batch.

## Sending it to a person

Draw an arrow, highlight the button, blur the account number, then press **Copy image** (or `Ctrl+Shift+C`). The marked-up screenshot is on the clipboard as an image, ready to paste into Teams, Slack, an email or a ticket. No note needed, and the shot is still saved in the batch if you want it later.

## Sending a few together

Just keep going: `Ctrl+Shift+1`, note, `Enter`, again. Each shot joins the open batch, and the note box header counts them. `Ctrl+Enter` on the last one (or **Finish quick batch and copy paths** in the tray) copies every shot in the batch with its note, plus a line naming the folder, in one paste, and closes the batch.

Closing matters: the next quick shot starts a fresh folder, so an agent is never pointed at shots you have already dealt with. **New batch** in the note box moves the shot you are noting into a fresh folder without closing the old one.

## Your own wording

**Prompt library…** in the tray (or **Hand off → Prompt library…** in the bundle window) lets you rewrite what goes on the clipboard: the single-shot line, the batch preamble, and the bundle prompts. Words in braces such as `{path}` and `{note}` are filled in. Reset puts a default back.

You can also add prompts of your own and pick one before a hand-off. A quick-shot prompt wraps the shots: write the instruction and put `{shots}` where the paths and notes should go, or leave it out and they are appended. Once you have one, the quick shot window shows a "Hand off as" picker above the note.

## When to use a bundle instead

Use quick shots for one-off fixes and short sessions. Use a [bundle](/docs/qacut) when there are enough shots across enough pages that an agent needs groups and master notes to keep them straight, when you want auto-capture to record a process, or when the hand-off should carry a purpose-specific prompt and your brand kit.
