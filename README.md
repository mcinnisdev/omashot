# QACut

A tray-resident capture tool for QA passes. Grab a region, type what is wrong,
keep going. Group the shots as you move between pages, then hand the whole
bundle to an agent as a folder path or as markdown.

Tauri 2 (Rust backend, vanilla TS frontend). Windows, macOS and Linux.

## Hotkeys

| Key | Does |
| --- | --- |
| `Ctrl/Cmd + Shift + 2` | Freeze the screen, drag a region, then type a note |
| `Ctrl/Cmd + Shift + R` | Drag a region, adjust it, Record; press again to stop, then type a note |
| `Ctrl/Cmd + Shift + G` | Wrap up the current group with a master note and start the next |
| `Ctrl/Cmd + Shift + Q` | Show or hide the current bundle |
| `Ctrl/Cmd + Shift + Enter` | Write the bundle, copy the folder path, show the result |
| `Ctrl/Cmd + Shift + N` | Start a new bundle and open the window to name it and its first group |

Inside the note box: `Enter` saves, `Shift + Enter` adds a line, `Esc` keeps
the screenshot with no note. The header reads "Group 1 / Shot 1"; click either
and type to name it. Inside the capture overlay: `Esc` or right-click cancels.

The two workflows from the brief map to this:

- **One detail.** Capture hotkey, drag, type, Enter, finish hotkey. Five
  actions, one of them a mouse drag.
- **Full pass.** Capture and note repeatedly on the settings page, group
  hotkey to wrap that group up with a master note, carry on in the next one,
  finish hotkey at the end.

Recordings are for showing a process rather than a fault. After the drag the
box can be moved and its edges pulled; Record (or `Enter`) then starts a
three-second countdown so windows and the mouse can be put in place. While
it counts down and records, everything outside the region stays tinted and
the region is outlined, so it is obvious when something has drifted out of
shot. The hotkey cancels during the countdown. The region is
grabbed at ten frames a second with the cursor drawn on as a ring, scaled to
at most 720 px wide and streamed into a looping GIF, so stopping is instant.
Every two seconds a still is saved beside it (thinned to twelve at most).
Agents cannot play a GIF, so `bundle.md` links those stills in order under the
recording; a human reading the resulting document just sees the GIF.

A bundle has a purpose, chosen in the bundle window: **Fix issues** (the
default), **Write process doc**, or **Custom prompt**. It only changes the
prompt that "Copy agent prompt" produces, so one bundle of shots and
recordings can be handed off any way you like. The custom template is
written in the bundle window and kept in `~/QACut/custom-prompt.txt` for
every bundle after that; `{root}` becomes the folder path and `{name}` the
bundle name, and a template that never mentions `{root}` gets the path
appended so the agent can always find the folder.

## Brand kit

Anything the agent produces from a bundle, a process document especially,
should sound and look like the business. Put whatever describes it in
`~/QACut/brand/`: logo, colour swatches, fonts, a style guide, a document
whose voice to imitate. "Brand kit" in the bundle window opens that folder
and takes voice notes (tone, audience, terminology, things never to say),
which are saved as `brand/brand.md`.

On finish, the folder is copied into the bundle as `brand/`, so the bundle
stays self-contained, and `bundle.md` gets a "Brand kit" section near the
top that inlines the notes and lists the files. The built-in prompts add a
line telling the agent to match it. Untick "Include in this bundle" for a
bundle where it does not apply; the copy is removed on the next finish.

A bundle starts on its own at the first capture. Finishing writes it out but
does not close it: keep capturing, finish again, and the files are rewritten.
Only **New bundle** (hotkey, tray menu or bundle window) ends one and starts
the next, saving any unwritten changes first. A bundle with no shots is
deleted rather than left as an empty folder.

To go back to an earlier bundle, **Open** in the bundle window (or "Open
bundle..." in the tray) lists everything under `~/QACut/`, newest first.
Opening one puts the current bundle away the same way New bundle does, then
reloads the chosen one from its `manifest.json`, so you can add shots,
reorder, or finish it again. A bundle folder that was moved by hand still
opens; its paths are rebuilt from wherever it is now.

New shots go into the most recent group by default. To add to an earlier
group, open the bundle window and press **Capture here** on that group.
Shots can be reordered there too: drag one onto another to place it before
it, onto a group heading to append it there, or use the arrow buttons. The
folder is laid out to match on the next finish, so `01.png` is always
step 1, whatever order things were captured in.

## Output

Everything lands under `~/QACut/`. The folder is named by timestamp; give
the bundle a name in the bundle window and a slug is appended, so
`2026-09-17_143022-settings-review`:

```
~/QACut/2026-09-17_143022-settings-review/
  bundle.md            everything in reading order, images linked relatively
  manifest.json        the same data, structured
  01-settings-page/
    01.png  02.png  03.png
  02-billing/
    01.png
    02.gif             a recording
    02-frames/         its key frames, 01.png 02.png ...
  brand/               copy of ~/QACut/brand, if included
```

`bundle.md` is the agent-facing file. It opens with a short note on how to
read it, then group headings carry the master note as a blockquote, and each
shot is a numbered heading (with its name, if you gave it one) followed by
its image, its note, and its pixel size.
Image links are relative to the file, so the folder can be moved or handed to
a CLI as-is.

## Handing it off

**CLI.** Finish copies the folder path. **Copy agent prompt** in the bundle
window copies a ready-made instruction with the path filled in:

```
claude "Work through the QA bundle at /Users/nick/QACut/2026-09-17_143022.
Start with bundle.md: each group is a page or area, its quoted master note
applies to every screenshot under it, and each screenshot's note says what is
wrong. Open each screenshot it references before changing anything."
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

- Hotkeys: the six `HK_*` constants at the top of `src-tauri/src/main.rs`.
  Anything the Tauri shortcut parser accepts works.
- Output location: `base_dir()` in the same file.
- Markdown shape: `render_markdown()` in `src-tauri/src/export.rs`. This is
  the function to edit if you want the bundle to match a prompt format you
  already use. The two agent prompts are in `agent_prompt()` in `main.rs`.
- Recording rate, width and key-frame interval: the constants at the top of
  the recording section in `src-tauri/src/capture.rs`.

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

Unit tests cover slugs, shot numbering, folder renames, group targeting and
the markdown shape: `cd src-tauri && cargo test`.

## Known gaps

- No annotation on the screenshot itself. Arrows and boxes would mean a canvas
  layer between capture and note, which is a real feature rather than a tweak.
- No undo for a deleted shot. The file is removed immediately.
- Groups cannot be reordered or merged after the fact.
- Multi-monitor capture opens one overlay per display. Scale is applied per
  monitor when cropping, but a selection cannot span two displays and mixed
  DPI placement is unverified (see above).
