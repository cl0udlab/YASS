<script lang="ts">
  import { api } from "#lib/api.ts";
  import { showAlert } from "#lib/alerts.svelte.ts";
  import type { RegistrationInfo } from "#lib/types.ts";
  import Modal from "./Modal.svelte";

  let {
    oncreated,
    onclose,
  }: { oncreated: (info: RegistrationInfo) => void; onclose: () => void } =
    $props();

  let name = $state("");
  let submitting = $state(false);

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) {
      showAlert("請輸入名稱", "warning");
      return;
    }

    submitting = true;
    try {
      oncreated(
        await api<RegistrationInfo>("/api/servers", {
          method: "POST",
          json: { name: trimmed },
        }),
      );
    } catch (error) {
      showAlert("建立伺服器失敗: " + (error as Error).message, "error");
    } finally {
      submitting = false;
    }
  }
</script>

<Modal {onclose}>
  <h3>新增伺服器</h3>
  <p class="hint">
    取一個好認的名稱。面板會配發專屬的 ID 與 token，並產生一行安裝指令，
    到目標機器上貼著執行即可完成安裝與設定。
  </p>
  <form onsubmit={submit}>
    <div class="form-group">
      <label for="new-server-name">名稱</label>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        id="new-server-name"
        type="text"
        placeholder="例如 hk-node-01"
        bind:value={name}
        autofocus
      />
    </div>
    <div class="modal-actions">
      <button type="button" class="btn btn-secondary" onclick={onclose}
        >取消</button
      >
      <button type="submit" class="btn btn-primary" disabled={submitting}
        >建立並產生指令</button
      >
    </div>
  </form>
</Modal>
