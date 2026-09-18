// The markup editor. Marks are kept as data and rendered over the untouched
// original every time, so an edit can be reopened and adjusted rather than
// painted over a flattened image. Saving writes the composite PNG in place
// (the original is kept beside it on first save) and the marks as JSON.
//
// A recording's click stills arrive with a "click" mark already in place,
// put there by the recorder, so the cursor ring is movable like anything
// else.
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

type Tool = "move" | "arrow" | "rect" | "blur" | "step";
type Mark =
  | { kind: "arrow"; x1: number; y1: number; x2: number; y2: number }
  | { kind: "rect"; x: number; y: number; w: number; h: number }
  | { kind: "blur"; x: number; y: number; w: number; h: number }
  | { kind: "step"; x: number; y: number; n: number }
  | { kind: "click"; x: number; y: number };

interface Markup {
  original: string;
  marks: Mark[];
}

const params = new URLSearchParams(location.search);
const path = params.get("path") ?? "";
const label = params.get("label") ?? "Edit";

const title = document.getElementById("title") as HTMLSpanElement;
const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const bodyEl = document.getElementById("body") as HTMLDivElement;
const toolButtons = Array.from(
  document.querySelectorAll<HTMLButtonElement>(".tool"),
);

const ctx = canvas.getContext("2d") as CanvasRenderingContext2D;
const img = new Image();
let marks: Mark[] = [];
let tool: Tool = "arrow";
let draft: Mark | null = null;
let selected: number | null = null;
let scale = 1;
let done = false;

const ACCENT = "#ff5b5b";
const CLICK = "rgb(255, 196, 0)";

title.textContent = label;

function setTool(t: Tool) {
  tool = t;
  for (const b of toolButtons) b.classList.toggle("on", b.dataset.tool === t);
  canvas.style.cursor = t === "move" ? "default" : t === "step" ? "pointer" : "crosshair";
  if (t !== "move") {
    selected = null;
    render();
  }
}

// Stroke and glyph sizes follow the image so a mark reads the same on a
// 300 px crop and a 4K grab.
function unit() {
  return Math.max(3, Math.round(Math.min(img.width, img.height) / 150));
}

function clickRadius() {
  return Math.max(8, unit() * 4);
}

function drawMark(m: Mark) {
  const u = unit();
  ctx.save();
  ctx.strokeStyle = ACCENT;
  ctx.fillStyle = ACCENT;
  ctx.lineWidth = u;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";

  if (m.kind === "arrow") {
    const angle = Math.atan2(m.y2 - m.y1, m.x2 - m.x1);
    const head = u * 4;
    ctx.beginPath();
    ctx.moveTo(m.x1, m.y1);
    ctx.lineTo(m.x2, m.y2);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(m.x2, m.y2);
    ctx.lineTo(
      m.x2 - head * Math.cos(angle - Math.PI / 6),
      m.y2 - head * Math.sin(angle - Math.PI / 6),
    );
    ctx.lineTo(
      m.x2 - head * Math.cos(angle + Math.PI / 6),
      m.y2 - head * Math.sin(angle + Math.PI / 6),
    );
    ctx.closePath();
    ctx.fill();
  } else if (m.kind === "rect") {
    ctx.fillStyle = "rgba(255, 91, 91, 0.12)";
    ctx.fillRect(m.x, m.y, m.w, m.h);
    ctx.strokeRect(m.x, m.y, m.w, m.h);
  } else if (m.kind === "blur") {
    // Pixelate: draw the region tiny, then back up with smoothing off.
    // Unlike a blur this cannot be undone by sharpening.
    const block = Math.max(8, Math.round(Math.min(img.width, img.height) / 60));
    const w = Math.max(1, Math.round(m.w));
    const h = Math.max(1, Math.round(m.h));
    const small = document.createElement("canvas");
    small.width = Math.max(1, Math.round(w / block));
    small.height = Math.max(1, Math.round(h / block));
    const sctx = small.getContext("2d") as CanvasRenderingContext2D;
    sctx.imageSmoothingEnabled = true;
    sctx.drawImage(img, m.x, m.y, w, h, 0, 0, small.width, small.height);
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(small, 0, 0, small.width, small.height, m.x, m.y, w, h);
    ctx.imageSmoothingEnabled = true;
  } else if (m.kind === "step") {
    const r = u * 4.5;
    ctx.beginPath();
    ctx.arc(m.x, m.y, r, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = "#fff";
    ctx.font = `bold ${Math.round(r * 1.25)}px ${getComputedStyle(document.body).fontFamily}`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(String(m.n), m.x, m.y + r * 0.05);
  } else {
    // The recorder's click ring: amber, filled, like the GIF shows it.
    const r = clickRadius();
    ctx.globalAlpha = 0.7;
    ctx.fillStyle = CLICK;
    ctx.beginPath();
    ctx.arc(m.x, m.y, r, 0, Math.PI * 2);
    ctx.fill();
    ctx.globalAlpha = 0.9;
    ctx.strokeStyle = CLICK;
    ctx.lineWidth = Math.max(2, u * 0.8);
    ctx.beginPath();
    ctx.arc(m.x, m.y, r, 0, Math.PI * 2);
    ctx.stroke();
  }
  ctx.restore();
}

function bounds(m: Mark) {
  const u = unit();
  if (m.kind === "arrow") {
    return {
      x: Math.min(m.x1, m.x2) - u * 2,
      y: Math.min(m.y1, m.y2) - u * 2,
      w: Math.abs(m.x2 - m.x1) + u * 4,
      h: Math.abs(m.y2 - m.y1) + u * 4,
    };
  }
  if (m.kind === "rect" || m.kind === "blur") return { x: m.x, y: m.y, w: m.w, h: m.h };
  const r = m.kind === "click" ? clickRadius() : u * 4.5;
  return { x: m.x - r, y: m.y - r, w: r * 2, h: r * 2 };
}

function drawSelection(m: Mark) {
  const b = bounds(m);
  const pad = unit();
  ctx.save();
  ctx.strokeStyle = "#fff";
  ctx.lineWidth = 1;
  ctx.setLineDash([4, 3]);
  ctx.strokeRect(b.x - pad, b.y - pad, b.w + pad * 2, b.h + pad * 2);
  ctx.restore();
}

function render() {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.drawImage(img, 0, 0);
  // Blurs first so a highlight or arrow can sit on top of one.
  for (const m of marks) if (m.kind === "blur") drawMark(m);
  for (const m of marks) if (m.kind !== "blur") drawMark(m);
  if (draft) drawMark(draft);
  if (selected !== null && marks[selected]) drawSelection(marks[selected]);
}

function fit() {
  const maxW = bodyEl.clientWidth - 24;
  const maxH = bodyEl.clientHeight - 24;
  scale = Math.min(1, maxW / img.width, maxH / img.height);
  canvas.style.width = `${Math.round(img.width * scale)}px`;
  canvas.style.height = `${Math.round(img.height * scale)}px`;
}

function pos(e: MouseEvent) {
  const r = canvas.getBoundingClientRect();
  return {
    x: Math.max(0, Math.min(img.width, (e.clientX - r.left) / scale)),
    y: Math.max(0, Math.min(img.height, (e.clientY - r.top) / scale)),
  };
}

function normalized(x: number, y: number, x2: number, y2: number) {
  return {
    x: Math.min(x, x2),
    y: Math.min(y, y2),
    w: Math.abs(x2 - x),
    h: Math.abs(y2 - y),
  };
}

// Topmost mark under a point, with a little slack so thin arrows and
// small rings are easy to grab.
function hit(x: number, y: number): number | null {
  const slack = unit() * 2;
  for (let i = marks.length - 1; i >= 0; i--) {
    const b = bounds(marks[i]);
    if (x >= b.x - slack && x <= b.x + b.w + slack && y >= b.y - slack && y <= b.y + b.h + slack) {
      return i;
    }
  }
  return null;
}

function shift(m: Mark, dx: number, dy: number): Mark {
  if (m.kind === "arrow") {
    return { ...m, x1: m.x1 + dx, y1: m.y1 + dy, x2: m.x2 + dx, y2: m.y2 + dy };
  }
  return { ...m, x: m.x + dx, y: m.y + dy };
}

let anchor: { x: number; y: number } | null = null;
let dragFrom: { x: number; y: number } | null = null;

canvas.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  const p = pos(e);
  if (tool === "move") {
    selected = hit(p.x, p.y);
    dragFrom = selected === null ? null : p;
    render();
    return;
  }
  if (tool === "step") {
    const n = marks.filter((m) => m.kind === "step").length + 1;
    marks.push({ kind: "step", x: p.x, y: p.y, n });
    render();
    return;
  }
  anchor = p;
});

canvas.addEventListener("mousemove", (e) => {
  const p = pos(e);
  if (tool === "move") {
    if (dragFrom !== null && selected !== null) {
      marks[selected] = shift(marks[selected], p.x - dragFrom.x, p.y - dragFrom.y);
      dragFrom = p;
      render();
    } else {
      canvas.style.cursor = hit(p.x, p.y) === null ? "default" : "move";
    }
    return;
  }
  if (!anchor) return;
  if (tool === "arrow") {
    draft = { kind: "arrow", x1: anchor.x, y1: anchor.y, x2: p.x, y2: p.y };
  } else if (tool === "rect" || tool === "blur") {
    const r = normalized(anchor.x, anchor.y, p.x, p.y);
    draft = tool === "rect" ? { kind: "rect", ...r } : { kind: "blur", ...r };
  }
  render();
});

window.addEventListener("mouseup", () => {
  dragFrom = null;
  if (!anchor) return;
  anchor = null;
  if (draft) {
    const big =
      draft.kind === "arrow"
        ? Math.hypot(draft.x2 - draft.x1, draft.y2 - draft.y1) > 6
        : draft.kind === "rect" || draft.kind === "blur"
          ? draft.w > 4 && draft.h > 4
          : true;
    if (big) marks.push(draft);
    draft = null;
    render();
  }
});

async function save() {
  if (done) return;
  done = true;
  selected = null;
  render();
  const blob = await new Promise<Blob | null>((res) =>
    canvas.toBlob(res, "image/png"),
  );
  if (!blob) {
    done = false;
    return;
  }
  const bytes = new Uint8Array(await blob.arrayBuffer());
  let bin = "";
  for (let i = 0; i < bytes.length; i += 0x8000) {
    bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  try {
    await invoke("save_markup", {
      path,
      pngBase64: btoa(bin),
      marks: JSON.stringify(marks),
    });
  } catch (err) {
    done = false;
    title.textContent = String(err);
    return;
  }
  await getCurrentWindow().close();
}

async function cancel() {
  if (done) return;
  done = true;
  await getCurrentWindow().close();
}

function removeSelected() {
  if (selected === null) return;
  marks.splice(selected, 1);
  selected = null;
  render();
}

for (const b of toolButtons) {
  b.addEventListener("click", () => setTool(b.dataset.tool as Tool));
}
(document.getElementById("undo") as HTMLButtonElement).addEventListener(
  "click",
  () => {
    marks.pop();
    selected = null;
    render();
  },
);
(document.getElementById("clear") as HTMLButtonElement).addEventListener(
  "click",
  () => {
    marks = [];
    selected = null;
    render();
  },
);
(document.getElementById("save") as HTMLButtonElement).addEventListener(
  "click",
  () => void save(),
);
(document.getElementById("cancel") as HTMLButtonElement).addEventListener(
  "click",
  () => void cancel(),
);

window.addEventListener("keydown", (e) => {
  const nudge: Record<string, [number, number]> = {
    ArrowLeft: [-1, 0],
    ArrowRight: [1, 0],
    ArrowUp: [0, -1],
    ArrowDown: [0, 1],
  };
  if (e.key in nudge && selected !== null) {
    e.preventDefault();
    const step = e.shiftKey ? 10 : 1;
    const [dx, dy] = nudge[e.key];
    marks[selected] = shift(marks[selected], dx * step, dy * step);
    render();
    return;
  }
  if ((e.key === "Delete" || e.key === "Backspace") && selected !== null) {
    e.preventDefault();
    removeSelected();
    return;
  }
  if (e.key === "Escape") {
    e.preventDefault();
    if (selected !== null) {
      selected = null;
      render();
      return;
    }
    void cancel();
  } else if (e.key === "Enter" || (e.ctrlKey && e.key.toLowerCase() === "s")) {
    e.preventDefault();
    void save();
  } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
    e.preventDefault();
    marks.pop();
    selected = null;
    render();
  } else if (!e.ctrlKey && !e.metaKey) {
    const k = e.key.toLowerCase();
    if (k === "m") setTool("move");
    if (k === "a") setTool("arrow");
    if (k === "h") setTool("rect");
    if (k === "b") setTool("blur");
    if (k === "s") setTool("step");
  }
});

window.addEventListener("resize", () => {
  fit();
  render();
});

async function boot() {
  const markup = await invoke<Markup>("load_markup", { path });
  marks = markup.marks;
  img.onload = () => {
    canvas.width = img.width;
    canvas.height = img.height;
    fit();
    // A still that already carries a click mark opens on Move, since
    // nudging that ring is the likely reason for opening it.
    setTool(marks.some((m) => m.kind === "click") ? "move" : "arrow");
    render();
  };
  img.src = `${convertFileSrc(markup.original)}?v=${Date.now()}`;
}

void boot();
