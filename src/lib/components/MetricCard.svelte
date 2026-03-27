<script lang="ts">
  interface Props {
    label: string;
    value: number | string;
    unit?: string;
    sub?: string;
    color?: string;
    loading?: boolean;
    threshold?: { warn: number; danger: number };
  }
  let { label, value, unit = '', sub, color = 'var(--accent)', loading = false, threshold }: Props = $props();

  const numVal = $derived(typeof value === 'number' ? value : parseFloat(String(value)));

  function getStatusClass(v: number): string {
    if (threshold) {
      if (v >= threshold.danger) return 'danger';
      if (v >= threshold.warn) return 'warning';
      return 'ok';
    }
    if (v >= 90) return 'danger';
    if (v >= 70) return 'warning';
    return 'ok';
  }

  const statusClass = $derived(!isNaN(numVal) && unit === '%' ? getStatusClass(numVal) : 'ok');
</script>

<div class="metric-card card">
  {#if loading}
    <div class="skeleton" style="height:20px;width:60%;margin-bottom:8px"></div>
    <div class="skeleton" style="height:36px;width:80%"></div>
  {:else}
    <div class="metric-label">{label}</div>
    <div class="metric-value-row">
      <span class="metric-value status-{statusClass}" style:color={statusClass === 'ok' ? color : undefined}>
        {typeof value === 'number' ? value.toFixed(1) : value}
      </span>
      {#if unit}<span class="metric-unit">{unit}</span>{/if}
    </div>
    {#if sub}<div class="metric-sub">{sub}</div>{/if}
  {/if}
</div>

<style>
  .metric-card { display: flex; flex-direction: column; gap: 0.25rem; }
  .metric-value-row { display: flex; align-items: baseline; gap: 0.25rem; }
  .metric-value { font-size: 1.75rem; font-weight: 700; line-height: 1; letter-spacing: -0.02em; }
  .metric-unit { font-size: 0.9rem; color: var(--text-muted); font-weight: 500; }
  .metric-sub { font-size: 0.72rem; color: var(--text-muted); margin-top: 0.1rem; }
  .status-warning { color: var(--warning) !important; }
  .status-danger  { color: var(--danger) !important; }
</style>
