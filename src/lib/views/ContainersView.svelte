<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { RotateCw, Play, Square, RefreshCw, FileText, Search, Box, HardDrive, Network } from 'lucide-svelte';
  import { activeServerId, activeServer } from '$lib/stores';
  import { api } from '$lib/api';
  import type { DockerContainer } from '$lib/types';

  let containers = $state<DockerContainer[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let filter = $state('');
  
  // Logs modal state
  let showLogs = $state(false);
  let logsContainer = $state('');
  let logsContent = $state('');
  let logsLoading = $state(false);

  let refreshInterval: ReturnType<typeof setInterval>;

  async function loadContainers() {
    if (!$activeServerId) return;
    try {
      loading = true;
      error = null;
      const list = await api.containers.listDocker($activeServerId);
      containers = list as DockerContainer[];
    } catch (e: any) {
      error = e.message || String(e);
    } finally {
      loading = false;
    }
  }

  async function handleAction(containerName: string, action: 'start' | 'stop' | 'restart') {
    if (!$activeServerId) return;
    try {
      await api.containers.dockerAction($activeServerId, containerName, action);
      setTimeout(loadContainers, 1500); // Wait for Docker to process
    } catch (e: any) {
      alert(`Action ${action} failed: ${e.message}`);
    }
  }

  async function viewLogs(containerName: string) {
    if (!$activeServerId) return;
    logsContainer = containerName;
    showLogs = true;
    logsLoading = true;
    logsContent = '';
    try {
      logsContent = await api.containers.getDockerLogs($activeServerId, containerName, 100);
    } catch (e: any) {
      logsContent = `Error fetching logs: ${e.message}`;
    } finally {
      logsLoading = false;
    }
  }

  $effect(() => {
    const id = $activeServerId;
    if (id) {
      loadContainers();
      clearInterval(refreshInterval);
      refreshInterval = setInterval(loadContainers, 6000); // 6s auto-refresh
    } else {
      containers = [];
      clearInterval(refreshInterval);
    }
  });

  onDestroy(() => {
    clearInterval(refreshInterval);
  });

  const filtered = $derived(
    filter ? containers.filter(c => c.name.toLowerCase().includes(filter.toLowerCase()) || c.image.toLowerCase().includes(filter.toLowerCase())) : containers
  );

  function isUp(status: string) {
    return status.toLowerCase().startsWith('up');
  }
</script>

<div class="view-content">
  <div class="view-header">
    <div>
      <h1>Docker Containers</h1>
      <span class="header-sub">{$activeServer?.host || 'No Server'}</span>
    </div>
    <div class="header-actions">
      <div class="search-box">
        <span class="search-icon"><Search size={16} /></span>
        <input type="text" bind:value={filter} placeholder="Search containers..." />
      </div>
      <button class="btn btn-outline" onclick={loadContainers} disabled={loading}>
        <RotateCw size={14} class={loading ? 'spin' : ''} /> Refresh
      </button>
    </div>
  </div>

  {#if !$activeServerId}
    <div class="empty-state">
      <HardDrive size={48} />
      <h2>No server selected</h2>
      <p>Please select a server from the dashboard to view Docker containers.</p>
    </div>
  {:else if error && containers.length === 0}
    <div class="error-state card">
      Oops! {error}
    </div>
  {:else}
    <div class="grid list-grid">
      {#each filtered as container}
        <div class="card pcard">
          <div class="pcard-header">
            <div class="p-title">
              <span class="status-indicator {isUp(container.status) ? 'status-online' : 'status-offline'}"></span>
              <h3 title={container.name}>{container.name}</h3>
            </div>
          </div>
          
          <div class="pcard-metrics">
            <div class="p-metric full-width" title={container.image}>
              <Box size={12} style="min-width: 12px;"/> <span class="truncate">{container.image}</span>
            </div>
            <div class="p-metric full-width" title={container.ports || 'No ports mapped'}>
              <Network size={12} style="min-width: 12px;"/> <span class="truncate">{container.ports || '—'}</span>
            </div>
            <div class="p-metric full-width">
              <RotateCw size={12} style="min-width: 12px;"/> <span class="truncate">{container.status}</span>
            </div>
          </div>

          <div class="pcard-actions">
            {#if isUp(container.status)}
              <button class="btn btn-sm btn-outline-danger" onclick={() => handleAction(container.name, 'stop')} title="Stop">
                <Square size={12} fill="currentColor" />
              </button>
              <button class="btn btn-sm btn-outline-warning" onclick={() => handleAction(container.name, 'restart')} title="Restart">
                <RefreshCw size={12} />
              </button>
            {:else}
              <button class="btn btn-sm btn-outline-success" onclick={() => handleAction(container.name, 'start')} title="Start">
                <Play size={12} fill="currentColor" />
              </button>
            {/if}
            <button class="btn btn-sm btn-outline" style="flex:1" onclick={() => viewLogs(container.name)}>
              <FileText size={12} /> Logs
            </button>
          </div>
        </div>
      {/each}
      {#if filtered.length === 0 && !loading && !error}
        <div class="empty-state" style="grid-column: 1/-1">
          <p>No Docker containers found on this server.</p>
        </div>
      {/if}
    </div>
  {/if}
</div>

{#if showLogs}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={() => { showLogs = false; }}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal card" onclick={e => e.stopPropagation()}>
      <div class="modal-header">
        <h3>Logs: {logsContainer}</h3>
        <button class="btn btn-sm btn-outline" onclick={() => { showLogs = false; }}>Close</button>
      </div>
      <div class="modal-body terminal">
        {#if logsLoading}
          <div class="skeleton" style="height: 100%"></div>
        {:else}
          <pre>{logsContent || 'No logs found.'}</pre>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .truncate {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .full-width {
    grid-column: 1 / -1;
  }
  .view-content {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 1.5rem;
    gap: 1.25rem;
    overflow-y: auto;
  }
  .view-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
  }
  .view-header h1 { margin: 0; font-size: 1.3rem; }
  .header-sub { font-size: 0.8rem; color: var(--text-muted); font-family: 'JetBrains Mono', monospace; }
  .header-actions { display: flex; gap: 0.75rem; }

  .search-box {
    position: relative;
    display: flex;
    align-items: center;
  }
  .search-icon {
    position: absolute;
    left: 10px;
    color: var(--text-muted);
    pointer-events: none;
    display: flex;
    align-items: center;
  }
  .search-box input {
    background: var(--bg-hover);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: 0.5rem 0.75rem 0.5rem 2.2rem;
    color: var(--text-primary);
    font-size: 0.85rem;
    width: 200px;
    outline: none;
    transition: var(--transition);
  }
  .search-box input:focus {
    border-color: var(--accent);
    background: var(--bg-surface);
    box-shadow: 0 0 0 2px var(--accent-glow);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .pcard {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    background: var(--bg-elevated);
    border: 1px solid rgba(255,255,255,0.04);
  }
  .pcard-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .p-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .p-title h3 { margin: 0; font-size: 1rem; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 200px; }
  .status-indicator {
    width: 8px; height: 8px; border-radius: 50%;
  }
  .status-online { background-color: var(--success); box-shadow: 0 0 8px var(--success); }
  .status-offline { background-color: var(--text-muted); box-shadow: 0 0 8px var(--text-muted); }

  .pcard-metrics {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
    background: rgba(0,0,0,0.15);
    padding: 0.75rem;
    border-radius: 6px;
  }
  .p-metric { display: flex; align-items: center; gap: 0.4rem; font-family: 'JetBrains Mono', monospace; }

  .pcard-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 4px;
  }

  /* Logs Modal */
  .modal-backdrop {
    position: fixed; top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0,0,0,0.5); backdrop-filter: blur(2px);
    display: flex; align-items: center; justify-content: center;
    z-index: 1000;
  }
  .modal {
    width: 90%; max-width: 1000px;
    height: 80vh;
    display: flex; flex-direction: column;
    padding: 1.5rem; gap: 1rem;
  }
  .modal-header { display: flex; justify-content: space-between; align-items: center; }
  .modal-header h3 { margin: 0; font-family: 'JetBrains Mono', monospace; font-size: 1rem; }
  .modal-body { flex: 1; overflow: auto; background: #0f172a; border-radius: 6px; padding: 1rem; }
  .terminal pre { margin: 0; font-family: 'JetBrains Mono', monospace; font-size: 0.8rem; color: #cbd5e1; white-space: pre-wrap; word-break: break-all; }
</style>
