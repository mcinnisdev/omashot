// QACut Studio: opens a recording, plays it back composited, and keeps the
// edits in project.json. Playback is driven by the hidden source <video>;
// every presented frame is drawn through the compositor, so the preview is
// the export.
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { buildTrack, draw, type Track } from "./compositor";
import { withDefaults, type Edits, type Events, type Project, type StudioInfo } from "./model";

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

let dir: string | null = null;
let project: Project | null = null;
let events: Events | null = null;
let edits: Edits = withDefaults(undefined);
let track: Track | null = null;
let saveTimer = 0;
let rafPending = false;

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
    if (project && events) track = buildTrack(project, events, edits);
  });
  on("ripple", "change", (el) => (edits.cursor.ripple = (el as HTMLInputElement).checked));
  on("show-keys", "change", (el) => (edits.keys.show = (el as HTMLInputElement).checked));
  on("cam-show", "change", (el) => (edits.camera.show = (el as HTMLInputElement).checked));
  on("cam-size", "input", (el) => (edits.camera.size = Number(el.value)));
  on("cam-corner", "change", (el) => (edits.camera.corner = el.value as Edits["camera"]["corner"]));
  on("cam-shape", "change", (el) => (edits.camera.shape = el.value as Edits["camera"]["shape"]));
}

function showInspector() {
  $<HTMLInputElement>("padding").value = String(edits.frame.padding);
  $<HTMLInputElement>("radius").value = String(edits.frame.radius);
  $<HTMLSelectElement>("background").value = edits.frame.background;
  $<HTMLInputElement>("shadow").checked = edits.frame.shadow;
  $<HTMLInputElement>("cursor-size").value = String(edits.cursor.size);
  $<HTMLInputElement>("smoothing").value = String(edits.cursor.smoothing);
  $<HTMLInputElement>("ripple").checked = edits.cursor.ripple;
  $<HTMLInputElement>("show-keys").checked = edits.keys.show;
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
  track = buildTrack(project, events, edits);
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
