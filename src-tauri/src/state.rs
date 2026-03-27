use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct SshSession {
    pub server_id: String,
    pub host: String,
    pub username: String,
    pub connected_at: DateTime<Utc>,
    pub session: Arc<Mutex<ssh2::Session>>,
}

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub ssh_sessions: Arc<Mutex<HashMap<String, SshSession>>>,
}

impl AppState {
    pub fn new(conn: Connection) -> Self {
        Self {
            db: Arc::new(Mutex::new(conn)),
            ssh_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}