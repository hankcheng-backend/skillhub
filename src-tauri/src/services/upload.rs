use crate::db::models::{Agent, Source};
use crate::error::AppError;
use crate::services::sources::get_source_token;
use crate::services::token_store::TokenStore;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Upload a local skill to a remote source.
pub async fn upload_skill(
    db: &Arc<Mutex<Connection>>,
    token_store: &dyn TokenStore,
    source_id: &str,
    skill_id: &str,
    force: bool,
) -> Result<(), AppError> {
    let parts: Vec<&str> = skill_id.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(AppError::NotFound(format!(
            "Invalid skill ID format: {}",
            skill_id
        )));
    }
    let (agent_id, folder_name) = (parts[0], parts[1]);

    let (url, skill_dir, token) = {
        let conn = db
            .lock()
            .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;

        let source = Source::find_by_id(&conn, source_id)
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

        let agent = Agent::find_by_id(&conn, agent_id)
            .map_err(|_| AppError::NotFound(format!("Agent not found: {}", agent_id)))?;

        let skill_dir = agent.resolved_skill_dir();
        let token = get_source_token(&conn, token_store, source_id)?;

        (url, skill_dir, token)
    };

    let local_path = skill_dir.join(folder_name);
    if !local_path.exists() {
        return Err(AppError::NotFound(format!(
            "Skill folder '{}' not found on disk",
            folder_name
        )));
    }

    crate::remote::gitlab::upload_skill(&url, folder_name, &token, &skill_dir, force).await
}
