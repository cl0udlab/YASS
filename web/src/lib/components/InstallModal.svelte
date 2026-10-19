<script lang="ts">
  import { api } from "#lib/api.ts";
  import { showAlert } from "#lib/alerts.svelte.ts";
  import type { RegistrationInfo } from "#lib/types.ts";
  import Modal from "./Modal.svelte";

  let {
    info: initialInfo,
    isNew = false,
    onclose,
  }: {
    info: RegistrationInfo;
    isNew?: boolean;
    onclose: () => void;
  } = $props();

  let info = $derived(initialInfo);

  async function copy() {
    try {
      await navigator.clipboard.writeText(info.install_command);
      showAlert("已複製到剪貼簿", "success");
    } catch (error) {
      showAlert("複製失敗: " + (error as Error).message, "error");
    }
  }

  async function resetToken() {
    if (
      !confirm(
        "重置後舊的 token 會立刻失效，該機器上的 client 必須重新執行安裝指令才能繼續回報。確定要重置嗎？",
      )
    )
      return;

    try {
      info = await api<RegistrationInfo>(
        `/api/servers/${info.id}/reset_token`,
        { method: "POST" },
      );
      isNew = false;
      showAlert("Token 已重置", "success");
    } catch (error) {
      showAlert("重置 Token 失敗: " + (error as Error).message, "error");
    }
  }
</script>

<Modal {onclose}>
  <h3>{isNew ? "已建立" : "安裝指令"}：{info.name}</h3>
  <p class="hint">在目標機器上以 root 執行這一行：</p>
  <div class="command-box">{info.install_command}</div>
  <p class="hint">
    腳本會下載對應架構的 client、把 ID 與 token 寫進
    <code>/etc/yass/client.env</code>，再註冊成 systemd 服務
    <code>yass-client</code> 並啟動。首次回報後這張卡片就會轉為線上。
    <br />
    伺服器 ID：<code>{info.id}</code>
  </p>
  <div class="modal-actions">
    <button class="btn btn-warning" onclick={resetToken}>重置 Token</button>
    <button class="btn btn-secondary" onclick={copy}>複製</button>
    <button class="btn btn-primary" onclick={onclose}>完成</button>
  </div>
</Modal>

<style>
  .command-box {
    background-color: var(--bg-color);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    padding: 0.75rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 0.8rem;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-all;
    margin: 0.5rem 0;
  }
</style>
