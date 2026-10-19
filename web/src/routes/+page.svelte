<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "#lib/api.ts";
  import { showAlert } from "#lib/alerts.svelte.ts";
  import { displayName, formatTime } from "#lib/format.ts";
  import type { RegistrationInfo, Server } from "#lib/types.ts";
  import AddServerModal from "#lib/components/AddServerModal.svelte";
  import InstallModal from "#lib/components/InstallModal.svelte";
  import RankingCard from "#lib/components/RankingCard.svelte";
  import ServerCard from "#lib/components/ServerCard.svelte";

  const REFRESH_MS = 30_000;

  let servers = $state<Server[] | null>(null);
  let refreshKey = $state(0);
  let adding = $state(false);
  let install = $state<{ info: RegistrationInfo; isNew: boolean } | null>(null);

  const onlineCount = $derived(servers?.filter((s) => s.online).length ?? 0);
  const lastUpdate = $derived(
    servers?.length
      ? formatTime(
          Math.max(...servers.map((s) => new Date(s.last_seen).getTime())),
        )
      : "無資料",
  );

  async function load() {
    try {
      const data = await api<{ servers: Record<string, Server> }>(
        "/api/servers",
      );
      servers = Object.values(data.servers ?? {});
    } catch (error) {
      showAlert("載入資料失敗: " + (error as Error).message, "error");
    }
  }

  onMount(() => {
    load();
    const timer = setInterval(load, REFRESH_MS);
    const onVisible = () => !document.hidden && load();
    document.addEventListener("visibilitychange", onVisible);
    return () => {
      clearInterval(timer);
      document.removeEventListener("visibilitychange", onVisible);
    };
  });

  async function showInstall(server: Server) {
    try {
      install = {
        info: await api<RegistrationInfo>(`/api/servers/${server.id}/install`),
        isNew: false,
      };
    } catch (error) {
      showAlert("取得安裝指令失敗: " + (error as Error).message, "error");
    }
  }

  async function remove(server: Server) {
    if (!confirm(`確定要刪除「${displayName(server)}」嗎？此動作無法復原。`))
      return;
    try {
      await api(`/api/servers/${server.id}`, { method: "DELETE" });
      showAlert("伺服器已刪除", "success");
      await load();
      refreshKey++;
    } catch (error) {
      showAlert("刪除伺服器失敗: " + (error as Error).message, "error");
    }
  }
</script>

<div class="stats-grid">
  <div class="card">
    <h3>線上伺服器</h3>
    <div class="stat-number">{onlineCount}</div>
  </div>
  <div class="card">
    <h3>總伺服器數</h3>
    <div class="stat-number">{servers?.length ?? 0}</div>
  </div>
  <div class="card">
    <h3>最近更新</h3>
    <div class="stat-text">{lastUpdate}</div>
  </div>
</div>

<RankingCard {refreshKey} />

<section>
  <div class="section-header">
    <h2>伺服器概覽</h2>
    <button class="btn btn-primary" onclick={() => (adding = true)}
      >+ 新增伺服器</button
    >
  </div>
  <div class="server-list">
    {#if servers === null}
      <div class="loading">載入中...</div>
    {:else if servers.length === 0}
      <div class="loading">尚無伺服器資料</div>
    {:else}
      {#each servers as server (server.id)}
        <ServerCard
          {server}
          oninstall={() => showInstall(server)}
          ondelete={() => remove(server)}
        />
      {/each}
    {/if}
  </div>
</section>

{#if adding}
  <AddServerModal
    onclose={() => (adding = false)}
    oncreated={(info) => {
      adding = false;
      install = { info, isNew: true };
      load();
    }}
  />
{/if}

{#if install}
  <InstallModal
    info={install.info}
    isNew={install.isNew}
    onclose={() => (install = null)}
  />
{/if}

<style>
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .stat-number {
    font-size: 2.5rem;
    font-weight: bold;
    color: var(--primary-color);
  }

  .stat-text {
    font-size: 1.125rem;
    color: var(--text-secondary);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }

  .server-list {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
    gap: 1.5rem;
  }

  @media (max-width: 768px) {
    .stats-grid,
    .server-list {
      grid-template-columns: 1fr;
    }
  }
</style>
