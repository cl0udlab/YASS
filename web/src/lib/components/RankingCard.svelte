<script lang="ts">
  import { api } from "#lib/api.ts";
  import { formatBytes } from "#lib/format.ts";
  import type { RankMetric, RankRow } from "#lib/types.ts";

  let { refreshKey = 0 }: { refreshKey?: number } = $props();

  let metric = $state<RankMetric>("cpu");
  let limit = $state(5);
  let rows = $state<RankRow[] | null>(null);
  let failed = $state(false);

  const isTraffic = $derived(["rx", "tx", "traffic"].includes(metric));
  const max = $derived(Math.max(1, ...(rows ?? []).map((r) => r.value)));

  $effect(() => {
    void refreshKey;
    const m = metric;
    const l = limit;
    api<{ rankings: RankRow[] }>(`/api/rankings?metric=${m}&limit=${l}`)
      .then((data) => {
        if (m !== metric || l !== limit) return;
        rows = data.rankings;
        failed = false;
      })
      .catch(() => (failed = true));
  });
</script>

<div class="chart-card">
  <div class="chart-toolbar">
    <h3>排行</h3>
    <div class="range-group">
      <select bind:value={metric} aria-label="排行指標">
        <option value="cpu">CPU 使用率</option>
        <option value="mem">記憶體使用率</option>
        <option value="disk">磁碟使用率</option>
        <option value="traffic">30 天總流量</option>
        <option value="rx">30 天下載</option>
        <option value="tx">30 天上傳</option>
      </select>
      <select bind:value={limit} aria-label="顯示筆數">
        {#each [5, 10, 15, 20] as n (n)}
          <option value={n}>Top {n}</option>
        {/each}
      </select>
    </div>
  </div>

  {#if failed}
    <div class="chart-empty">載入排行失敗</div>
  {:else if rows === null}
    <div class="chart-empty">載入中...</div>
  {:else if rows.length === 0}
    <div class="chart-empty">尚無資料</div>
  {:else}
    {#each rows as row, index (row.id)}
      <div class="rank-row">
        <div class="rank-index">{index + 1}</div>
        <div>
          <div>
            {row.name}{#if !row.online}<span class="offline"> (離線)</span>{/if}
          </div>
          <div
            class="rank-bar"
            style:width="{((row.value / max) * 100).toFixed(1)}%"
          ></div>
        </div>
        <div class="rank-value">
          {isTraffic ? formatBytes(row.value) : `${row.value.toFixed(1)}%`}
        </div>
      </div>
    {/each}
  {/if}
</div>

<style>
  .offline {
    opacity: 0.5;
  }
</style>
