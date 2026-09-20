---
description: "Install Omashot on Omarchy, wire it into your keys, bar and theme, and take your first shot and first brief. Free and open source, no account needed."
---

# Getting started

Omashot has no window to open first. Press a key, drag a region, type what is
wrong. It appears in your bar only while it is holding something.

## Install

Not in the AUR yet. You need Node 22 or newer, a Rust toolchain and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
git clone https://github.com/mcinnisdev/omashot
cd omashot
npm install
npm run tauri build
```

Then two installers, which do separate jobs:

```bash
./scripts/install-local.sh    # binary, launcher and icons into ~/.local
./omarchy/install.sh          # theme, keys, menu, window rules, bar widget
```

`scripts/install-local.sh` puts `omashot` on your PATH. `omarchy/install.sh`
wires it into the desktop: a theme template, the key bindings, the menu
entries, a window rule and the bar widget. It is additive and idempotent — run
it again after an update and it fills in only what is missing, skips anything
you have edited yourself, and `--uninstall` takes it back out.

Reload Hyprland (<kbd>Super</kbd><kbd>Escape</kbd>) to pick up the keys.

::: tip Not on Omarchy?
The app runs on any Wayland compositor. Only the wiring is Omarchy-specific:
without `omarchy-capture-region` it falls back to plain `slurp`, and you bind
the keys yourself. `omarchy/install.sh` will tell you it is not for you and
stop rather than half-configure something.
:::

## Your first shot

Press <kbd>Super</kbd><kbd>Alt</kbd><kbd>Q</kbd>. The screen freezes and you
drag a region — the same picker Omarchy uses for screenshots, so it snaps to
windows and monitors and the keyboard works the way it does everywhere else.

The shot opens with the markup tools and a note box beside it. Type what is
wrong and press <kbd>Enter</kbd>.

Take a few more. When you are done, **Finish and hand off** puts the lot on
your clipboard, ready to paste into an agent:

```
/home/you/Omashot/Quick/2026-09-19_101512/01.png
The save button is clipped at 125% scaling.

/home/you/Omashot/Quick/2026-09-19_101512/02.png
Same at 150%. The footer overlaps the form.
```

Those are **loose shots**: shots outside any brief. Copying them clears them,
so the next one starts fresh and an agent is never pointed at work you have
already dealt with.

## Your first brief

A brief is the thing you hand over: a folder of shots in reading order with a
`brief.md` written for whoever gets it.

1. <kbd>Super</kbd><kbd>Alt</kbd><kbd>A</kbd> adds a shot to the brief instead
   of leaving it loose. Note it and press <kbd>Enter</kbd>.
2. <kbd>Super</kbd><kbd>Alt</kbd><kbd>N</kbd> closes the current **section** and
   starts the next. A section is one page, one screen, one step.
3. <kbd>Super</kbd><kbd>Alt</kbd><kbd>B</kbd> opens the brief to review,
   reorder, rename and re-note anything.
4. <kbd>Super</kbd><kbd>Alt</kbd><kbd>D</kbd> is done: it writes the brief out
   and copies its path.

What lands on disk:

```
~/Omashot/2026-09-19_143022-settings-review/
  brief.md               everything in reading order, images linked relatively
  manifest.json          the same data, structured
  01-settings-page/      01.png 02.png 03.png
  02-billing/            01.png
```

Point an agent at the folder and it reads `brief.md` top to bottom, opens the
images beside it, and knows which section each note belongs to.

Set `$OMASHOT_DIR` if you want that folder somewhere other than `~/Omashot`.

## Where it lives

In your bar, but only while it is holding something — a brief and its shot
count, loose shots waiting to be copied, or a live trail or recording you can
stop with a click. Idle, it shows nothing.

Everything is also in the Omarchy menu
(<kbd>Super</kbd><kbd>Alt</kbd><kbd>Space</kbd>) under **Omashot**, and
`omashot help` lists every verb.

## Next

- [The keys](/docs/keys) — all of them, and the ones inside each window
- [Shots](/docs/shots) — loose shots and markup
- [Briefs](/docs/briefs) — sections, trails and handing off
- [Prompts](/docs/prompts) — what gets copied, and how to change it
