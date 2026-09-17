// The recording badge. During the countdown it shows the seconds left; once
// Rust reports the recorder has started it shows elapsed time. The recorder
// itself runs in Rust and this window is closed from there.
import { listen } from "@tauri-apps/api/event";

const pill = document.getElementById("pill") as HTMLDivElement;
const label = document.getElementById("label") as HTMLSpanElement;
const time = document.getElementById("time") as HTMLSpanElement;
const stop = document.getElementById("stop") as HTMLSpanElement;

const countdownMs = Number(new URLSearchParams(location.search).get("countdown") ?? 0);
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
    time.textContent = String(Math.max(1, Math.ceil((countdownMs - (Date.now() - opened)) / 1000)));
    stop.innerHTML = "<kbd>Ctrl/Cmd</kbd>+<kbd>Shift</kbd>+<kbd>R</kbd> cancels";
    return;
  }
  pill.classList.remove("arming");
  label.textContent = "REC";
  time.textContent = fmt(Date.now() - started);
  stop.innerHTML = "<kbd>Ctrl/Cmd</kbd>+<kbd>Shift</kbd>+<kbd>R</kbd> stops";
}

void listen("recording-started", () => {
  started = Date.now();
  tick();
});

tick();
window.setInterval(tick, 250);
