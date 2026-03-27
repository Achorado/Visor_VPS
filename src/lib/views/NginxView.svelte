<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { RefreshCw, Play, Square, Save, Trash2, FileCode2, Globe } from 'lucide-svelte';
  import { activeServerId, activeServer } from '$lib/stores';
  import { api } from '$lib/api';
  import type { NginxConfig } from '$lib/types';

  let configs = $state<NginxConfig[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Editor State
  let activeConfig = $state<NginxConfig | null>(null);
  let editContent = $state('');
  let saving = $state(false);
  let toggling = $state(false);

  // New config modal
  let showNewModal = $state(false);
  let newName = $state('');

  async function loadConfigs() {
    if (!$activeServerId) return;
    try {
      loading = true;
      error = null;
      configs = await api.nginx.list($activeServerId) as NginxConfig[];
    } catch (e: any) {
      error = e.message || String(e);
    } finally {
      loading = false;
    }
  }

  async function loadContent(config: NginxConfig) {
    if (!$activeServerId) return;
    try {
      activeConfig = config;
      editContent = 'Loading...';
      const result = await api.nginx.getContent($activeServerId, config.name);
      editContent = result as string;
    } catch (e: any) {
      editContent = `Error loading config: ${e.message}`;
    }
  }

  async function createConfig() {
    if (!$activeServerId || !newName.trim()) return;
    let name = newName.trim();
    if (!name.endsWith('.conf')) name += '.conf';
    
    saving = true;
    try {
      const template = `server {\n    listen 80;\n    server_name example.com;\n    \n    location / {\n        proxy_pass http://localhost:3000;\n    }\n}\n`;
      await api.nginx.save($activeServerId, name, template);
      showNewModal = false;
      newName = '';
      await loadConfigs();
      const newlyCreated = configs.find(c => c.name === name);
      if (newlyCreated) loadContent(newlyCreated);
    } catch (e: any) {
      alert(`Error creating config: ${e.message}`);
    } finally {
      saving = false;
    }
  }

  async function saveConfig() {
    if (!$activeServerId || !activeConfig) return;
    saving = true;
    try {
      await api.nginx.save($activeServerId, activeConfig.name, editContent);
    } catch (e: any) {
      alert(`Error saving config: ${e.message}`);
    } finally {
      saving = false;
    }
  }

  async function toggleConfig(config: NginxConfig) {
    if (!$activeServerId) return;
    toggling = true;
    try {
      await api.nginx.toggle($activeServerId, config.name, !config.enabled);
      await loadConfigs();
    } catch (e: any) {
      alert(`Error toggling config: ${e.message}`);
    } finally {
      toggling = false;
    }
  }

  async function deleteConfig(config: NginxConfig) {
    if (!$activeServerId || !confirm(`Are you sure you want to completely delete ${config.name}?`)) return;
    try {
      await api.nginx.delete($activeServerId, config.name);
      if (activeConfig?.name === config.name) activeConfig = null;
      await loadConfigs();
    } catch (e: any) {
      alert(`Error deleting: ${e.message}`);
    }
  }

  async function reloadNginx() {
    if (!$activeServerId) return;
    try {
      const out = await api.nginx.reload($activeServerId);
      alert('Nginx Reloaded Successfully!\n' + out);
    } catch (e: any) {
      alert(`Nginx Reload Error (Syntax bad?):\n${e.message}`);
    }
  }

  $effect(() => {
    const id = $activeServerId;
    if (id) {
      loadConfigs();
      activeConfig = null;
    } else {
      configs = [];
      activeConfig = null;
    }
  });
</script>

<div class="view-content">
  <div class="view-header">
    <div>
      <h1>Nginx Sites & Modules</h1>
      <span class="header-sub">{$activeServer?.host || 'No Server'} &bull; /etc/nginx/sites-available</span>
    </div>
    <div class="header-actions">
      <button class="btn btn-outline" onclick={loadConfigs} disabled={loading}>
        <RefreshCw size={14} class={loading ? 'spin' : ''} /> Refresh
      </button>
      <button class="btn btn-primary" onclick={() => showNewModal = true}>
        <FileCode2 size={14} /> New Config
      </button>
      <button class="btn btn-warning" onclick={reloadNginx}>
        <RefreshCw size={14} /> Reload Daemon
      </button>
    </div>
  </div>

  {#if !$activeServerId}
    <div class="empty-state">
      <Globe size={48} />
      <h2>No server selected</h2>
    </div>
  {:else if error && configs.length === 0}
    <div class="error-state card">
      Oops! {error}
    </div>
  {:else}
    <div class="layout-grid">
      <!-- Left sidebar: the config list -->
      <div class="config-sidebar card">
        <h3 class="list-title">Sites Available</h3>
        <div class="list">
          {#each configs as conf}
            <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
            <div 
              class="list-item {activeConfig?.name === conf.name ? 'active' : ''}"
              onclick={() => loadContent(conf)}
            >
              <div class="conf-name">
                <span class="status-indicator {conf.enabled ? 'status-online' : 'status-offline'}"></span>
                <span>{conf.name}</span>
              </div>
              <div class="conf-actions" onclick={(e) => e.stopPropagation()}>
                <button class="btn-icon {conf.enabled ? 'btn-danger' : 'btn-success'}" 
                        onclick={() => toggleConfig(conf)} 
                        title={conf.enabled ? "Disable Site" : "Enable Site"}>
                  {#if conf.enabled}<Square size={12} fill="currentColor" />{:else}<Play size={12} />{/if}
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Right area: config editor -->
      <div class="editor-area card">
        {#if activeConfig}
          <div class="editor-header">
            <h3>Editing: {activeConfig.name}</h3>
            <div class="editor-actions">
              <button class="btn btn-sm btn-outline-danger" onclick={() => deleteConfig(activeConfig)}>
                <Trash2 size={14} /> Delete
              </button>
              <button class="btn btn-sm btn-primary" onclick={saveConfig} disabled={saving}>
                <Save size={14} /> {saving ? 'Saving...' : 'Save File'}
              </button>
            </div>
          </div>
          <div class="editor-body">
            <textarea
              class="code-editor"
              bind:value={editContent}
              spellcheck="false"
            ></textarea>
          </div>
        {:else}
          <div class="empty-state" style="height: 100%">
            <FileCode2 size={48} style="opacity: 0.2" />
            <p style="color: var(--text-muted); margin-top: 1rem;">Select a config on the left to edit</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

{#if showNewModal}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={() => { showNewModal = false; }}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal card compact-modal" onclick={e => e.stopPropagation()}>
      <div class="modal-header">
        <h3>New Nginx Site</h3>
        <button class="btn btn-sm btn-outline" onclick={() => { showNewModal = false; }}>Close</button>
      </div>
      <div class="modal-body" style="padding: 1.5rem;">
        <label>Configuration Name (.conf)</label>
        <div style="display: flex; gap: 0.5rem; margin-top: 0.5rem">
          <input type="text" bind:value={newName} placeholder="e.g. backend.conf" class="form-input" />
          <button class="btn btn-primary" onclick={createConfig} disabled={saving}>Create</button>
        </div>
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
  }
  .view-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
  }
  .view-header h1 { margin: 0; font-size: 1.3rem; }
  .header-sub { font-size: 0.8rem; color: var(--text-muted); font-family: 'JetBrains Mono', monospace; }
  .header-actions { display: flex; gap: 0.75rem; }

  .layout-grid {
    display: grid;
    grid-template-columns: 300px 1fr;
    gap: 1.25rem;
    flex: 1;
    min-height: 0;
  }

  .config-sidebar {
    display: flex;
    flex-direction: column;
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }
  .list-title {
    padding: 1rem;
    margin: 0;
    font-size: 0.9rem;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-secondary);
  }
  .list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }
  .list-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 0.5rem;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background 0.2s;
    margin-bottom: 2px;
  }
  .list-item:hover { background: var(--bg-hover); }
  .list-item.active { background: rgba(99,102,241,0.15); border-left: 3px solid var(--accent); }
  
  .conf-name { display: flex; align-items: center; gap: 0.5rem; font-family: 'JetBrains Mono', monospace; font-size: 0.85rem;}
  .status-indicator { width: 8px; height: 8px; border-radius: 50%; }
  .status-online { background-color: var(--success); box-shadow: 0 0 8px var(--success); }
  .status-offline { background-color: var(--text-muted); box-shadow: 0 0 8px var(--text-muted); }

  .btn-icon {
    display: flex; align-items: center; justify-content: center;
    width: 24px; height: 24px; border-radius: 4px;
    border: 1px solid transparent; background: transparent;
    cursor: pointer; color: var(--text-muted);
  }
  .btn-icon:hover { background: rgba(255,255,255,0.1); }
  .btn-success:hover { color: var(--success); border-color: var(--success); }
  .btn-danger:hover { color: var(--danger); border-color: var(--danger); }

  .editor-area {
    display: flex;
    flex-direction: column;
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    min-height: 0;
  }
  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--border-subtle);
  }
  .editor-header h3 { margin: 0; font-size: 1rem; font-family: 'JetBrains Mono', monospace; }
  .editor-actions { display: flex; gap: 0.5rem; }
  
  .editor-body {
    flex: 1;
    padding: 0;
    position: relative;
    min-height: 0;
  }
  .code-editor {
    position: absolute;
    top: 0; left: 0; right: 0; bottom: 0;
    width: 100%; height: 100%;
    background: #0f172a;
    color: #e2e8f0;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.85rem;
    line-height: 1.5;
    padding: 1rem;
    border: none;
    resize: none;
    outline: none;
  }
  .code-editor:focus {
    box-shadow: inset 0 0 0 1px var(--accent);
  }

  .form-input {
    flex: 1;
    background: var(--bg-base);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 0.5rem 0.75rem;
    color: var(--text-primary);
  }
  
  /* Modal */
  .modal-backdrop {
    position: fixed; top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0,0,0,0.5); backdrop-filter: blur(2px);
    display: flex; align-items: center; justify-content: center;
    z-index: 1000;
  }
  .modal {
    width: 90%; max-width: 800px;
    display: flex; flex-direction: column;
  }
  .compact-modal { max-width: 400px; }
  .modal-header { display: flex; justify-content: space-between; align-items: center; padding: 1rem 1.5rem; border-bottom: 1px solid var(--border-subtle); }
  .modal-header h3 { margin: 0; font-size: 1rem; }
</style>
