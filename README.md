# QACut

A tray-resident capture tool for QA passes. Grab a region, type what is wrong,
keep going. Group the shots as you move between pages, then hand the whole
bundle to an agent as a folder path or as markdown.

Tauri 2 (Rust backend, vanilla TS frontend). Windows, macOS and Linux.

## Hotkeys

| Key | Does |
| --- | --- |
| `Ctrl/Cmd + Shift + 2` | Freeze the screen, drag a region, then type a note |
| `Ctrl/Cmd + Shift + G` | Start a new group and give it a name and master note |
| `Ctrl/Cmd + Shift + Q` | Show or hide the current bundle |
| `Ctrl/Cmd + Shift + Enter` | Write the bundle, copy the folder path, show the result |

Inside the note box: `Enter` saves, `Shift + Enter` adds a line, `Esc` keeps
the screenshot with no note. Inside the capture overlay: `Esc` or right-click
cancels.

The two workflows from the brief map to this:

- **One detail.** Capture hotkey, drag, type, Enter, finish hotkey. Five
  actions, one of them a mouse drag.
- **Full pass.** Capture and note repeatedly on the settings page, group
  hotkey to open the next group with its master note, carry on, finish hotkey
  at the end.

A session starts on its own at the first capture and ends at the first finish.
There is nothing to open and nothing to save.

## Output

Everything lands under `~/QACut/`:

```
~/QACut/2026-09-17_143022/
  bundle.md            everything in reading order, images linked relatively
  manifest.json        the same data, structured
  01-settings-page/
    01.png  02.png  03.png
  02-billing/
    01.png
```

`bundle.md` is the agent-facing file. Group headings carry the master note as
a blockquote, each shot is a heading with its image and its note underneath.
Image links are relative to the file, so the folder can be moved or handed to
a CLI as-is.

## Handing it off

**CLI.** Finish copies the folder path. Point an agent at it:

```
claude "Work through the QA bundle at /Users/nick/QACut/2026-09-17_143022.
Start with bundle.md, then look at each screenshot it references."
```

**Chat.** "Copy markdown" in the bundle window puts the full `bundle.md` text
on the clipboard. Paste it, then drag the PNGs in from the folder.

That second step is a real limitation, not an oversight. No OS clipboard
carries "markdown plus N images" as a single payload that a chat app will
accept, so any tool that claims to do it is really doing one of the two. If
you want the images to ride along, the path is a platform shim that writes a
file list to the clipboard next to the text: `CF_HDROP` on Windows,
`NSPasteboard` file URLs on macOS, `text/uri-list` on Linux. That is maybe
150 lines of `#[cfg(target_os)]` code in `main.rs` and it does not exist yet.

## Running it

Needs Node 18+ and a Rust toolchain. Platform prerequisites are the standard
Tauri ones (`https://v2.tauri.app/start/prerequisites/`): Visual Studio Build
Tools and WebView2 on Windows, Xcode command line tools on macOS,
`libwebkit2gtk-4.1-dev` and friends on Linux.

```bash
npm install
npm run tauri dev
```

Release build: `npm run tauri build`.

Icons are committed under `src-tauri/icons/`. If the mark changes, replace
`assets/logo.png` (square, at least 1024px, transparent) and regenerate:

```bash
npx tauri icon assets/logo.png
rm -rf src-tauri/icons/android src-tauri/icons/ios   # desktop only
```

`src-tauri/icons/tray.png` is not produced by that command and has to be
redone by hand. It is the mark cropped to its alpha bounding box and repadded
to a 6% margin at 256px. The app icon carries roughly 11 to 16% margin, which
is right for a dock but leaves the glyph small in a menubar, and this app
lives in the tray. `main.rs` embeds it with `include_bytes!`, so a stale
`tray.png` is a stale tray icon with no build error.

On macOS the tray icon stays full colour rather than a monochrome template
image, so it will not invert with the menubar. The coral reads on both light
and dark, but a template variant is the correct fix if it bothers you.

The accent colour throughout the UI is the mark's coral, `--signal` at the top
of `src/styles.css`. Because the accent is red, destructive controls are
distinguished by fill rather than hue: the accent draws strokes and text, the
remove button fills solid on hover.

### Permissions

- **macOS** will ask for Screen Recording the first time you capture. Grant it
  in System Settings, then restart the app. It will not prompt again but it
  also will not work until you do.
- **Linux/Wayland** does not let applications read the screen directly. `xcap`
  goes through the portal, so you may see a one-time share prompt. X11 works
  without ceremony.
- **Windows** needs nothing, but note that `Ctrl+Shift+2` is claimed by some
  IDEs. If a hotkey is taken the app logs it and keeps running with the rest.

## Changing things

- Hotkeys: the four `HK_*` constants at the top of `src-tauri/src/main.rs`.
  Anything the Tauri shortcut parser accepts works.
- Output location: `base_dir()` in the same file.
- Markdown shape: `render_markdown()` in `src-tauri/src/export.rs`. This is
  the function to edit if you want the bundle to match a prompt format you
  already use.

## Layout

```
index.html, capture.html, note.html, peek.html   Vite entry points, one per window
src/                    frontend: capture.ts, note.ts, peek.ts, types.ts, styles.css
src-tauri/
  src/                  main.rs (tray, hotkeys, commands), model.rs, capture.rs,
                        export.rs, overlay.rs
  capabilities/         window permissions
  icons/                generated app icons plus the hand-cropped tray.png
  tauri.conf.json
assets/logo.png         source mark for `tauri icon`
```

## What has and has not been verified

- The frontend typechecks and builds clean (`npm run build`).
- The Rust compiles: `cargo check`, `cargo clippy --all-targets` and
  `cargo test` all pass on Windows with Rust 1.93, Tauri 2.11 and xcap 0.9.
- The app has not yet been run end to end. The parts most likely to need
  adjustment on first launch, in order:
  1. **Monitor geometry.** `src-tauri/src/capture.rs` takes `x`/`y` from
     `xcap` as-is and hands them to Tauri as logical coordinates. On a single
     display, or displays all at 100%, that is correct. With mixed DPI across
     several displays the overlay may land offset on the scaled one.
  2. **Capability names.** If a window action is denied at runtime, the
     message names the exact permission string to add to
     `src-tauri/capabilities/default.json`.
  3. **Hotkey collisions.** A taken shortcut is logged at boot and skipped;
     the rest keep working.

There is one unit test, on directory slugs: `cd src-tauri && cargo test`.

## Known gaps

- No annotation on the screenshot itself. Arrows and boxes would mean a canvas
  layer between capture and note, which is a real feature rather than a tweak.
- No undo for a deleted shot. The file is removed immediately.
- Groups cannot be reordered or merged after the fact.
- Multi-monitor capture opens one overlay per display. Scale is applied per
  monitor when cropping, but a selection cannot span two displays and mixed
  DPI placement is unverified (see above).
