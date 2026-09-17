import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { Frame } from "./types";

const params = new URLSearchParams(location.search);
const monitor = params.get("m") ?? "0";
// "shot" crops the frozen frame; "record" starts recording the live region.
const recording = params.get("mode") === "record";

const frameEl = document.getElementById("frame") as HTMLImageElement;
const scrim = document.getElementById("scrim") as HTMLDivElement;
const guideV = document.getElementById("guide-v") as HTMLDivElement;
const guideH = document.getElementById("guide-h") as HTMLDivElement;
const sel = document.getElementById("sel") as HTMLDivElement;
const dims = document.getElementById("dims") as HTMLDivElement;
const hint = document.getElementById("hint") as HTMLDivElement;

// Ignore accidental click-drags; anything smaller than this is a misfire.
const MIN_EDGE = 8;

let dragging = false;
let sent = false;
let ax = 0;
let ay = 0;

async function boot() {
  if (recording) {
    hint.innerHTML =
      "<b>Drag</b> the region to record <kbd>Esc</kbd> cancel";
  }
  const frame = await invoke<Frame | null>("frame_for", { monitor });
  if (!frame) {
    await cancel();
    return;
  }
  frameEl.src = convertFileSrc(frame.png_path);
}

function rect(bx: number, by: number) {
  return {
    x: Math.min(ax, bx),
    y: Math.min(ay, by),
    width: Math.abs(bx - ax),
    height: Math.abs(by - ay),
  };
}

function paint(r: { x: number; y: number; width: number; height: number }) {
  sel.style.display = "block";
  sel.style.left = `${r.x}px`;
  sel.style.top = `${r.y}px`;
  sel.style.width = `${r.width}px`;
  sel.style.height = `${r.height}px`;

  dims.style.display = "block";
  dims.textContent = `${Math.round(r.width)} x ${Math.round(r.height)}`;
  // Keep the readout inside the viewport, above the selection when there is
  // room and below it when the drag started near the top edge.
  const below = r.y < 28;
  dims.style.left = `${Math.min(r.x, window.innerWidth - 90)}px`;
  dims.style.top = below ? `${r.y + r.height + 6}px` : `${r.y - 24}px`;
}

function reset() {
  dragging = false;
  sel.style.display = "none";
  dims.style.display = "none";
  scrim.style.display = "block";
  hint.style.display = "flex";
}

async function cancel() {
  if (sent) return;
  sent = true;
  await invoke("cancel_capture");
}

async function commit(r: {
  x: number;
  y: number;
  width: number;
  height: number;
}) {
  if (sent) return;
  sent = true;
  await invoke(recording ? "start_recording" : "commit_selection", {
    monitor,
    x: r.x,
    y: r.y,
    width: r.width,
    height: r.height,
  });
}

window.addEventListener("mousemove", (e) => {
  if (!dragging) {
    guideV.style.left = `${e.clientX}px`;
    guideH.style.top = `${e.clientY}px`;
    return;
  }
  paint(rect(e.clientX, e.clientY));
});

window.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  dragging = true;
  ax = e.clientX;
  ay = e.clientY;
  scrim.style.display = "none";
  hint.style.display = "none";
  guideV.style.display = "none";
  guideH.style.display = "none";
  paint(rect(ax, ay));
});

window.addEventListener("mouseup", async (e) => {
  if (!dragging) return;
  dragging = false;
  const r = rect(e.clientX, e.clientY);
  if (r.width < MIN_EDGE || r.height < MIN_EDGE) {
    guideV.style.display = "block";
    guideH.style.display = "block";
    reset();
    return;
  }
  await commit(r);
});

window.addEventListener("keydown", async (e) => {
  if (e.key === "Escape") {
    e.preventDefault();
    await cancel();
  }
});

// Right-click is the other muscle-memory way out of a selection tool.
window.addEventListener("contextmenu", async (e) => {
  e.preventDefault();
  await cancel();
});

boot();
