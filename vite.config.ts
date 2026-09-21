import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import { resolve } from "node:path";

// Multiple HTML entries: one per native window (main app, overlay, HUD, region picker).
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  resolve: { alias: { "@": resolve(__dirname, "src") } },
  server: { port: 1420, strictPort: true, watch: { ignored: ["**/src-tauri/**"] } },
  build: {
    target: "chrome120",
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        overlay: resolve(__dirname, "overlay.html"),
        hud: resolve(__dirname, "hud.html"),
        region: resolve(__dirname, "region.html"),
      },
    },
  },
  test: { environment: "node", include: ["src/**/*.test.ts"] },
});
