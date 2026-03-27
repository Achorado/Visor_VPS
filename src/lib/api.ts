import { invoke } from '@tauri-apps/api/core';
import type {
  ServerConfig,
  ServerConfigInput,
  ServerConnectionState,
  ServerMetrics,
  MetricsHistoryPoint,
  ServiceInfo,
  Project,
  ProjectInput,
  ExecResult,
  ConnectedUser,
  DeploymentCandidate
} from '$lib/types';

// Helper: invoke with typed error
async function cmd<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (e: any) {
    if (e && typeof e === 'object' && 'message' in e) {
      throw new Error(e.message);
    }
    throw new Error(String(e));
  }
}

// ─── Servers ─────────────────────────────────────────────────────────────────
export const api = {
  servers: {
    list: () => cmd<ServerConfig[]>('list_servers'),
    add: (input: ServerConfigInput) => cmd<ServerConfig>('add_server', { input }),
    remove: (serverId: string) => cmd<void>('remove_server', { serverId }),
  },

  // ─── SSH ──────────────────────────────────────────────────────────────────
  ssh: {
    testConnection: (
      host: string, port: number, username: string,
      password?: string, keyPath?: string
    ) => cmd<boolean>('test_connection', { host, port, username, password, keyPath }),

    connect: (serverId: string) => cmd<ServerConnectionState>('connect_server', { serverId }),
    disconnect: (serverId: string) => cmd<void>('disconnect_server', { serverId }),

    exec: (serverId: string, command: string) =>
      cmd<ExecResult>('exec_command', { serverId, command }),
  },

  // ─── Metrics ──────────────────────────────────────────────────────────────
  metrics: {
    get: (serverId: string) => cmd<ServerMetrics>('get_metrics', { serverId }),
    history: (serverId: string, metricType: string, limit?: number) =>
      cmd<MetricsHistoryPoint[]>('get_metric_history', { serverId, metricType, limit }),
  },

  // ─── Services ──────────────────────────────────────────────────────────────
  services: {
    list: (serverId: string, filter?: string) =>
      cmd<ServiceInfo[]>('list_services', { serverId, filter }),
    start: (serverId: string, serviceName: string) =>
      cmd<boolean>('start_service', { serverId, serviceName }),
    stop: (serverId: string, serviceName: string) =>
      cmd<boolean>('stop_service', { serverId, serviceName }),
    restart: (serverId: string, serviceName: string) =>
      cmd<boolean>('restart_service', { serverId, serviceName }),
    logs: (serverId: string, serviceName: string, lines?: number) =>
      cmd<string>('get_service_logs', { serverId, serviceName, lines }),
  },

  // ─── Projects ─────────────────────────────────────────────────────────────
  projects: {
    list: (serverId: string) => cmd<Project[]>('list_projects', { serverId }),
    add: (input: ProjectInput) => cmd<Project>('add_project', { input }),
    remove: (projectId: string) => cmd<void>('remove_project', { projectId }),
  },

  // ─── Containers & PM2 ────────────────────────────────────────────────────
  containers: {
    listDocker: (serverId: string) => cmd<any[]>('list_docker_containers', { serverId }),
    dockerAction: (serverId: string, container: string, action: string) =>
      cmd<string>('docker_action', { serverId, container, action }),
    getDockerLogs: (serverId: string, container: string, lines?: number) =>
      cmd<string>('get_docker_logs', { serverId, container, lines }),

    listPm2: (serverId: string) => cmd<any[]>('list_pm2_processes', { serverId }),
    pm2Action: (serverId: string, process: string, action: string) =>
      cmd<string>('pm2_action', { serverId, process, action }),
    getPm2Logs: (serverId: string, process: string, lines?: number) =>
      cmd<string>('get_pm2_logs', { serverId, process, lines }),
    discoverDeployments: (serverId: string) => invoke<DeploymentCandidate[]>('discover_deployments', { serverId }),
    deployPm2Project: (serverId: string, path: string, name: string, entry: string) => 
      invoke<string>('deploy_pm2_project', { serverId, path, name, entry })
  },
  nginx: {
    list: (serverId: string) => invoke('list_nginx_configs', { serverId }),
    getContent: (serverId: string, name: string) => invoke('get_nginx_config_content', { serverId, name }),
    save: (serverId: string, name: string, content: string) => invoke('save_nginx_config', { serverId, name, content }),
    toggle: (serverId: string, name: string, enable: boolean) => invoke('toggle_nginx_config', { serverId, name, enable }),
    reload: (serverId: string) => invoke('reload_nginx', { serverId }),
    delete: (serverId: string, name: string) => invoke('delete_nginx_config', { serverId, name })
  },
  system: {
    connectedUsers: (serverId: string) => invoke<ConnectedUser[]>('get_connected_users', { serverId }),
    saveFile: (path: string, base64Data: string) => invoke('save_base64_file', { path, base64Data })
  }
};
