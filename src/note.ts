import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Session } from "./types";

const mode = new URLSearchParams(location.search).get("mode") ?? "shot";
const isGroup = mode === "group";
const isRecording = mode === "recording";
const isQuick = mode === "quick";

const label = document.getElementById("head-label") as HTMLSpanElement;
const groupTitle = document.getElementById("group-title") as HTMLInputElement;
const sep = document.getElementById("sep") as HTMLSpanElement;
const shotTitle = document.getElementById("shot-title") as HTMLInputElement;
const groupHint = document.getElementById("group-hint") as HTMLParagraphElement;
const note = document.getElementById("note") as HTMLTextAreaElement;
const secondary = document.getElementById("secondary") as HTMLButtonElement;
const keysHint = document.getElementById("keys") as HTMLSpanElement;

let done = false;

async function boot() {
  // The names in the header are editable in place; the number is the
  // fallback when they are left blank.
  if (isGroup) {
    label.textContent = "Wrap up";
    sep.hidden = true;
    shotTitle.hidden = true;
    groupHint.hidden = false;
    note.placeholder = "Master note, e.g. problems on the settings page";
    secondary.textContent = "Cancel";
  } else if (isRecording) {
    label.textContent = "Auto-capture";
    note.placeholder = "What is happening in this clip?";
    secondary.textContent = "Discard clip";
  } else if (isQuick) {
    // No bundle, no group: the shot is already saved under Quick/<today>.
    // Enter copies its path and note; Ctrl+Enter copies every shot from
    // today so a handful can be pasted in one go.
    label.textContent = "Quick shot";
    sep.hidden = true;
    groupTitle.hidden = true;
    shotTitle.hidden = true;
    note.placeholder = "What should the agent do with this?";
    secondary.textContent = "Discard shot";
    keysHint.innerHTML =
      "<kbd>Enter</kbd> copy path + note <kbd>Ctrl</kbd>+<kbd>Enter</kbd> copy all of today";
    note.focus();
    return;
  } else {
    label.textContent = "Note";
    secondary.textContent = "Discard shot";
  }
  note.focus();

  const session = await invoke<Session | null>("get_session");
  if (!session) return;
  const g =
    session.groups.find((x) => x.index === session.current) ??
    session.groups[session.groups.length - 1];
  if (!g) return;

  groupTitle.value = g.title;
  groupTitle.placeholder = `Group ${g.index}`;
  shotTitle.placeholder = `${isRecording ? "Recording" : "Shot"} ${g.shots.length}`;

  if (isGroup) {
    const name = g.title.trim() || `Group ${g.index}`;
    groupHint.textContent =
      g.shots.length === 0
        ? `${name} has no shots yet. Enter saves its name and note; captures keep landing here.`
        : `Add a master note for ${name} (optional). Enter saves it and starts the next group. To add more shots to this group later, press Capture here in the bundle window.`;
  }
}

async function saveShot() {
  if (done) return;
  done = true;
  await invoke("save_note", {
    note: note.value,
    title: shotTitle.value,
    groupTitle: groupTitle.value,
  });
}

async function saveGroup() {
  if (done) return;
  done = true;
  await invoke("close_group", {
    title: groupTitle.value,
    masterNote: note.value,
  });
}

async function saveQuick(all: boolean) {
  if (done) return;
  done = true;
  await invoke("save_quick", { note: note.value, all });
}

async function commit(all = false) {
  if (isGroup) await saveGroup();
  else if (isQuick) await saveQuick(all);
  else await saveShot();
}

async function bail() {
  if (done) return;
  done = true;
  if (isGroup) {
    // Nothing was created yet, so backing out is just closing the window.
    await getCurrentWindow().close();
  } else if (isQuick) {
    await invoke("discard_quick");
  } else {
    await invoke("discard_pending");
  }
}

function keys(e: KeyboardEvent) {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    void commit(isQuick && (e.ctrlKey || e.metaKey));
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

groupTitle.addEventListener("keydown", keys);
shotTitle.addEventListener("keydown", keys);
note.addEventListener("keydown", keys);
secondary.addEventListener("click", () => void bail());

boot();
