<script lang="ts">
  import { onDestroy } from 'svelte';
  import {
    activeServer, activeServerId, latestMetrics, metricsHistory,
    connectionStates, connectToServer, startPolling, stopPolling, disconnectServer
  } from '$lib/stores';
  import MetricCard from '$lib/components/MetricCard.svelte';
  import MetricChart from '$lib/components/MetricChart.svelte';
  import { Activity, Cpu, Network, Clock, Power, Users } from 'lucide-svelte';
  import { api } from '$lib/api';
  import type { ConnectedUser } from '$lib/types';

  let loading = $state(false);
  let currentServerId: string | null = null;
  let connectedUsers = $state<ConnectedUser[]>([]);
  
  let histCpu = $state<Array<[number, number]>>([]);
  let histRam = $state<Array<[number, number]>>([]);
  let histRx = $state<Array<[number, number]>>([]);
  let histTx = $state<Array<[number, number]>>([]);

  function formatUptime(seconds: number): string {
    const d = Math.floor(seconds / 86400);
    const h = Math.floor((seconds % 86400) / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    if (d > 0) return `${d}d ${h}h`;
    if (h > 0) return `${h}h ${m}m`;
    return `${m}m`;
  }

  function handleDisconnect() {
    if (currentServerId) {
      disconnectServer(currentServerId);
      activeServerId.set(null);
    }
  }

  const metrics = $derived($activeServerId ? $latestMetrics[$activeServerId] : null);
  const liveHistory = $derived($activeServerId ? ($metricsHistory[$activeServerId] ?? []) : []);

  // Format live history into echarts coordinates
  const liveCpu = $derived(liveHistory.map(m => [new Date(m.timestamp).getTime(), m.cpu_percent] as [number, number]));
  const liveRam = $derived(liveHistory.map(m => [new Date(m.timestamp).getTime(), m.ram_percent] as [number, number]));
  const liveRx = $derived(liveHistory.map(m => [new Date(m.timestamp).getTime(), m.net_rx_kbps] as [number, number]));
  const liveTx = $derived(liveHistory.map(m => [new Date(m.timestamp).getTime(), m.net_tx_kbps] as [number, number]));

  // Build combined chart series, merging DB history and live buffer, sorting by timestamp
  const cpuSeries = $derived([...histCpu, ...liveCpu].sort((a,b) => a[0] - b[0]));
  const ramSeries = $derived([...histRam, ...liveRam].sort((a,b) => a[0] - b[0]));
  const netRxSeries = $derived([...histRx, ...liveRx].sort((a,b) => a[0] - b[0]));
  const netTxSeries = $derived([...histTx, ...liveTx].sort((a,b) => a[0] - b[0]));

  async function loadHistoricalData(sid: string) {
    try {
      const limit = 720; // 30 mins * 60s / 2.5s
      const [c, r, rx, tx] = await Promise.all([
        api.metrics.history(sid, 'cpu', limit),
        api.metrics.history(sid, 'ram', limit),
        api.metrics.history(sid, 'net_rx', limit),
        api.metrics.history(sid, 'net_tx', limit)
      ]);
      // The DB returns descending by timestamp usually.
      histCpu = c.map(p => [p.timestamp, p.value] as [number, number]);
      histRam = r.map(p => [p.timestamp, p.value] as [number, number]);
      histRx = rx.map(p => [p.timestamp, p.value] as [number, number]);
      histTx = tx.map(p => [p.timestamp, p.value] as [number, number]);
      
      const users = await api.system.connectedUsers(sid);
      connectedUsers = users;
    } catch (e) {
      console.warn('Failed to load metric history or users', e);
    }
  }

  $effect(() => {
    const newId = $activeServerId;
    if (newId === currentServerId) return;

    if (currentServerId) stopPolling(currentServerId);
    currentServerId = newId;

    if (!newId) return;

    loadHistoricalData(newId);

    const status = $connectionStates[newId]?.status;
    if (status !== 'connected') {
      loading = true;
      connectToServer(newId)
        .then(() => { startPolling(newId); })
        .catch(() => {})
        .finally(() => { loading = false; });
    } else {
      startPolling(newId);
    }
  });

  onDestroy(() => {
    if (currentServerId) stopPolling(currentServerId);
  });
</script>

<div class="dashboard">
  <!-- Header -->
  <div class="dash-header">
    <div>
      <h1>{$activeServer?.name ?? 'No server selected'}</h1>
      {#if $activeServer}
        <span class="header-sub">{$activeServer.host}:{$activeServer.port}</span>
      {/if}
    </div>
    <div class="header-actions">
      {#if metrics}
        <div class="uptime-pill">
          <Clock size={13} /> {formatUptime(metrics.uptime_seconds)} uptime
        </div>
      {/if}
      <button class="btn btn-danger btn-sm" onclick={handleDisconnect} title="Disconnect from server">
        <Power size={14} /> Disconnect
      </button>
    </div>
  </div>

  {#if !$activeServer}
    <div class="empty-state">
      <Activity size={48} color="var(--text-muted)" strokeWidth={1.5} />
      <h2>Select a server to start monitoring</h2>
      <p>Add a server from the sidebar to view real-time metrics.</p>
    </div>
  {:else}
    <!-- Stat cards row -->
    <div class="stat-cards">
      <MetricCard label="CPU Usage" value={metrics?.cpu_percent ?? 0} unit="%" loading={!metrics}
        sub="Load: {metrics ? `${metrics.load_1.toFixed(2)} / ${metrics.load_5.toFixed(2)} / ${metrics.load_15.toFixed(2)}` : '—'}"
        color="var(--chart-cpu)"
        threshold={{ warn: 70, danger: 90 }} />

      <MetricCard label="RAM Usage" value={metrics?.ram_percent ?? 0} unit="%"
        sub={metrics ? `${metrics.ram_used_mb.toLocaleString()}MB / ${metrics.ram_total_mb.toLocaleString()}MB` : '—'}
        loading={!metrics} color="var(--chart-ram)" />

      <MetricCard label="Disk Usage" value={metrics?.disk_percent ?? 0} unit="%"
        sub={metrics ? `${metrics.disk_used_gb.toFixed(1)}GB / ${metrics.disk_total_gb.toFixed(1)}GB` : '—'}
        loading={!metrics} color="var(--chart-disk)" />

      <MetricCard label="Processes" value={metrics?.process_count ?? '—'}
        loading={!metrics} color="var(--info)" />

      <MetricCard label="Net RX" value={metrics?.net_rx_kbps ?? 0} unit=" KB/s"
        loading={!metrics} color="var(--chart-net-rx)" />

      <MetricCard label="Net TX" value={metrics?.net_tx_kbps ?? 0} unit=" KB/s"
        loading={!metrics} color="var(--chart-net-tx)" />
    </div>

    <!-- Connected IPs Panel -->
    {#if connectedUsers.length > 0}
      <div class="active-ips-panel">
        <div class="panel-header">
          <Users size={16} color="var(--accent)" />
          <h3>Active SSH Sessions</h3>
        </div>
        <div class="ips-list">
          {#each connectedUsers as user}
            <div class="ip-badge">
              <span class="ip-user">{user.username}</span>
              <span class="ip-addr">{user.ip}</span>
              <span class="ip-time" title="Connected Since">{user.connected_since}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Charts grid -->
    <div class="charts-grid">
      <div class="card chart-card">
        <div class="section-header">
          <div class="chart-label"><Cpu size={14} /> CPU Usage</div>
          {#if metrics}
            <span class="badge badge-{metrics.cpu_percent >= 90 ? 'danger' : metrics.cpu_percent >= 70 ? 'warning' : 'success'}">
              {metrics.cpu_percent.toFixed(1)}%
            </span>
          {/if}
        </div>
        <MetricChart
          data={cpuSeries}
          color="#6366f1"
          unit="%"
          max={100}
          threshold={{ warn: 70, danger: 90 }}
          height="180px"
          loading={!metrics && loading}
        />
      </div>

      <div class="card chart-card">
        <div class="section-header">
          <div class="chart-label"><Activity size={14} /> RAM Usage</div>
          {#if metrics}
            <span class="badge badge-{metrics.ram_percent >= 90 ? 'danger' : metrics.ram_percent >= 70 ? 'warning' : 'success'}">
              {metrics.ram_percent.toFixed(1)}%
            </span>
          {/if}
        </div>
        <MetricChart
          data={ramSeries}
          color="#22c55e"
          unit="%"
          max={100}
          threshold={{ warn: 70, danger: 90 }}
          height="180px"
          loading={!metrics && loading}
        />
      </div>

      <div class="card chart-card chart-full">
        <div class="section-header">
          <div class="chart-label"><Network size={14} /> Network Traffic</div>
        </div>
        <div class="net-legend">
          <span style="color:var(--chart-net-rx)">⬇ RX</span>
          <span style="color:var(--chart-net-tx)">⬆ TX</span>
        </div>
        <div class="net-charts">
          <MetricChart
            data={netRxSeries}
            color="#38bdf8"
            unit=" KB/s"
            max={Math.max(1000, ...netRxSeries.map(d => d[1]))}
            height="140px"
            loading={!metrics && loading}
          />
          <MetricChart
            data={netTxSeries}
            color="#f59e0b"
            unit=" KB/s"
            max={Math.max(1000, ...netTxSeries.map(d => d[1]))}
            height="140px"
            loading={!metrics && loading}
          />
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    padding: 1.5rem;
    gap: 1.25rem;
  }

  .dash-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
  }
  .dash-header h1 { font-size: 1.3rem; margin: 0; }
  .header-sub { font-size: 0.8rem; color: var(--text-muted); font-family: 'JetBrains Mono', monospace; }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .uptime-pill {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    border-radius: 999px;
    padding: 0.3rem 0.75rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    color: var(--text-muted);
    text-align: center;
  }
  .empty-state h2 { color: var(--text-secondary); font-size: 1.1rem; }
  .empty-state p { font-size: 0.85rem; }

  .stat-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 0.75rem;
  }

  .charts-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }
  .chart-card { padding: 1rem; }
  .chart-full { grid-column: 1 / -1; }

  .chart-label {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
    color: var(--text-secondary);
    font-weight: 600;
  }

  .net-legend {
    display: flex;
    gap: 1rem;
    font-size: 0.75rem;
    font-weight: 500;
    margin-bottom: 0.5rem;
  }
  .net-charts {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }
  
  /* Active IPs Panel */
  .active-ips-panel {
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 1rem;
  }
  .panel-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }
  .panel-header h3 {
    margin: 0;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }
  .ips-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }
  .ip-badge {
    display: flex;
    align-items: center;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
    gap: 0.5rem;
  }
  .ip-user { font-weight: 600; color: var(--accent); }
  .ip-addr { font-family: 'JetBrains Mono', monospace; color: var(--text-primary); }
  .ip-time { color: var(--text-muted); font-size: 0.7rem; }

  @media (max-width: 1200px) {
    .charts-grid { grid-template-columns: 1fr; }
    .chart-full { grid-column: 1; }
    .net-charts { grid-template-columns: 1fr; }
  }
</style>
