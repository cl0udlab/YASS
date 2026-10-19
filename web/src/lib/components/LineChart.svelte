<script lang="ts" module>
  export interface Series {
    label: string;
    color: string;
    values: { ts: number; value: number }[];
  }
</script>

<script lang="ts">
  let {
    series,
    title,
    yMax: fixedYMax,
    formatY,
  }: {
    series: Series[];
    title: string;
    yMax?: number;
    formatY: (value: number) => string;
  } = $props();

  const W = 800;
  const H = 200;
  const L = 64;
  const R = 12;
  const T = 12;
  const B = 26;

  const empty = $derived(!series[0]?.values.length);

  const yMax = $derived(
    fixedYMax ??
      Math.max(1, ...series.flatMap((s) => s.values.map((v) => v.value))) *
        1.15,
  );
  const xMin = $derived(series[0]?.values[0]?.ts ?? 0);
  const xSpan = $derived(
    Math.max(1, (series[0]?.values.at(-1)?.ts ?? 0) - xMin),
  );

  const px = (ts: number) => L + ((ts - xMin) / xSpan) * (W - L - R);
  const py = (value: number) =>
    T + (1 - Math.min(value, yMax) / yMax) * (H - T - B);

  function timeLabel(ts: number) {
    const d = new Date(ts * 1000);
    return xSpan > 3 * 86400
      ? `${d.getMonth() + 1}/${d.getDate()}`
      : d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }

  const anchors = ["start", "middle", "end"] as const;
</script>

{#if empty}
  <div class="chart-empty">此區間尚無資料</div>
{:else}
  <svg class="chart-svg" viewBox="0 0 {W} {H}" role="img" aria-label={title}>
    {#each [0, 0.25, 0.5, 0.75, 1] as frac (frac)}
      {@const y = py(yMax * frac)}
      <line
        x1={L}
        y1={y}
        x2={W - R}
        y2={y}
        stroke="currentColor"
        stroke-opacity="0.15"
        stroke-width="1"
        vector-effect="non-scaling-stroke"
      />
      <text
        x={L - 8}
        y={y + 4}
        text-anchor="end"
        font-size="11"
        fill="currentColor"
        fill-opacity="0.6">{formatY(yMax * frac)}</text
      >
    {/each}

    {#each series as s (s.label)}
      <polyline
        points={s.values
          .map((v) => `${px(v.ts).toFixed(1)},${py(v.value).toFixed(1)}`)
          .join(" ")}
        fill="none"
        stroke={s.color}
        stroke-width="1.8"
        stroke-linejoin="round"
        stroke-linecap="round"
        vector-effect="non-scaling-stroke"
      />
    {/each}

    {#each [0, 0.5, 1] as frac, i (frac)}
      <text
        x={px(xMin + xSpan * frac)}
        y={H - 6}
        text-anchor={anchors[i]}
        font-size="11"
        fill="currentColor"
        fill-opacity="0.6">{timeLabel(xMin + xSpan * frac)}</text
      >
    {/each}
  </svg>
  <div class="chart-legend">
    {#each series as s (s.label)}
      <span style:color={s.color}>{s.label}</span>
    {/each}
  </div>
{/if}

<style>
  .chart-svg {
    width: 100%;
    height: auto;
    display: block;
  }
</style>
