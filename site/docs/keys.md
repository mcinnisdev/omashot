---
description: "Every Omashot key: the Hyprland bindings that reach the app, and the keys inside the shot, brief, markup and recording windows."
---

# The keys

There are two kinds. The ones that work from anywhere belong to the
compositor. The ones inside a window belong to that window.

## From anywhere

Wayland has no global hotkey API, so Hyprland holds these. Each runs
`omashot <verb>`, which reaches the app over a socket — and starts it first if
it is not running, so the first press after a login just works.

| Key | Command | Does |
| --- | --- | --- |
| <kbd>Super</kbd><kbd>Alt</kbd><kbd>Q</kbd> | `omashot shot` | Take one shot and note it |
| <kbd>Super</kbd><kbd>Alt</kbd><kbd>A</kbd> | `omashot add` | Add a shot to the open brief |
| <kbd>Super</kbd><kbd>Alt</kbd><kbd>T</kbd> | `omashot trail` | Trail what you do, as shots. Again to stop |
| <kbd>Super</kbd><kbd>Alt</kbd><kbd>R</kbd> | `omashot record` | Record the screen. Again to stop |
| <kbd>Super</kbd><kbd>Alt</kbd><kbd>N</kbd> | `omashot section` | Close this section, start the next |
| <kbd>Super</kbd><kbd>Alt</kbd><kbd>B</kbd> | `omashot brief` | Open the brief |
| <kbd>Super</kbd><kbd>Alt</kbd><kbd>D</kbd> | `omashot done` | Write the brief out, copy its path |
| <kbd>Super</kbd><kbd>Alt</kbd><kbd>Z</kbd> | `omashot zoom` | Mark a zoom, while recording |

Every letter is the first letter of its verb, so the keys and the command line
are one vocabulary.

Two more verbs have no key by default:

| Command | Does |
| --- | --- |
| `omashot copy` | Copy the loose shots and clear them. No key, because it is the one worth thinking about before you press it |
| `omashot edit FILE` | Mark up an image that already exists and hand it off |

### Nothing of yours is unbound

Every chord above is one Omarchy leaves free, checked against
`omarchy menu keybindings --print` on a stock install.
<kbd>Super</kbd><kbd>Shift</kbd><kbd>1</kbd>–<kbd>9</kbd> stays on *move window
to workspace*, <kbd>Super</kbd><kbd>Alt</kbd><kbd>1</kbd>–<kbd>5</kbd> on the
window groups, <kbd>Super</kbd><kbd>Alt</kbd><kbd>S</kbd> on the scratchpad,
and <kbd>Super</kbd><kbd>Shift</kbd><kbd>Return</kbd> on the browser.

### Changing them

They are ordinary Hyprland bindings, in a marked block in
`~/.config/hypr/bindings.lua`:

```lua
-- >>> omashot >>>
o.bind("SUPER + ALT + Q", "Omashot: take a shot", "omashot shot")
...
-- <<< omashot <<<
```

Edit them as you would any other binding. The app prints whatever is in its
settings as the hint text, so if you change a key, change it in
**Help → Keyboard shortcuts** too, or the hints will name a key that does
nothing.

## In the note box

| Key | Does |
| --- | --- |
| <kbd>Enter</kbd> | Copy the picture with the note under it, for a person |
| <kbd>Ctrl</kbd><kbd>Shift</kbd><kbd>A</kbd> | Hand the path and note to an agent as text, and clear the loose shots |
| <kbd>Shift</kbd><kbd>Enter</kbd> | New line |
| <kbd>Esc</kbd> | Keep the shot with no note; cancel a section wrap-up |

The **A** is for agent. Everything else goes to a person — see
[shots](/docs/shots#two-keys-because-there-are-two-audiences) for why it is
one or the other and never both.

## In the markup editor

| Key | Does |
| --- | --- |
| <kbd>M</kbd> <kbd>A</kbd> <kbd>H</kbd> <kbd>B</kbd> <kbd>S</kbd> | Move, arrow, highlight, blur, step counter |
| Arrow keys | Nudge the selected mark; with <kbd>Shift</kbd>, by ten |
| <kbd>Delete</kbd> | Remove the selected mark |
| <kbd>Ctrl</kbd><kbd>Z</kbd> | Undo |
| <kbd>Ctrl</kbd><kbd>S</kbd> | Save |
| <kbd>Ctrl</kbd><kbd>Shift</kbd><kbd>C</kbd> | Copy the marked-up image on its own, with no note under it |
| <kbd>Esc</kbd> | Cancel |

Blur really removes pixels rather than covering them, so a blurred shot is
safe to hand over.

## In the brief

| Key | Does |
| --- | --- |
| <kbd>←</kbd> <kbd>→</kbd> | Previous and next shot, when reviewing one |
| <kbd>Ctrl</kbd><kbd>E</kbd> | Open the markup editor on the shot |
| <kbd>Esc</kbd> | Close the panel, then the window |

## In the recording editor

| Key | Does |
| --- | --- |
| <kbd>Space</kbd> | Play or pause |
| <kbd>←</kbd> <kbd>→</kbd> | Step one second; with <kbd>Shift</kbd>, five |
| <kbd>I</kbd> <kbd>O</kbd> | Video starts or ends at the playhead |
| <kbd>X</kbd> | Cut: once at the start of a stretch, once where it resumes |
| <kbd>Delete</kbd> | Remove the selected zoom or cut |
| <kbd>Esc</kbd> | Deselect, close a panel, then close the window |
