<script lang="ts">
  import { servers, activeServerId, connectionStates, loadServers } from '$lib/stores';
  import { api } from '$lib/api';
  import { Trash2, Server } from 'lucide-svelte';

  async function removeServer(id: string) {
    if (!confirm('Remove this server? All its projects and metric history will be deleted.')) return;
    if ($activeServerId === id) activeServerId.set(null);
    await api.servers.remove(id);
    await loadServers();
  }

  function statusText(id: string): string {
    const s = $connectionStates[id]?.status;
    if (s === 'connected') return 'Connected';
    if (s === 'connecting') return 'Connecting…';
    if (typeof s === 'object' && 'error' in s) return `Error: ${s.error}`;
    return 'Disconnected';
  }
</script>

<div class="settings-shell">
  <h1 style="margin-bottom:1.5rem">Settings</h1>

  <section class="section">
    <h2>Configured Servers</h2>
    <p class="section-desc">Manage your VPS connections. Remove a server to delete its data.</p>
    <div class="server-table">
      {#if $servers.length === 0}
        <div class="empty-row">No servers configured.</div>
      {:else}
        {#each $servers as server (server.id)}
          <div class="server-row">
            <div class="srv-color" style:background={server.color}></div>
            <div class="srv-details">
              <div class="srv-name">{server.name}</div>
              <div class="srv-addr">{server.username}@{server.host}:{server.port}</div>
            </div>
            <span class="srv-status {$connectionStates[server.id]?.status === 'connected' ? 'connected' : 'disconnected'}">
              {statusText(server.id)}
            </span>
            <button class="btn btn-danger btn-sm btn-icon" onclick={() => removeServer(server.id)} title="Remove">
              <Trash2 size={14} />
            </button>
          </div>
        {/each}
      {/if}
    </div>
  </section>

  <section class="section">
    <h2>About</h2>
    <div class="about-card card">
      <div style="display:flex;align-items:center;gap:0.75rem;margin-bottom:0.5rem">
        <div style="width:36px;height:36px;background:linear-gradient(135deg,#6366f1,#a78bfa);border-radius:8px;display:flex;align-items:center;justify-content:center;font-weight:800;color:white">V</div>
        <div>
          <div style="font-weight:700">Visor VPS</div>
          <div style="font-size:0.75rem;color:var(--text-muted)">v0.1.0</div>
        </div>
      </div>
      <p style="font-size:0.8rem;color:var(--text-secondary)">
        Real-time VPS monitoring and management. Built with Tauri v2, SvelteKit, and Apache ECharts.
      </p>
    </div>
  </section>
</div>

<style>
  .settings-shell { padding: 1.5rem; overflow-y: auto; height: 100%; }
  .section { margin-bottom: 2rem; }
  .section h2 { font-size: 1rem; margin-bottom: 0.25rem; }
  .section-desc { font-size: 0.8rem; color: var(--text-muted); margin-bottom: 0.875rem; }

  .server-table { display: flex; flex-direction: column; gap: 4px; }
  .server-row {
    display: flex; align-items: center; gap: 0.75rem;
    background: var(--bg-surface); border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm); padding: 0.7rem 0.875rem;
  }
  .srv-color { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
  .srv-details { flex: 1; min-width: 0; }
  .srv-name { font-size: 0.85rem; font-weight: 500; }
  .srv-addr { font-size: 0.72rem; color: var(--text-muted); font-family: 'JetBrains Mono', monospace; }
  .srv-status { font-size: 0.75rem; font-weight: 500; flex-shrink: 0; }
  .srv-status.connected { color: var(--success); }
  .srv-status.disconnected { color: var(--text-muted); }

  .empty-row { text-align: center; padding: 1.5rem; color: var(--text-muted); font-size: 0.85rem; }
  .about-card { max-width: 400px; }
</style>
