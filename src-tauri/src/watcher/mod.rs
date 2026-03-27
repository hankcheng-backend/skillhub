use crate::db::models::Agent;
use notify_debouncer_mini::{new_debouncer, Debouncer, DebouncedEventKind};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Emitter;

/// Managed watcher state — stored via `app.manage()` instead of `Box::leak`.
/// The inner `Option` is `None` only if the debouncer failed to initialize.
pub type WatcherState = Arc<Mutex<Option<Debouncer<notify::RecommendedWatcher>>>>;

pub fn start_watching(
    db: Arc<Mutex<Connection>>,
    app_handle: tauri::AppHandle,
) -> Result<WatcherState, crate::error::AppError> {
    let db_clone = db.clone();

    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            if let Ok(events) = events {
                let has_changes = events
                    .iter()
                    .any(|e| matches!(e.kind, DebouncedEventKind::Any));
                if has_changes {
                    let conn = match db_clone.lock() {
                        Ok(c) => c,
                        Err(_) => return, // Inside callback — cannot propagate error
                    };
                    let _ = crate::scanner::scan_all(&conn);
                    let _ = app_handle.emit("skills-changed", ());
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
        let conn = match db.lock() {
            Ok(c) => c,
            Err(_) => {
                // DB lock poisoned at startup — return an empty watcher state
                return Ok(Arc::new(Mutex::new(None)));
            }
        };
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

    Ok(Arc::new(Mutex::new(Some(debouncer))))
}

/// Dynamically register additional paths with the running watcher.
/// Called when an agent is enabled after startup so its skill directory
/// is watched without requiring an app restart (D-11).
///
/// Returns `Ok(())` silently if the watcher has not been initialized.
pub fn add_paths(
    watcher_state: &WatcherState,
    paths: &[std::path::PathBuf],
) -> Result<(), crate::error::AppError> {
    let mut guard = watcher_state
        .lock()
        .map_err(|e| crate::error::AppError::Internal(format!("Watcher lock poisoned: {}", e)))?;
    let Some(ref mut debouncer) = *guard else {
        return Ok(()); // Watcher not initialized — no-op
    };
    for path in paths {
        if path.exists() {
            let _ = debouncer
                .watcher()
                .watch(path, notify::RecursiveMode::NonRecursive);
        }
    }
    Ok(())
}
