#!/usr/bin/env bash
set -euo pipefail

PANEL_URL=""
SERVER_ID=""
TOKEN=""
INTERVAL="60"
REPO="cl0udlab/YASS"
VERSION="latest"

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
    --repo)     REPO="${2:-}"; shift 2 ;;
    --version)  VERSION="${2:-}"; shift 2 ;;
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

ASSET="yass-client-${ARCH}"
if [ "$VERSION" = "latest" ]; then
  RELEASE_URL="https://github.com/${REPO}/releases/latest/download"
else
  RELEASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
fi

TMP_BIN="$(mktemp)"
TMP_SUMS="$(mktemp)"
trap 'rm -f "$TMP_BIN" "$TMP_SUMS"' EXIT

info "從 GitHub ${REPO} (${VERSION}) 下載 ${ASSET}"
curl -fsSL -o "$TMP_BIN" "${RELEASE_URL}/${ASSET}" \
  || die "下載失敗：${RELEASE_URL}/${ASSET}"
[ -s "$TMP_BIN" ] || die "下載到的檔案是空的"

if command -v sha256sum >/dev/null 2>&1; then
  curl -fsSL -o "$TMP_SUMS" "${RELEASE_URL}/SHA256SUMS" || die "無法取得 SHA256SUMS"
  expected="$(awk -v f="$ASSET" '$2 == f || $2 == "*"f { print $1 }' "$TMP_SUMS")"
  [ -n "$expected" ] || die "SHA256SUMS 裡沒有 ${ASSET}"
  actual="$(sha256sum "$TMP_BIN" | awk '{ print $1 }')"
  [ "$expected" = "$actual" ] || die "${ASSET} 校驗失敗"
  info "SHA256 校驗通過"
else
  info "找不到 sha256sum，略過校驗"
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
