import path from "node:path";
import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import wasm from "vite-plugin-wasm";
// import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [
    solidPlugin({
      babel: {
        parserOpts: {
          plugins: ["explicitResourceManagement"]
        }
      }
    }),
    wasm(),
    // topLevelAwait()
  ],
  optimizeDeps: {
    exclude: ["tools_wasm"],
  },
  server: {
    port: 3000,
    fs: {
      allow: [process.cwd(), path.resolve(__dirname, "../tools_wasm/pkg")],
    },
  },
  build: {
    target: "esnext",
  },
});
