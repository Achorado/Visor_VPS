<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { RotateCw, Play, Square, RefreshCw, FileText, Search, Activity, Cpu, Package, Clock, Plus, Box, Terminal, Power, Rocket } from 'lucide-svelte';
  import { activeServerId, activeServer } from '$lib/stores';
  import { api } from '$lib/api';
  import type { Pm2Process, DeploymentCandidate } from '$lib/types';

  let processes = $state<Pm2Process[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let filter = $state('');
  
  // Logs modal state
  let showLogs = $state(false);
  let logsProcess = $state('');
  let logsContent = $state('');
  let logsLoading = $state(false);

  let logsContainer = $state<HTMLElement | null>(null);
  
  // Deployer Modal State
  let showDeployModal = $state(false);
  let deployLoading = $state(false);
  let deployCandidates = $state<DeploymentCandidate[]>([]);
  let selectedCandidate = $state<DeploymentCandidate | null>(null);
  let deployName = $state('');
  let deployEntry = $state('');
  let deployError = $state<string | null>(null);
  let executingDeploy = $state(false);

  let refreshInterval: ReturnType<typeof setInterval>;

  async function loadProcesses() {
    if (!$activeServerId) return;
    try {
      loading = true;
      error = null;
      // In Rust, we return a serde_json::Value Array directly matching the interface
      const list = await api.containers.listPm2($activeServerId);
      processes = list as Pm2Process[];
    } catch (e: any) {
      error = e.message || String(e);
      // Suppress network errors from polluting UI aggressively if they are just timeouts
    } finally {
      loading = false;
    }
  }

  async function handleAction(processName: string, action: 'start' | 'stop' | 'restart') {
    if (!$activeServerId) return;
    try {
      await api.containers.pm2Action($activeServerId, processName, action);
      setTimeout(loadProcesses, 1500); // Wait for PM2 to digest
    } catch (e: any) {
      alert(`Acton ${action} failed: ${e.message}`);
    }
  }

  async function viewLogs(processName: string) {
    if (!$activeServerId) return;
    logsProcess = processName;
    showLogs = true;
    logsLoading = true;
    logsContent = '';
    try {
      logsContent = await api.containers.getPm2Logs($activeServerId, processName, 100);
    } catch (e: any) {
      logsContent = `Failed to get PM2 logs: ${e.message || String(e)}`;
    } finally {
      logsLoading = false;
    }
  }

  async function openDeployer() {
    if (!$activeServerId) return;
    showDeployModal = true;
    deployLoading = true;
    deployError = null;
    deployCandidates = [];
    selectedCandidate = null;
    
    try {
      deployCandidates = await api.containers.discoverDeployments($activeServerId) || [];
    } catch (e: any) {
      deployError = `Discovery failed: ${e.message}`;
    } finally {
      deployLoading = false;
    }
  }

  function selectCandidate(c: DeploymentCandidate) {
    selectedCandidate = c;
    deployName = c.name;
    deployEntry = c.entry;
  }

  async function executeDeploy() {
    if (!$activeServerId || !selectedCandidate) return;
    executingDeploy = true;
    deployError = null;
    try {
      const out = await api.containers.deployPm2Project($activeServerId, selectedCandidate.path, deployName, deployEntry);
      alert('Deployment Started Successfully!\n\n' + out);
      showDeployModal = false;
      await loadProcesses();
    } catch(e: any) {
      deployError = `Deployment failed: ${e.message}`;
    } finally {
      executingDeploy = false;
    }
  }

  function startAutoRefresh() {
    clearInterval(refreshInterval);
    refreshInterval = setInterval(loadProcesses, 180000); // 3 minutes
  }

  $effect(() => {
    const id = $activeServerId;
    if (id) {
      loadProcesses();
      startAutoRefresh();
    } else {
      processes = [];
      clearInterval(refreshInterval);
    }
  });

  onDestroy(() => {
    clearInterval(refreshInterval);
  });

  const filtered = $derived(
    filter ? processes.filter(p => p.name.toLowerCase().includes(filter.toLowerCase())) : processes
  );

  function formatMemory(bytes: number) {
    const mb = bytes / 1024 / 1024;
    return `${mb.toFixed(1)} MB`;
  }
  function formatUptime(uptimeMs: number) {
    if (uptimeMs === 0) return '0s';
    const now = Date.now();
    const diff = Math.floor((now - uptimeMs) / 1000);
    if (diff < 0) return '0s'; // edge case
    const d = Math.floor(diff / 86400);
    const h = Math.floor((diff % 86400) / 3600);
    const m = Math.floor((diff % 3600) / 60);
    if (d > 0) return `${d}d ${h}h`;
    if (h > 0) return `${h}h ${m}m`;
    return `${m}m`;
  }
</script>

<div class="view-content">
  <div class="view-header">
    <div>
      <h1>PM2 Processes</h1>
      <span class="header-sub">{$activeServer?.host || 'No Server'}</span>
    </div>
    <div class="header-actions">
      <div class="search-box">
        <span class="search-icon"><Search size={16} /></span>
        <input type="text" bind:value={filter} placeholder="Search processes..." />
      </div>
      <button class="btn btn-outline" onclick={() => { loadProcesses(); if ($activeServerId) startAutoRefresh(); }} disabled={loading}>
        <RefreshCw size={14} class={loading ? 'spin' : ''} /> {loading ? 'Refreshing...' : 'Refresh'}
      </button>
      <button class="btn btn-primary" onclick={openDeployer}>
        <Rocket size={14} /> Deploy New App
      </button>
    </div>
  </div>

  {#if !$activeServerId}
    <div class="empty-state">
      <Package size={48} />
      <h2>No server selected</h2>
      <p>Please select a server from the dashboard to view PM2 processes.</p>
    </div>
  {:else if error && processes.length === 0}
    <div class="error-state card">
      Oops! {error}
    </div>
  {:else}
    <div class="grid list-grid">
      {#each filtered as proc}
        <div class="card pcard">
          <div class="pcard-header">
            <div class="p-title">
              <span class="status-indicator {proc.status === 'online' ? 'status-online' : 'status-offline'}"></span>
              <h3>{proc.name}</h3>
            </div>
            <div class="p-id">ID: {proc.pm_id}</div>
          </div>
          
          <div class="pcard-metrics">
            <div class="p-metric">
              <Cpu size={12}/> {proc.cpu}%
            </div>
            <div class="p-metric">
              <Activity size={12}/> {formatMemory(proc.memory)}
            </div>
            <div class="p-metric">
              <RotateCw size={12}/> {proc.restarts} restarts
            </div>
            <div class="p-metric">
              <Clock size={12}/> {proc.status === 'online' ? formatUptime(proc.uptime) : '—'}
            </div>
          </div>

          <div class="pcard-actions">
            {#if proc.status === 'online'}
              <button class="btn btn-sm btn-outline-danger" onclick={() => handleAction(proc.name, 'stop')} title="Stop">
                <Square size={12} fill="currentColor" />
              </button>
              <button class="btn btn-sm btn-outline-warning" onclick={() => handleAction(proc.name, 'restart')} title="Restart">
                <RefreshCw size={12} />
              </button>
            {:else}
              <button class="btn btn-sm btn-outline-success" onclick={() => handleAction(proc.name, 'start')} title="Start">
                <Play size={12} fill="currentColor" />
              </button>
            {/if}
            <button class="btn btn-sm btn-outline" style="flex:1" onclick={() => viewLogs(proc.name)}>
              <FileText size={12} /> Logs
            </button>
          </div>
        </div>
      {/each}
      {#if filtered.length === 0 && !loading && !error}
        <div class="empty-state" style="grid-column: 1/-1">
          <p>No PM2 processes found on this server.</p>
        </div>
      {/if}
    </div>
  {/if}
</div>

{#if showLogs}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-overlay" onclick={() => { showLogs = false; }}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal card" style="max-width: 800px; padding: 1.5rem; display:flex; flex-direction:column; gap:1rem;" onclick={e => e.stopPropagation()}>
      <div style="display:flex; justify-content:space-between; align-items:center;">
        <h3 style="font-family:'JetBrains Mono', monospace; font-size:1rem; margin:0">Logs: {logsProcess}</h3>
        <button class="btn btn-sm btn-outline" onclick={() => { showLogs = false; }}>✕ Close</button>
      </div>
      <div class="terminal" style="flex:1; overflow:auto; background:#0f172a; border-radius:6px; padding:1rem; min-height:400px; max-height:70vh;">
        {#if logsLoading}
          <div class="skeleton" style="height: 100%"></div>
        {:else}
          <pre style="margin:0; font-family:'JetBrains Mono', monospace; font-size:0.8rem; color:#cbd5e1; white-space:pre-wrap; word-break:break-all;">{logsContent || 'No logs found.'}</pre>
        {/if}
      </div>
    </div>
  </div>
{/if}

{#if showDeployModal}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-overlay" onclick={() => { showDeployModal = false; }}>
    <div class="modal card" style="max-width: 600px; padding: 1.5rem;" onclick={e => e.stopPropagation()}>
      <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:1rem;">
        <h3 style="display:flex; align-items:center; gap:0.5rem"><Rocket size={18} color="var(--accent)"/> PM2 Smart Deployer</h3>
        <button class="btn btn-sm btn-outline" onclick={() => { showDeployModal = false; }}>✕ Close</button>
      </div>
      
      <div class="modal-body" style="padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem; max-height: 70vh; overflow-y: auto;">
        <p style="color:var(--text-muted); font-size:0.85rem">
          Scanning your home directory (`~/*`) for Node/PM2 projects...
        </p>

        {#if deployLoading}
          <div class="skeleton" style="height: 100px; width: 100%"></div>
        {:else if deployError}
          <div class="error-msg">{deployError}</div>
        {:else if !selectedCandidate}
          {#if deployCandidates.length === 0}
            <div class="empty-state" style="padding: 2rem 0;">
              No NodeJS projects found in ~/
            </div>
          {:else}
            <div class="candidates-list" style="display:flex; flex-direction:column; gap:0.5rem;">
              {#each deployCandidates as c}
                <button class="deploy-card {c.stack === 'UNKNOWN' ? 'unknown' : ''}" onclick={() => selectCandidate(c)}>
                  <div style="font-weight:600; font-family:'JetBrains Mono', monospace">{c.name}</div>
                  <div style="font-size:0.8rem; color:var(--text-muted); margin-top:0.2rem">
                    Path: {c.path} <br/> 
                    Type: <span class="badge badge-info">{c.stack}</span> &bull; Extracted Command: <code>{c.entry || 'none'}</code>
                  </div>
                </button>
              {/each}
            </div>
          {/if}
        {:else}
          <!-- Configuration Step -->
          <div class="deploy-setup-form" style="display:flex; flex-direction:column; gap:1rem;">
            <div class="form-group">
              <label>Project Selected</label>
              <div class="badge badge-info" style="align-self: flex-start; padding:0.5rem; font-family:'JetBrains Mono', monospace;">
                {selectedCandidate.path}
              </div>
            </div>
            
            <div class="form-row" style="display:flex; gap:1rem;">
              <div class="form-group" style="flex:1">
                <label>App Name (for PM2)</label>
                <input class="form-input" type="text" bind:value={deployName} />
              </div>
            </div>
            
            <div class="form-group">
              <label>Startup Command / Script</label>
              <input class="form-input" type="text" bind:value={deployEntry} />
              <p style="font-size:0.7rem; color:var(--warning); margin-top:0.3rem">
                Note: A silent <code>npm install</code> will be fired first if dependencies are missing!
              </p>
            </div>
            
            <div style="display:flex; justify-content:space-between; margin-top: 1rem;">
              <button class="btn btn-ghost" onclick={() => selectedCandidate = null}>&larr; Back</button>
              <button class="btn btn-primary" onclick={executeDeploy} disabled={executingDeploy}>
                {#if executingDeploy} <span class="spin"><RefreshCw size={14}/></span> Executing... {:else} <Rocket size={14}/> Start Project {/if}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
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
  .pcard-header { display: flex; justify-content: space-between; align-items: center; }
  .p-title { display: flex; align-items: center; gap: 0.5rem; }
  .p-title h3 { margin: 0; font-size: 1rem; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 150px; }
  .status-indicator { width: 8px; height: 8px; border-radius: 50%; }
  .status-online { background-color: var(--success); box-shadow: 0 0 8px var(--success); }
  .status-offline { background-color: var(--danger); box-shadow: 0 0 8px var(--danger); }
  .p-id { font-size: 0.7rem; color: var(--text-muted); background: rgba(255,255,255,0.05); padding: 2px 6px; border-radius: 4px; }

  .pcard-metrics {
    display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem; font-size: 0.8rem;
    color: var(--text-secondary); background: rgba(0,0,0,0.15); padding: 0.75rem; border-radius: 6px;
  }
  .p-metric { display: flex; align-items: center; gap: 0.4rem; font-family: 'JetBrains Mono', monospace; }
  .pcard-actions { display: flex; gap: 0.5rem; margin-top: 4px; }
  
  .deploy-card {
    text-align: left; background: var(--bg-surface); border: 1px solid var(--border-default);
    padding: 1rem; border-radius: var(--radius-sm); cursor: pointer; transition: all 0.2s;
  }
  .deploy-card:hover {
    background: var(--bg-hover); border-color: var(--accent); transform: translateY(-1px);
  }
  .deploy-card.unknown { opacity: 0.7; }
  .form-group { display: flex; flex-direction: column; gap: 0.3rem; }
  .form-group label { font-size: 0.8rem; font-weight: 500; color: var(--text-secondary); }
  .form-input {
    background: #0f172a; border: 1px solid var(--border-subtle); padding: 0.6rem 0.8rem;
    border-radius: 4px; color: #e2e8f0; font-family: inherit; width: 100%; box-sizing: border-box;
  }
  .form-input:focus { border-color: var(--accent); outline: none; }
</style>
