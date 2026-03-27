use tauri::State;

use crate::commands::ssh::ssh_exec_cached;
use crate::models::AppError;
use crate::state::AppState;

fn parse_json_lines(output: &str) -> Vec<serde_json::Value> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || !line.starts_with('{') {
                None
            } else {
                serde_json::from_str(line).ok()
            }
        })
        .collect()
}

#[tauri::command]
pub async fn list_docker_containers(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<serde_json::Value, AppError> {
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    let result = tokio::task::spawn_blocking(move || {
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, "docker ps -a --format '{{json .}}'")
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    let res = parse_json_lines(&result.stdout);
    Ok(serde_json::json!(res))
}

#[tauri::command]
pub async fn docker_action(
    state: State<'_, AppState>,
    server_id: String,
    container: String,
    action: String,
) -> Result<String, AppError> {
    if !["start", "stop", "restart"].contains(&action.as_str()) {
        return Err(AppError::new("INVALID_ACTION", "Action must be start, stop, or restart"));
    }
    
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    let result = tokio::task::spawn_blocking(move || {
        let cmd = format!("docker {} {}", action, container);
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, &cmd)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    if result.exit_code != 0 {
        return Err(AppError::new("DOCKER_ERROR", result.stderr));
    }
    Ok(result.stdout)
}

#[tauri::command]
pub async fn list_pm2_processes(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<serde_json::Value, AppError> {
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    let result = tokio::task::spawn_blocking(move || {
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, "pm2 jlist")
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    let out = result.stdout.trim();
    if out.is_empty() {
        return Ok(serde_json::json!([]));
    }
    
    let parsed: serde_json::Value = serde_json::from_str(out)
        .map_err(|e| AppError::new("PARSE_ERROR", format!("Failed parsing pm2 JSON: {}", e)))?;
        
    let empty_arr = vec![];
    let arr = parsed.as_array().unwrap_or(&empty_arr);
    
    let mut flat_list = Vec::new();
    for p in arr {
        let name = p["name"].as_str().unwrap_or("").to_string();
        let pm_id = p["pm_id"].as_i64().unwrap_or(0);
        let pid = p["pid"].as_i64().unwrap_or(0);
        
        let monit = &p["monit"];
        let cpu = monit["cpu"].as_f64().unwrap_or(0.0);
        let memory = monit["memory"].as_i64().unwrap_or(0);
        
        let env = &p["pm2_env"];
        let status = env["status"].as_str().unwrap_or("unknown").to_string();
        let uptime = env["pm_uptime"].as_i64().unwrap_or(0);
        let restarts = env["restart_time"].as_i64().unwrap_or(0);
        
        flat_list.push(serde_json::json!({
            "name": name,
            "pm_id": pm_id,
            "pid": pid,
            "status": status,
            "cpu": cpu,
            "memory": memory,
            "uptime": uptime,
            "restarts": restarts
        }));
    }
    
    Ok(serde_json::json!(flat_list))
}

#[tauri::command]
pub async fn pm2_action(
    state: State<'_, AppState>,
    server_id: String,
    process: String,
    action: String,
) -> Result<String, AppError> {
    if !["start", "stop", "restart", "delete"].contains(&action.as_str()) {
        return Err(AppError::new("INVALID_ACTION", "Action must be start, stop, restart, or delete"));
    }
    
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    let result = tokio::task::spawn_blocking(move || {
        let cmd = format!("pm2 {} {}", action, process);
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, &cmd)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    if result.exit_code != 0 {
        return Err(AppError::new("PM2_ERROR", result.stderr));
    }
    Ok(result.stdout)
}

#[tauri::command]
pub async fn get_docker_logs(
    state: State<'_, AppState>,
    server_id: String,
    container: String,
    lines: Option<u32>,
) -> Result<String, AppError> {
    let n = lines.unwrap_or(200);
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    let result = tokio::task::spawn_blocking(move || {
        let cmd = format!("docker logs --tail {} {} 2>&1", n, container);
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, &cmd)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    Ok(result.stdout)
}

#[tauri::command]
pub async fn get_pm2_logs(
    state: State<'_, AppState>,
    server_id: String,
    process: String,
    lines: Option<u32>,
) -> Result<String, AppError> {
    let n = lines.unwrap_or(200);
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    let result = tokio::task::spawn_blocking(move || {
        let cmd = format!("pm2 logs {} --lines {} --nostream 2>&1", process, n);
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, &cmd)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    Ok(result.stdout)
}

#[derive(serde::Serialize)]
pub struct DeploymentCandidate {
    pub path: String,
    pub name: String,
    pub stack: String,
    pub entry: String,
}

#[tauri::command]
pub async fn discover_deployments(
    state: tauri::State<'_, crate::state::AppState>,
    server_id: String,
) -> Result<Vec<DeploymentCandidate>, crate::models::AppError> {
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();
    
    let script = r#"
for d in ~/*/; do
  if [ -d "$d" ]; then
    name=$(basename "$d")
    if [ -f "${d}ecosystem.config.js" ]; then
      echo "PM2|$d|$name|ecosystem.config.js"
    elif [ -f "${d}ecosystem.config.cjs" ]; then
      echo "PM2|$d|$name|ecosystem.config.cjs"
    elif [ -f "${d}package.json" ]; then
      if grep -q '"start"' "${d}package.json"; then
          echo "NODE|$d|$name|npm run start"
      elif grep -q '"dev"' "${d}package.json"; then
          echo "NODE|$d|$name|npm run dev"
      else
          echo "NODE|$d|$name|npm install"
      fi
    else
      echo "UNKNOWN|$d|$name|"
    fi
  fi
done
"#;

    let result = tokio::task::spawn_blocking(move || {
        crate::commands::ssh::ssh_exec_cached(&db_arc, &sessions_arc, &server_id, script)
    })
    .await
    .map_err(|e| crate::models::AppError::new("TASK_ERROR", format!("{:#}", e)))??;

    let mut candidates = Vec::new();
    for line in result.stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            candidates.push(DeploymentCandidate {
                stack: parts[0].to_string(),
                path: parts[1].to_string(),
                name: parts[2].to_string(),
                entry: parts[3].to_string(),
            });
        }
    }
    
    Ok(candidates)
}

#[tauri::command]
pub async fn deploy_pm2_project(
    state: tauri::State<'_, crate::state::AppState>,
    server_id: String,
    path: String,
    name: String,
    entry: String,
) -> Result<String, crate::models::AppError> {
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();
    
    let script = format!(r#"
cd "{path}"
if [ -f "package.json" ] && [ ! -d "node_modules" ]; then
  npm install
fi

if [[ "{entry}" == "npm run"* ]]; then
  cmd=$(echo "{entry}" | sed 's/npm run //')
  pm2 start npm --name "{name}" -- run "$cmd"
else
  pm2 start "{entry}" --name "{name}"
fi
pm2 save
"#);

    let result = tokio::task::spawn_blocking(move || {
        crate::commands::ssh::ssh_exec_cached(&db_arc, &sessions_arc, &server_id, &script)
    })
    .await
    .map_err(|e| crate::models::AppError::new("TASK_ERROR", format!("{:#}", e)))??;

    if result.exit_code != 0 {
        return Err(crate::models::AppError::new("DEPLOY_FAILED", result.stderr));
    }
    
    Ok(result.stdout)
}
