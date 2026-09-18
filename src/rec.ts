// The recording overlay. It covers the monitor, ignores the mouse, tints
// everything outside the region and outlines the region so it is always
// clear what is in the shot. The badge counts down, then shows elapsed
// time once Rust reports the recorder has started. Rust closes the window.
//
// For a studio recording it also records the microphone and, if enabled,
// the camera: getUserMedia into a MediaRecorder whose chunks are streamed to
// Rust as they arrive, and a small live preview of the camera outside the
// region. Sync with the screen source is by timestamp: Rust is told the
// moment the recorder started.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const params = new URLSearchParams(location.search);
const num = (k: string) => Number(params.get(k) ?? 0);
const region = { x: num("x"), y: num("y"), w: num("w"), h: num("h") };
const countdownMs = num("countdown");
const studio = params.get("studio") === "1";

const hole = document.getElementById("hole") as HTMLDivElement;
const pill = document.getElementById("pill") as HTMLDivElement;
const label = document.getElementById("label") as HTMLSpanElement;
const time = document.getElementById("time") as HTMLSpanElement;
const stop = document.getElementById("stop") as HTMLSpanElement;
const cam = document.getElementById("cam") as HTMLVideoElement;

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

// The camera preview prefers a corner of the monitor the region does not
// cover, so it is not in the operator's way, and otherwise sits inside the
// region's bottom-right. Either way it is never in the source: the studio
// overlay is excluded from capture.
function placeCam() {
  const w = 200;
  const h = 150;
  const m = 16;
  const W = window.innerWidth;
  const H = window.innerHeight;
  const corners = [
    { left: W - w - m, top: H - h - m },
    { left: m, top: H - h - m },
    { left: W - w - m, top: m },
    { left: m, top: m },
  ];
  const clear = (c: { left: number; top: number }) =>
    c.left + w <= region.x ||
    c.left >= region.x + region.w ||
    c.top + h <= region.y ||
    c.top >= region.y + region.h;
  const spot = corners.find(clear) ?? {
    left: Math.max(m, region.x + region.w - w - m),
    top: Math.max(m, region.y + region.h - h - m),
  };
  cam.hidden = false;
  cam.style.left = `${spot.left}px`;
  cam.style.top = `${spot.top}px`;
}

const opened = Date.now();
let started: number | null = countdownMs > 0 ? null : opened;
let zoomed = false;

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
    pill.classList.toggle("zoomed", zoomed);
    label.textContent = zoomed ? "REC · ZOOM" : "REC";
    time.textContent = fmt(Date.now() - started);
    stop.innerHTML = studio
      ? zoomed
        ? "<kbd>Ctrl/Cmd</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd> zooms out"
        : "<kbd>Ctrl/Cmd</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd> zoom here · <kbd>Ctrl/Cmd</kbd>+<kbd>Shift</kbd>+<kbd>3</kbd> stops"
      : "<kbd>Ctrl/Cmd</kbd>+<kbd>Shift</kbd>+<kbd>R</kbd> stops";
  }
  placePill();
}

// ------------------------------------------------------ mic and camera

interface StudioSettings {
  keystrokes: boolean;
  mic: boolean;
  camera: boolean;
}

let recorder: MediaRecorder | null = null;
let stream: MediaStream | null = null;
// Chunks are appended in order, one at a time.
let flush: Promise<void> = Promise.resolve();
let finished = false;

async function setupMedia() {
  const s = await invoke<StudioSettings>("get_studio_settings");
  if (!s.mic && !s.camera) return;
  try {
    stream = await navigator.mediaDevices.getUserMedia({
      audio: s.mic ? { echoCancellation: true, noiseSuppression: true } : false,
      video: s.camera ? { width: 640, height: 480, frameRate: 30 } : false,
    });
  } catch (err) {
    await invoke("log_error", { message: `mic/camera unavailable: ${String(err)}` });
    return;
  }
  const hasVideo = stream.getVideoTracks().length > 0;
  const hasAudio = stream.getAudioTracks().length > 0;
  if (hasVideo) {
    cam.srcObject = stream;
    cam.muted = true;
    void cam.play();
    placeCam();
  }
  const mime = hasVideo ? "video/webm;codecs=vp9,opus" : "audio/webm;codecs=opus";
  recorder = new MediaRecorder(stream, {
    mimeType: MediaRecorder.isTypeSupported(mime) ? mime : undefined,
    videoBitsPerSecond: 1_500_000,
    audioBitsPerSecond: 96_000,
  });
  recorder.ondataavailable = (e) => {
    if (e.data.size === 0) return;
    const blob = e.data;
    flush = flush.then(async () => {
      const bytes = new Uint8Array(await blob.arrayBuffer());
      await invoke("append_camera", bytes);
    });
  };
  recorder.onstart = () => {
    void invoke("camera_started", {
      atUnixMs: Date.now(),
      hasVideo,
      hasAudio,
    });
  };
}

async function finishMedia() {
  if (finished) return;
  finished = true;
  if (recorder && recorder.state !== "inactive") {
    await new Promise<void>((resolve) => {
      recorder!.onstop = () => resolve();
      recorder!.stop();
    });
  }
  await flush;
  stream?.getTracks().forEach((t) => t.stop());
  await invoke("camera_stopped");
}

void listen("recording-started", () => {
  started = Date.now();
  tick();
  if (studio && recorder && recorder.state === "inactive") recorder.start(1000);
});

void listen("recording-stop", () => {
  void finishMedia();
});

void listen<boolean>("zoom-changed", (e) => {
  zoomed = e.payload;
  tick();
});

if (studio) void setupMedia();

tick();
window.setInterval(tick, 250);
