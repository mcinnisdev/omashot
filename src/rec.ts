// The recording badge. It only shows elapsed time; the recorder itself runs
// in Rust and this window is closed from there when the recording stops.

const time = document.getElementById("time") as HTMLSpanElement;
const started = Date.now();

function tick() {
  const s = Math.floor((Date.now() - started) / 1000);
  time.textContent = `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

tick();
window.setInterval(tick, 500);
