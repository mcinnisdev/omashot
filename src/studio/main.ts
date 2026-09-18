// QACut Studio: opens a recording, plays it back composited, and keeps the
// edits in project.json. Playback is driven by the hidden source <video>;
// every presented frame is drawn through the compositor, so the preview is
// the export.
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { buildTrack, draw, layout, viewAt, type Track } from "./compositor";
import {
  DEFAULT_ZOOM_SCALE,
  withDefaults,
  zoomsFromMarks,
  type Edits,
  type Events,
  type Project,
  type StudioInfo,
  type Zoom,
} from "./model";

const params = new URLSearchParams(location.search);

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const canvas = $<HTMLCanvasElement>("canvas");
const ctx = canvas.getContext("2d") as CanvasRenderingContext2D;
const stage = $<HTMLDivElement>("stage");
const empty = $<HTMLDivElement>("empty");
const src = $<HTMLVideoElement>("src");
const cam = $<HTMLVideoElement>("cam");
const nameInput = $<HTMLInputElement>("name");
const playBtn = $<HTMLButtonElement>("play");
const timeEl = $<HTMLSpanElement>("time");
const scrub = $<HTMLInputElement>("scrub");
const facts = $<HTMLDivElement>("facts");
const inspector = $<HTMLElement>("inspector");
const recordings = $<HTMLDivElement>("recordings");
const recordingsList = $<HTMLDivElement>("recordings-list");
const toastEl = $<HTMLDivElement>("toast");
const timeline = $<HTMLDivElement>("timeline");
const tlZooms = $<HTMLDivElement>("tl-zooms");
const tlMarks = $<HTMLDivElement>("tl-marks");
const tlHead = $<HTMLDivElement>("tl-head");

let dir: string | null = null;
let project: Project | null = null;
let events: Events | null = null;
let edits: Edits = withDefaults(undefined);
let track: Track | null = null;
let saveTimer = 0;
let rafPending = false;
let selectedZoom: Zoom | null = null;

function toast(text: string) {
  toastEl.textContent = text;
  toastEl.hidden = false;
  window.clearTimeout(saveTimer);
  window.setTimeout(() => (toastEl.hidden = true), 1600);
}

function fmt(ms: number) {
  const s = Math.max(0, ms / 1000);
  return `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}.${Math.floor((s % 1) * 10)}`;
}

// ------------------------------------------------------------- canvas

// 16:9 output, sized to the stage; drawn at device pixels for crispness.
function fitCanvas() {
  const dpr = window.devicePixelRatio || 1;
  const maxW = stage.clientWidth - 32;
  const maxH = stage.clientHeight - 32;
  let w = maxW;
  let h = (w * 9) / 16;
  if (h > maxH) {
    h = maxH;
    w = (h * 16) / 9;
  }
  canvas.style.width = `${Math.round(w)}px`;
  canvas.style.height = `${Math.round(h)}px`;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  render();
}

function currentMs() {
  return src.currentTime * 1000;
}

function render() {
  if (!project || !track) return;
  draw(ctx, { t: currentMs(), source: src, camera: project.camera ? cam : null }, project, edits, track);
  timeEl.textContent = `${fmt(currentMs())} / ${fmt(project.duration_ms)}`;
  if (document.activeElement !== scrub) {
    scrub.value = String(Math.round((currentMs() / Math.max(1, project.duration_ms)) * 1000));
  }
  tlHead.style.left = `${(currentMs() / Math.max(1, project.duration_ms)) * 100}%`;
}

function scheduleRender() {
  if (rafPending) return;
  rafPending = true;
  requestAnimationFrame(() => {
    rafPending = false;
    render();
  });
}

// Every presented source frame is composited; while paused, edits re-render.
function onFrame() {
  render();
  syncCamera();
  if (!src.paused && !src.ended) src.requestVideoFrameCallback(onFrame);
}

// ------------------------------------------------------------ playback

function syncCamera(force = false) {
  if (!project?.camera) return;
  const want = src.currentTime - project.camera.offset_ms / 1000;
  if (want < 0) {
    if (!cam.paused) cam.pause();
    return;
  }
  const drift = Math.abs(cam.currentTime - want);
  if (force || drift > 0.12) cam.currentTime = want;
  if (!src.paused && cam.paused) void cam.play().catch(() => {});
  if (src.paused && !cam.paused) cam.pause();
}

async function play() {
  if (!project) return;
  if (src.ended) src.currentTime = 0;
  await src.play();
  syncCamera(true);
  playBtn.textContent = "Pause";
  src.requestVideoFrameCallback(onFrame);
}

function pause() {
  src.pause();
  cam.pause();
  playBtn.textContent = "Play";
  render();
}

function seekMs(ms: number) {
  if (!project) return;
  src.currentTime = Math.max(0, Math.min(project.duration_ms, ms)) / 1000;
  syncCamera(true);
  // The frame arrives asynchronously; draw when it does.
  src.requestVideoFrameCallback(() => render());
}

// --------------------------------------------------------------- edits

function bindInspector() {
  const on = (id: string, ev: string, fn: (el: HTMLInputElement | HTMLSelectElement) => void) => {
    const el = $<HTMLInputElement | HTMLSelectElement>(id);
    el.addEventListener(ev, () => {
      fn(el);
      scheduleRender();
      saveSoon();
    });
  };
  on("padding", "input", (el) => (edits.frame.padding = Number(el.value)));
  on("radius", "input", (el) => (edits.frame.radius = Number(el.value)));
  on("background", "change", (el) => (edits.frame.background = el.value as Edits["frame"]["background"]));
  on("shadow", "change", (el) => (edits.frame.shadow = (el as HTMLInputElement).checked));
  on("cursor-size", "input", (el) => (edits.cursor.size = Number(el.value)));
  on("smoothing", "input", (el) => {
    edits.cursor.smoothing = Number(el.value);
    // A new cursor path means new camera paths too.
    if (project && events) track = buildTrack(project, events, edits);
  });
  on("ripple", "change", (el) => (edits.cursor.ripple = (el as HTMLInputElement).checked));
  on("show-keys", "change", (el) => (edits.keys.show = (el as HTMLInputElement).checked));
  on("key-mode", "change", (el) => {
    edits.keys.mode = el.value as Edits["keys"]["mode"];
    if (project && events) track = buildTrack(project, events, edits);
    renderTimeline();
  });
  on("cam-show", "change", (el) => (edits.camera.show = (el as HTMLInputElement).checked));
  on("cam-size", "input", (el) => (edits.camera.size = Number(el.value)));
  on("cam-corner", "change", (el) => (edits.camera.corner = el.value as Edits["camera"]["corner"]));
  on("cam-shape", "change", (el) => (edits.camera.shape = el.value as Edits["camera"]["shape"]));
}

// ------------------------------------------------------------ timeline

function pct(ms: number) {
  return `${(ms / Math.max(1, project?.duration_ms ?? 1)) * 100}%`;
}

function msAt(clientX: number) {
  const r = timeline.getBoundingClientRect();
  const f = Math.min(1, Math.max(0, (clientX - r.left) / r.width));
  return f * (project?.duration_ms ?? 0);
}

function selectZoom(z: Zoom | null) {
  selectedZoom = z;
  $<HTMLElement>("zoom-none").hidden = z !== null;
  $<HTMLElement>("zoom-edit").hidden = z === null;
  if (z) {
    $<HTMLInputElement>("zoom-scale").value = String(z.scale);
    $<HTMLInputElement>("zoom-follow").checked = z.follow ?? false;
  }
  $<HTMLInputElement>("zoom-follow-default").checked = edits.zoom_follow;
  $<HTMLInputElement>("follow-tightness").value = String(edits.follow_tightness);
  renderTimeline();
  scheduleRender();
}

function renderTimeline() {
  if (!project || !track) return;
  tlZooms.replaceChildren();
  tlMarks.replaceChildren();

  for (const z of edits.zooms) {
    const el = document.createElement("div");
    el.className = "tl-zoom" + (z === selectedZoom ? " selected" : "");
    el.style.left = pct(z.start);
    el.style.width = pct(z.end - z.start);
    el.title = `Zoom ${z.scale.toFixed(1)}×, ${fmt(z.start)} to ${fmt(z.end)}`;
    const l = document.createElement("div");
    l.className = "edge l";
    const r = document.createElement("div");
    r.className = "edge r";
    el.append(l, r);

    // Drag the body to move, an edge to retime; a click selects.
    const startDrag = (e: MouseEvent, mode: "move" | "l" | "r") => {
      e.stopPropagation();
      e.preventDefault();
      selectZoom(z);
      const from = { x: e.clientX, start: z.start, end: z.end };
      const onMove = (m: MouseEvent) => {
        const dms = msAt(m.clientX) - msAt(from.x);
        const D = project!.duration_ms;
        if (mode === "move") {
          const len = from.end - from.start;
          z.start = Math.min(Math.max(0, from.start + dms), D - len);
          z.end = z.start + len;
        } else if (mode === "l") {
          z.start = Math.min(Math.max(0, from.start + dms), from.end - 300);
        } else {
          z.end = Math.max(Math.min(D, from.end + dms), from.start + 300);
        }
        el.style.left = pct(z.start);
        el.style.width = pct(z.end - z.start);
        seekMs(mode === "r" ? z.end - 1 : z.start + 1);
      };
      const onUp = () => {
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
        edits.zooms.sort((a, b) => a.start - b.start);
        track?.follow.delete(z);
        saveSoon();
        renderTimeline();
      };
      window.addEventListener("mousemove", onMove);
      window.addEventListener("mouseup", onUp);
    };
    el.addEventListener("mousedown", (e) => startDrag(e, "move"));
    l.addEventListener("mousedown", (e) => startDrag(e, "l"));
    r.addEventListener("mousedown", (e) => startDrag(e, "r"));
    tlZooms.append(el);
  }

  for (const c of track.clicks) {
    const d = document.createElement("div");
    d.className = "tl-click";
    d.style.left = pct(c.t);
    tlMarks.append(d);
  }
  const hidden = new Set(edits.keys.hidden);
  for (const b of track.badges) {
    const k = document.createElement("div");
    k.className = "tl-key" + (hidden.has(b.t) ? " hidden-key" : "");
    k.style.left = pct(b.t);
    k.title = `${b.text} at ${fmt(b.t)}${hidden.has(b.t) ? " (hidden)" : ""}`;
    k.addEventListener("mousedown", (e) => {
      e.stopPropagation();
      e.preventDefault();
      if (hidden.has(b.t)) edits.keys.hidden = edits.keys.hidden.filter((t) => t !== b.t);
      else edits.keys.hidden.push(b.t);
      seekMs(b.t + 50);
      saveSoon();
      renderTimeline();
    });
    tlMarks.append(k);
  }
}

// Clicking the empty timeline seeks; clicking away deselects a zoom.
timeline.addEventListener("mousedown", (e) => {
  if (!project) return;
  selectZoom(null);
  const move = (m: MouseEvent) => seekMs(msAt(m.clientX));
  move(e);
  const up = () => {
    window.removeEventListener("mousemove", move);
    window.removeEventListener("mouseup", up);
  };
  window.addEventListener("mousemove", move);
  window.addEventListener("mouseup", up);
});

// Drag in the preview while a zoom is selected to move where it looks.
canvas.addEventListener("mousedown", (e) => {
  if (!project || !selectedZoom) return;
  const z = selectedZoom;
  const t = currentMs();
  if (t < z.start || t > z.end) seekMs(z.start + Math.min(700, (z.end - z.start) / 2));
  const rect = canvas.getBoundingClientRect();
  const L = layout(canvas.width, canvas.height, project, edits);
  const view = viewAt(edits.zooms, currentMs(), project.region, track ?? undefined, edits.follow_tightness);
  const perPx = (canvas.width / rect.width) / (L.s * view.scale);
  let last = { x: e.clientX, y: e.clientY };
  canvas.style.cursor = "grabbing";
  const onMove = (m: MouseEvent) => {
    z.cx = Math.min(Math.max(z.cx - (m.clientX - last.x) * perPx, 0), project!.region.width);
    z.cy = Math.min(Math.max(z.cy - (m.clientY - last.y) * perPx, 0), project!.region.height);
    last = { x: m.clientX, y: m.clientY };
    track?.follow.delete(z);
    scheduleRender();
  };
  const onUp = () => {
    canvas.style.cursor = "";
    window.removeEventListener("mousemove", onMove);
    window.removeEventListener("mouseup", onUp);
    saveSoon();
  };
  window.addEventListener("mousemove", onMove);
  window.addEventListener("mouseup", onUp);
});

$("zoom-add").addEventListener("click", () => {
  if (!project || !track) return;
  const t = currentMs();
  const path = track.path;
  let cx = project.region.width / 2;
  let cy = project.region.height / 2;
  if (path.length > 0) {
    const p = path.reduce((best, q) => (Math.abs(q.t - t) < Math.abs(best.t - t) ? q : best), path[0]);
    cx = p.x;
    cy = p.y;
  }
  const z: Zoom = {
    start: t,
    end: Math.min(project.duration_ms, t + 3000),
    cx,
    cy,
    scale: DEFAULT_ZOOM_SCALE,
    follow: edits.zoom_follow,
  };
  edits.zooms.push(z);
  edits.zooms.sort((a, b) => a.start - b.start);
  selectZoom(z);
  saveSoon();
});
$("zoom-remove").addEventListener("click", () => {
  if (!selectedZoom) return;
  edits.zooms = edits.zooms.filter((z) => z !== selectedZoom);
  selectZoom(null);
  saveSoon();
});
$<HTMLInputElement>("zoom-scale").addEventListener("input", (e) => {
  if (!selectedZoom) return;
  selectedZoom.scale = Number((e.target as HTMLInputElement).value);
  track?.follow.delete(selectedZoom);
  renderTimeline();
  scheduleRender();
  saveSoon();
});
$<HTMLInputElement>("zoom-follow").addEventListener("change", (e) => {
  if (!selectedZoom) return;
  selectedZoom.follow = (e.target as HTMLInputElement).checked;
  track?.follow.delete(selectedZoom);
  scheduleRender();
  saveSoon();
});
$<HTMLInputElement>("zoom-follow-default").addEventListener("change", (e) => {
  edits.zoom_follow = (e.target as HTMLInputElement).checked;
  saveSoon();
});
$<HTMLInputElement>("follow-tightness").addEventListener("input", (e) => {
  edits.follow_tightness = Number((e.target as HTMLInputElement).value);
  // Every follow path depends on it.
  if (track) track.follow = new WeakMap();
  scheduleRender();
  saveSoon();
});

function showInspector() {
  $<HTMLInputElement>("padding").value = String(edits.frame.padding);
  $<HTMLInputElement>("radius").value = String(edits.frame.radius);
  $<HTMLSelectElement>("background").value = edits.frame.background;
  $<HTMLInputElement>("shadow").checked = edits.frame.shadow;
  $<HTMLInputElement>("cursor-size").value = String(edits.cursor.size);
  $<HTMLInputElement>("smoothing").value = String(edits.cursor.smoothing);
  $<HTMLInputElement>("ripple").checked = edits.cursor.ripple;
  $<HTMLInputElement>("show-keys").checked = edits.keys.show;
  $<HTMLSelectElement>("key-mode").value = edits.keys.mode;
  $<HTMLInputElement>("cam-show").checked = edits.camera.show;
  $<HTMLInputElement>("cam-size").value = String(edits.camera.size);
  $<HTMLSelectElement>("cam-corner").value = edits.camera.corner;
  $<HTMLSelectElement>("cam-shape").value = edits.camera.shape;
  $<HTMLElement>("camera-section").hidden = !project?.camera?.has_video;

  const p = project!;
  const secs = Math.round(p.duration_ms / 1000);
  const fps = p.duration_ms > 0 ? Math.round((p.frames / p.duration_ms) * 1000) : 0;
  facts.replaceChildren();
  for (const line of [
    `${p.region.width} × ${p.region.height} region on a ${p.monitor.width} × ${p.monitor.height} monitor`,
    `${secs} s, ${p.frames} frames (${fps} fps delivered)`,
    p.camera ? `camera${p.camera.has_video ? " and mic" : " mic only"}, offset ${p.camera.offset_ms} ms` : "no camera or mic",
    `${events?.buttons.filter((b) => b[2] === "down").length ?? 0} clicks, ${events?.keys.filter((k) => k.down).length ?? 0} key presses`,
  ]) {
    const d = document.createElement("div");
    d.textContent = line;
    facts.append(d);
  }
}

function saveSoon() {
  window.clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => {
    if (!dir) return;
    void invoke("save_studio_edits", { dir, edits, name: nameInput.value });
  }, 400);
}

// -------------------------------------------------------------- loading

async function open(projectDir: string) {
  pause();
  const loaded = await invoke<{ project: Project; events: Events }>("load_studio_project", { dir: projectDir });
  dir = projectDir;
  project = loaded.project;
  events = loaded.events;
  edits = withDefaults(project.edits as Partial<Edits>);
  // The operator's zoom marks become blocks once; after that the blocks
  // are theirs to change or delete.
  if (!edits.zooms_seeded) {
    edits.zooms = zoomsFromMarks(events.zooms, project.region, project.duration_ms, edits.zoom_follow);
    edits.zooms_seeded = true;
    saveSoon();
  }
  track = buildTrack(project, events, edits);
  selectedZoom = null;
  nameInput.value = project.name;

  src.src = convertFileSrc(`${projectDir}/${project.source}`);
  if (project.camera) {
    cam.src = convertFileSrc(`${projectDir}/${project.camera.file}`);
    cam.muted = !project.camera.has_audio;
  } else {
    cam.removeAttribute("src");
  }
  empty.hidden = true;
  inspector.hidden = false;
  recordings.hidden = true;
  showInspector();
  selectZoom(null);
  await new Promise<void>((resolve) => {
    src.addEventListener("loadeddata", () => resolve(), { once: true });
  });
  seekMs(0);
}

async function showRecordings() {
  const list = await invoke<StudioInfo[]>("list_studio_projects");
  recordingsList.replaceChildren();
  if (list.length === 0) {
    const none = document.createElement("div");
    none.className = "empty";
    none.textContent = "No studio recordings yet.";
    recordingsList.append(none);
  }
  for (const r of list) {
    const row = document.createElement("div");
    row.className = "bundle-row";
    if (r.dir === dir) row.classList.add("current");
    const name = document.createElement("span");
    name.className = "bundle-name-cell";
    name.textContent = r.name || r.id;
    const meta = document.createElement("span");
    meta.className = "bundle-meta";
    meta.textContent = `${r.created_at.slice(0, 16).replace("T", " ")}  ${Math.round(r.duration_ms / 1000)} s${r.has_camera ? "  camera" : ""}`;
    const btn = document.createElement("button");
    btn.className = "btn";
    btn.textContent = r.dir === dir ? "Open now" : "Open";
    btn.disabled = r.dir === dir;
    btn.addEventListener("click", () => void open(r.dir).catch((e) => toast(String(e))));
    row.append(name, meta, btn);
    recordingsList.append(row);
  }
  recordings.hidden = false;
}

// ------------------------------------------------------------- wiring

bindInspector();

playBtn.addEventListener("click", () => (src.paused ? void play() : pause()));
scrub.addEventListener("input", () => {
  if (!project) return;
  seekMs((Number(scrub.value) / 1000) * project.duration_ms);
});
nameInput.addEventListener("change", saveSoon);
$("open-list").addEventListener("click", () => void showRecordings());
$("list-close").addEventListener("click", () => (recordings.hidden = true));
$("open-folder").addEventListener("click", () => {
  if (dir) void invoke("open_path", { path: dir });
});
$("close").addEventListener("click", () => void getCurrentWindow().close());

window.addEventListener("keydown", (e) => {
  const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement;
  if (e.key === "Escape") {
    if (!recordings.hidden) recordings.hidden = true;
    else void getCurrentWindow().close();
    return;
  }
  if (typing) return;
  if (e.key === " ") {
    e.preventDefault();
    if (src.paused) void play();
    else pause();
  } else if (e.key === "ArrowLeft") {
    seekMs(currentMs() - (e.shiftKey ? 5000 : 1000));
  } else if (e.key === "ArrowRight") {
    seekMs(currentMs() + (e.shiftKey ? 5000 : 1000));
  }
});

src.addEventListener("ended", () => pause());
window.addEventListener("resize", fitCanvas);
new ResizeObserver(fitCanvas).observe(stage);

const initial = params.get("project");
if (initial) {
  void open(initial).catch((e) => toast(String(e)));
} else {
  empty.hidden = false;
  inspector.hidden = true;
  void showRecordings();
}
fitCanvas();
