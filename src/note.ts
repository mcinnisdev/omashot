import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Session } from "./types";

const mode = new URLSearchParams(location.search).get("mode") ?? "shot";
const isGroup = mode === "group";

const label = document.getElementById("head-label") as HTMLSpanElement;
const where = document.getElementById("head-where") as HTMLSpanElement;
const title = document.getElementById("title") as HTMLInputElement;
const note = document.getElementById("note") as HTMLTextAreaElement;
const secondary = document.getElementById("secondary") as HTMLButtonElement;

let done = false;

async function boot() {
  if (isGroup) {
    label.textContent = "New group";
    title.hidden = false;
    note.placeholder = "Master note, e.g. problems on the settings page";
    secondary.textContent = "Cancel";
    title.focus();
  } else {
    label.textContent = "Note";
    secondary.textContent = "Discard shot";
    note.focus();
  }

  const session = await invoke<Session | null>("get_session");
  if (!session) return;
  const g = session.groups[session.groups.length - 1];
  if (!g) return;

  const name = g.title.trim() || `Group ${g.index}`;
  where.textContent = isGroup
    ? `after ${name}`
    : `${name} / shot ${g.shots.length}`;
}

async function saveShot() {
  if (done) return;
  done = true;
  await invoke("save_note", { note: note.value });
}

async function saveGroup() {
  if (done) return;
  done = true;
  await invoke("save_group", {
    title: title.value,
    masterNote: note.value,
  });
}

async function commit() {
  if (isGroup) await saveGroup();
  else await saveShot();
}

async function bail() {
  if (done) return;
  done = true;
  if (isGroup) {
    // Nothing was created yet, so backing out is just closing the window.
    await getCurrentWindow().close();
  } else {
    await invoke("discard_pending");
  }
}

function keys(e: KeyboardEvent) {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    void commit();
    return;
  }
  if (e.key === "Escape") {
    e.preventDefault();
    // Esc on a shot keeps the screenshot and leaves the note empty, which is
    // the fast path when the picture already says it. Esc on a group prompt
    // abandons the group, since nothing has been created yet.
    if (isGroup) void bail();
    else void commit();
  }
}

title.addEventListener("keydown", keys);
note.addEventListener("keydown", keys);
secondary.addEventListener("click", () => void bail());

boot();
