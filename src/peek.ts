import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Export, Session } from "./types";

const body = document.getElementById("body") as HTMLDivElement;
const count = document.getElementById("count") as HTMLSpanElement;
const exported = document.getElementById("exported") as HTMLDivElement;
const exportedPath = document.getElementById("exported-path") as HTMLElement;

function flash(btn: HTMLButtonElement, text: string) {
  const original = btn.textContent ?? "";
  btn.textContent = text;
  window.setTimeout(() => {
    btn.textContent = original;
  }, 1400);
}

async function render() {
  // Never yank the DOM out from under someone mid-sentence.
  const active = document.activeElement;
  if (
    active instanceof HTMLTextAreaElement ||
    active instanceof HTMLInputElement
  ) {
    return;
  }

  const session = await invoke<Session | null>("get_session");
  body.replaceChildren();

  const shots = session?.groups.reduce((n, g) => n + g.shots.length, 0) ?? 0;
  const used = session?.groups.filter((g) => g.shots.length > 0).length ?? 0;
  count.textContent = session ? `${shots} in ${used || 1} groups` : "";

  if (!session || shots === 0) {
    const empty = document.createElement("div");
    empty.className = "empty";
    empty.textContent =
      "Nothing captured yet. Press the capture hotkey, drag a region, type what is wrong, and hit Enter. Shots land in the current group until you start a new one.";
    body.append(empty);
    return;
  }

  for (const g of session.groups) {
    if (g.shots.length === 0 && !g.title && !g.master_note) continue;

    const wrap = document.createElement("div");
    wrap.className = "group";

    const head = document.createElement("div");
    head.className = "group-head";

    const num = document.createElement("span");
    num.className = "group-num";
    num.textContent = String(g.index).padStart(2, "0");

    const nameInput = document.createElement("input");
    nameInput.value = g.title;
    nameInput.placeholder = `Group ${g.index}`;
    nameInput.setAttribute("aria-label", `Name for group ${g.index}`);

    head.append(num, nameInput);

    const master = document.createElement("textarea");
    master.className = "master";
    master.value = g.master_note;
    master.placeholder = "Master note for this group";
    master.setAttribute("aria-label", `Master note for group ${g.index}`);

    const saveGroup = () =>
      void invoke("set_group_note", {
        group: g.index,
        title: nameInput.value,
        masterNote: master.value,
      });
    nameInput.addEventListener("change", saveGroup);
    master.addEventListener("change", saveGroup);

    wrap.append(head, master);

    g.shots.forEach((s, i) => {
      const row = document.createElement("div");
      row.className = "shot";

      const left = document.createElement("div");
      const img = document.createElement("img");
      img.src = convertFileSrc(s.abs_path);
      img.alt = `Screenshot ${g.index}.${i + 1}`;
      img.addEventListener("click", () =>
        void invoke("open_path", { path: s.abs_path }),
      );

      const meta = document.createElement("div");
      meta.className = "meta";
      meta.textContent = `${g.index}.${i + 1}  ${s.width}x${s.height}`;
      left.append(img, meta);

      const text = document.createElement("textarea");
      text.value = s.note;
      text.placeholder = "No note";
      text.setAttribute("aria-label", `Note for screenshot ${g.index}.${i + 1}`);
      text.addEventListener("change", () =>
        void invoke("set_shot_note", {
          group: g.index,
          shot: s.id,
          note: text.value,
        }),
      );

      const del = document.createElement("button");
      del.className = "remove";
      del.textContent = "\u00d7";
      del.title = "Remove this screenshot";
      del.setAttribute("aria-label", `Remove screenshot ${g.index}.${i + 1}`);
      del.addEventListener("click", async () => {
        await invoke("delete_shot", { group: g.index, shot: s.id });
        await render();
      });

      row.append(left, text, del);
      wrap.append(row);
    });

    body.append(wrap);
  }
}

function showExported(result: Export) {
  exported.style.display = "flex";
  exportedPath.textContent = result.root;
}

async function run(action: "path" | "markdown" | "open", btn: HTMLButtonElement) {
  try {
    const result = await invoke<Export>("finish", { action });
    showExported(result);
    if (action === "path") flash(btn, "Path copied");
    if (action === "markdown") flash(btn, "Markdown copied");
  } catch (err) {
    flash(btn, String(err));
  }
}

const copyPath = document.getElementById("copy-path") as HTMLButtonElement;
const copyMd = document.getElementById("copy-md") as HTMLButtonElement;
const openFolder = document.getElementById("open-folder") as HTMLButtonElement;
const close = document.getElementById("close") as HTMLButtonElement;

copyPath.addEventListener("click", () => void run("path", copyPath));
copyMd.addEventListener("click", () => void run("markdown", copyMd));
openFolder.addEventListener("click", () => void run("open", openFolder));
close.addEventListener("click", () => void getCurrentWindow().close());

window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") void getCurrentWindow().close();
});

void listen("session-changed", () => void render());

// If the window was opened by the finish hotkey, the export already happened.
void invoke<Export | null>("get_last_export").then((last) => {
  if (last) showExported(last);
});

void render();
