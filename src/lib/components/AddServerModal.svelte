<script lang="ts">
  import { api } from '$lib/api';
  import { servers, loadServers } from '$lib/stores';
  import { X } from 'lucide-svelte';

  interface Props {
    onClose?: () => void;
  }
  let { onClose }: Props = $props();

  const COLORS = ['#6366f1', '#22c55e', '#f59e0b', '#ef4444', '#38bdf8', '#a78bfa', '#ec4899'];

  let form = $state({
    name: '',
    host: '',
    port: 22,
    username: '',
    password: '',
    key_path: '',
    color: COLORS[0],
    auth: 'password' as 'password' | 'key',
  });

  let testing = $state(false);
  let saving = $state(false);
  let testResult = $state<'ok' | 'fail' | null>(null);
  let errorMsg = $state('');

  async function testConnection() {
    testing = true;
    testResult = null;
    errorMsg = '';
    try {
      const ok = await api.ssh.testConnection(
        form.host, form.port, form.username,
        form.password, // Pasamos el password (o passphrase si es llave)
        form.auth === 'key' ? form.key_path : undefined,
      );
      testResult = ok ? 'ok' : 'fail';
    } catch (e: unknown) {
      testResult = 'fail';
      errorMsg = e instanceof Error ? e.message : String(e);
    } finally {
      testing = false;
    }
  }

  async function save() {
    if (!form.name || !form.host || !form.username) return;
    saving = true;
    errorMsg = '';
    try {
      await api.servers.add({
        name: form.name,
        host: form.host,
        port: form.port,
        username: form.username,
        password: form.password, // Pasamos el password siempre
        key_path: form.auth === 'key' ? form.key_path : undefined,
        color: form.color,
      });
      await loadServers();
      onClose?.();
    } catch (e: unknown) {
      errorMsg = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose?.();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="modal-overlay" onclick={(e) => e.target === e.currentTarget && onClose?.()}>
  <div class="modal">
    <div class="modal-header">
      <h2>Add Server</h2>
      <button class="btn-icon" onclick={onClose}><X size={18} /></button>
    </div>

    <form onsubmit={(e) => { e.preventDefault(); save(); }} class="modal-form">
      <!-- Name + Color -->
      <div class="field-row">
        <div class="field" style="flex:1">
          <label class="label" for="srv-name">Server Name</label>
          <input id="srv-name" class="input" type="text" placeholder="My VPS" bind:value={form.name} required />
        </div>
        <div class="field">
          <label class="label">Color</label>
          <div class="color-swatches">
            {#each COLORS as c}
              <button
                type="button"
                class="color-swatch"
                class:selected={form.color === c}
                style:background={c}
                onclick={() => form.color = c}
              ></button>
            {/each}
          </div>
        </div>
      </div>

      <!-- Host + Port -->
      <div class="field-row">
        <div class="field" style="flex:1">
          <label class="label" for="srv-host">Hostname / IP</label>
          <input id="srv-host" class="input" type="text" placeholder="192.168.1.1" bind:value={form.host} required />
        </div>
        <div class="field" style="width:90px">
          <label class="label" for="srv-port">Port</label>
          <input id="srv-port" class="input" type="number" min="1" max="65535" bind:value={form.port} />
        </div>
      </div>

      <!-- Username -->
      <div class="field">
        <label class="label" for="srv-user">Username</label>
        <input id="srv-user" class="input" type="text" placeholder="root" bind:value={form.username} required />
      </div>

      <!-- Auth method -->
      <div class="field">
        <label class="label">Authentication</label>
        <div class="auth-tabs">
          <button type="button" class="auth-tab" class:active={form.auth === 'password'} onclick={() => form.auth = 'password'}>Password</button>
          <button type="button" class="auth-tab" class:active={form.auth === 'key'} onclick={() => form.auth = 'key'}>SSH Key</button>
        </div>
      </div>

      {#if form.auth === 'password'}
        <div class="field">
          <label class="label" for="srv-pass">Password</label>
          <input id="srv-pass" class="input" type="password" placeholder="••••••••" bind:value={form.password} />
        </div>
      {:else}
        <div class="field">
          <label class="label" for="srv-key">Key File Path</label>
          <input id="srv-key" class="input" type="text" placeholder="/home/user/.ssh/id_rsa" bind:value={form.key_path} />
        </div>
        <div class="field">
          <label class="label" for="srv-passphrase">Passphrase (optional)</label>
          <input id="srv-passphrase" class="input" type="password" placeholder="••••••••" bind:value={form.password} />
        </div>
      {/if}

      {#if errorMsg}
        <div class="error-msg">{errorMsg}</div>
      {/if}

      <div class="modal-actions">
        <button type="button" class="btn btn-ghost" onclick={testConnection} disabled={testing || !form.host}>
          {#if testing}
            <span class="spinner"></span> Testing...
          {:else if testResult === 'ok'}
            ✓ Connected
          {:else if testResult === 'fail'}
            ✗ Failed — Retry
          {:else}
            Test Connection
          {/if}
        </button>
        <button type="submit" class="btn btn-primary" disabled={saving}>
          {#if saving}<span class="spinner"></span>{/if}
          Save Server
        </button>
      </div>
    </form>
  </div>
</div>

<style>
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1.25rem;
  }
  .modal-form { display: flex; flex-direction: column; gap: 1rem; }
  .field-row { display: flex; gap: 0.75rem; align-items: flex-start; }
  .field { display: flex; flex-direction: column; }

  .color-swatches {
    display: flex;
    gap: 6px;
    padding: 6px 0;
  }
  .color-swatch {
    width: 20px; height: 20px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    transition: var(--transition);
  }
  .color-swatch.selected {
    border-color: white;
    transform: scale(1.2);
  }

  .auth-tabs {
    display: flex;
    gap: 4px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    padding: 3px;
  }
  .auth-tab {
    flex: 1;
    padding: 6px 12px;
    border-radius: 4px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.8rem;
    font-family: inherit;
    font-weight: 500;
    cursor: pointer;
    transition: var(--transition);
  }
  .auth-tab.active { background: var(--bg-surface); color: var(--text-primary); }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .error-msg {
    padding: 0.6rem 0.875rem;
    background: var(--danger-bg);
    border: 1px solid rgba(239,68,68,0.2);
    border-radius: var(--radius-sm);
    color: var(--danger);
    font-size: 0.8rem;
  }

  .spinner {
    display: inline-block;
    width: 12px; height: 12px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
