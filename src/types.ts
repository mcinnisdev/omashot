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

export interface Shot {
  id: string;
  file: string;
  abs_path: string;
  note: string;
  width: number;
  height: number;
  captured_at: string;
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
  groups: Group[];
}

export interface Export {
  root: string;
  markdown: string;
  groups: number;
  shots: number;
}

export interface AppState {
  session: Session | null;
  last_export: Export | null;
  dirty: boolean;
}
