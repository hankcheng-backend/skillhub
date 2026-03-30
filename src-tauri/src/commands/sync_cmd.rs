use crate::commands::skills::DbState;
use crate::error::AppError;
use crate::services::sync as sync_svc;
use tauri::State;

#[tauri::command]
pub fn sync_skill(
    db: State<'_, DbState>,
    skill_id: String,
    target_agent: String,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    sync_svc::sync_skill(&conn, &skill_id, &target_agent)
}

#[tauri::command]
pub fn unsync_skill(
    db: State<'_, DbState>,
    skill_id: String,
    agent: String,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    sync_svc::unsync_skill(&conn, &skill_id, &agent)
}
