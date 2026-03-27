use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ─── Server Config ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    /// Encrypted password blob (base64 encoded)
    pub encrypted_password: Option<String>,
    /// Path to private key file
    pub key_path: Option<String>,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigInput {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub color: String,
}

// ─── Connection Status ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConnectionState {
    pub server_id: String,
    pub status: ConnectionStatus,
    pub latency_ms: Option<u64>,
    pub last_seen: Option<DateTime<Utc>>,
}

// ─── Metrics ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub server_id: String,
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f64,
    pub ram_total_mb: u64,
    pub ram_used_mb: u64,
    pub ram_percent: f64,
    pub disk_total_gb: f64,
    pub disk_used_gb: f64,
    pub disk_percent: f64,
    pub net_rx_kbps: f64,
    pub net_tx_kbps: f64,
    pub load_1: f64,
    pub load_5: f64,
    pub load_15: f64,
    pub uptime_seconds: u64,
    pub process_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsHistoryPoint {
    pub timestamp: i64, // Unix ms
    pub value: f64,
}

// ─── Services ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    Active,
    Inactive,
    Failed,
    Activating,
    Deactivating,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub description: String,
    pub status: ServiceStatus,
    pub sub_state: String,
    pub enabled: bool,
    pub pid: Option<u32>,
    pub memory_kb: Option<u64>,
}

// ─── Projects ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub path: String,
    pub service_name: Option<String>,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInput {
    pub server_id: String,
    pub name: String,
    pub path: String,
    pub service_name: Option<String>,
    pub color: String,
}

// ─── SSH Exec Response ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

// ─── App Error ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}
