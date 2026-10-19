#!/usr/bin/env bash
set -euo pipefail

PANEL_URL=""
SERVER_ID=""
TOKEN=""
INTERVAL="60"

BIN_PATH="/usr/local/bin/yass-client"
CONF_DIR="/etc/yass"
CONF_PATH="${CONF_DIR}/client.env"
UNIT_PATH="/etc/systemd/system/yass-client.service"

die() { echo "錯誤: $*" >&2; exit 1; }
info() { echo "==> $*"; }

while [ $# -gt 0 ]; do
  case "$1" in
    --url)      PANEL_URL="${2:-}"; shift 2 ;;
    --id)       SERVER_ID="${2:-}"; shift 2 ;;
    --token)    TOKEN="${2:-}"; shift 2 ;;
    --interval) INTERVAL="${2:-}"; shift 2 ;;
    *) die "未知參數: $1" ;;
  esac
done

[ -n "$PANEL_URL" ] || die "缺少 --url"
[ -n "$SERVER_ID" ] || die "缺少 --id"
[ -n "$TOKEN" ]     || die "缺少 --token"
[ "$(id -u)" = "0" ] || die "請以 root 執行（在指令前加 sudo）"

command -v systemctl >/dev/null 2>&1 || die "這台機器沒有 systemd"
command -v curl >/dev/null 2>&1 || die "需要 curl"

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|aarch64|armv7l) ;;
  *) die "不支援的架構: ${ARCH}" ;;
esac

TMP_BIN="$(mktemp)"
trap 'rm -f "$TMP_BIN"' EXIT

info "從 ${PANEL_URL} 下載 ${ARCH} client"
curl -fsSL -o "$TMP_BIN" "${PANEL_URL}/download/client?arch=${ARCH}" \
  || die "下載失敗，請確認面板的 bin/ 目錄裡有 yass-client-${ARCH}"

[ -s "$TMP_BIN" ] || die "下載到的檔案是空的"

if head -c 1 "$TMP_BIN" | grep -q '{'; then
  cat "$TMP_BIN" >&2
  die "面板沒有提供 ${ARCH} 的 client 二進位"
fi

info "安裝到 ${BIN_PATH}"
install -m 0755 "$TMP_BIN" "$BIN_PATH"

info "寫入設定 ${CONF_PATH}"
mkdir -p "$CONF_DIR"
cat > "$CONF_PATH" <<EOF
YASS_URL=${PANEL_URL}
YASS_SERVER_ID=${SERVER_ID}
YASS_TOKEN=${TOKEN}
YASS_INTERVAL=${INTERVAL}
EOF
chmod 0600 "$CONF_PATH"

info "寫入 systemd unit ${UNIT_PATH}"
cat > "$UNIT_PATH" <<EOF
[Unit]
Description=YASS monitoring client
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=${CONF_PATH}
ExecStart=${BIN_PATH}
Restart=always
RestartSec=10

NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=read-only
PrivateTmp=yes

[Install]
WantedBy=multi-user.target
EOF

info "啟動服務"
systemctl daemon-reload
systemctl enable --now yass-client.service

sleep 2
if systemctl is-active --quiet yass-client.service; then
  info "完成，client 每 ${INTERVAL} 秒回報一次"
  info "查看紀錄: journalctl -u yass-client -f"
else
  journalctl -u yass-client -n 30 --no-pager >&2 || true
  die "服務啟動失敗"
fi
