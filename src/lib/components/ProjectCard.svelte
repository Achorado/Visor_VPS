<script lang="ts">
  import { api } from '$lib/api';
  import SparklineChart from './SparklineChart.svelte';
  import type { Project, ServiceStatus } from '$lib/types';
  import { Play, Square, RotateCcw, Terminal, Trash2 } from 'lucide-svelte';

  interface Props {
    project: Project;
    serverId: string;
    serviceStatus?: ServiceStatus;
    cpuHistory?: number[];
    onRemove?: (id: string) => void;
    onShowLogs?: (serviceName: string) => void;
  }
  let {
    project, serverId, serviceStatus, cpuHistory = [], onRemove, onShowLogs
  }: Props = $props();

  let actionLoading = $state<'start' | 'stop' | 'restart' | null>(null);

  const statusClass = $derived(
    serviceStatus === 'active' ? 'active'
    : serviceStatus === 'failed' ? 'error'
    : serviceStatus === 'activating' ? 'pending'
    : 'offline'
  );

  const statusLabel = $derived(
    serviceStatus === 'active' ? 'Running'
    : serviceStatus === 'failed' ? 'Failed'
    : serviceStatus === 'activating' ? 'Starting…'
    : serviceStatus === 'inactive' ? 'Stopped'
    : 'Unknown'
  );

  async function doAction(action: 'start' | 'stop' | 'restart') {
    if (!project.service_name) return;
    actionLoading = action;
    try {
      if (action === 'start') await api.services.start(serverId, project.service_name);
      if (action === 'stop') await api.services.stop(serverId, project.service_name);
      if (action === 'restart') await api.services.restart(serverId, project.service_name);
    } finally {
      actionLoading = null;
    }
  }
</script>

<div class="project-card fade-in" style:border-left-color={project.color}>
  <div class="card-header">
    <div class="card-title-row">
      {#if project.service_name}
        <span class="pulse-dot {statusClass}"></span>
      {/if}
      <div>
        <h3 class="project-name">{project.name}</h3>
        <span class="project-path">{project.path}</span>
      </div>
    </div>
    <div class="card-right">
      {#if cpuHistory.length > 0}
        <SparklineChart data={cpuHistory} color={project.color} width="70px" height="28px" />
      {/if}
      {#if project.service_name}
        <span class="badge badge-{statusClass === 'active' ? 'success' : statusClass === 'error' ? 'danger' : statusClass === 'pending' ? 'warning' : 'muted'}">
          {statusLabel}
        </span>
      {/if}
    </div>
  </div>

  {#if project.service_name}
    <div class="card-actions">
      <button class="btn btn-ghost btn-sm" onclick={() => doAction('start')}
        disabled={!!actionLoading || serviceStatus === 'active'} title="Start">
        {#if actionLoading === 'start'}<span class="spinner-sm"></span>{:else}<Play size={13}/>{/if}
      </button>
      <button class="btn btn-ghost btn-sm" onclick={() => doAction('stop')}
        disabled={!!actionLoading || serviceStatus === 'inactive'} title="Stop">
        {#if actionLoading === 'stop'}<span class="spinner-sm"></span>{:else}<Square size={13}/>{/if}
      </button>
      <button class="btn btn-ghost btn-sm" onclick={() => doAction('restart')}
        disabled={!!actionLoading} title="Restart">
        {#if actionLoading === 'restart'}<span class="spinner-sm"></span>{:else}<RotateCcw size={13}/>{/if}
      </button>
      <button class="btn btn-ghost btn-sm" onclick={() => onShowLogs?.(project.service_name!)} title="Logs">
        <Terminal size={13} />
      </button>
      <button class="btn btn-ghost btn-sm" onclick={() => onRemove?.(project.id)} title="Remove" style="margin-left:auto;color:var(--danger)">
        <Trash2 size={13} />
      </button>
    </div>
  {:else}
    <div style="margin-top:0.5rem;display:flex;justify-content:flex-end">
      <button class="btn btn-ghost btn-sm" onclick={() => onRemove?.(project.id)} title="Remove" style="color:var(--danger)">
        <Trash2 size={13} />
      </button>
    </div>
  {/if}
</div>

<style>
  .project-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-left: 3px solid var(--accent);
    border-radius: var(--radius-md);
    padding: 1rem;
    transition: var(--transition);
  }
  .project-card:hover { border-color: var(--border-default); box-shadow: var(--shadow-md); }

  .card-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 0.75rem; }
  .card-title-row { display: flex; align-items: center; gap: 0.6rem; flex: 1; min-width: 0; }
  .project-name { font-size: 0.9rem; font-weight: 600; color: var(--text-primary); }
  .project-path { font-size: 0.72rem; color: var(--text-muted); font-family: 'JetBrains Mono', monospace; }
  .card-right { display: flex; align-items: center; gap: 0.5rem; flex-shrink: 0; }

  .card-actions {
    display: flex;
    gap: 4px;
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border-subtle);
    align-items: center;
  }

  .spinner-sm {
    display: inline-block;
    width: 10px; height: 10px;
    border: 1.5px solid rgba(255,255,255,0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
