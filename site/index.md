---
layout: page
sidebar: false
aside: false
title: QACut
titleTemplate: Screen capture that hands off
---

<div class="qc">
<section class="qc-hero">
<div>
<span class="eyebrow">Free and open source · Windows</span>
<h1>Capture what's wrong.<br />Record <em>how it's done.</em></h1>
<p class="lead">QACut turns screenshots and clips into a bundle an AI agent can act on. QACut Studio turns a screen recording into a walkthrough people actually watch. One tray icon. No account. Nothing leaves your machine unless you send it.</p>
<div class="qc-actions">
<a class="qc-btn primary" href="https://github.com/mcinnisdev/qacut/releases/latest">Download for Windows</a>
<a class="qc-btn ghost" href="/docs/getting-started">Getting started</a>
<a class="qc-btn ghost" href="https://github.com/mcinnisdev/qacut">Source on GitHub</a>
</div>
<p class="qc-fine">Windows 10 (2004+) or 11, x64. MIT licensed. No telemetry, no sign-in.</p>
</div>
<div class="qc-stage" aria-label="A QACut Studio frame: a window on a gradient background with a cursor, a click ripple, a keystroke badge, a camera bubble and a title">
<img class="logo" src="/logo.png" alt="" />
<div class="title">Unlink OneDrive from this computer</div>
<div class="win">
<div class="bar"><i></i><i></i><i></i></div>
<div class="side"><b></b><b></b><b></b><b></b><b></b></div>
<div class="main"><b class="w1"></b><b class="w2"></b><b class="w3"></b><div class="field"></div><span class="btn">Unlink this PC</span></div>
</div>
<div class="ripple"></div>
<svg class="cursor" viewBox="0 0 20 20" aria-hidden="true"><path d="M0 0v16l4.2-3.4L7.2 19l3-1.3-2.9-6.3h5.1z" fill="#fff" stroke="rgba(0,0,0,.9)" stroke-width="1.3" stroke-linejoin="round"/></svg>
<div class="keys">Ctrl+Shift+3</div>
<div class="cam"></div>
</div>
</section>
<section>
<div class="center">
<span class="eyebrow">Two tools, one tray</span>
<h2>Built for the two audiences a screen ends up in front of</h2>
<p class="lead">An agent needs files it can read and act on. A person needs something worth watching. QACut makes both from the same three hotkeys.</p>
</div>
<div class="qc-split" style="margin-top: 32px">
<div class="qc-card">
<span class="eyebrow">QACut</span>
<h3>Bundles for agents</h3>
<p>Freeze the screen, drag a region, type what's wrong, keep going. Group shots as you move between pages. Finish, and the bundle is a folder with a markdown file that reads in order, every image linked relatively.</p>
<ul>
<li>Auto-capture a process: a clip plus a still at every click, labelled with where it landed.</li>
<li>Mark up: arrows, highlights, step counters, and a blur that actually removes text.</li>
<li>Hand off as a folder path to a CLI agent or as a ZIP to a chat, with the prompt written for you.</li>
</ul>
<pre class="visual qc-bundle"><span class="h"># QA bundle: Settings review</span>
<span></span>
<span class="h">## 1. Settings page</span>
<span class="q">&gt; Everything on this page is a bit off</span>
<span></span>
<span class="h">### 1.1 Save button</span>
<span class="l">![1.1](01-settings-page/01.png)</span>
Clipped at 125% scaling; label wraps.
<span></span>
<span class="h">### 1.2</span>
<span class="l">![1.2](01-settings-page/02.gif)</span>
Toggle animates but the state never saves.
<span class="m">Key frames: 0 s start · 3 s click at 412,188 · 9 s end</span></pre>
</div>
<div class="qc-card">
<span class="eyebrow">QACut Studio</span>
<h3>Recordings for people</h3>
<p>Record with the cursor as data, not pixels. Then draw it back in, smoothed, enlarged, with a ripple on every click. Zoom to where the work is, live with one key or afterwards on a timeline. Your camera in a bubble, your voice in sync, your logo in the corner.</p>
<ul>
<li>Zooms that follow the cursor, with a dead zone so they never twitch.</li>
<li>Trim, cut the middle out, drag blocks, and the preview follows every move.</li>
<li>Export exactly what you previewed: H.264 MP4 with AAC narration, 720p to 1440p.</li>
</ul>
<div class="visual qc-timeline" aria-label="A studio timeline with zoom blocks, a cut, click and key markers">
<div class="film"></div>
<div class="cut" style="left: 44%; width: 9%"></div>
<div class="zoom" style="left: 12%; width: 14%"></div>
<div class="zoom" style="left: 31%; width: 10%"></div>
<div class="zoom" style="left: 60%; width: 18%"></div>
<div class="click" style="left: 16%"></div><div class="click" style="left: 23%"></div><div class="click" style="left: 35%"></div><div class="click" style="left: 66%"></div><div class="click" style="left: 74%"></div>
<div class="key" style="left: 70%"></div><div class="key" style="left: 83%"></div>
<div class="handle" style="left: 5%"></div><div class="handle" style="left: 93%"></div>
<div class="head"></div>
</div>
</div>
</div>
</section>
<section>
<div class="center">
<span class="eyebrow">How it works</span>
<h2>Three keys and you're done</h2>
</div>
<div class="qc-steps">
<div class="qc-step"><span class="n">1</span><h3>Press a key</h3><p><kbd>Ctrl+Shift+2</kbd> for a screenshot, <kbd>Ctrl+Shift+R</kbd> to auto-capture a process, <kbd>Ctrl+Shift+3</kbd> for a Studio recording. Drag the region. A three-second countdown lets you get in place.</p></div>
<div class="qc-step"><span class="n">2</span><h3>Do the thing</h3><p>Type a note under each shot. Press <kbd>Ctrl+Shift+Z</kbd> mid-recording to zoom in where you are and again to zoom out. Every click, key and cursor move is recorded as data on the same clock as the frames.</p></div>
<div class="qc-step"><span class="n">3</span><h3>Hand it off</h3><p><kbd>Ctrl+Shift+Enter</kbd> writes the bundle and copies the path; the prompt is one click away. Or the Studio opens on your recording, you tidy it in minutes, and Export writes the MP4.</p></div>
</div>
</section>
<section>
<div class="center">
<span class="eyebrow">Studio</span>
<h2>The polish is automatic. The control is yours.</h2>
<p class="lead">Everything Screen Studio and Loom charge for, drawn from data you already recorded, editable after the fact.</p>
</div>
<div class="qc-grid">
<div class="qc-feature"><div class="ico">⌖</div><h3>Zoom that follows</h3><p>A zoom block pushes in with an eased 600 ms move, holds still while the cursor moves inside the middle of the view, and eases after it near the edges. Tightness is a slider.</p></div>
<div class="qc-feature"><div class="ico">◎</div><h3>Clicks and keys</h3><p>A ripple where every click landed. Keystroke badges for shortcuts only, or every key. Hide any single badge from the timeline.</p></div>
<div class="qc-feature"><div class="ico">◉</div><h3>Camera and voice</h3><p>Your camera in a circle or rounded bubble, any corner, any size. Narration recorded on the same clock and cut with the video.</p></div>
<div class="qc-feature"><div class="ico">⟷</div><h3>Trim and cut</h3><p>Start and end handles you can actually grab. Cut a stretch out of the middle, then drag it, resize it, or put it back. A filmstrip shows where you are.</p></div>
<div class="qc-feature"><div class="ico">◈</div><h3>Title and logo</h3><p>A title and subtitle in the padding above or below the frame, and a logo from your brand folder in a corner of it. Five backgrounds, or plain.</p></div>
<div class="qc-feature"><div class="ico">▶</div><h3>Export that matches</h3><p>The preview and the export share one renderer. Hardware H.264 where the machine has it, AAC narration, 30 or 60 fps, written straight into the recording's folder.</p></div>
</div>
</section>
<section>
<div class="center">
<span class="eyebrow">Made for agents</span>
<h2>A bundle is a folder an agent can read</h2>
<p class="lead">No API, no plugin, no format to learn. The markdown explains itself at the top, images are linked relatively, and the prompt is written for the purpose you picked: fix the issues, write the process doc, or your own.</p>
</div>
<div class="qc-split" style="margin-top: 32px">
<div class="qc-card">
<h3>To a CLI agent</h3>
<p>Finish copies the folder path. <strong>Copy agent prompt</strong> gives you the instruction with the path filled in.</p>
<pre class="visual qc-bundle">claude "Work through the QA bundle at
  C:\Users\nick\QACut\2026-09-17_143022-settings-review.
Start with bundle.md: each group is a page or area,
its quoted master note applies to every screenshot
under it, and each screenshot's note says what is
wrong. Open each screenshot, and the key frames of
any recording, before changing anything."</pre>
</div>
<div class="qc-card">
<h3>To a chat agent</h3>
<p><strong>Save ZIP for chat</strong> zips the folder, shows it in Explorer ready to drag in, and copies a prompt that says "the attached ZIP".</p>
<ul>
<li>The brand kit rides along, so the output sounds like you.</li>
<li>Recordings include stills at every click, so an agent that can't play video still sees each step.</li>
<li>Everything stays plain files. Move the folder anywhere and it still works.</li>
</ul>
</div>
</div>
</section>
<section>
<div class="center">
<span class="eyebrow">Why not the tools you already know?</span>
<h2>Own the files. Skip the seat.</h2>
</div>
<div class="qc-scroll">
<table class="qc-compare">
<thead><tr><th></th><th>QACut</th><th>Loom</th><th>Scribe</th><th>Screen Studio</th></tr></thead>
<tbody>
<tr><td>Runs on your machine, no upload</td><td class="yes">Yes</td><td class="no">Cloud</td><td class="no">Cloud</td><td class="yes">Yes</td></tr>
<tr><td>No account required</td><td class="yes">Yes</td><td class="no">No</td><td class="no">No</td><td class="some">License</td></tr>
<tr><td>Bundles an AI agent can act on</td><td class="yes">Yes</td><td class="no">No</td><td class="some">Docs only</td><td class="no">No</td></tr>
<tr><td>Cursor smoothing and follow zoom</td><td class="yes">Yes</td><td class="no">No</td><td class="no">No</td><td class="yes">Yes</td></tr>
<tr><td>Zooms you mark while recording</td><td class="yes">Yes</td><td class="no">No</td><td class="no">No</td><td class="no">No</td></tr>
<tr><td>Step documents from clicks</td><td class="yes">Yes</td><td class="no">No</td><td class="yes">Yes</td><td class="no">No</td></tr>
<tr><td>Your brand on the output</td><td class="yes">Yes</td><td class="some">Paid</td><td class="some">Paid</td><td class="some">Partial</td></tr>
<tr><td>Open source</td><td class="yes">MIT</td><td class="no">No</td><td class="no">No</td><td class="no">No</td></tr>
<tr class="total"><td>Price</td><td class="yes">Free</td><td class="some">Per seat</td><td class="some">Per seat</td><td class="some">One-time</td></tr>
</tbody>
</table>
</div>
<p class="qc-note">Comparison reflects each product's public plans as of September 2026. They are good tools; they solve a different problem than "hand this to an agent" and "own the files".</p>
</section>
<section>
<div class="qc-oss">
<div>
<span class="eyebrow">Open source</span>
<h2>Free, MIT, and built in the open</h2>
<p class="lead">Rust and TypeScript on Tauri. A few thousand lines you can read in an afternoon. Fork it, package it, ship it inside your own tools.</p>
</div>
<div class="qc-actions">
<a class="qc-btn primary" href="https://github.com/mcinnisdev/qacut">Star on GitHub</a>
<a class="qc-btn ghost" href="https://github.com/mcinnisdev/qacut/releases/latest">Download</a>
</div>
</div>
</section>
</div>
