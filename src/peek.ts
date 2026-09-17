import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { AppState, Export } from "./types";

const body = document.getElementById("body") as HTMLDivElement;
const count = document.getElementById("count") as HTMLSpanElement;
const bundleName = document.getElementById("bundle-name") as HTMLInputElement;
const purpose = document.getElementById("purpose") as HTMLSelectElement;
const custom = document.getElementById("custom") as HTMLDivElement;
const customPrompt = document.getElementById("custom-prompt") as HTMLTextAreaElement;
const brandToggle = document.getElementById("brand-toggle") as HTMLButtonElement;
const brand = document.getElementById("brand") as HTMLDivElement;
const brandInclude = document.getElementById("brand-include") as HTMLInputElement;
const brandFiles = document.getElementById("brand-files") as HTMLSpanElement;
const brandOpen = document.getElementById("brand-open") as HTMLButtonElement;
const brandNotes = document.getElementById("brand-notes") as HTMLTextAreaElement;
const exported = document.getElementById("exported") as HTMLDivElement;
const exportedLabel = document.getElementById("exported-label") as HTMLElement;
const exportedPath = document.getElementById("exported-path") as HTMLElement;

function clock(ms: number) {
  const s = Math.round(ms / 1000);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

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

  const state = await invoke<AppState>("get_state");
  const session = state.session;
  body.replaceChildren();

  // Name, purpose and brand can be set before the first capture; the
  // backend starts a session on demand.
  bundleName.value = session?.name ?? "";
  purpose.value = session?.purpose ?? "fix";
  custom.hidden = purpose.value !== "custom";
  customPrompt.value = state.custom_prompt;

  brandInclude.checked = session?.include_brand ?? true;
  brandNotes.value = state.brand.notes;
  const n = state.brand.files.length;
  const hasKit = n > 0 || state.brand.notes.trim() !== "";
  brandFiles.textContent =
    n === 0 ? "no files yet" : `${n} file${n === 1 ? "" : "s"}`;
  brandToggle.classList.toggle("on", hasKit && brandInclude.checked);

  const shots = session?.groups.reduce((n, g) => n + g.shots.length, 0) ?? 0;
  const used = session?.groups.filter((g) => g.shots.length > 0).length ?? 0;
  count.textContent = session ? `${shots} in ${used || 1} groups` : "";

  if (state.last_export) {
    showExported(state.last_export, state.dirty);
  } else {
    exported.style.display = "none";
  }

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

    // Where the next capture lands. Clicking an earlier group points new
    // shots back at it.
    const target = document.createElement("button");
    target.className = "target";
    if (g.index === session.current) {
      target.classList.add("active");
      target.textContent = "Capturing here";
      target.disabled = true;
      wrap.classList.add("current");
    } else {
      target.textContent = "Capture here";
      target.addEventListener("click", () =>
        void invoke("set_current_group", { group: g.index }),
      );
    }

    head.append(num, nameInput, target);

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

      const isRec = s.kind === "recording";
      const left = document.createElement("div");
      const img = document.createElement("img");
      img.src = convertFileSrc(s.abs_path);
      img.alt = `${isRec ? "Recording" : "Screenshot"} ${g.index}.${i + 1}`;
      img.addEventListener("click", () =>
        void invoke("open_path", { path: s.abs_path }),
      );

      const meta = document.createElement("div");
      meta.className = "meta";
      meta.textContent = isRec
        ? `${g.index}.${i + 1}  ${clock(s.duration_ms)}  ${s.width}x${s.height}`
        : `${g.index}.${i + 1}  ${s.width}x${s.height}`;
      left.append(img, meta);

      const middle = document.createElement("div");
      middle.className = "shot-fields";

      const title = document.createElement("input");
      title.className = "shot-title";
      title.value = s.title;
      title.placeholder = `${isRec ? "Recording" : "Shot"} ${i + 1}`;
      title.setAttribute("aria-label", `Name for screenshot ${g.index}.${i + 1}`);

      const text = document.createElement("textarea");
      text.value = s.note;
      text.placeholder = "No note";
      text.setAttribute("aria-label", `Note for screenshot ${g.index}.${i + 1}`);

      const saveShot = () =>
        void invoke("set_shot_note", {
          group: g.index,
          shot: s.id,
          note: text.value,
          title: title.value,
        });
      title.addEventListener("change", saveShot);
      text.addEventListener("change", saveShot);
      middle.append(title, text);

      const del = document.createElement("button");
      del.className = "remove";
      del.textContent = "\u00d7";
      del.title = "Remove this screenshot";
      del.setAttribute("aria-label", `Remove screenshot ${g.index}.${i + 1}`);
      del.addEventListener("click", async () => {
        await invoke("delete_shot", { group: g.index, shot: s.id });
        await render();
      });

      row.append(left, middle, del);
      wrap.append(row);
    });

    body.append(wrap);
  }
}

function showExported(result: Export, dirty: boolean) {
  exported.style.display = "flex";
  exportedLabel.textContent = dirty
    ? "Changed since it was last written to"
    : "Bundle written to";
  exportedPath.textContent = result.root;
}

type Action = "path" | "prompt" | "markdown" | "open";

const flashText: Record<Action, string> = {
  path: "Path copied",
  prompt: "Prompt copied",
  markdown: "Markdown copied",
  open: "Opened",
};

// Every action writes the bundle first, so what gets copied or opened is
// never stale.
async function run(action: Action, btn: HTMLButtonElement) {
  try {
    const result = await invoke<Export>("finish", { action });
    showExported(result, false);
    flash(btn, flashText[action]);
  } catch (err) {
    flash(btn, String(err));
  }
}

const copyPath = document.getElementById("copy-path") as HTMLButtonElement;
const copyPrompt = document.getElementById("copy-prompt") as HTMLButtonElement;
const copyMd = document.getElementById("copy-md") as HTMLButtonElement;
const openFolder = document.getElementById("open-folder") as HTMLButtonElement;
const newBundle = document.getElementById("new-bundle") as HTMLButtonElement;
const close = document.getElementById("close") as HTMLButtonElement;

copyPath.addEventListener("click", () => void run("path", copyPath));
copyPrompt.addEventListener("click", () => void run("prompt", copyPrompt));
copyMd.addEventListener("click", () => void run("markdown", copyMd));
openFolder.addEventListener("click", () => void run("open", openFolder));
close.addEventListener("click", () => void getCurrentWindow().close());

newBundle.addEventListener("click", async () => {
  try {
    await invoke("new_bundle");
    flash(newBundle, "Started fresh");
  } catch (err) {
    flash(newBundle, String(err));
  }
  await render();
});

bundleName.addEventListener("change", async () => {
  try {
    await invoke("rename_bundle", { name: bundleName.value });
  } catch (err) {
    bundleName.title = String(err);
  }
  bundleName.blur();
  await render();
});
bundleName.addEventListener("keydown", (e) => {
  if (e.key === "Enter") bundleName.blur();
});

purpose.addEventListener("change", async () => {
  await invoke("set_purpose", { purpose: purpose.value });
  custom.hidden = purpose.value !== "custom";
  if (!custom.hidden) customPrompt.focus();
});

customPrompt.addEventListener("change", () => {
  void invoke("set_custom_prompt", { text: customPrompt.value });
});

brandToggle.addEventListener("click", () => {
  brand.hidden = !brand.hidden;
  if (!brand.hidden) brandNotes.focus();
});
brandInclude.addEventListener("change", () => {
  void invoke("set_include_brand", { include: brandInclude.checked });
});
brandOpen.addEventListener("click", () => void invoke("open_brand_folder"));
brandNotes.addEventListener("change", async () => {
  await invoke("set_brand_notes", { text: brandNotes.value });
  await render();
});

window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") void getCurrentWindow().close();
});

void listen("session-changed", () => void render());

void render();
