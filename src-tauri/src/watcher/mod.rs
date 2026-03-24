use crate::db::models::Agent;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Emitter;

pub fn start_watching(
    db: Arc<Mutex<Connection>>,
    app_handle: tauri::AppHandle,
) -> Result<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>, crate::error::AppError> {
    let db_clone = db.clone();

    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            if let Ok(events) = events {
                let has_changes = events
                    .iter()
                    .any(|e| matches!(e.kind, DebouncedEventKind::Any));
                if has_changes {
                    if let Ok(conn) = db_clone.lock() {
                        let _ = crate::scanner::scan_all(&conn);
                        let _ = app_handle.emit("skills-changed", ());
                    }
                }
            }
        },
    )
    .map_err(|e| {
        crate::error::AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    {
        let conn = db.lock().unwrap();
        let agents = Agent::enabled(&conn)?;
        for agent in &agents {
            let dir = agent.resolved_skill_dir();
            if dir.exists() {
                let _ = debouncer
                    .watcher()
                    .watch(&dir, notify::RecursiveMode::NonRecursive);
            }
        }
    }

    Ok(debouncer)
}
