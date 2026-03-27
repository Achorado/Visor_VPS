// Shared TypeScript types mirroring the Rust models

export interface ServerConfig {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  encrypted_password?: string;
  key_path?: string;
  color: string;
  created_at: string;
}

export interface ServerConfigInput {
  name: string;
  host: string;
  port: number;
  username: string;
  password?: string;
  key_path?: string;
  color: string;
}

export type ConnectionStatus =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | { error: string };

export interface ServerConnectionState {
  server_id: string;
  status: ConnectionStatus;
  latency_ms?: number;
  last_seen?: string;
}

export interface ServerMetrics {
  server_id: string;
  timestamp: string;
  cpu_percent: number;
  ram_total_mb: number;
  ram_used_mb: number;
  ram_percent: number;
  disk_total_gb: number;
  disk_used_gb: number;
  disk_percent: number;
  net_rx_kbps: number;
  net_tx_kbps: number;
  load_1: number;
  load_5: number;
  load_15: number;
  uptime_seconds: number;
  process_count: number;
}

export interface MetricsHistoryPoint {
  timestamp: number;
  value: number;
}

export type ServiceStatus = 'active' | 'inactive' | 'failed' | 'activating' | 'deactivating' | 'unknown';

export interface ServiceInfo {
  name: string;
  description: string;
  status: ServiceStatus;
  sub_state: string;
  enabled: boolean;
  pid?: number;
  memory_kb?: number;
}

export interface Project {
  id: string;
  server_id: string;
  name: string;
  path: string;
  service_name?: string;
  color: string;
  created_at: string;
}

export interface ProjectInput {
  server_id: string;
  name: string;
  path: string;
  service_name?: string;
  color: string;
}

export interface ExecResult {
  stdout: string;
  stderr: string;
  exit_code: number;
}

export interface AppError {
  code: string;
  message: string;
}

export interface Pm2Process {
  name: string;
  pm_id: number;
  status: 'online' | 'stopped' | 'errored' | string;
  cpu: number;
  memory: number; // in bytes
  uptime: number; // in timestamp MS
  restarts: number;
  pid: number;
}

export interface DockerContainer {
  name: string;
  status: string;
  image: string;
  ports: string;
  cpu_percent?: number;
  memory_percent?: number;
}

export interface NginxConfig {
  name: string;
  enabled: boolean;
  content: string;
}

export interface ConnectedUser {
  username: string;
  ip: string;
  session_type: string;
  connected_since: string;
}

export interface DeploymentCandidate {
  path: string;
  name: string;
  stack: string;
  entry: string;
}
