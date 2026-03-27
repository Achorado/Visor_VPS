import { writable, derived, get } from 'svelte/store';
import type { ServerConfig, ServerConnectionState, ServerMetrics, MetricsHistoryPoint } from '$lib/types';
import { api } from '$lib/api';

// ─── Theme ────────────────────────────────────────────────────────────────────
function createThemeStore() {
  const stored = typeof localStorage !== 'undefined'
    ? (localStorage.getItem('visor-theme') as 'dark' | 'light') ?? 'dark'
    : 'dark';
  const { subscribe, set, update } = writable<'dark' | 'light'>(stored);

  return {
    subscribe,
    toggle() {
      update(t => {
        const next = t === 'dark' ? 'light' : 'dark';
        if (typeof localStorage !== 'undefined') localStorage.setItem('visor-theme', next);
        if (typeof document !== 'undefined') {
          document.documentElement.classList.toggle('light', next === 'light');
        }
        return next;
      });
    },
    init() {
      const current = get({ subscribe });
      if (typeof document !== 'undefined') {
        document.documentElement.classList.toggle('light', current === 'light');
      }
    }
  };
}

export const theme = createThemeStore();

// ─── Servers ──────────────────────────────────────────────────────────────────
export const servers = writable<ServerConfig[]>([]);
export const activeServerId = writable<string | null>(null);

// Connection states keyed by server_id
export const connectionStates = writable<Record<string, ServerConnectionState>>({});

// Derived: currently active server object
export const activeServer = derived(
  [servers, activeServerId],
  ([$servers, $activeServerId]) => $servers.find(s => s.id === $activeServerId) ?? null
);

export async function loadServers() {
  try {
    const list = await api.servers.list();
    servers.set(list);
    // Select first server if none selected
    const current = get(activeServerId);
    if (!current && list.length > 0) {
      activeServerId.set(list[0].id);
    }
  } catch (e) {
    console.error('Failed to load servers:', e);
  }
}

export async function connectToServer(serverId: string) {
  connectionStates.update(s => ({
    ...s,
    [serverId]: { server_id: serverId, status: 'connecting' }
  }));
  try {
    const state = await api.ssh.connect(serverId);
    connectionStates.update(s => ({ ...s, [serverId]: state }));
    return state;
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    connectionStates.update(s => ({
      ...s,
      [serverId]: { server_id: serverId, status: { error: msg } }
    }));
    throw e;
  }
}

export async function disconnectServer(serverId: string) {
  await api.ssh.disconnect(serverId);
  connectionStates.update(s => ({
    ...s,
    [serverId]: { server_id: serverId, status: 'disconnected' }
  }));
}

// ─── Metrics ──────────────────────────────────────────────────────────────────
// Ring buffer: last N metric snapshots per server
const METRICS_BUFFER = 720;

export const metricsHistory = writable<Record<string, ServerMetrics[]>>({});
export const latestMetrics = writable<Record<string, ServerMetrics>>({});

export function pushMetrics(metrics: ServerMetrics) {
  const sid = metrics.server_id;
  metricsHistory.update(h => {
    const buf = h[sid] ?? [];
    const next = [...buf, metrics].slice(-METRICS_BUFFER);
    return { ...h, [sid]: next };
  });
  latestMetrics.update(m => ({ ...m, [sid]: metrics }));
}

// ─── Polling ──────────────────────────────────────────────────────────────────
const pollingIntervals: Record<string, ReturnType<typeof setInterval>> = {};

export function startPolling(serverId: string, intervalMs = 2500) {
  if (pollingIntervals[serverId]) return;
  const poll = async () => {
    try {
      const m = await api.metrics.get(serverId);
      pushMetrics(m);
    } catch (e) {
      console.warn(`Metrics poll failed for ${serverId}:`, e);
    }
  };
  poll(); // immediate first poll
  pollingIntervals[serverId] = setInterval(poll, intervalMs);
}

export function stopPolling(serverId: string) {
  if (pollingIntervals[serverId]) {
    clearInterval(pollingIntervals[serverId]);
    delete pollingIntervals[serverId];
  }
}

// ─── UI State ─────────────────────────────────────────────────────────────────
export const sidebarOpen = writable(true);
export const activeTab = writable<'dashboard' | 'services' | 'projects' | 'processes' | 'containers' | 'nginx' | 'settings'>('dashboard');
