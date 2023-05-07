import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [solidPlugin(), wasm(), topLevelAwait()],
  optimizeDeps: {
    exclude: ["tools_wasm"],
  },
  server: {
    port: 3000,
  },
  build: {
    target: "esnext",
  },
});
