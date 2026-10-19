import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

// 開發時把 API 轉給本機的 Rust 面板（cargo run 預設監聽 127.0.0.1:9000）
const backend = process.env.YASS_BACKEND ?? "http://127.0.0.1:9000";

export default defineConfig({
  plugins: [
    sveltekit({
      compilerOptions: {
        // Force runes mode for the project, except for libraries. Can be removed in svelte 6.
        runes: ({ filename }) =>
          filename.split(/[/\\]/).includes("node_modules") ? undefined : true,
      },

      // 純 SPA：所有路由都由 200.html 接手，後端對未知路徑回退到這個檔案
      adapter: adapter({ fallback: "200.html" }),
    }),
  ],
  server: {
    proxy: {
      "/api": backend,
      "/install.sh": backend,
      "/download": backend,
    },
  },
});
