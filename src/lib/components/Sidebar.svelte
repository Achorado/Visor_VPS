<script lang="ts">
  import { servers, activeServerId, connectionStates, connectToServer, disconnectServer, activeTab, theme } from '$lib/stores';
  import type { ServerConfig } from '$lib/types';
  import { Plus, Server, Moon, Sun, Settings, LayoutDashboard, Box, Wrench, Cpu, Package, Globe } from 'lucide-svelte';

  interface Props {
    onAddServer?: () => void;
  }
  let { onAddServer }: Props = $props();

  function statusColor(id: string): string {
    const s = $connectionStates[id]?.status;
    if (s === 'connected') return 'active';
    if (s === 'connecting') return 'pending';
    if (typeof s === 'object' && 'error' in s) return 'error';
    return 'offline';
  }

  function latency(id: string): string {
    const l = $connectionStates[id]?.latency_ms;
    return l !== undefined ? `${l}ms` : '';
  }

  async function handleServerClick(server: ServerConfig) {
    activeServerId.set(server.id);
    const s = $connectionStates[server.id]?.status;
    if (s !== 'connected' && s !== 'connecting') {
      try { await connectToServer(server.id); } catch {}
    }
  }
</script>

<aside class="sidebar">
  <!-- Logo -->
  <div class="sidebar-logo">
    <div class="logo-icon">V</div>
    <span class="logo-text">Visor VPS</span>
  </div>

  <!-- Nav -->
  <nav class="sidebar-nav">
    <button
      class="nav-item" class:active={$activeTab === 'dashboard'}
      onclick={() => activeTab.set('dashboard')}
    >
      <LayoutDashboard size={16} />
      <span>Dashboard</span>
    </button>
    <button
      class="nav-item" class:active={$activeTab === 'services'}
      onclick={() => activeTab.set('services')}
    >
      <Wrench size={16} />
      <span>Services</span>
    </button>
    <button
      class="nav-item" class:active={$activeTab === 'projects'}
      onclick={() => activeTab.set('projects')}
    >
      <Box size={16} />
      <span>Projects</span>
    </button>
    <button
      class="nav-item" class:active={$activeTab === 'processes'}
      onclick={() => activeTab.set('processes')}
    >
      <Cpu size={16} />
      <span>Processes</span>
    </button>
    <button
      class="nav-item" class:active={$activeTab === 'containers'}
      onclick={() => activeTab.set('containers')}
    >
      <Package size={16} />
      <span>Containers</span>
    </button>
    <button
      class="nav-item" class:active={$activeTab === 'nginx'}
      onclick={() => activeTab.set('nginx')}
    >
      <Globe size={16} />
      <span>Nginx Sites</span>
    </button>
  </nav>

  <div class="divider" style="margin: 0.5rem 1rem;"></div>

  <!-- Servers list -->
  <div class="servers-header">
    <span class="section-label">SERVERS</span>
    <button class="btn-icon-sm" onclick={onAddServer} title="Add server">
      <Plus size={14} />
    </button>
  </div>

  <div class="servers-list">
    {#if $servers.length === 0}
      <div class="empty-servers">
        <Server size={24} color="var(--text-muted)" />
        <p>No servers yet</p>
        <button class="btn btn-primary btn-sm" onclick={onAddServer}>Add Server</button>
      </div>
    {:else}
      {#each $servers as server (server.id)}
        <button
          class="server-item"
          class:active={$activeServerId === server.id}
          onclick={() => handleServerClick(server)}
        >
          <span class="pulse-dot {statusColor(server.id)}"></span>
          <div class="server-info">
            <span class="server-name">{server.name}</span>
            <span class="server-host">{server.host}</span>
          </div>
          {#if latency(server.id)}
            <span class="server-latency">{latency(server.id)}</span>
          {/if}
          <div class="server-color-dot" style:background={server.color}></div>
        </button>
      {/each}
    {/if}
  </div>

  <!-- Bottom -->
  <div class="sidebar-bottom">
    <button
      class="nav-item" class:active={$activeTab === 'settings'}
      onclick={() => activeTab.set('settings')}
    >
      <Settings size={16} />
      <span>Settings</span>
    </button>
    <button class="nav-item" onclick={theme.toggle} title="Toggle theme">
      {#if $theme === 'dark'}
        <Sun size={16} />
        <span>Light Mode</span>
      {:else}
        <Moon size={16} />
        <span>Dark Mode</span>
      {/if}
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    background: var(--sidebar-bg);
    border-right: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    height: 100%;
    flex-shrink: 0;
    overflow: hidden;
  }

  .sidebar-logo {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1.25rem 1rem;
    border-bottom: 1px solid var(--border-subtle);
  }
  .logo-icon {
    width: 32px; height: 32px;
    background: linear-gradient(135deg, var(--accent), #a78bfa);
    border-radius: 8px;
    display: flex; align-items: center; justify-content: center;
    font-weight: 800; font-size: 1rem; color: white;
    flex-shrink: 0;
  }
  .logo-text { font-weight: 700; font-size: 0.9rem; color: var(--text-primary); }

  .sidebar-nav {
    padding: 0.75rem 0.5rem 0.25rem;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.55rem 0.75rem;
    border-radius: var(--radius-sm);
    cursor: pointer;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.85rem;
    font-family: inherit;
    font-weight: 500;
    width: 100%;
    transition: var(--transition);
    text-align: left;
  }
  .nav-item:hover { background: var(--bg-hover); color: var(--text-primary); }
  .nav-item.active {
    background: rgba(99,102,241,0.12);
    color: var(--accent);
  }

  .servers-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 1rem 0.25rem;
  }
  .section-label {
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--text-muted);
    letter-spacing: 0.08em;
  }
  .btn-icon-sm {
    width: 22px; height: 22px;
    border-radius: 4px;
    background: transparent;
    border: 1px solid var(--border-default);
    color: var(--text-muted);
    cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: var(--transition);
  }
  .btn-icon-sm:hover { background: var(--bg-hover); color: var(--text-primary); }

  .servers-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.25rem 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .empty-servers {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 2rem 1rem;
    color: var(--text-muted);
    font-size: 0.8rem;
  }

  .server-item {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.6rem 0.75rem;
    border-radius: var(--radius-sm);
    cursor: pointer;
    border: none;
    background: transparent;
    width: 100%;
    transition: var(--transition);
    text-align: left;
    position: relative;
  }
  .server-item:hover { background: var(--bg-hover); }
  .server-item.active {
    background: rgba(99,102,241,0.1);
    border-left: 2px solid var(--accent);
    margin-left: 2px;
    width: calc(100% - 2px);
  }

  .server-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .server-name {
    font-size: 0.83rem;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .server-host {
    font-size: 0.7rem;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .server-latency {
    font-size: 0.65rem;
    color: var(--success);
    font-family: 'JetBrains Mono', monospace;
    flex-shrink: 0;
  }
  .server-color-dot {
    width: 6px; height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
    opacity: 0.7;
  }

  .sidebar-bottom {
    padding: 0.5rem;
    border-top: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
