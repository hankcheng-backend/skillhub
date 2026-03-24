use crate::commands::skills::DbState;
use crate::commands::sources::get_source_token;
use crate::db::models::{Agent, Source};
use crate::error::AppError;
use tauri::State;

#[tauri::command]
pub async fn upload_skill(
    db: State<'_, DbState>,
    source_id: String,
    skill_id: String,
    force: Option<bool>,
) -> Result<(), AppError> {
    let (url, folder_name, skill_dir, token) = {
        let conn = db.lock().unwrap();

        let source = Source::find_by_id(&conn, &source_id)
            .map_err(|_| AppError::NotFound(format!("Source not found: {}", source_id)))?;

        if source.source_type != "gitlab" {
            return Err(AppError::Remote(format!(
                "Unsupported source type: {}",
                source.source_type
            )));
        }

        let url = source
            .url
            .ok_or_else(|| AppError::Remote("GitLab source has no URL configured".into()))?;

        let parts: Vec<&str> = skill_id.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AppError::NotFound(format!(
                "Invalid skill ID format: {}",
                skill_id
            )));
        }
        let (agent_id, folder_name) = (parts[0], parts[1]);

        let agent = Agent::find_by_id(&conn, agent_id)
            .map_err(|_| AppError::NotFound(format!("Agent not found: {}", agent_id)))?;

        let skill_dir = agent.resolved_skill_dir();
        let token = get_source_token(&conn, &source_id)?;

        (url, folder_name.to_string(), skill_dir, token)
    };

    let local_path = skill_dir.join(&folder_name);
    if !local_path.exists() {
        return Err(AppError::NotFound(format!(
            "Skill folder '{}' not found on disk",
            folder_name
        )));
    }

    crate::remote::gitlab::upload_skill(
        &url,
        &folder_name,
        &token,
        &skill_dir,
        force.unwrap_or(false),
    )
    .await
}
