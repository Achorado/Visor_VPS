<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { api } from '$lib/api';
  import { Search, RefreshCw } from 'lucide-svelte';

  interface Props {
    serverId: string;
    serviceName: string;
    lines?: number;
    autoRefresh?: boolean;
  }
  let { serverId, serviceName, lines = 200, autoRefresh = false }: Props = $props();

  let logContent = $state('');
  let loading = $state(false);
  let container: HTMLDivElement;
  let searchQuery = $state('');
  let autoScrollEnabled = $state(true);
  let interval: ReturnType<typeof setInterval> | null = null;

  function highlight(text: string): string {
    const escaped = text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    return escaped
      .replace(/(error|fatal|exception|failed|critical)/gi, '<span class="log-error">$1</span>')
      .replace(/(warn|warning)/gi, '<span class="log-warn">$1</span>')
      .replace(/(info|notice)/gi, '<span class="log-info">$1</span>')
      .replace(/(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2})/g, '<span class="log-time">$1</span>')
      .replace(/(https?:\/\/[^\s]+)/g, '<a class="log-url" href="#">$1</a>');
  }

  async function fetchLogs() {
    loading = true;
    try {
      logContent = await api.services.logs(serverId, serviceName, lines);
      if (autoScrollEnabled && container) {
        setTimeout(() => { container.scrollTop = container.scrollHeight; }, 50);
      }
    } catch (e) {
      logContent = `Error fetching logs: ${e}`;
    } finally {
      loading = false;
    }
  }

  const filteredLines = $derived(
    logContent.split('\n').filter(l =>
      searchQuery ? l.toLowerCase().includes(searchQuery.toLowerCase()) : true
    )
  );

  onMount(() => {
    fetchLogs();
    if (autoRefresh) interval = setInterval(fetchLogs, 5000);
  });
  onDestroy(() => { if (interval) clearInterval(interval); });
</script>

<div class="log-viewer">
  <div class="log-toolbar">
    <div class="search-wrap">
      <Search size={14} color="var(--text-muted)" />
      <input class="log-search" type="text" placeholder="Filter logs…" bind:value={searchQuery} />
    </div>
    <button class="btn btn-ghost btn-sm" onclick={fetchLogs} disabled={loading}>
      <RefreshCw size={13} class={loading ? 'spin' : ''} />
      Refresh
    </button>
  </div>

  <div class="log-container" bind:this={container} onscroll={(e) => {
    const el = e.currentTarget as HTMLDivElement;
    autoScrollEnabled = el.scrollTop + el.clientHeight >= el.scrollHeight - 10;
  }}>
    {#if loading && !logContent}
      <div class="skeleton" style="height:100%;border-radius:0"></div>
    {:else}
      {#each filteredLines as line, i}
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        <div class="log-line">{@html highlight(line)}</div>
      {/each}
      {#if filteredLines.length === 0}
        <div class="log-empty">No matching log entries</div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .log-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    overflow: hidden;
    background: #050a14;
  }
  .log-toolbar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
  }
  .search-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: 0.35rem 0.6rem;
  }
  .log-search {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 0.8rem;
    font-family: inherit;
  }
  .log-container {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.75rem;
    line-height: 1.6;
  }
  .log-line {
    white-space: pre-wrap;
    word-break: break-all;
    color: #94a3b8;
    padding: 1px 0;
  }
  .log-line:hover { background: rgba(255,255,255,0.03); }
  .log-empty {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
    font-size: 0.8rem;
  }
  :global(.log-error) { color: #ef4444; font-weight: 600; }
  :global(.log-warn)  { color: #f59e0b; }
  :global(.log-info)  { color: #38bdf8; }
  :global(.log-time)  { color: #64748b; }
  :global(.log-url)   { color: #6366f1; text-decoration: none; }
  :global(.spin) { animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
