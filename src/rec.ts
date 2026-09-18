// The recording overlay. It covers the monitor, ignores the mouse, tints
// everything outside the region and outlines the region so it is always
// clear what is in the shot. The badge counts down, then shows elapsed
// time once Rust reports the recorder has started. Rust closes the window.
import { listen } from "@tauri-apps/api/event";

const params = new URLSearchParams(location.search);
const num = (k: string) => Number(params.get(k) ?? 0);
const region = { x: num("x"), y: num("y"), w: num("w"), h: num("h") };
const countdownMs = num("countdown");

const hole = document.getElementById("hole") as HTMLDivElement;
const pill = document.getElementById("pill") as HTMLDivElement;
const label = document.getElementById("label") as HTMLSpanElement;
const time = document.getElementById("time") as HTMLSpanElement;
const stop = document.getElementById("stop") as HTMLSpanElement;

hole.style.left = `${region.x}px`;
hole.style.top = `${region.y}px`;
hole.style.width = `${region.w}px`;
hole.style.height = `${region.h}px`;

// Badge above the region, else below, else tucked into its top-right
// corner (the only case where it ends up in the recording).
function placePill() {
  const pw = pill.offsetWidth;
  const ph = pill.offsetHeight;
  const gap = 10;
  let top: number;
  if (region.y - ph - gap >= 0) top = region.y - ph - gap;
  else if (region.y + region.h + gap + ph <= window.innerHeight)
    top = region.y + region.h + gap;
  else top = region.y + gap;
  const left = Math.max(8, Math.min(region.x, window.innerWidth - pw - 8));
  pill.style.left = `${left}px`;
  pill.style.top = `${top}px`;
}

const opened = Date.now();
let started: number | null = countdownMs > 0 ? null : opened;

function fmt(ms: number) {
  const s = Math.floor(ms / 1000);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

function tick() {
  if (started === null) {
    pill.classList.add("arming");
    label.textContent = "Recording in";
    time.textContent = String(
      Math.max(1, Math.ceil((countdownMs - (Date.now() - opened)) / 1000)),
    );
    stop.innerHTML = "<kbd>Ctrl/Cmd</kbd>+<kbd>Shift</kbd>+<kbd>R</kbd> cancels";
  } else {
    pill.classList.remove("arming");
    label.textContent = "REC";
    time.textContent = fmt(Date.now() - started);
    stop.innerHTML = "<kbd>Ctrl/Cmd</kbd>+<kbd>Shift</kbd>+<kbd>R</kbd> stops";
  }
  placePill();
}

void listen("recording-started", () => {
  started = Date.now();
  tick();
});

tick();
window.setInterval(tick, 250);
