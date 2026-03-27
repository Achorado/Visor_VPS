use tauri::State;
use serde_json;
use crate::models::{AppError, ServiceInfo, ServiceStatus, ExecResult};
use crate::state::AppState;
use crate::db;
use crate::commands::ssh::ssh_exec_cached;

fn parse_service_status(status_str: &str) -> ServiceStatus {
    match status_str {
        "active" => ServiceStatus::Active,
        "inactive" => ServiceStatus::Inactive,
        "failed" => ServiceStatus::Failed,
        "activating" => ServiceStatus::Activating,
        "deactivating" => ServiceStatus::Deactivating,
        _ => ServiceStatus::Unknown,
    }
}

fn parse_services_output(output: &str) -> Vec<ServiceInfo> {
    let mut services = Vec::new();
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.splitn(5, '\t').collect();
        if parts.len() >= 4 {
            let name = parts[0].trim().trim_end_matches(".service").to_string();
            let status_str = parts[2].trim();
            let sub_state = parts[3].trim().to_string();
            let description = parts.get(4).unwrap_or(&"").trim().to_string();
            services.push(ServiceInfo {
                name,
                description,
                status: parse_service_status(status_str),
                sub_state,
                enabled: true, // simplified; could run systemctl is-enabled separately
                pid: None,
                memory_kb: None,
            });
        }
    }
    services
}

async fn with_server_ssh(
    state: &State<'_, AppState>,
    server_id: &str,
    cmd: String,
) -> Result<ExecResult, AppError>
{
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();
    let sid = server_id.to_string();

    let result = tokio::task::spawn_blocking(move || {
        ssh_exec_cached(&db_arc, &sessions_arc, &sid, &cmd)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))??;
    Ok(result)
}

#[tauri::command]
pub async fn list_services(
    state: State<'_, AppState>,
    server_id: String,
    filter: Option<String>,
) -> Result<Vec<ServiceInfo>, AppError> {
    let filter_str = filter.unwrap_or_default();
    let cmd = format!(
        r#"systemctl list-units --type=service --all --no-pager --no-legend \
           --output=json 2>/dev/null || \
           systemctl list-units --type=service --all --no-pager \
             --output=cat -l 2>/dev/null | head -100"#
    );
    let result = with_server_ssh(&state, &server_id, cmd).await?;

    // Try JSON parse first, fall back to text parsing
    let out = result.stdout.trim();
    if out.starts_with('[') {
        // JSON format
        let items: Vec<serde_json::Value> = serde_json::from_str(out)
            .unwrap_or_default();
        let services: Vec<ServiceInfo> = items.iter().filter_map(|v| {
            let name = v["unit"].as_str()?.trim_end_matches(".service").to_string();
            if !filter_str.is_empty() && !name.contains(&filter_str) { return None; }
            Some(ServiceInfo {
                name,
                description: v["description"].as_str().unwrap_or("").to_string(),
                status: parse_service_status(v["active"].as_str().unwrap_or("unknown")),
                sub_state: v["sub"].as_str().unwrap_or("").to_string(),
                enabled: v["unit_file_state"].as_str().map(|s| s == "enabled").unwrap_or(false),
                pid: None,
                memory_kb: None,
            })
        }).collect();
        return Ok(services);
    }

    let parsed = parse_services_output(out);
    let filtered = if filter_str.is_empty() {
        parsed
    } else {
        parsed.into_iter().filter(|s| s.name.contains(&filter_str)).collect()
    };
    Ok(filtered)
}

#[tauri::command]
pub async fn start_service(
    state: State<'_, AppState>,
    server_id: String,
    service_name: String,
) -> Result<bool, AppError> {
    let cmd = format!("sudo systemctl start {}.service 2>&1 && echo OK", service_name);
    let result = with_server_ssh(&state, &server_id, cmd).await?;
    Ok(result.stdout.contains("OK") || result.exit_code == 0)
}

#[tauri::command]
pub async fn stop_service(
    state: State<'_, AppState>,
    server_id: String,
    service_name: String,
) -> Result<bool, AppError> {
    let cmd = format!("sudo systemctl stop {}.service 2>&1 && echo OK", service_name);
    let result = with_server_ssh(&state, &server_id, cmd).await?;
    Ok(result.stdout.contains("OK") || result.exit_code == 0)
}

#[tauri::command]
pub async fn restart_service(
    state: State<'_, AppState>,
    server_id: String,
    service_name: String,
) -> Result<bool, AppError> {
    let cmd = format!("sudo systemctl restart {}.service 2>&1 && echo OK", service_name);
    let result = with_server_ssh(&state, &server_id, cmd).await?;
    Ok(result.stdout.contains("OK") || result.exit_code == 0)
}

#[tauri::command]
pub async fn get_service_logs(
    state: State<'_, AppState>,
    server_id: String,
    service_name: String,
    lines: Option<u32>,
) -> Result<String, AppError> {
    let n = lines.unwrap_or(200);
    let cmd = format!("journalctl -u {}.service -n {} --no-pager 2>&1", service_name, n);
    let result = with_server_ssh(&state, &server_id, cmd).await?;
    Ok(result.stdout)
}
