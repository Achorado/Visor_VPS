use tauri::State;
use serde_json::json;

use crate::commands::ssh::ssh_exec_cached;
use crate::models::AppError;
use crate::state::AppState;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NginxConfig {
    pub name: String,
    pub enabled: bool,
    pub content: String,
}

#[tauri::command]
pub async fn list_nginx_configs(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<Vec<NginxConfig>, AppError> {
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    // Command returns the file name followed by TAB followed by CONTENT for sites-available 
    // and we also check which ones exist in sites-enabled.
    // To do this simply, we will list sites-available, list sites-enabled.
    let script = r#"
#!/bin/bash
AVAILABLE=$(ls -1 /etc/nginx/sites-available 2>/dev/null || echo "")
ENABLED=$(ls -1 /etc/nginx/sites-enabled 2>/dev/null || echo "")

echo "===AVAILABLE==="
echo "$AVAILABLE"
echo "===ENABLED==="
echo "$ENABLED"
"#;

    let result = tokio::task::spawn_blocking(move || {
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, script)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    let out = result.stdout;
    let available_str = out.split("===ENABLED===").next().unwrap_or("").replace("===AVAILABLE===", "");
    let enabled_str = out.split("===ENABLED===").nth(1).unwrap_or("");
    
    let mut enabled_set = std::collections::HashSet::new();
    for line in enabled_str.lines() {
        let name = line.trim();
        if !name.is_empty() {
            enabled_set.insert(name.to_string());
        }
    }

    let mut configs = Vec::new();
    for line in available_str.lines() {
        let name = line.trim();
        if !name.is_empty() {
            // We fetch the contents individually, or just load them here?
            // Since there may be large files, let's just fetch them in a separate command, 
            // OR we can fetch them all in one go. Fetching all is fine if small count. 
            // Actually, for list, we just return empty content, and load content on demand!
            configs.push(NginxConfig {
                name: name.to_string(),
                enabled: enabled_set.contains(name),
                content: String::new(),
            });
        }
    }
    
    // Sort array: enabled first
    configs.sort_by(|a, b| b.enabled.cmp(&a.enabled).then(a.name.cmp(&b.name)));

    Ok(configs)
}

#[tauri::command]
pub async fn get_nginx_config_content(
    state: State<'_, AppState>,
    server_id: String,
    name: String,
) -> Result<String, AppError> {
    // Basic security: avoid path traversal
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(AppError::new("INVALID_NAME", "Invalid module name"));
    }
    
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    let result = tokio::task::spawn_blocking(move || {
        let cmd = format!("cat /etc/nginx/sites-available/{}", name);
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, &cmd)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    if result.exit_code != 0 {
        return Err(AppError::new("NGINX_ERROR", result.stderr));
    }

    Ok(result.stdout)
}

#[tauri::command]
pub async fn save_nginx_config(
    state: State<'_, AppState>,
    server_id: String,
    name: String,
    content: String,
) -> Result<bool, AppError> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(AppError::new("INVALID_NAME", "Invalid module name"));
    }
    
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    // We write text to a temp file, then sudo mv
    let script = format!(
        r#"
cat << 'EOF' > /tmp/nginx_{}.conf
{}
EOF
sudo mv /tmp/nginx_{}.conf /etc/nginx/sites-available/{}
"#,
        name, content.replace('\'', "'\\''"), name, name
    );

    let result = tokio::task::spawn_blocking(move || {
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, &script)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    if result.exit_code != 0 {
        return Err(AppError::new("NGINX_ERROR", result.stderr));
    }

    Ok(true)
}

#[tauri::command]
pub async fn toggle_nginx_config(
    state: State<'_, AppState>,
    server_id: String,
    name: String,
    enable: bool,
) -> Result<bool, AppError> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(AppError::new("INVALID_NAME", "Invalid module name"));
    }
    
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    let script = if enable {
        format!("sudo ln -sf /etc/nginx/sites-available/{} /etc/nginx/sites-enabled/{}", name, name)
    } else {
        format!("sudo rm -f /etc/nginx/sites-enabled/{}", name)
    };

    let result = tokio::task::spawn_blocking(move || {
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, &script)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    if result.exit_code != 0 {
        return Err(AppError::new("NGINX_ERROR", result.stderr));
    }

    Ok(true)
}

#[tauri::command]
pub async fn reload_nginx(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<String, AppError> {
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    let result = tokio::task::spawn_blocking(move || {
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, "sudo nginx -t && sudo systemctl reload nginx 2>&1")
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    if result.exit_code != 0 {
        return Err(AppError::new("NGINX_ERROR", result.stderr.clone() + &result.stdout));
    }

    Ok(result.stdout)
}

#[tauri::command]
pub async fn delete_nginx_config(
    state: State<'_, AppState>,
    server_id: String,
    name: String,
) -> Result<bool, AppError> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(AppError::new("INVALID_NAME", "Invalid module name"));
    }
    
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();

    let script = format!(
        "sudo rm -f /etc/nginx/sites-enabled/{} && sudo rm -f /etc/nginx/sites-available/{}",
        name, name
    );

    let result = tokio::task::spawn_blocking(move || {
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, &script)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", e.to_string()))?
    .map_err(|e| AppError::new("SSH_ERROR", e.to_string()))?;

    if result.exit_code != 0 {
        return Err(AppError::new("NGINX_ERROR", result.stderr));
    }

    Ok(true)
}
