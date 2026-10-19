<script lang="ts">
  import { page } from "$app/state";
  import { api } from "#lib/api.ts";
  import { showAlert } from "#lib/alerts.svelte.ts";
  import {
    SERIES_COLORS,
    cpuUsage,
    formatBytes,
    formatTime,
    formatUptime,
    memoryUsage,
  } from "#lib/format.ts";
  import type {
    HistoryPoint,
    HistoryRange,
    Server,
    TrafficResponse,
  } from "#lib/types.ts";
  import LineChart from "#lib/components/LineChart.svelte";

  const RANGES: { value: HistoryRange; label: string }[] = [
    { value: "1h", label: "1 小時" },
    { value: "6h", label: "6 小時" },
    { value: "24h", label: "24 小時" },
    { value: "7d", label: "7 天" },
    { value: "30d", label: "30 天" },
  ];

  const id = $derived(page.params.id!);

  let server = $state<Server | null>(null);
  let range = $state<HistoryRange>("1h");
  let history = $state<HistoryPoint[] | null>(null);
  let historyFailed = $state(false);
  let traffic = $state<TrafficResponse | null>(null);
  let trafficFailed = $state(false);

  $effect(() => {
    const sid = id;
    server = null;
    api<Server>(`/api/servers/${sid}`)
      .then((data) => sid === id && (server = data))
      .catch((error) =>
        showAlert("載入伺服器詳情失敗: " + error.message, "error"),
      );

    traffic = null;
    trafficFailed = false;
    api<TrafficResponse>(`/api/servers/${sid}/traffic?days=30`)
      .then((data) => sid === id && (traffic = data))
      .catch(() => (trafficFailed = true));
  });

  $effect(() => {
    const sid = id;
    const r = range;
    history = null;
    historyFailed = false;
    api<{ points: HistoryPoint[] }>(`/api/servers/${sid}/history?range=${r}`)
      .then((data) => sid === id && r === range && (history = data.points))
      .catch(() => (historyFailed = true));
  });

  const series = (key: keyof Omit<HistoryPoint, "ts">) =>
    (history ?? []).map((p) => ({ ts: p.ts, value: p[key] }));

  const trafficMax = $derived(
    Math.max(1, ...(traffic?.days ?? []).map((d) => d.rx_bytes + d.tx_bytes)),
  );
  const activeInterfaces = $derived(
    server?.network_info?.network_info.filter(
      (n) => n.received_bytes > 0 || n.transmitted_bytes > 0,
    ) ?? [],
  );
  const pct = (part: number, whole: number) =>
    whole ? Math.round((part / whole) * 100) : 0;
</script>

<div class="page-header">
  <h1>{server ? `${server.pc_name} - 詳情` : "伺服器詳情"}</h1>
  <a class="btn btn-secondary btn-sm" href="/">← 返回</a>
</div>

<div class="chart-card">
  <div class="chart-toolbar">
    <h3>歷史趨勢</h3>
    <div class="range-group">
      {#each RANGES as r (r.value)}
        <button
          class="range-btn"
          class:active={range === r.value}
          onclick={() => (range = r.value)}
        >
          {r.label}
        </button>
      {/each}
    </div>
  </div>
  {#if historyFailed}
    <div class="chart-empty">載入歷史資料失敗</div>
  {:else if history === null}
    <div class="chart-empty">載入中...</div>
  {:else}
    <LineChart
      title="使用率"
      yMax={100}
      formatY={(v) => `${Math.round(v)}%`}
      series={[
        { label: "CPU", color: SERIES_COLORS.cpu, values: series("cpu") },
        { label: "記憶體", color: SERIES_COLORS.mem, values: series("mem") },
        { label: "磁碟", color: SERIES_COLORS.disk, values: series("disk") },
      ]}
    />
    <div class="spacer"></div>
    <LineChart
      title="網路速率"
      formatY={(v) => `${formatBytes(v)}/s`}
      series={[
        { label: "下載", color: SERIES_COLORS.rx, values: series("rx_rate") },
        { label: "上傳", color: SERIES_COLORS.tx, values: series("tx_rate") },
      ]}
    />
  {/if}
</div>

<div class="chart-card">
  <div class="chart-toolbar">
    <h3>近 30 天流量</h3>
    {#if traffic}
      <div class="chart-legend">
        <span style:color={SERIES_COLORS.rx}
          >下載 {formatBytes(traffic.total_rx)}</span
        >
        <span style:color={SERIES_COLORS.tx}
          >上傳 {formatBytes(traffic.total_tx)}</span
        >
      </div>
    {/if}
  </div>
  {#if trafficFailed}
    <div class="chart-empty">載入流量失敗</div>
  {:else if traffic === null}
    <div class="chart-empty">載入中...</div>
  {:else if traffic.days.length === 0}
    <div class="chart-empty">尚無流量資料</div>
  {:else}
    {#each traffic.days as day (day.date)}
      <div class="rank-row">
        <div class="rank-index">{day.date.slice(5)}</div>
        <div>
          <div
            class="rank-bar"
            style:width="{((day.rx_bytes / trafficMax) * 100).toFixed(1)}%"
            style:background-color={SERIES_COLORS.rx}
          ></div>
          <div
            class="rank-bar"
            style:width="{((day.tx_bytes / trafficMax) * 100).toFixed(1)}%"
            style:background-color={SERIES_COLORS.tx}
          ></div>
        </div>
        <div class="rank-value">{formatBytes(day.rx_bytes + day.tx_bytes)}</div>
      </div>
    {/each}
  {/if}
</div>

{#if server === null}
  <div class="loading">載入中...</div>
{:else}
  {@const sys = server.system_info}
  {@const mem = server.memory_info}
  <div class="detail-grid">
    <section class="card detail-section">
      <h3>系統資訊</h3>
      <dl class="info-list">
        <div class="info-item">
          <dt>電腦名稱:</dt>
          <dd>{server.pc_name}</dd>
        </div>
        <div class="info-item">
          <dt>IP 位址:</dt>
          <dd>{server.ip_address || "未知"}</dd>
        </div>
        <div class="info-item">
          <dt>作業系統:</dt>
          <dd>{sys?.os_name || "Unknown"} {sys?.os_version || ""}</dd>
        </div>
        <div class="info-item">
          <dt>核心版本:</dt>
          <dd>{sys?.kernel_version || "Unknown"}</dd>
        </div>
        <div class="info-item">
          <dt>CPU:</dt>
          <dd>{sys?.cpu_name || "Unknown"}</dd>
        </div>
        <div class="info-item">
          <dt>運行時間:</dt>
          <dd>{formatUptime(sys?.uptime)}</dd>
        </div>
        <div class="info-item">
          <dt>進程數:</dt>
          <dd>{sys?.process_count || 0}</dd>
        </div>
        <div class="info-item">
          <dt>狀態:</dt>
          <dd>
            <span
              class="status-badge"
              class:online={server.online}
              class:offline={!server.online}
            >
              {server.online ? "線上" : "離線"}
            </span>
          </dd>
        </div>
        <div class="info-item">
          <dt>最後更新:</dt>
          <dd>{formatTime(server.last_seen)}</dd>
        </div>
      </dl>
    </section>

    <section class="card detail-section">
      <h3>記憶體使用</h3>
      <dl class="info-list">
        <div class="info-item">
          <dt>總記憶體:</dt>
          <dd>{formatBytes(mem?.total_memory)}</dd>
        </div>
        <div class="info-item">
          <dt>已使用:</dt>
          <dd>
            {formatBytes(mem?.used_memory)} ({Math.round(memoryUsage(server))}%)
          </dd>
        </div>
        <div class="info-item">
          <dt>可用記憶體:</dt>
          <dd>
            {formatBytes((mem?.total_memory ?? 0) - (mem?.used_memory ?? 0))}
          </dd>
        </div>
        <div class="info-item">
          <dt>總交換空間:</dt>
          <dd>{formatBytes(mem?.total_swap)}</dd>
        </div>
        <div class="info-item">
          <dt>已使用交換空間:</dt>
          <dd>{formatBytes(mem?.used_swap)}</dd>
        </div>
      </dl>
    </section>

    {#if server.cpu_info?.cpu_info.length}
      <section class="card detail-section">
        <h3>CPU 使用率</h3>
        <dl class="info-list">
          <div class="info-item">
            <dt>平均使用率:</dt>
            <dd>{cpuUsage(server).toFixed(2)}%</dd>
          </div>
          {#each server.cpu_info.cpu_info as cpu, i (i)}
            <div class="info-item">
              <dt>{cpu.cpu_name}:</dt>
              <dd>{cpu.cpu_usage.toFixed(2)}%</dd>
            </div>
          {/each}
        </dl>
      </section>
    {/if}

    {#if server.disks_info?.disksinfo.length}
      <section class="card detail-section">
        <h3>硬碟使用</h3>
        <dl class="info-list">
          {#each server.disks_info.disksinfo as disk, i (i)}
            <div class="info-item">
              <dt>{disk.name} ({disk.file_system}):</dt>
              <dd>
                {formatBytes(disk.used_space)} / {formatBytes(disk.total_space)}
                ({pct(disk.used_space, disk.total_space)}%)
              </dd>
            </div>
            <div class="info-item">
              <dt>可用空間:</dt>
              <dd>{formatBytes(disk.available_space)}</dd>
            </div>
          {/each}
        </dl>
      </section>
    {/if}

    {#if activeInterfaces.length}
      <section class="card detail-section">
        <h3>網路介面 (有流量)</h3>
        <dl class="info-list">
          {#each activeInterfaces as net (net.interface_name)}
            <div class="info-item">
              <dt>{net.interface_name}:</dt>
              <dd>
                接收: {formatBytes(net.received_bytes)}, 傳送: {formatBytes(
                  net.transmitted_bytes,
                )}
              </dd>
            </div>
          {/each}
        </dl>
      </section>
    {/if}
  </div>
{/if}

<style>
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 2px solid var(--border-color);
  }

  .range-btn {
    padding: 0.25rem 0.6rem;
    border: 1px solid var(--border-color);
    background: transparent;
    color: var(--text-secondary);
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.8rem;
  }

  .range-btn.active {
    background-color: var(--primary-color);
    border-color: var(--primary-color);
    color: #fff;
  }

  .spacer {
    height: 1rem;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: 2rem;
  }

  .detail-section h3 {
    margin-bottom: 1rem;
    color: var(--primary-color);
    border-bottom: 2px solid var(--primary-color);
    padding-bottom: 0.5rem;
  }

  .info-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .info-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem;
    background-color: var(--item-bg);
    border-radius: 4px;
  }

  .info-item dt {
    font-weight: bold;
    color: var(--text-secondary);
  }

  .info-item dd {
    text-align: right;
    overflow-wrap: anywhere;
  }

  @media (max-width: 768px) {
    .page-header {
      flex-direction: column;
      align-items: stretch;
    }

    .detail-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
