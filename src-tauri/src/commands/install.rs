use crate::commands::skills::DbState;
use crate::commands::sources::get_source_token;
use crate::db::models::{Agent, Source};
use crate::error::AppError;
use crate::scanner::frontmatter::parse_frontmatter;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tauri::State;

#[tauri::command]
pub async fn install_skill(
    db: State<'_, DbState>,
    source_id: String,
    folder_name: String,
    target_agent: String,
    force: Option<bool>,
) -> Result<(), AppError> {
    install_skill_with_db(
        db.inner(),
        &source_id,
        &folder_name,
        &target_agent,
        force.unwrap_or(false),
    )
    .await
}

pub(crate) async fn install_skill_with_db(
    db: &Arc<Mutex<Connection>>,
    source_id: &str,
    folder_name: &str,
    target_agent: &str,
    force: bool,
) -> Result<(), AppError> {
    let (source_type, repo_url, token, config_dir, skill_dir) = {
        let conn = db.lock().unwrap();

        let source = Source::find_by_id(&conn, source_id)
            .map_err(|_| AppError::NotFound(format!("Source not found: {}", source_id)))?;

        let agent = Agent::find_by_id(&conn, target_agent)
            .map_err(|_| AppError::NotFound(format!("Agent not found: {}", target_agent)))?;

        if !agent.enabled {
            return Err(AppError::Conflict(format!(
                "Agent '{}' is not enabled",
                target_agent
            )));
        }

        let token = get_source_token(&conn, source_id)?;
        let config_dir = agent.resolved_config_dir();
        let skill_dir = agent.resolved_skill_dir();

        (source.source_type, source.url, token, config_dir, skill_dir)
    };

    if !config_dir.is_dir() {
        return Err(AppError::NotFound(format!(
            "Agent config directory not found: {}",
            config_dir.display()
        )));
    }

    let target_path = skill_dir.join(folder_name);
    if target_path.exists() && !force {
        return Err(AppError::Conflict(format!(
            "Skill folder '{}' already exists in {}",
            folder_name, target_agent
        )));
    }

    let temp_dir = tempfile::TempDir::new()?;
    match source_type.as_str() {
        "gitlab" => {
            let url = repo_url
                .ok_or_else(|| AppError::Remote("GitLab source has no URL configured".into()))?;
            crate::remote::gitlab::download_skill(&url, folder_name, &token, temp_dir.path())
                .await?;
        }
        other => {
            return Err(AppError::Remote(format!(
                "Unsupported source type: {}",
                other
            )));
        }
    }

    let temp_skill_dir = temp_dir.path().join(folder_name);
    let skill_md_path = if temp_skill_dir.join("skill.md").exists() {
        temp_skill_dir.join("skill.md")
    } else if temp_skill_dir.join("SKILL.md").exists() {
        temp_skill_dir.join("SKILL.md")
    } else {
        return Err(AppError::Frontmatter(format!(
            "Downloaded folder '{}' does not contain skill.md or SKILL.md",
            folder_name
        )));
    };

    let content = std::fs::read_to_string(&skill_md_path)?;
    let _ = parse_frontmatter(&content)?;

    if target_path.exists() {
        std::fs::remove_dir_all(&target_path)?;
    }

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if std::fs::rename(&temp_skill_dir, &target_path).is_err() {
        copy_dir_recursive(&temp_skill_dir, &target_path)?;
    }

    {
        let conn = db.lock().unwrap();
        let _ = crate::scanner::scan_all(&conn)?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(&entry.path(), &dest_path)?;
        }
    }
    Ok(())
}
