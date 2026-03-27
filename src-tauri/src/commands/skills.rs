use crate::db::models::Skill;
use crate::error::AppError;
use crate::scanner;
use crate::services::skills as skills_svc;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tauri::State;

pub type DbState = Arc<Mutex<Connection>>;

#[tauri::command]
pub fn list_skills(db: State<'_, DbState>) -> Result<Vec<Skill>, AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    skills_svc::list_skills(&conn)
}

#[tauri::command]
pub fn scan_skills(db: State<'_, DbState>) -> Result<Vec<Skill>, AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    scanner::scan_all(&conn)
}

#[tauri::command]
pub fn delete_skill(db: State<'_, DbState>, skill_id: String) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    skills_svc::delete_skill(&conn, &skill_id)
}

#[tauri::command]
pub fn get_skill_content(db: State<'_, DbState>, skill_id: String) -> Result<String, AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    skills_svc::get_skill_content(&conn, &skill_id)
}

#[tauri::command]
pub fn update_skill_meta(
    db: State<'_, DbState>,
    skill_id: String,
    tags: Option<String>,
    notes: Option<String>,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    skills_svc::update_skill_meta(&conn, &skill_id, tags.as_deref(), notes.as_deref())
}
