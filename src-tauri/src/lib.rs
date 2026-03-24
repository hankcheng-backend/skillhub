mod commands;
mod db;
mod error;
mod mcp;
mod remote;
mod scanner;
mod sync;
mod watcher;

use std::sync::{Arc, Mutex};
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = dirs::data_dir()
        .expect("no data dir")
        .join("skillhub")
        .join("skillhub.db");

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("cannot create data dir");
    }

    let (conn, is_fresh) = db::init_db(&db_path).expect("failed to init DB");

    // First launch: auto-detect installed agents by checking config dirs
    if is_fresh {
        let home = dirs::home_dir().expect("cannot find home dir");
        for agent_id in &["claude", "codex", "gemini"] {
            // Default all built-in agents to disabled on first launch.
            let _ = db::models::Agent::update(&conn, agent_id, false, None);
            let config_dir = home.join(format!(".{}", agent_id));
            if config_dir.is_dir() {
                let _ = db::models::Agent::update(&conn, agent_id, true, None);
            }
        }
    }

    let db = Arc::new(Mutex::new(conn));

    // Initial scan
    {
        let conn_guard = db.lock().unwrap();
        let _ = scanner::scan_all(&conn_guard);
    }

    let db_for_watcher = db.clone();
    let db_for_mcp = db.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(db)
        .setup(move |app| {
            let handle = app.handle().clone();
            let watcher = watcher::start_watching(db_for_watcher, handle)?;
            Box::leak(Box::new(watcher));

            // Read port from DB (braces required — MutexGuard is !Send, must drop before tokio::spawn)
            let mcp_port: u16 = {
                let conn = db_for_mcp.lock().unwrap();
                conn.query_row(
                    "SELECT value FROM app_config WHERE key = 'mcp_port'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(9800)
            }; // MutexGuard dropped here

            tauri::async_runtime::spawn(async move {
                if let Err(e) = mcp::start_server(db_for_mcp, mcp_port).await {
                    log::error!("MCP server error: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::skills::list_skills,
            commands::skills::scan_skills,
            commands::skills::delete_skill,
            commands::skills::get_skill_content,
            commands::skills::update_skill_meta,
            commands::sync_cmd::sync_skill,
            commands::sync_cmd::unsync_skill,
            commands::settings::get_agents,
            commands::settings::update_agent,
            commands::settings::get_agent_versions,
            commands::settings::get_latest_versions,
            commands::sources::list_sources,
            commands::sources::add_source,
            commands::sources::remove_source,
            commands::sources::browse_source,
            commands::sources::get_remote_skill_content,
            commands::settings::get_config,
            commands::settings::set_config,
            commands::settings::get_autostart,
            commands::settings::set_autostart,
            commands::settings::check_agent_dir,
            commands::settings::pick_agent_dir,
            commands::install::install_skill,
            commands::upload::upload_skill,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
