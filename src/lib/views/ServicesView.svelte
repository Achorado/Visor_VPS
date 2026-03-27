<script lang="ts">
  import { onMount } from 'svelte';
  import { activeServerId } from '$lib/stores';
  import { api } from '$lib/api';
  import type { ServiceInfo } from '$lib/types';
  import LogViewer from '$lib/components/LogViewer.svelte';
  import { Search, RefreshCw, Play, Square, RotateCcw, FileText } from 'lucide-svelte';

  let services = $state<ServiceInfo[]>([]);
  let loading = $state(false);
  let searchQuery = $state('');
  let actionLoading = $state<string | null>(null);
  let logsModal = $state<{ open: boolean; service: string }>({ open: false, service: '' });

  const filteredServices = $derived(
    services.filter(s => s.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         s.description.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  async function loadServices() {
    if (!$activeServerId) return;
    loading = true;
    try {
      services = await api.services.list($activeServerId);
    } catch (e) {
      console.error('Failed to list services:', e);
    } finally {
      loading = false;
    }
  }

  async function doAction(service: string, action: 'start' | 'stop' | 'restart') {
    if (!$activeServerId) return;
    actionLoading = `${action}-${service}`;
    try {
      if (action === 'start') await api.services.start($activeServerId, service);
      if (action === 'stop') await api.services.stop($activeServerId, service);
      if (action === 'restart') await api.services.restart($activeServerId, service);
      await loadServices();
    } finally {
      actionLoading = null;
    }
  }

  onMount(loadServices);
  $effect(() => { if ($activeServerId) loadServices(); });

  function statusBadge(s: ServiceInfo): string {
    if (s.status === 'active') return 'badge-success';
    if (s.status === 'failed') return 'badge-danger';
    if (s.status === 'activating') return 'badge-warning';
    return 'badge-muted';
  }
</script>

<div class="view-shell">
  <div class="view-header">
    <h1>Services</h1>
    <div class="header-actions">
      <div class="search-wrap">
        <Search size={14} color="var(--text-muted)" />
        <input class="log-search" type="text" placeholder="Filter services…" bind:value={searchQuery} />
      </div>
      <button class="btn btn-ghost btn-sm" onclick={loadServices} disabled={loading}>
        <RefreshCw size={13} />
        Refresh
      </button>
    </div>
  </div>

  {#if !$activeServerId}
    <div class="empty-view">Select a server to manage services.</div>
  {:else if loading && services.length === 0}
    <div class="services-list">
      {#each {length: 6} as _}
        <div class="service-row skeleton" style="height:52px;border-radius:8px"></div>
      {/each}
    </div>
  {:else}
    <div class="services-list">
      {#each filteredServices as svc (svc.name)}
        <div class="service-row fade-in">
          <div class="svc-info">
            <span class="svc-name">{svc.name}</span>
            <span class="svc-desc">{svc.description}</span>
          </div>
          <span class="badge svc-badge {statusBadge(svc)}">
            {svc.status === 'active' ? 'Active' : svc.status === 'failed' ? 'Failed' : svc.status === 'inactive' ? 'Inactive' : svc.sub_state}
          </span>
          <div class="svc-actions">
            <button class="btn btn-ghost btn-sm btn-icon" onclick={() => doAction(svc.name, 'start')}
              disabled={!!actionLoading || svc.status === 'active'} title="Start">
              {#if actionLoading === `start-${svc.name}`}<span class="spinner-xs"></span>{:else}<Play size={12}/>{/if}
            </button>
            <button class="btn btn-ghost btn-sm btn-icon" onclick={() => doAction(svc.name, 'stop')}
              disabled={!!actionLoading || svc.status === 'inactive'} title="Stop">
              {#if actionLoading === `stop-${svc.name}`}<span class="spinner-xs"></span>{:else}<Square size={12}/>{/if}
            </button>
            <button class="btn btn-ghost btn-sm btn-icon" onclick={() => doAction(svc.name, 'restart')}
              disabled={!!actionLoading} title="Restart">
              {#if actionLoading === `restart-${svc.name}`}<span class="spinner-xs"></span>{:else}<RotateCcw size={12}/>{/if}
            </button>
            <button class="btn btn-ghost btn-sm btn-icon" title="Logs"
              onclick={() => logsModal = { open: true, service: svc.name }}>
              <FileText size={12} />
            </button>
          </div>
        </div>
      {/each}
      {#if filteredServices.length === 0}
        <div class="empty-view">No matching services found.</div>
      {/if}
    </div>
  {/if}
</div>

{#if logsModal.open && $activeServerId}
  <div class="modal-overlay" onclick={() => logsModal.open = false}>
    <div class="modal" style="max-width:800px;height:70vh;display:flex;flex-direction:column"
      onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>Logs — {logsModal.service}</h2>
        <button class="btn-ghost btn btn-sm" onclick={() => logsModal.open = false}>✕ Close</button>
      </div>
      <div style="flex:1;overflow:hidden">
        <LogViewer serverId={$activeServerId} serviceName={logsModal.service} autoRefresh={true} />
      </div>
    </div>
  </div>
{/if}

<style>
  .view-shell { display: flex; flex-direction: column; height: 100%; padding: 1.5rem; gap: 1rem; overflow: hidden; }
  .view-header { display: flex; align-items: center; justify-content: space-between; gap: 1rem; }
  .view-header h1 { font-size: 1.3rem; }
  .header-actions { display: flex; align-items: center; gap: 0.75rem; }
  .search-wrap {
    display: flex; align-items: center; gap: 0.5rem;
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-sm); padding: 0.35rem 0.75rem; flex: 1;
  }
  .log-search { background: transparent; border: none; outline: none; color: var(--text-primary); font-size: 0.8rem; font-family: inherit; min-width: 200px; }
  .services-list { display: flex; flex-direction: column; gap: 4px; overflow-y: auto; flex: 1; }
  .service-row {
    display: flex; align-items: center; gap: 0.75rem;
    background: var(--bg-surface); border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm); padding: 0.65rem 0.875rem;
    transition: var(--transition);
  }
  .service-row:hover { border-color: var(--border-default); }
  .svc-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
  .svc-name { font-size: 0.85rem; font-weight: 500; color: var(--text-primary); font-family: 'JetBrains Mono', monospace; }
  .svc-desc { font-size: 0.72rem; color: var(--text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .svc-badge { font-size: 0.7rem; flex-shrink: 0; }
  .svc-actions { display: flex; gap: 3px; flex-shrink: 0; }
  .empty-view { display: flex; align-items: center; justify-content: center; flex: 1; color: var(--text-muted); font-size: 0.85rem; }
  .modal-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .spinner-xs { display: inline-block; width: 10px; height: 10px; border: 1.5px solid rgba(255,255,255,0.3); border-top-color: white; border-radius: 50%; animation: spin 0.7s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
