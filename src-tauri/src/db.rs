use anyhow::Result;
use rusqlite::{Connection, params};
use chrono::Utc;
use uuid::Uuid;

use crate::models::{Project, ProjectInput, ServerConfig, ServerConfigInput};
use crate::crypto::{encrypt_password, decrypt_password};

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;

        CREATE TABLE IF NOT EXISTS servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            host TEXT NOT NULL,
            port INTEGER NOT NULL DEFAULT 22,
            username TEXT NOT NULL,
            encrypted_password TEXT,
            key_path TEXT,
            color TEXT NOT NULL DEFAULT '#6366f1',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            server_id TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            service_name TEXT,
            color TEXT NOT NULL DEFAULT '#22c55e',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS metric_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            server_id TEXT NOT NULL,
            metric_type TEXT NOT NULL,  -- 'cpu', 'ram', 'net_rx', 'net_tx', 'disk'
            value REAL NOT NULL,
            timestamp INTEGER NOT NULL  -- unix ms
        );

        CREATE INDEX IF NOT EXISTS idx_metric_history_server_time
            ON metric_history(server_id, metric_type, timestamp DESC);
        "#,
    )?;
    Ok(())
}

// ─── Servers ─────────────────────────────────────────────────────────────────

pub fn create_server(conn: &Connection, input: ServerConfigInput) -> Result<ServerConfig> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let encrypted = input
        .password
        .as_deref()
        .map(encrypt_password)
        .transpose()?;

    conn.execute(
        "INSERT INTO servers (id, name, host, port, username, encrypted_password, key_path, color, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id, input.name, input.host, input.port, input.username,
            encrypted, input.key_path, input.color, now.to_rfc3339()
        ],
    )?;

    Ok(ServerConfig {
        id,
        name: input.name,
        host: input.host,
        port: input.port,
        username: input.username,
        encrypted_password: encrypted,
        key_path: input.key_path,
        color: input.color,
        created_at: now,
    })
}

pub fn list_servers(conn: &Connection) -> Result<Vec<ServerConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, host, port, username, encrypted_password, key_path, color, created_at
         FROM servers ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ServerConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            host: row.get(2)?,
            port: row.get(3)?,
            username: row.get(4)?,
            encrypted_password: row.get(5)?,
            key_path: row.get(6)?,
            color: row.get(7)?,
            created_at: {
                let s: String = row.get(8)?;
                s.parse().unwrap_or_else(|_| Utc::now())
            },
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn delete_server(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM servers WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_server_password(conn: &Connection, id: &str) -> Result<Option<String>> {
    let encrypted: Option<String> = conn.query_row(
        "SELECT encrypted_password FROM servers WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    match encrypted {
        Some(enc) => Ok(Some(decrypt_password(&enc)?)),
        None => Ok(None),
    }
}

// ─── Projects ────────────────────────────────────────────────────────────────

pub fn create_project(conn: &Connection, input: ProjectInput) -> Result<Project> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    conn.execute(
        "INSERT INTO projects (id, server_id, name, path, service_name, color, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, input.server_id, input.name, input.path, input.service_name, input.color, now.to_rfc3339()],
    )?;
    Ok(Project {
        id,
        server_id: input.server_id,
        name: input.name,
        path: input.path,
        service_name: input.service_name,
        color: input.color,
        created_at: now,
    })
}

pub fn list_projects(conn: &Connection, server_id: &str) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, server_id, name, path, service_name, color, created_at
         FROM projects WHERE server_id = ?1 ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(params![server_id], |row| {
        Ok(Project {
            id: row.get(0)?,
            server_id: row.get(1)?,
            name: row.get(2)?,
            path: row.get(3)?,
            service_name: row.get(4)?,
            color: row.get(5)?,
            created_at: {
                let s: String = row.get(6)?;
                s.parse().unwrap_or_else(|_| Utc::now())
            },
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn delete_project(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

// ─── Metric History ──────────────────────────────────────────────────────────

pub fn insert_metric(
    conn: &Connection,
    server_id: &str,
    metric_type: &str,
    value: f64,
    timestamp_ms: i64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO metric_history (server_id, metric_type, value, timestamp) VALUES (?1, ?2, ?3, ?4)",
        params![server_id, metric_type, value, timestamp_ms],
    )?;
    // Keep only last 2000 points per server+metric
    conn.execute(
        "DELETE FROM metric_history WHERE server_id = ?1 AND metric_type = ?2
         AND id NOT IN (
             SELECT id FROM metric_history WHERE server_id = ?1 AND metric_type = ?2
             ORDER BY timestamp DESC LIMIT 2000
         )",
        params![server_id, metric_type],
    )?;
    Ok(())
}

pub fn get_metric_history(
    conn: &Connection,
    server_id: &str,
    metric_type: &str,
    limit: usize,
) -> Result<Vec<(i64, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, value FROM (
           SELECT timestamp, value FROM metric_history
           WHERE server_id = ?1 AND metric_type = ?2
           ORDER BY timestamp DESC
           LIMIT ?3
         ) ORDER BY timestamp ASC",
    )?;
    let rows = stmt.query_map(params![server_id, metric_type, limit as i64], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
