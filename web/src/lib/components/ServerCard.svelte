<script lang="ts">
  import {
    cpuUsage,
    displayName,
    formatTime,
    memoryUsage,
  } from "#lib/format.ts";
  import type { Server } from "#lib/types.ts";

  let {
    server,
    oninstall,
    ondelete,
  }: { server: Server; oninstall: () => void; ondelete: () => void } = $props();
</script>

<div class="card server-card">
  <div class="server-header">
    <h3>{displayName(server)}</h3>
    <span
      class="status-indicator"
      class:online={server.online}
      title={server.online ? "線上" : "離線"}
    ></span>
  </div>
  <div class="server-details">
    <p>
      <strong>OS:</strong>
      {server.system_info?.os_name || "Unknown"}
      {server.system_info?.os_version || ""}
    </p>
    <p><strong>記憶體使用:</strong> {Math.round(memoryUsage(server))}%</p>
    <p><strong>CPU 使用率:</strong> {Math.round(cpuUsage(server))}%</p>
    <p><strong>最後更新:</strong> {formatTime(server.last_seen)}</p>
  </div>
  <div class="server-actions">
    <a class="btn btn-primary btn-sm" href="/servers/{server.id}">查看詳情</a>
    <button class="btn btn-secondary btn-sm" onclick={oninstall}
      >安裝指令</button
    >
    <button class="btn btn-danger btn-sm" onclick={ondelete}>刪除</button>
  </div>
</div>

<style>
  .server-card {
    transition:
      transform 0.3s,
      box-shadow 0.3s;
  }

  .server-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px var(--shadow-color);
  }

  .server-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .status-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    display: inline-block;
    background-color: var(--danger-color);
  }

  .status-indicator.online {
    background-color: var(--success-color);
  }

  .server-actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-top: 0.75rem;
  }
</style>
