import { defineConfig } from "vite";
import { resolve } from "node:path";

export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    // Cargo writes into src-tauri/target while Vite is running; watching it
    // trips EBUSY on Windows and floods the watcher everywhere else.
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    target: "es2021",
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        capture: resolve(__dirname, "capture.html"),
        note: resolve(__dirname, "note.html"),
        peek: resolve(__dirname, "peek.html"),
      },
    },
  },
});
