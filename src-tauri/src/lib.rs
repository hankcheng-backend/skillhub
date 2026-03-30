mod commands;
pub mod db;
pub mod error;
pub mod mcp;
mod remote;
pub mod scanner;
pub mod services;
mod sync;
mod watcher;

use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_updater::UpdaterExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = dirs::data_dir()
        .expect("no data dir")
        .join("skillhub")
        .join("skillhub.db");

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("cannot create data dir");
    }

    let (conn, is_fresh, migration_error) = match db::init_db(&db_path) {
        Ok((conn, is_fresh)) => (conn, is_fresh, None),
        Err(crate::error::AppError::Migration(msg)) => {
            log::error!("Migration failed, attempting fallback: {}", msg);
            // init_db already restored the backup; open the restored DB directly
            let conn = rusqlite::Connection::open(&db_path).expect("failed to open restored DB");
            conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
                .expect("failed to set pragmas on restored DB");
            (conn, false, Some(msg))
        }
        Err(e) => panic!("failed to init DB: {}", e),
    };

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

    // Initial scan — non-fatal if lock is poisoned at startup
    if let Ok(conn_guard) = db.lock() {
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(db)
        .setup(move |app| {
            // D-11: Show error dialog if migration failed
            if let Some(ref msg) = migration_error {
                use tauri_plugin_dialog::DialogExt;
                app.dialog()
                    .message(msg)
                    .title("SkillHub - Database Migration Error")
                    .blocking_show();
            }

            let handle = app.handle().clone();
            let watcher_state = watcher::start_watching(db_for_watcher, handle)?;
            app.manage(watcher_state);

            // Read port from DB (braces required — MutexGuard is !Send, must drop before tokio::spawn)
            let mcp_port: u16 = if let Ok(conn) = db_for_mcp.lock() {
                conn.query_row(
                    "SELECT value FROM app_config WHERE key = 'mcp_port'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(9800)
            } else {
                9800
            }; // MutexGuard dropped here

            tauri::async_runtime::spawn(async move {
                if let Err(e) = mcp::start_server(db_for_mcp, mcp_port).await {
                    log::error!("MCP server error: {}", e);
                }
            });

            // Check for updates on startup
            let update_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let updater = match update_handle.updater() {
                    Ok(u) => u,
                    Err(e) => {
                        log::error!("Failed to create updater: {}", e);
                        return;
                    }
                };
                match updater.check().await {
                    Ok(Some(update)) => {
                        log::info!(
                            "Update available: {} -> {}",
                            update.current_version,
                            update.version
                        );

                        // Ask user before updating
                        let msg = format!(
                            "A new version {} is available (current: {}). Update now?",
                            update.version, update.current_version
                        );
                        use tauri_plugin_dialog::MessageDialogButtons;
                        let confirmed = update_handle
                            .dialog()
                            .message(msg)
                            .title("SkillHub Update")
                            .buttons(MessageDialogButtons::OkCancelCustom(
                                "Update".into(),
                                "Later".into(),
                            ))
                            .blocking_show();

                        if confirmed {
                            if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
                                log::error!("Failed to install update: {}", e);
                                update_handle
                                    .dialog()
                                    .message(format!("Update failed: {}", e))
                                    .title("SkillHub Update")
                                    .blocking_show();
                            }
                        }
                    }
                    Ok(None) => log::info!("No update available"),
                    Err(e) => log::error!("Update check failed: {}", e),
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
            commands::sources::update_source_token,
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
