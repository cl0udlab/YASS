# YASS web

YASS 面板前端：SvelteKit（Svelte 5 + TypeScript）以 `adapter-static` 建成純 SPA，資料全部來自後端 `/api`。

- `pnpm dev`：開發伺服器，API 轉給 `YASS_BACKEND`（預設 `http://127.0.0.1:9000`）
- `pnpm build`：輸出到 `build/`，由 Rust 後端送出
- `pnpm check`：型別檢查

開發與部署流程見 [server/readme.md](../server/readme.md)。
