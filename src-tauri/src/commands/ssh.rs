use anyhow::{Result, Context};
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;
use tauri::State;
use chrono::Utc;

use crate::models::{AppError, ConnectionStatus, ExecResult, ServerConnectionState};
use crate::state::AppState;
use crate::db;

pub fn ssh_exec(
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    key_path: Option<&str>,
    command: &str,
) -> Result<ExecResult> {
    let addr = format!("{}:{}", host, port);
    let tcp = TcpStream::connect(&addr).context("Failed to connect to TCP socket")?; // standard connect para evitar problemas de socket state en Windows
    tcp.set_nodelay(true)?; // Desactivar Nagle's algorithm

    let mut session = Session::new().context("Failed to create SSH session")?;
    session.set_tcp_stream(tcp);
    session.set_timeout(30000); // 30 segundos usando el timeout interno de libssh2
    session.handshake().context("SSH Handshake failed")?;

    if let Some(kp) = key_path {
        // Intento 1: Usar SSH-Agent si está disponible (la mejor forma en Windows para Ed25519)
        let agent_success = session.userauth_agent(username).is_ok();

        // Intento 2: Usar archivo directamente si Agent falló o no estaba
        if !agent_success {
            let pass = password.filter(|p| !p.is_empty()); 
            let pub_path_str = format!("{}.pub", kp);
            let pub_path = std::path::Path::new(&pub_path_str);
            let pubkey_opt = if pub_path.exists() { Some(pub_path) } else { None };

            session.userauth_pubkey_file(username, pubkey_opt, std::path::Path::new(kp), pass.as_deref())
                .context("SSH Key auto-auth and direct-file auth failed (Ed25519 is often unsupported on Windows libssh2 natively without ssh-agent)")?;
        }
    } else if let Some(pwd) = password.filter(|p| !p.is_empty()) {
        session.userauth_password(username, pwd)
            .context("SSH Password authentication failed")?;
    }

    if !session.authenticated() {
        return Err(anyhow::anyhow!("SSH authentication failed"));
    }

    let mut channel = session.channel_session().context("Failed to open channel")?;
    channel.exec(command).context("Failed to execute command")?;

    let mut stdout = String::new();
    channel.read_to_string(&mut stdout)?;

    let mut stderr = String::new();
    channel.stderr().read_to_string(&mut stderr)?;

    channel.wait_close()?;
    let exit_code = channel.exit_status()?;
    Ok(ExecResult { stdout, stderr, exit_code })
}

pub fn get_cached_session(
    db_arc: &std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
    sessions_arc: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, crate::state::SshSession>>>,
    server_id: &str,
) -> Result<std::sync::Arc<std::sync::Mutex<Session>>, AppError> {
    {
        let sessions = sessions_arc.lock().map_err(|e| AppError::new("SESSION_LOCK_ERROR", e.to_string()))?;
        if let Some(s) = sessions.get(server_id) {
            return Ok(s.session.clone());
        }
    }

    let (server, password) = {
        let db_guard = db_arc.lock().map_err(|e| AppError::new("DB_LOCK_ERROR", e.to_string()))?;
        let servers = db::list_servers(&db_guard).map_err(|e| AppError::new("DB_ERROR", e.to_string()))?;
        let server = servers.iter().find(|s| s.id == server_id)
            .ok_or_else(|| AppError::new("NOT_FOUND", "Server not found"))?.clone();
        let password = db::get_server_password(&db_guard, server_id)
            .map_err(|e| AppError::new("CRYPTO_ERROR", e.to_string()))?;
        (server, password)
    };

    let addr = format!("{}:{}", server.host, server.port);
    let tcp = TcpStream::connect(&addr).map_err(|e| AppError::new("NETWORK_ERROR", format!("Failed to connect to TCP socket: {}", e)))?;
    tcp.set_nodelay(true).unwrap_or(());

    let mut session = Session::new().map_err(|e| AppError::new("SSH_ERROR", format!("Failed to create SSH session: {}", e)))?;
    session.set_tcp_stream(tcp);
    session.set_timeout(30000);
    session.handshake().map_err(|e| AppError::new("SSH_ERROR", format!("SSH Handshake failed: {}", e)))?;

    if let Some(kp) = &server.key_path {
        let agent_success = session.userauth_agent(&server.username).is_ok();
        if !agent_success {
            let pass = password.as_deref().filter(|p| !p.is_empty()); 
            let pub_path_str = format!("{}.pub", kp);
            let pub_path = std::path::Path::new(&pub_path_str);
            let pubkey_opt = if pub_path.exists() { Some(pub_path) } else { None };

            session.userauth_pubkey_file(&server.username, pubkey_opt, std::path::Path::new(kp), pass)
                .map_err(|e| AppError::new("AUTH_ERROR", format!("SSH Key auth failed: {}", e)))?;
        }
    } else if let Some(pwd) = password.as_deref().filter(|p| !p.is_empty()) {
        session.userauth_password(&server.username, pwd)
            .map_err(|e| AppError::new("AUTH_ERROR", format!("SSH Password authentication failed: {}", e)))?;
    }

    if !session.authenticated() {
        return Err(AppError::new("AUTH_ERROR", "SSH authentication failed"));
    }

    let session_arc = std::sync::Arc::new(std::sync::Mutex::new(session));

    let mut sessions = sessions_arc.lock().map_err(|e| AppError::new("SESSION_LOCK_ERROR", e.to_string()))?;
    sessions.insert(
        server_id.to_string(),
        crate::state::SshSession {
            server_id: server_id.to_string(),
            host: server.host,
            username: server.username,
            connected_at: Utc::now(),
            session: session_arc.clone(),
        },
    );

    Ok(session_arc)
}

pub fn ssh_exec_cached(
    db_arc: &std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
    sessions_arc: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, crate::state::SshSession>>>,
    server_id: &str,
    command: &str,
) -> Result<ExecResult, AppError> {
    for attempt in 1..=2 {
        let session_arc = get_cached_session(db_arc, sessions_arc, server_id)?;
        
        // Ejecutamos el comando
        let result = (|| -> anyhow::Result<ExecResult> {
            let session = session_arc.lock().unwrap();
            let mut channel = session.channel_session()?;
            channel.exec(command)?;
            
            let mut stdout = String::new();
            channel.read_to_string(&mut stdout)?;
            
            let mut stderr = String::new();
            channel.stderr().read_to_string(&mut stderr)?;
            
            channel.wait_close()?;
            let exit_code = channel.exit_status()?;
            
            Ok(ExecResult { stdout, stderr, exit_code })
        })();

        match result {
            Ok(r) => return Ok(r),
            Err(e) => {
                if attempt == 1 {
                    // Si falla el primer intento, asumimos que la conexión TCP/SSH se cerró/rompió. 
                    // Eliminamos la sesión de la caché para forzar una re-conexión en attempt 2.
                    let mut s = sessions_arc.lock().unwrap();
                    s.remove(server_id);
                    continue;
                } else {
                    return Err(AppError::new("SSH_ERROR", format!("Cached exec failed: {}", e)));
                }
            }
        }
    }
    Err(AppError::new("SSH_ERROR", "Unreachable"))
}

#[tauri::command]
pub async fn test_connection(
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    key_path: Option<String>,
) -> Result<bool, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        ssh_exec(
            &host, port, &username,
            password.as_deref(), key_path.as_deref(),
            "echo visor-vps-ok",
        )
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", format!("{:#}", e)))?;

    match result {
        Ok(r) => Ok(r.stdout.trim() == "visor-vps-ok"),
        Err(e) => Err(AppError::new("SSH_ERROR", format!("{:#}", e))),
    }
}

#[tauri::command]
pub async fn connect_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ServerConnectionState, AppError> {
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();
    let sid = server_id.clone();
    let db_arc_2 = db_arc.clone();
    let sid_2 = sid.clone();

    // Ensure session is open
    tokio::task::spawn_blocking(move || {
        ssh_exec_cached(&db_arc, &sessions_arc, &sid, "echo connected")
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", format!("{:#}", e)))??;

    // Measure raw TCP latency to get accurate true ping
    let latency_ms = tokio::task::spawn_blocking(move || {
        let (host, port) = {
            let db_guard = db_arc_2.lock().unwrap();
            let servers = crate::db::list_servers(&db_guard).unwrap_or_default();
            if let Some(s) = servers.iter().find(|s| s.id == sid_2) {
                (s.host.clone(), s.port)
            } else {
                return 0;
            }
        };
        let start = std::time::Instant::now();
        let addr = format!("{}:{}", host, port);
        if let Ok(mut addrs) = std::net::ToSocketAddrs::to_socket_addrs(&addr) {
            if let Some(addr) = addrs.next() {
                let _ = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(3));
            }
        }
        start.elapsed().as_millis() as u64
    })
    .await
    .unwrap_or(0);

    Ok(ServerConnectionState {
        server_id: server_id,
        status: ConnectionStatus::Connected,
        latency_ms: Some(latency_ms),
        last_seen: Some(Utc::now()),
    })
}

#[tauri::command]
pub async fn disconnect_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<(), AppError> {
    let mut sessions = state.ssh_sessions.lock().map_err(|e| AppError::new("SESSION_LOCK_ERROR", e.to_string()))?;
    sessions.remove(&server_id);
    Ok(())
}

#[tauri::command]
pub async fn exec_command(
    state: State<'_, AppState>,
    server_id: String,
    command: String,
) -> Result<ExecResult, AppError> {
    let db_arc = state.inner().db.clone();
    let sessions_arc = state.inner().ssh_sessions.clone();
    
    let result = tokio::task::spawn_blocking(move || {
        ssh_exec_cached(&db_arc, &sessions_arc, &server_id, &command)
    })
    .await
    .map_err(|e| AppError::new("TASK_ERROR", format!("{:#}", e)))??;

    Ok(result)
}