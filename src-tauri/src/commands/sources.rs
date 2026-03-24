use crate::commands::skills::DbState;
use crate::db::models::Source;
use crate::error::AppError;
use crate::remote::{oauth, RemoteSkill};
use tauri::State;

#[tauri::command]
pub fn list_sources(db: State<'_, DbState>) -> Result<Vec<Source>, AppError> {
    let conn = db.lock().unwrap();
    Source::all(&conn).map_err(AppError::from)
}

#[tauri::command]
pub async fn add_source(
    db: State<'_, DbState>,
    name: String,
    source_type: String,
    url: Option<String>,
    folder_id: Option<String>,
    token: Option<String>,
) -> Result<Source, AppError> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err(AppError::Conflict("Source name cannot be empty".into()));
    }

    match source_type.as_str() {
        "gitlab" => {
            let has_url = url.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false);
            let has_token = token
                .as_ref()
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false);
            if !has_url {
                return Err(AppError::Conflict("GitLab source URL is required".into()));
            }
            if !has_token {
                return Err(AppError::Conflict(
                    "GitLab Personal Access Token is required".into(),
                ));
            }
        }
        "gdrive" => {
            return Err(AppError::Remote(
                "Google Drive source is not implemented yet".into(),
            ));
        }
        other => {
            return Err(AppError::Conflict(format!(
                "Unsupported source type: {}",
                other
            )));
        }
    }

    let source_id = uuid::Uuid::new_v4().to_string();
    let added_at = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    );

    let source = Source {
        id: source_id.clone(),
        name: trimmed_name.to_string(),
        source_type,
        url: url.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()),
        folder_id: folder_id
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty()),
        keychain_key: None,
        refresh_token_key: None,
        added_at,
    };
    {
        let conn = db.lock().unwrap();
        Source::insert(&conn, &source)?;
    }

    if let Some(tok) = token
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
    {
        let keychain_key = format!("skillhub-{}", source_id);
        match oauth::store_token("skillhub", &keychain_key, &tok) {
            Ok(()) => {
                let conn = db.lock().unwrap();
                Source::update_keychain_key(&conn, &source_id, &keychain_key)?;
            }
            Err(e) => {
                let conn = db.lock().unwrap();
                let _ = Source::delete(&conn, &source_id);
                return Err(e);
            }
        }
    }

    let conn = db.lock().unwrap();
    Source::find_by_id(&conn, &source_id).map_err(AppError::from)
}

#[tauri::command]
pub fn remove_source(db: State<'_, DbState>, source_id: String) -> Result<(), AppError> {
    let keychain_key = {
        let conn = db.lock().unwrap();
        let source = Source::find_by_id(&conn, &source_id)?;
        Source::delete(&conn, &source_id)?;
        source.keychain_key
    };
    if let Some(ref key) = keychain_key {
        if let Ok(entry) = keyring::Entry::new("skillhub", key) {
            let _ = entry.delete_credential();
        }
    }
    Ok(())
}

pub fn get_source_token(conn: &rusqlite::Connection, source_id: &str) -> Result<String, AppError> {
    let source = Source::find_by_id(conn, source_id)
        .map_err(|_| AppError::NotFound(format!("Source not found: {}", source_id)))?;
    match source.keychain_key {
        Some(ref key) => oauth::get_token("skillhub", key),
        None => Err(AppError::OAuth(
            "No token configured for this source".into(),
        )),
    }
}

#[tauri::command]
pub async fn browse_source(
    db: State<'_, DbState>,
    source_id: String,
) -> Result<Vec<RemoteSkill>, AppError> {
    let (source_name, source_type, url) = {
        let conn = db.lock().unwrap();
        let source = Source::find_by_id(&conn, &source_id)
            .map_err(|_| AppError::NotFound(format!("Source not found: {}", source_id)))?;
        (
            source.name.clone(),
            source.source_type.clone(),
            source.url.clone(),
        )
    };

    let token = {
        let conn = db.lock().unwrap();
        get_source_token(&conn, &source_id)?
    };

    match source_type.as_str() {
        "gitlab" => {
            let repo_url =
                url.ok_or_else(|| AppError::Remote("GitLab source has no URL configured".into()))?;
            let mut skills = crate::remote::gitlab::list_skills(&repo_url, &token).await?;
            for skill in &mut skills {
                skill.source_id = source_id.clone();
                skill.source_name = source_name.clone();
            }
            Ok(skills)
        }
        other => Err(AppError::Remote(format!(
            "Unsupported source type: {}",
            other
        ))),
    }
}

#[tauri::command]
pub async fn get_remote_skill_content(
    db: State<'_, DbState>,
    source_id: String,
    folder_name: String,
) -> Result<Option<String>, AppError> {
    let (source_type, url, token) = {
        let conn = db.lock().unwrap();
        let source = Source::find_by_id(&conn, &source_id)
            .map_err(|_| AppError::NotFound(format!("Source not found: {}", source_id)))?;
        let token = get_source_token(&conn, &source_id)?;
        (source.source_type, source.url, token)
    };

    match source_type.as_str() {
        "gitlab" => {
            let repo_url =
                url.ok_or_else(|| AppError::Remote("GitLab source has no URL configured".into()))?;
            crate::remote::gitlab::get_skill_content(&repo_url, &folder_name, &token).await
        }
        other => Err(AppError::Remote(format!(
            "Unsupported source type: {}",
            other
        ))),
    }
}
