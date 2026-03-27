// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod crypto;
mod db;
mod models;
mod state;

use rusqlite::Connection;
use tauri::Manager;



pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");
            let db_path = app_dir.join("visor_vps.db");

            let conn = Connection::open(&db_path)
                .expect("Failed to open SQLite database");
            db::init_db(&conn).expect("Failed to initialize database schema");

            let app_state = state::AppState::new(conn);
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::storage::list_servers,
            commands::storage::add_server,
            commands::storage::remove_server,
            commands::storage::list_projects,
            commands::storage::add_project,
            commands::storage::remove_project,
            commands::ssh::test_connection,
            commands::ssh::connect_server,
            commands::ssh::disconnect_server,
            commands::ssh::exec_command,
            commands::metrics::get_metrics,
            commands::metrics::get_metric_history,
            commands::metrics::get_connected_users,
            commands::metrics::save_base64_file,
            commands::services::list_services,
            commands::services::start_service,
            commands::services::stop_service,
            commands::services::restart_service,
            commands::services::get_service_logs,
            commands::containers::list_docker_containers,
            commands::containers::docker_action,
            commands::containers::list_pm2_processes,
            commands::containers::pm2_action,
            commands::containers::get_docker_logs,
            commands::containers::get_pm2_logs,
            commands::containers::discover_deployments,
            commands::containers::deploy_pm2_project,
            commands::nginx::list_nginx_configs,
            commands::nginx::get_nginx_config_content,
            commands::nginx::save_nginx_config,
            commands::nginx::toggle_nginx_config,
            commands::nginx::reload_nginx,
            commands::nginx::delete_nginx_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}