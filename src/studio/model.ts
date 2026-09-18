// Types mirroring src-tauri/src/studio/{project,events}.rs, plus the edit
// model the studio keeps in project.json's `edits` block.

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Camera {
  file: string;
  offset_ms: number;
  has_video: boolean;
  has_audio: boolean;
}

export interface Project {
  version: number;
  id: string;
  name: string;
  created_at: string;
  source: string;
  events: string;
  camera: Camera | null;
  monitor: Rect;
  scale: number;
  region: Rect;
  fps: number;
  first_frame_ts: number;
  frames: number;
  duration_ms: number;
  keystrokes: boolean;
  edits: Partial<Edits> | Record<string, never>;
}

export interface KeyOut {
  t: number;
  key: string;
  down: boolean;
  mods: string;
}

export interface Events {
  version: number;
  cursor: [number, number, number][];
  shapes: [number, string][];
  buttons: [number, string, string, number, number][];
  keys: KeyOut[];
  windows: [number, string][];
}

export interface StudioInfo {
  dir: string;
  id: string;
  name: string;
  created_at: string;
  duration_ms: number;
  frames: number;
  has_camera: boolean;
}

export type Background = "midnight" | "sunset" | "ocean" | "slate" | "plain";
export type Corner = "br" | "bl" | "tr" | "tl";

export interface Edits {
  frame: { padding: number; radius: number; background: Background; shadow: boolean };
  cursor: { size: number; smoothing: number; ripple: boolean };
  keys: { show: boolean };
  camera: { show: boolean; size: number; corner: Corner; shape: "circle" | "rounded" };
  trim: { in_ms: number; out_ms: number | null };
  /// Zoom blocks come in milestone 3; kept here so the format is stable.
  zooms: { start: number; end: number; cx: number; cy: number; scale: number }[];
}

export const DEFAULT_EDITS: Edits = {
  frame: { padding: 0.06, radius: 14, background: "midnight", shadow: true },
  cursor: { size: 1.6, smoothing: 0.35, ripple: true },
  keys: { show: true },
  camera: { show: true, size: 0.22, corner: "br", shape: "circle" },
  trim: { in_ms: 0, out_ms: null },
  zooms: [],
};

/// Fills in whatever an older project.json lacks.
export function withDefaults(e: Partial<Edits> | undefined): Edits {
  const d = DEFAULT_EDITS;
  return {
    frame: { ...d.frame, ...(e?.frame ?? {}) },
    cursor: { ...d.cursor, ...(e?.cursor ?? {}) },
    keys: { ...d.keys, ...(e?.keys ?? {}) },
    camera: { ...d.camera, ...(e?.camera ?? {}) },
    trim: { ...d.trim, ...(e?.trim ?? {}) },
    zooms: e?.zooms ?? [],
  };
}

export const BACKGROUNDS: Record<Background, [string, string]> = {
  midnight: ["#141a2b", "#2a1f4d"],
  sunset: ["#3a1c3f", "#c2503a"],
  ocean: ["#0d2b3e", "#1e6f8c"],
  slate: ["#2b2f36", "#4a515b"],
  plain: ["#1b1e23", "#1b1e23"],
};
