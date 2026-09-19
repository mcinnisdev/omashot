<p align="center">
  <img src="assets/mark.png" alt="Omashot" width="120" />
</p>

<h1 align="center">Omashot</h1>

<p align="center">
  Shoot the screen, note it, hand it off. Built for Omarchy.
</p>

<p align="center">
  <a href="LICENSE"><img alt="MIT" src="https://img.shields.io/badge/license-MIT-2f4e6f" /></a>
  <a href="https://omashot.com"><img alt="Docs" src="https://img.shields.io/badge/docs-omashot.com-ff9e64" /></a>
</p>

---

Omarchy already takes screenshots and records the screen. What it has no answer
for is the **hand-off**: turning what you just saw into something an agent can
act on. That is all Omashot does.

Press a key, drag a region, type what is wrong. Keep going. When you are done,
the whole lot is on your clipboard as paths and notes, or as a folder with a
`brief.md` an agent reads top to bottom. No account, no upload, no telemetry:
everything is plain files under `~/Omashot/`.

Omashot is a fork of [QACut](https://github.com/mcinnisdev/qacut), rebuilt
around Omarchy's own picker, keys, menu, bar and themes. The capture and markup
code is shared; the desktop integration, and the vocabulary, are not.

## The words

Omashot does not have product tiers, and it does not call anything a "bundle".

| Word | What it is |
| --- | --- |
| **shot** | One screenshot, with a note on it. |
| **brief** | A folder of shots in reading order, with a `brief.md` written for whoever gets it. The thing you hand over. |
| **section** | A part of a brief. One page, one screen, one step of a process. |
| **loose shots** | Shots taken outside any brief. Copy them and they clear. |
| **trail** | Hands-free capture: a shot at every step while you do the thing. |
| **recording** | A screen recording, for a person rather than an agent. |

A brief on disk:

```
~/Omashot/2026-09-19_143022-settings-review/
  brief.md               everything in reading order, images linked relatively
  manifest.json          the same data, structured
  01-settings-page/      01.png 02.png 03.png
  02-billing/            01.png
```

Set `$OMASHOT_DIR` to put that somewhere else.

## It wears your theme

Omashot has no palette of its own. Omarchy renders its colours from whatever
theme you are on, and the app follows along — windows and bar mark alike — the
moment you switch. Nothing to configure.

That is one template, `omarchy/themed/omashot.css.tpl`. There is no theme-set
hook: the app watches the rendered stylesheet and re-skins itself.

## In the bar

Omashot puts a widget in the Omarchy bar that appears **only when it is holding
something**: a brief with a shot count, loose shots waiting to be copied, or a
live trail or recording you can stop with a click. Idle, it shows nothing. A bar
is yours, and an app that sits in it to announce that it is doing nothing has
not earned the space.

Click it to open the brief, middle-click to finish and copy.

## Keys

The compositor owns the keys, because Wayland has no global hotkey API and an
app that claims otherwise is lying to you. Every binding runs `omashot <verb>`,
which reaches the running app over a socket — and starts it first if it is not
up, so the first press after a login just works.

| Key | Command | Does |
| --- | --- | --- |
| `SUPER+ALT+Q` | `omashot shot` | Take one shot and note it |
| `SUPER+ALT+A` | `omashot add` | **A**dd a shot to the open brief |
| `SUPER+ALT+T` | `omashot trail` | **T**rail what you do, as shots |
| `SUPER+ALT+R` | `omashot record` | **R**ecord the screen |
| `SUPER+ALT+N` | `omashot section` | **N**ext section |
| `SUPER+ALT+B` | `omashot brief` | Open the **b**rief |
| `SUPER+ALT+D` | `omashot done` | **D**one: write it out and copy its path |
| `SUPER+ALT+Z` | `omashot zoom` | Mark a **z**oom, while recording |

Every letter is the first letter of its verb, so the keys and the command line
are the same vocabulary. `omashot copy` takes the loose shots; it has no key by
default because it is the one you want to think about.

Every chord is one Omarchy leaves free, so installing Omashot **unbinds
nothing**: `SUPER+SHIFT+1..9` stays on *move window to workspace*,
`SUPER+ALT+1..5` on the window groups, `SUPER+ALT+S` on the scratchpad.

Everything is also in the Omarchy menu under **Omashot**, and `omashot help`
lists the verbs.

## Install

Not in the AUR yet. From source, with Node 22+, a Rust toolchain and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/):

```bash
git clone https://github.com/mcinnisdev/omashot
cd omashot
npm install
npm run tauri build
./scripts/install-local.sh    # binary, launcher and icons into ~/.local
./omarchy/install.sh          # theme, keys, menu, window rule, bar widget
```

`omarchy/install.sh` is additive and idempotent, skips anything you have edited
yourself, and `--uninstall` takes it back out. `packaging/PKGBUILD` is the Arch
package.

## Status

Honest about where this is:

- **Shots, briefs and trails** work on Omarchy. Region picking goes through
  `omarchy-capture-region`, so it snaps to windows and monitors exactly like a
  screenshot does, and stills come through wlroots screencopy.
- **Recording** edits, composites and exports on Linux, but **capturing a
  source is still Windows-only**. The Linux path will go through
  `gpu-screen-recorder`, which Omarchy already ships.
- **Keystroke badges** and click-triggered trailing need `/dev/input` access
  that Wayland does not hand out. A trail falls back to interval shots; badges
  are not available yet.
- Windows still builds. The Omarchy-specific pieces are behind `cfg`, and the
  capture and markup code is shared.

## Development

Tauri 2: a Rust backend and vanilla TypeScript built with Vite.

```bash
npm run tauri dev
npm run build                 # type-check and bundle
cd src-tauri && cargo test
```

```
src/                  the windows: picker, note, brief, edit, prompts, theme
src/studio/           compositor, timeline, export (WebCodecs)
src-tauri/src/        main.rs, capture.rs, export.rs, model.rs
src-tauri/src/        picker.rs (Omarchy region picking), ipc.rs (the CLI),
                      theme.rs (live theming), status.rs (what the bar reads)
omarchy/              theme template, keys, menu, window rule, bar widget
packaging/            PKGBUILD and the .desktop entry
scripts/logo.py       the mark, as pixel art, at any size and any colour
```

Internally the code still says `group` where the user says *section*, and
`peek` where the user says *brief*. The vocabulary is a seam at the edges —
`ipc::VERBS` maps one to the other — rather than a rename of 6,000 lines that
would risk the parts that already work.

The mark is a 16x16 pixel grid scaled by whole numbers, so it is exact at every
icon size and can be re-rendered in any accent:

```bash
python3 scripts/logo.py --ascii
./scripts/icons.sh            # regenerate every icon
```

---

MIT. Made by [Nick McInnis](https://github.com/mcinnisdev).
