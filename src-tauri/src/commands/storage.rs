use tauri::State;
use crate::models::{AppError, Project, ProjectInput, ServerConfig, ServerConfigInput};
use crate::state::AppState;
use crate::db;

// ─── Servers ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_servers(
    state: State<'_, AppState>,
) -> Result<Vec<ServerConfig>, AppError> {
    let db = state.db.lock().map_err(|e| AppError::new("DB_LOCK_ERROR", e.to_string()))?;
    db::list_servers(&db).map_err(|e| AppError::new("DB_ERROR", e.to_string()))
}

#[tauri::command]
pub async fn add_server(
    state: State<'_, AppState>,
    input: ServerConfigInput,
) -> Result<ServerConfig, AppError> {
    let db = state.db.lock().map_err(|e| AppError::new("DB_LOCK_ERROR", e.to_string()))?;
    db::create_server(&db, input).map_err(|e| AppError::new("DB_ERROR", e.to_string()))
}

#[tauri::command]
pub async fn remove_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<(), AppError> {
    // Primero eliminar de las sesiones SSH
    {
        let mut sessions = state.ssh_sessions.lock().map_err(|e| AppError::new("SESSION_LOCK_ERROR", e.to_string()))?;
        sessions.remove(&server_id);
    }
    
    // Luego eliminar de la base de datos
    let db = state.db.lock().map_err(|e| AppError::new("DB_LOCK_ERROR", e.to_string()))?;
    db::delete_server(&db, &server_id).map_err(|e| AppError::new("DB_ERROR", e.to_string()))
}

// ─── Projects ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_projects(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<Vec<Project>, AppError> {
    let db = state.db.lock().map_err(|e| AppError::new("DB_LOCK_ERROR", e.to_string()))?;
    db::list_projects(&db, &server_id).map_err(|e| AppError::new("DB_ERROR", e.to_string()))
}

#[tauri::command]
pub async fn add_project(
    state: State<'_, AppState>,
    input: ProjectInput,
) -> Result<Project, AppError> {
    let db = state.db.lock().map_err(|e| AppError::new("DB_LOCK_ERROR", e.to_string()))?;
    db::create_project(&db, input).map_err(|e| AppError::new("DB_ERROR", e.to_string()))
}

#[tauri::command]
pub async fn remove_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|e| AppError::new("DB_LOCK_ERROR", e.to_string()))?;
    db::delete_project(&db, &project_id).map_err(|e| AppError::new("DB_ERROR", e.to_string()))
}