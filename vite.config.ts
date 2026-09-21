import { defineConfig } from "vite";

// Vite serves the settings window while `npm run tauri dev` is running, and builds it into dist/
// for a packaged application. src-tauri/tauri.conf.json names both this port and that folder, so
// a change here is a change there as well.
export default defineConfig({
  // A fixed port, and a failure rather than a silent move to the next one: Tauri waits for
  // exactly this address, and a dev server that quietly moved to 1421 leaves it waiting.
  server: {
    port: 1420,
    strictPort: true,
  },

  build: {
    outDir: "dist",
    emptyOutDir: true,
    // Two pages, not one: the settings window and the overlay that covers a monitor while a shot
    // is being taken. Without naming both here, only index.html is built and the overlay window
    // opens on nothing.
    rollupOptions: {
      input: {
        main: "index.html",
        overlay: "overlay.html",
      },
    },
    // The window runs on WebView2 on Windows and WKWebView on macOS, both of which are current
    // browsers. There is no old engine to compile down for.
    target: "es2022",
    // A packaged application has no source maps to fetch, and shipping them would publish the
    // source of the window with it.
    sourcemap: false,
  },

  // Tauri reports build failures through this process, so silence would hide them.
  clearScreen: false,
});
