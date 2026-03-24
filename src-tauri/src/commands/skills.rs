use crate::db::models::Skill;
use crate::error::AppError;
use crate::scanner;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tauri::State;

pub type DbState = Arc<Mutex<Connection>>;

#[tauri::command]
pub fn list_skills(db: State<'_, DbState>) -> Result<Vec<Skill>, AppError> {
    let conn = db.lock().unwrap();
    Skill::all_with_syncs(&conn).map_err(AppError::from)
}

#[tauri::command]
pub fn scan_skills(db: State<'_, DbState>) -> Result<Vec<Skill>, AppError> {
    let conn = db.lock().unwrap();
    scanner::scan_all(&conn)
}

#[tauri::command]
pub fn delete_skill(db: State<'_, DbState>, skill_id: String) -> Result<(), AppError> {
    let conn = db.lock().unwrap();
    let skills = Skill::all_with_syncs(&conn)?;
    let skill = skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| AppError::NotFound(format!("Skill not found: {}", skill_id)))?;

    for synced_agent in &skill.synced_to {
        let agents = crate::db::models::Agent::all(&conn)?;
        if let Some(agent) = agents.iter().find(|a| &a.id == synced_agent) {
            let link_path = agent.resolved_skill_dir().join(&skill.folder_name);
            let _ = crate::sync::remove_sync_link(&link_path);
        }
        crate::db::models::SkillSync::delete(&conn, &skill_id, synced_agent)?;
    }

    let origin_agents = crate::db::models::Agent::all(&conn)?;
    if let Some(origin) = origin_agents.iter().find(|a| a.id == skill.origin_agent) {
        let folder_path = origin.resolved_skill_dir().join(&skill.folder_name);
        if folder_path.exists() {
            std::fs::remove_dir_all(&folder_path)?;
        }
    }

    Skill::delete(&conn, &skill_id)?;
    Ok(())
}

#[tauri::command]
pub fn get_skill_content(db: State<'_, DbState>, skill_id: String) -> Result<String, AppError> {
    let conn = db.lock().unwrap();

    // skill_id format: "agent:folder_name"
    let parts: Vec<&str> = skill_id.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(AppError::NotFound(format!(
            "Invalid skill ID: {}",
            skill_id
        )));
    }
    let (agent_id, folder_name) = (parts[0], parts[1]);

    let agent = crate::db::models::Agent::find_by_id(&conn, agent_id)
        .map_err(|_| AppError::NotFound(format!("Agent not found: {}", agent_id)))?;

    let skill_dir = agent.resolved_skill_dir().join(folder_name);
    let candidates = ["skill.md", "SKILL.md"];
    for name in &candidates {
        let path = skill_dir.join(name);
        if path.exists() {
            return std::fs::read_to_string(&path).map_err(AppError::from);
        }
    }
    Err(AppError::NotFound("skill.md not found".to_string()))
}

#[tauri::command]
pub fn update_skill_meta(
    db: State<'_, DbState>,
    skill_id: String,
    tags: Option<String>,
    notes: Option<String>,
) -> Result<(), AppError> {
    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE skills SET tags = ?1, notes = ?2 WHERE id = ?3",
        rusqlite::params![tags, notes, skill_id],
    )?;
    Ok(())
}
