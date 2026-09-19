import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { Frame } from "./types";

const params = new URLSearchParams(location.search);
const monitor = params.get("m") ?? "0";
// "shot" crops the frozen frame the moment the drag ends. "record" and
// "studio" let the selection be adjusted first, then start recording the
// live region after a countdown: "record" makes v1's GIF and stills,
// "studio" makes a v2 source for the studio.
const mode = params.get("mode") ?? "shot";
const recording = mode === "record" || mode === "studio";

const frameEl = document.getElementById("frame") as HTMLImageElement;
const scrim = document.getElementById("scrim") as HTMLDivElement;
const guideV = document.getElementById("guide-v") as HTMLDivElement;
const guideH = document.getElementById("guide-h") as HTMLDivElement;
const sel = document.getElementById("sel") as HTMLDivElement;
const dims = document.getElementById("dims") as HTMLDivElement;
const hint = document.getElementById("hint") as HTMLDivElement;
const tools = document.getElementById("tools") as HTMLDivElement;
const toolRecord = document.getElementById("tool-record") as HTMLButtonElement;
const toolCancel = document.getElementById("tool-cancel") as HTMLButtonElement;

// Ignore accidental click-drags; anything smaller than this is a misfire.
const MIN_EDGE = 8;

type Rect = { x: number; y: number; width: number; height: number };
type Drag =
  | { kind: "new"; ax: number; ay: number }
  | { kind: "move"; ax: number; ay: number; from: Rect }
  | { kind: "resize"; ax: number; ay: number; from: Rect; edges: string };

type Phase = "idle" | "drag" | "adjust";

let phase: Phase = "idle";
let drag: Drag | null = null;
let current: Rect | null = null;
let sent = false;

const HINTS = {
  shot: "<b>Drag</b> to select the region <kbd>Esc</kbd> cancel",
  record:
    mode === "studio"
      ? "<b>Drag</b> the region for the Studio recording <kbd>Esc</kbd> cancel"
      : "<b>Drag</b> the region to auto-capture <kbd>Esc</kbd> cancel",
  adjust:
    "<b>Drag</b> the box or its edges to adjust <kbd>Enter</kbd> record <kbd>Esc</kbd> cancel",
};

async function boot() {
  hint.innerHTML = recording ? HINTS.record : HINTS.shot;
  const frame = await invoke<Frame | null>("frame_for", { monitor });
  if (!frame) {
    await cancel();
    return;
  }
  frameEl.src = convertFileSrc(frame.png_path);
}

function clamp(v: number, lo: number, hi: number) {
  return Math.min(Math.max(v, lo), hi);
}

function fromCorners(ax: number, ay: number, bx: number, by: number): Rect {
  return {
    x: Math.min(ax, bx),
    y: Math.min(ay, by),
    width: Math.abs(bx - ax),
    height: Math.abs(by - ay),
  };
}

function paint(r: Rect) {
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

  if (phase === "adjust") placeTools(r);
}

function placeTools(r: Rect) {
  tools.style.display = "flex";
  const w = tools.offsetWidth;
  const h = tools.offsetHeight;
  const left = clamp(r.x + r.width - w, 8, window.innerWidth - w - 8);
  const fitsBelow = r.y + r.height + 10 + h < window.innerHeight - 8;
  const top = fitsBelow ? r.y + r.height + 10 : Math.max(8, r.y - h - 10);
  tools.style.left = `${left}px`;
  tools.style.top = `${top}px`;
}

function setPhase(p: Phase) {
  phase = p;
  document.body.classList.toggle("adjust", p === "adjust");
  sel.classList.toggle("adjust", p === "adjust");
  const idle = p === "idle";
  scrim.style.display = idle ? "block" : "none";
  guideV.style.display = idle ? "block" : "none";
  guideH.style.display = idle ? "block" : "none";
  hint.style.display = p === "drag" ? "none" : "flex";
  if (idle) {
    sel.style.display = "none";
    dims.style.display = "none";
    hint.innerHTML = recording ? HINTS.record : HINTS.shot;
  }
  if (p === "adjust") hint.innerHTML = HINTS.adjust;
  if (p !== "adjust") tools.style.display = "none";
}

async function cancel() {
  if (sent) return;
  sent = true;
  await invoke("cancel_capture");
}

async function send(r: Rect) {
  if (sent) return;
  sent = true;
  try {
    const command =
      mode === "studio"
        ? "start_studio"
        : recording
          ? "start_recording"
          : mode === "quick"
            ? "commit_quick"
            : "commit_selection";
    await invoke(command, {
      monitor,
      x: r.x,
      y: r.y,
      width: r.width,
      height: r.height,
    });
  } catch (err) {
    // The overlay is already closing; make sure the reason is not lost.
    await invoke("log_error", { message: `selection failed: ${String(err)}` });
  }
}

/// Applies a move or resize to the rectangle the drag started from.
function apply(d: Drag, cx: number, cy: number): Rect {
  const dx = cx - d.ax;
  const dy = cy - d.ay;
  const W = window.innerWidth;
  const H = window.innerHeight;

  if (d.kind === "new") return fromCorners(d.ax, d.ay, cx, cy);

  if (d.kind === "move") {
    return {
      x: clamp(d.from.x + dx, 0, W - d.from.width),
      y: clamp(d.from.y + dy, 0, H - d.from.height),
      width: d.from.width,
      height: d.from.height,
    };
  }

  let { x, y, width, height } = d.from;
  const right = x + width;
  const bottom = y + height;
  if (d.edges.includes("w")) {
    x = clamp(d.from.x + dx, 0, right - MIN_EDGE);
    width = right - x;
  }
  if (d.edges.includes("e")) {
    width = clamp(d.from.width + dx, MIN_EDGE, W - x);
  }
  if (d.edges.includes("n")) {
    y = clamp(d.from.y + dy, 0, bottom - MIN_EDGE);
    height = bottom - y;
  }
  if (d.edges.includes("s")) {
    height = clamp(d.from.height + dy, MIN_EDGE, H - y);
  }
  return { x, y, width, height };
}

window.addEventListener("mousedown", (e) => {
  if (e.button !== 0 || sent) return;
  const target = e.target as HTMLElement;
  if (tools.contains(target)) return;

  if (phase === "adjust" && current) {
    const handle = target.closest(".handle") as HTMLElement | null;
    if (handle) {
      drag = {
        kind: "resize",
        ax: e.clientX,
        ay: e.clientY,
        from: current,
        edges: handle.dataset.edges ?? "",
      };
      return;
    }
    if (sel.contains(target)) {
      drag = { kind: "move", ax: e.clientX, ay: e.clientY, from: current };
      return;
    }
  }

  drag = { kind: "new", ax: e.clientX, ay: e.clientY };
  setPhase("drag");
  paint(apply(drag, e.clientX, e.clientY));
});

window.addEventListener("mousemove", (e) => {
  if (!drag) {
    if (phase === "idle") {
      guideV.style.left = `${e.clientX}px`;
      guideH.style.top = `${e.clientY}px`;
    }
    return;
  }
  paint(apply(drag, e.clientX, e.clientY));
});

window.addEventListener("mouseup", async (e) => {
  if (!drag) return;
  const d = drag;
  drag = null;
  const r = apply(d, e.clientX, e.clientY);

  if (d.kind === "new") {
    if (r.width < MIN_EDGE || r.height < MIN_EDGE) {
      current = null;
      setPhase("idle");
      return;
    }
    if (!recording) {
      await send(r);
      return;
    }
  }

  current = r;
  setPhase("adjust");
  paint(r);
});

window.addEventListener("keydown", async (e) => {
  if (e.key === "Escape") {
    e.preventDefault();
    await cancel();
    return;
  }
  if (e.key === "Enter" && phase === "adjust" && current) {
    e.preventDefault();
    await send(current);
  }
});

// Right-click is the other muscle-memory way out of a selection tool.
window.addEventListener("contextmenu", async (e) => {
  e.preventDefault();
  await cancel();
});

toolRecord.addEventListener("click", () => {
  if (current) void send(current);
});
toolCancel.addEventListener("click", () => void cancel());

boot();
