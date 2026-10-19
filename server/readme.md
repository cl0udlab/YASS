## 開發



```bash
# 127.0.0.1:9000
RUST_LOG=info cargo run -p server
$env:RUST_LOG="info"; cargo run -p server

# http://localhost:5173
cd web && pnpm install && pnpm dev
```

## 部署

```bash
cd web && pnpm build
cargo build --release -p server
```


| 環境變數                                  | 預設             | 說明                                           |
| ----------------------------------------- | ---------------- | ---------------------------------------------- |
| `YASS_BIND`                               | `127.0.0.1:9000` | 監聽位址                                       |
| `YASS_WEB_DIR`                            | `web/build`      | 前端建置產物目錄                               |
| `YASS_BIN_DIR`                            | `bin`            | client 執行檔目錄，檔名為 `yass-client-<arch>` |
| `YASS_ADMIN_USER` / `YASS_ADMIN_PASSWORD` | `admin` / 隨機   | 首次啟動建立的管理員帳號                       |
