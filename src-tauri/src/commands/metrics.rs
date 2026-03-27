use anyhow::Result;
use chrono::Utc;
use std::str::FromStr;
use tauri::State;

use crate::commands::ssh::ssh_exec_cached;
use crate::db;
use crate::models::{AppError, MetricsHistoryPoint, ServerMetrics};
use crate::state::AppState;

#[tauri::command]
pub fn save_base64_file(path: String, base64_data: String) -> Result<(), String> {
    use std::io::Write;
    let b64_cleaned = match base64_data.find(',') {
        Some(idx) => &base64_data[idx + 1..],
        None => &base64_data,
    };
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let data = STANDARD.decode(b64_cleaned).map_err(|e| e.to_string())?;
    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(&data).map_err(|e| e.to_string())?;
    Ok(())
}

/// Parse /proc/stat for CPU usage (two-sample approach collapsed to single call)
fn parse_cpu_percent(output: &str) -> f64 {
    // We use `top -bn2 -d0.5` and grab the second sample's CPU line
    for line in output.lines() {
        if line.starts_with("%Cpu") || line.starts_with("Cpu") {
            // Format: %Cpu(s):  3.2 us,  0.8 sy, ...
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Try to find "us" value
            for (i, p) in parts.iter().enumerate() {
                if *p == "us," || *p == "us" {
                    if i > 0 {
                        return parts[i - 1].trim_end_matches(',').parse().unwrap_or(0.0);
                    }
                }
            }
        }
    }
    // Fallback: parse first number after the colon
    if let Some(rest) = output.lines().find(|l| l.contains("Cpu") || l.contains("cpu")).and_then(|l| l.split(':').nth(1)) {
        return rest.split(',').next().and_then(|s| s.trim().split_whitespace().next()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    }
    0.0
}

fn parse_meminfo(output: &str) -> (u64, u64) {
    let mut total = 0u64;
    let mut available = 0u64;
    for line in output.lines() {
        if line.starts_with("MemTotal:") {
            total = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            available = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        }
    }
    let total_mb = total / 1024;
    let used_mb = (total - available) / 1024;
    (total_mb, used_mb)
}

fn parse_disk(output: &str) -> (f64, f64) {
    // df -h / output: Filesystem Size Used Avail Use% Mounted
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let parse_gb = |s: &str| -> f64 {
                let s = s.trim_end_matches(|c: char| !c.is_numeric() && c != '.');
                s.parse().unwrap_or(0.0)
            };
            let total = parse_gb(parts[1]);
            let used = parse_gb(parts[2]);
            // Normalize (df -h might show G/T/M)
            let multiplier = |s: &str| -> f64 {
                if s.ends_with('T') { 1024.0 }
                else if s.ends_with('M') { 1.0 / 1024.0 }
                else { 1.0 }
            };
            return (used * multiplier(parts[2]), total * multiplier(parts[1]));
        }
    }
    (0.0, 0.0)
}

fn parse_net(output: &str, prev_rx: f64, prev_tx: f64, interval_s: f64) -> (f64, f64, f64, f64) {
    // /proc/net/dev: eth0: rx_bytes ... tx_bytes
    let mut rx_bytes = 0u64;
    let mut tx_bytes = 0u64;
    for line in output.lines().skip(2) {
        let line = line.trim();
        if line.starts_with("lo") { continue; }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 10 {
            rx_bytes += parts[1].parse::<u64>().unwrap_or(0);
            tx_bytes += parts[9].parse::<u64>().unwrap_or(0);
        }
    }
    let rx_kbps = if prev_rx > 0.0 && interval_s > 0.0 {
        ((rx_bytes as f64 - prev_rx) / 1024.0 / interval_s).max(0.0)
    } else { 0.0 };
    let tx_kbps = if prev_tx > 0.0 && interval_s > 0.0 {
        ((tx_bytes as f64 - prev_tx) / 1024.0 / interval_s).max(0.0)
    } else { 0.0 };
    (rx_kbps, tx_kbps, rx_bytes as f64, tx_bytes as f64)
}

fn parse_uptime(output: &str) -> u64 {
    output.split_whitespace().next()
        .and_then(|s| s.split('.').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn parse_load(output: &str) -> (f64, f64, f64) {
    let parts: Vec<&str> = output.split_whitespace().collect();
    let l1 = parts.get(0).and_then(|s| s.trim_end_matches(',').parse().ok()).unwrap_or(0.0);
    let l5 = parts.get(1).and_then(|s| s.trim_end_matches(',').parse().ok()).unwrap_or(0.0);
    let l15 = parts.get(2).and_then(|s| s.trim_end_matches(',').parse().ok()).unwrap_or(0.0);
    (l1, l5, l15)
}

fn parse_proc_count(output: &str) -> u32 {
    output.lines().count().saturating_sub(1) as u32
}

fn get_server_metrics_internal(
    db_arc: &std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
    sessions_arc: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, crate::state::SshSession>>>,
    server_id: &str,
) -> Result<ServerMetrics> {
    let script = r#"
echo "===CPU===" && top -bn2 -d0.5 | grep -i "cpu(s)" | tail -1
echo "===MEM===" && cat /proc/meminfo
echo "===DISK===" && df -BG /
echo "===NET===" && cat /proc/net/dev
echo "===UPTIME===" && cat /proc/uptime
echo "===LOAD===" && cat /proc/loadavg
echo "===PROCS===" && ps -e --no-headers
"#;
    let result = ssh_exec_cached(db_arc, sessions_arc, server_id, script)
        .map_err(|e| anyhow::anyhow!("{}", e.message))?;
    let out = result.stdout;

    let section = |name: &str| -> &str {
        let marker = format!("==={}===", name);
        let next_markers = ["===CPU===", "===MEM===", "===DISK===", "===NET===", "===UPTIME===", "===LOAD===", "===PROCS==="];
        let start = out.find(&marker).map(|i| i + marker.len()).unwrap_or(0);
        let end = next_markers.iter()
            .filter_map(|m| if *m != marker.as_str() { out[start..].find(m).map(|i| start + i) } else { None })
            .min()
            .unwrap_or(out.len());
        out[start..end].trim()
    };

    let cpu_section = section("CPU");
    let mem_section = section("MEM");
    let disk_section = section("DISK");
    let net_section = section("NET");
    let uptime_section = section("UPTIME");
    let load_section = section("LOAD");
    let procs_section = section("PROCS");

    let cpu_percent = parse_cpu_percent(cpu_section);
    let (ram_total_mb, ram_used_mb) = parse_meminfo(mem_section);
    let ram_percent = if ram_total_mb > 0 { ram_used_mb as f64 / ram_total_mb as f64 * 100.0 } else { 0.0 };
    let (disk_used_gb, disk_total_gb) = parse_disk(disk_section);
    let disk_percent = if disk_total_gb > 0.0 { disk_used_gb / disk_total_gb * 100.0 } else { 0.0 };
    let (net_rx_kbps, net_tx_kbps, _, _) = parse_net(net_section, 0.0, 0.0, 0.0);
    let uptime_seconds = parse_uptime(uptime_section);
    let (load_1, load_5, load_15) = parse_load(load_section);
    let process_count = parse_proc_count(procs_section);

    Ok(ServerMetrics {
        server_id: server_id.to_string(),
        timestamp: Utc::now(),
        cpu_percent,
        ram_total_mb,
        ram_used_mb,
        ram_percent,
        disk_total_gb,
        disk_used_gb,
        disk_percent,
        net_rx_kbps,
        net_tx_kbps,
        load_1,
        load_5,
        load_15,
        uptime_seconds,
        process_count,
    })
}

#[derive(serde::Serialize)]
pub struct ConnectedUser {
    pub username: String,
    pub ip: String,
    pub session_type: String,
    pub connected_since: String,
}

#[tauri::command]
pub async fn get_connected_users(
    state: tauri::State<'_, crate::state::AppState>,
    server_id: String,
) -> Result<Vec<ConnectedUser>, crate::models::AppError> {
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();
    
    let result = tokio::task::spawn_blocking(move || {
        crate::commands::ssh::ssh_exec_cached(&db_arc, &sessions_arc, &server_id, "who -u")
    })
    .await
    .map_err(|e| crate::models::AppError::new("TASK_ERROR", format!("{:#}", e)))??;

    if result.exit_code != 0 {
        return Err(crate::models::AppError::new("COMMAND_FAILED", "Failed to run who"));
    }

    let mut users = Vec::new();
    for line in result.stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let username = parts[0].to_string();
            let session_type = parts[1].to_string();
            let connected_since = format!("{} {}", parts[2], parts[3]);
            
            let last_part = parts.last().unwrap_or(&"");
            let ip = if last_part.starts_with('(') && last_part.ends_with(')') {
                last_part[1..last_part.len()-1].to_string()
            } else {
                "Local".to_string()
            };
            
            users.push(ConnectedUser { username, session_type, connected_since, ip });
        }
    }
    
    Ok(users)
}

#[tauri::command]
pub async fn get_metrics(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ServerMetrics, AppError> {
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    let sid = server_id.clone();
    let result = tokio::task::spawn_blocking(move || {
        get_server_metrics_internal(&db_arc, &sessions_arc, &sid)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("METRICS_ERROR", e.to_string()))?;

    // Persist to history
    let now_ms = Utc::now().timestamp_millis();
    if let Ok(db) = state.db.lock() {
        let _ = db::insert_metric(&db, &server_id, "cpu", result.cpu_percent, now_ms);
        let _ = db::insert_metric(&db, &server_id, "ram", result.ram_percent, now_ms);
        let _ = db::insert_metric(&db, &server_id, "net_rx", result.net_rx_kbps, now_ms);
        let _ = db::insert_metric(&db, &server_id, "net_tx", result.net_tx_kbps, now_ms);
        let _ = db::insert_metric(&db, &server_id, "disk", result.disk_percent, now_ms);
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_metric_history(
    state: State<'_, AppState>,
    server_id: String,
    metric_type: String,
    limit: Option<usize>,
) -> Result<Vec<MetricsHistoryPoint>, AppError> {
    let db = state.db.lock().map_err(|e| AppError::new("DB_LOCK_ERROR", e.to_string()))?;
    let rows = db::get_metric_history(&db, &server_id, &metric_type, limit.unwrap_or(120))
        .map_err(|e| AppError::new("DB_ERROR", e.to_string()))?;
    Ok(rows.into_iter().map(|(ts, val)| MetricsHistoryPoint { timestamp: ts, value: val }).collect())
}
