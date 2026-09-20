---
layout: page
sidebar: false
aside: false
title: Omashot
titleTemplate: Shoot the screen, hand off the work
---

<div class="qc">

<section class="qc-hero">
<div>
<span class="eyebrow">Free and open source · built for Omarchy</span>
<h1>Shoot the screen.<br /><em>Hand off the work.</em></h1>
<p class="lead">Omarchy already takes screenshots. What it has no answer for is the hand-off: turning what you just saw into something an agent can act on. That is all Omashot does.</p>
<div class="qc-actions">
<a class="qc-btn primary" href="/docs/getting-started">Get started</a>
<a class="qc-btn outline" href="https://github.com/mcinnisdev/omashot">Source on GitHub</a>
</div>
<ul class="qc-trust">
<li>MIT</li>
<li>No account</li>
<li>No upload</li>
<li>No telemetry</li>
<li>Plain files on disk</li>
</ul>
</div>

<div class="qc-artifact" role="img" aria-label="A brief on disk: a folder of screenshots in sections, and a brief.md that reads top to bottom">
<div class="qc-artifact-head"><span class="dot"></span>brief.md</div>

```markdown
# Brief: Settings review

3 shots across 2 sections, captured 19 Sep 2026.
Image paths are relative to this file.

## 01 · Settings page

> The whole page is the wrong width at 125% scaling.

![01](01-settings-page/01.png)
The save button is clipped on the right.

![02](01-settings-page/02.png)
Same at 150%. The footer overlaps the form.

## 02 · Billing

![01](02-billing/01.png)
Card expiry accepts a past date.
```

</div>
</section>

<section class="qc-band">
<h2>The words</h2>
<p class="qc-lede">Omashot has no product tiers and does not call anything a “bundle”. It has six nouns, and they are the whole model.</p>

<div class="qc-words">
<dl>
<dt>shot</dt><dd>One screenshot, with a note on it.</dd>
<dt>brief</dt><dd>A folder of shots in reading order, with a <code>brief.md</code> written for whoever gets it. The thing you hand over.</dd>
<dt>section</dt><dd>A part of a brief. One page, one screen, one step.</dd>
<dt>loose shots</dt><dd>Shots taken outside any brief. Copy them and they clear.</dd>
<dt>trail</dt><dd>Hands-free capture: a shot at every step while you do the thing.</dd>
<dt>recording</dt><dd>A screen recording, for a person rather than an agent.</dd>
</dl>
</div>
</section>

<section class="qc-spot">
<div class="qc-spot-copy">
<h2>It wears your theme</h2>
<p>Omashot has no palette of its own. Omarchy renders its colours from whatever theme you are on, and the app follows along — windows and bar mark alike — the moment you switch.</p>
<p>That is one template file and no theme hook. The app watches the stylesheet Omarchy renders and re-skins itself.</p>
</div>
<div class="qc-spot-visual">
<div class="qc-themes">
<div class="qc-theme t1"><i></i><i></i><i></i></div>
<div class="qc-theme t2"><i></i><i></i><i></i></div>
<div class="qc-theme t3"><i></i><i></i><i></i></div>
</div>
</div>
</section>

<section class="qc-spot flip">
<div class="qc-spot-copy">
<h2>In the bar, only when it matters</h2>
<p>The bar widget appears when Omashot is holding something: a brief and its shot count, loose shots waiting to be copied, or a live trail or recording you can stop with a click.</p>
<p>Idle, it shows nothing. A bar is yours, and an app that sits in it to announce that it is doing nothing has not earned the space.</p>
</div>
<div class="qc-spot-visual">
<div class="qc-bar">
<span class="w"></span><span class="w"></span><span class="w"></span>
<span class="mark"><img src="/logo.png" alt="" width="16" height="16" />3</span>
<span class="w"></span><span class="w"></span>
</div>
<div class="qc-bar-note">A brief with three shots in it</div>
</div>
</section>

<section class="qc-band">
<h2>The compositor owns the keys</h2>
<p class="qc-lede">Wayland has no global hotkey API, and an app that claims otherwise is lying to you. Every binding runs <code>omashot &lt;verb&gt;</code>, which reaches the app over a socket — and starts it first if it is not up.</p>

<div class="qc-keys">
<table>
<tr><td><kbd>Super</kbd><kbd>Alt</kbd><kbd>Q</kbd></td><td><code>omashot shot</code></td><td>Take one shot and note it</td></tr>
<tr><td><kbd>Super</kbd><kbd>Alt</kbd><kbd>A</kbd></td><td><code>omashot add</code></td><td>Add a shot to the open brief</td></tr>
<tr><td><kbd>Super</kbd><kbd>Alt</kbd><kbd>T</kbd></td><td><code>omashot trail</code></td><td>Trail what you do, as shots</td></tr>
<tr><td><kbd>Super</kbd><kbd>Alt</kbd><kbd>R</kbd></td><td><code>omashot record</code></td><td>Record the screen</td></tr>
<tr><td><kbd>Super</kbd><kbd>Alt</kbd><kbd>N</kbd></td><td><code>omashot section</code></td><td>Next section</td></tr>
<tr><td><kbd>Super</kbd><kbd>Alt</kbd><kbd>B</kbd></td><td><code>omashot brief</code></td><td>Open the brief</td></tr>
<tr><td><kbd>Super</kbd><kbd>Alt</kbd><kbd>D</kbd></td><td><code>omashot done</code></td><td>Write it out, copy the path</td></tr>
</table>
</div>

<p class="qc-foot">Every letter is the first letter of its verb, and every chord is one Omarchy leaves free — so installing Omashot <strong>unbinds nothing</strong>. <code>Super+Shift+1..9</code> stays on move-to-workspace, <code>Super+Alt+1..5</code> on the window groups, <code>Super+Alt+S</code> on the scratchpad.</p>
</section>

<section class="qc-band">
<h2>It borrows Omarchy's own picker</h2>
<p class="qc-lede">Region selection goes through <code>omarchy-capture-region</code>: hyprpicker freezes the screen, slurp snaps to windows and monitors, and the keyboard works the way it does everywhere else on the system. Selecting a region in Omashot is the same muscle memory as taking a screenshot, because it is the same code.</p>
<p class="qc-lede">The pixels come from the grab taken the moment you pressed the key, so the picker's own overlay can never end up in the shot.</p>
</section>

<section class="qc-band qc-install">
<h2>Install</h2>
<p class="qc-lede">Not in the AUR yet. From source, with Node 22+, a Rust toolchain and the Tauri prerequisites:</p>

```bash
git clone https://github.com/mcinnisdev/omashot
cd omashot
npm install
npm run tauri build
./scripts/install-local.sh    # binary, launcher and icons into ~/.local
./omarchy/install.sh          # theme, keys, menu, window rules, bar widget
```

<p class="qc-foot"><code>omarchy/install.sh</code> is additive and idempotent, skips anything you have edited yourself, and <code>--uninstall</code> takes it back out.</p>
</section>

<section class="qc-band qc-status">
<h2>Where this actually is</h2>
<div class="qc-status-grid">
<div class="ok">
<h3>Working</h3>
<p>Shots, briefs, sections and trails. Markup with arrows, highlight, blur that really removes pixels, and step counters. Prompt library, brand kit, exported process docs. Live theming and the bar widget.</p>
</div>
<div class="part">
<h3>Half there</h3>
<p>Recordings edit, composite and export on Linux, but capturing a source is still Windows-only. The Linux path will go through <code>gpu-screen-recorder</code>, which Omarchy already ships.</p>
</div>
<div class="no">
<h3>Not yet</h3>
<p>Keystroke badges and click-triggered trailing need <code>/dev/input</code> access Wayland does not hand out. A trail falls back to interval shots.</p>
</div>
</div>
</section>

<section class="qc-band qc-end">
<h2>Free, MIT, and built in the open</h2>
<p class="qc-lede">Everything Omashot makes is a plain file under <code>~/Omashot/</code>. No account, nothing uploaded, nothing phoned home. Omashot is a fork of <a href="https://github.com/mcinnisdev/qacut">QACut</a>, rebuilt around Omarchy's picker, keys, menu, bar and themes.</p>
<div class="qc-actions">
<a class="qc-btn primary" href="/docs/getting-started">Get started</a>
<a class="qc-btn outline" href="https://github.com/mcinnisdev/omashot">Source on GitHub</a>
</div>
</section>

</div>
