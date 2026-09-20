---
description: "Edit the text Omashot puts on your clipboard. Built-in prompts with placeholders, your own shot and brief prompts, and a live preview as you type."
---

# Prompts

Every text Omashot puts on your clipboard is a template, and the prompt
library is where you edit them and add your own. Open it from **Hand off →
Prompt library…** in the brief.

## Built in

Six texts ship with Omashot. Each can be rewritten, and **Reset** puts the
default back.

| Prompt | Used when | Filled in |
| --- | --- | --- |
| Shot: one | <kbd>Ctrl</kbd><kbd>Enter</kbd> on a single shot | `{path}`, `{note}` |
| Shot: several loose | <kbd>Ctrl</kbd><kbd>Enter</kbd> with more than one loose shot | `{count}`, `{dir}`, `{entries}` |
| Brief: Fix issues | Copy agent prompt, purpose Fix issues | `{location}`, `{root}`, `{name}` |
| Brief: Write process doc | Copy agent prompt, purpose Write process doc | `{location}`, `{deliverable}`, `{root}`, `{name}` |
| Process doc as Markdown | Becomes `{deliverable}` for a Markdown document | |
| Process doc as web page | Becomes `{deliverable}` for a web page | |

`{location}` is the folder path for a CLI agent, or "the attached ZIP" for a
chat hand-off.

A built-in left at its default is stored as empty rather than as a copy of the
text, so if a default improves in a later release you get the improvement
instead of a stale duplicate.

## Your own

**+ New** adds a prompt with a name and a kind:

- **Shots.** Wraps the loose shots you hand off. Put `{shots}` where the paths
  and notes should go, or leave it out and they are appended. Once you have
  one, the shot window shows a **Hand off as** picker above the note.
- **Briefs.** The whole instruction for a brief, with `{root}` for the folder
  and `{name}` for its name. It appears by name in the brief's purpose menu,
  and the choice is saved with the brief.

**Duplicate as mine** on *Fix issues* or *Write process doc* is the quickest
way to start from something known to work.

## Editing

The placeholders for the selected prompt are chips — click one to insert it at
the cursor. The preview underneath fills them with sample values, so you can
see the shape before it is used on real work.

Nothing is written until **Save** (<kbd>Ctrl</kbd><kbd>S</kbd>), and switching
prompts or closing with unsaved changes asks first. Prompts live in
`~/Omashot/prompts.json`.
