use crate::commands::skills::DbState;
use crate::db::models::{Agent, Skill, SkillSync};
use crate::error::AppError;
use tauri::State;

#[tauri::command]
pub fn sync_skill(
    db: State<'_, DbState>,
    skill_id: String,
    target_agent: String,
) -> Result<(), AppError> {
    let conn = db.lock().unwrap();
    let skills = Skill::all_with_syncs(&conn)?;
    let skill = skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| AppError::NotFound(format!("Skill not found: {}", skill_id)))?;

    let agents = Agent::all(&conn)?;
    let origin = agents
        .iter()
        .find(|a| a.id == skill.origin_agent)
        .ok_or_else(|| AppError::NotFound("Origin agent not found".into()))?;
    let target = agents
        .iter()
        .find(|a| a.id == target_agent)
        .ok_or_else(|| AppError::NotFound("Target agent not found".into()))?;

    if !target.enabled {
        return Err(AppError::Conflict("Target agent is not enabled".into()));
    }

    // Verify target agent's config directory exists before syncing.
    let target_config_root = target.resolved_config_dir();
    if !target_config_root.is_dir() {
        return Err(AppError::NotFound(format!(
            "Agent config directory not found: {}",
            target_config_root.display()
        )));
    }

    let target_path = target.resolved_skill_dir().join(&skill.folder_name);
    let origin_path = origin.resolved_skill_dir().join(&skill.folder_name);

    crate::sync::create_sync_link(&origin_path, &target_path)?;

    SkillSync::insert(
        &conn,
        &SkillSync {
            skill_id: skill_id.clone(),
            agent: target_agent.clone(),
            symlink_path: Some(target_path.to_string_lossy().to_string()),
        },
    )?;

    Ok(())
}

#[tauri::command]
pub fn unsync_skill(
    db: State<'_, DbState>,
    skill_id: String,
    agent: String,
) -> Result<(), AppError> {
    let conn = db.lock().unwrap();
    let agents = Agent::all(&conn)?;
    let skills = Skill::all_with_syncs(&conn)?;
    let skill = skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| AppError::NotFound(format!("Skill not found: {}", skill_id)))?;
    let target = agents
        .iter()
        .find(|a| a.id == agent)
        .ok_or_else(|| AppError::NotFound("Agent not found".into()))?;

    let link_path = target.resolved_skill_dir().join(&skill.folder_name);
    crate::sync::remove_sync_link(&link_path)?;

    SkillSync::delete(&conn, &skill_id, &agent)?;
    Ok(())
}
