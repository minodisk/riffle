import { resolve } from "node:path";
import { defineConfig } from "vite-plus";

const root = "crates/app/ui";

export default defineConfig({
  root,
  clearScreen: false,
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/crates/*/src/**", "**/target/**"] },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rollupOptions: {
      input: {
        main: resolve(root, "index.html"),
        settings: resolve(root, "settings.html"),
      },
    },
  },
  fmt: {
    ignorePatterns: [
      "**/*.md",
      ".release-please-manifest.json",
      "release-please-config.json",
      "crates/app/capabilities/**",
      "crates/app/tauri.conf.json",
      "target/**",
      "crates/app/ui/dist/**",
      ".claude/**",
    ],
  },
  lint: {
    ignorePatterns: ["crates/app/ui/dist/**", ".claude/**"],
    options: { typeAware: true, typeCheck: true },
  },
});
