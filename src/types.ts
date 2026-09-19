// Mirrors the serde output of src-tauri/src/{model,capture,export}.rs.

export interface Frame {
  monitor_id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  scale: number;
  png_path: string;
}

export type ShotKind = "image" | "recording";
export type Purpose = "fix" | "document" | "custom";
export type DocFormat = "markdown" | "html";

export interface KeyFrame {
  file: string;
  at_ms: number;
  event: string;
  x: number | null;
  y: number | null;
}

export interface Shot {
  id: string;
  file: string;
  abs_path: string;
  title: string;
  note: string;
  width: number;
  height: number;
  captured_at: string;
  kind: ShotKind;
  duration_ms: number;
  frames: KeyFrame[];
  video: string | null;
  /** Set on a still that auto-capture took: when, and on what action. */
  moment: { at_ms: number; event: string; x: number | null; y: number | null } | null;
}

export interface Group {
  index: number;
  title: string;
  master_note: string;
  dir: string;
  shots: Shot[];
}

export interface Session {
  id: string;
  name: string;
  started_at: string;
  root: string;
  current: number;
  purpose: Purpose;
  doc_format: DocFormat;
  include_brand: boolean;
  groups: Group[];
}

export interface BundleInfo {
  path: string;
  id: string;
  name: string;
  started_at: string;
  groups: number;
  shots: number;
}

export interface BrandKit {
  notes: string;
  files: string[];
}

export interface Export {
  root: string;
  markdown: string;
  groups: number;
  shots: number;
  zip_path: string | null;
}

export interface AppState {
  session: Session | null;
  last_export: Export | null;
  dirty: boolean;
  finished: boolean;
  custom_prompt: string;
  brand: BrandKit;
}
