use crate::db::models::Source;
use crate::error::AppError;
use crate::remote::RemoteSkill;
use crate::services::token_store::TokenStore;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::token_store::InMemoryTokenStore;
    use rusqlite_migration::{Migrations, M};
    use std::sync::{Arc, Mutex};

    const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS agents (
        id TEXT PRIMARY KEY, enabled INTEGER DEFAULT 0, skill_dir TEXT
    );
    CREATE TABLE IF NOT EXISTS skills (
        id TEXT PRIMARY KEY, folder_name TEXT NOT NULL, origin_agent TEXT NOT NULL,
        name TEXT, description TEXT, tags TEXT, notes TEXT,
        discovered_at INTEGER, updated_at INTEGER
    );
    CREATE TABLE IF NOT EXISTS skill_syncs (
        skill_id TEXT NOT NULL, agent TEXT NOT NULL, symlink_path TEXT,
        PRIMARY KEY (skill_id, agent),
        FOREIGN KEY (skill_id) REFERENCES skills(id) ON DELETE CASCADE,
        FOREIGN KEY (agent) REFERENCES agents(id)
    );
    CREATE TABLE IF NOT EXISTS sources (
        id TEXT PRIMARY KEY, name TEXT NOT NULL, type TEXT NOT NULL,
        url TEXT, folder_id TEXT, keychain_key TEXT, refresh_token_key TEXT,
        added_at INTEGER
    );
    INSERT OR IGNORE INTO agents (id, enabled, skill_dir) VALUES
        ('claude', 0, NULL), ('codex', 0, NULL), ('gemini', 0, NULL);
    CREATE TABLE IF NOT EXISTS app_config (key TEXT PRIMARY KEY, value TEXT NOT NULL);
    INSERT OR IGNORE INTO app_config (key, value) VALUES ('mcp_port', '9800');";

    fn test_setup() -> (Arc<Mutex<rusqlite::Connection>>, InMemoryTokenStore) {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .unwrap();
        let migrations = Migrations::new(vec![M::up(SCHEMA)]);
        migrations.to_latest(&mut conn).unwrap();
        (Arc::new(Mutex::new(conn)), InMemoryTokenStore::default())
    }

    #[test]
    fn test_add_source_empty_name_rejected() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let (db, token_store) = test_setup();
            let result = add_source(
                &db,
                &token_store,
                "",
                "gitlab",
                Some("https://gitlab.com/test/repo"),
                None,
                Some("glpat-abc123"),
            )
            .await;
            assert!(result.is_err());
            let msg = result.unwrap_err().to_string().to_lowercase();
            assert!(msg.contains("empty"), "Expected 'empty' in: {}", msg);
        });
    }

    #[test]
    fn test_add_source_gitlab_missing_url_rejected() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let (db, token_store) = test_setup();
            let result = add_source(
                &db,
                &token_store,
                "My Source",
                "gitlab",
                None,
                None,
                Some("glpat-abc123"),
            )
            .await;
            assert!(result.is_err());
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.to_uppercase().contains("URL"),
                "Expected 'URL' in: {}",
                msg
            );
        });
    }

    #[test]
    fn test_add_source_gitlab_missing_token_rejected() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let (db, token_store) = test_setup();
            let result = add_source(
                &db,
                &token_store,
                "My Source",
                "gitlab",
                Some("https://gitlab.com/test/repo"),
                None,
                None,
            )
            .await;
            assert!(result.is_err());
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.to_uppercase().contains("TOKEN"),
                "Expected 'Token' in: {}",
                msg
            );
        });
    }

    #[test]
    fn test_get_source_token_retrieves_stored_token() {
        let (db, token_store) = test_setup();
        let source_id = "test-source-id-1234";
        let keychain_key = format!("skillhub-{}", source_id);

        // Manually insert a Source row
        {
            let conn = db.lock().unwrap();
            let source = Source {
                id: source_id.to_string(),
                name: "Test Source".to_string(),
                source_type: "gitlab".to_string(),
                url: Some("https://gitlab.com/test/repo".to_string()),
                folder_id: None,
                keychain_key: Some(keychain_key.clone()),
                refresh_token_key: None,
                added_at: Some(0),
            };
            Source::insert(&conn, &source).unwrap();
        }

        // Store token in the InMemoryTokenStore
        token_store
            .store_token("skillhub", &keychain_key, "glpat-secret")
            .unwrap();

        // Now retrieve via get_source_token
        let conn = db.lock().unwrap();
        let retrieved = get_source_token(&conn, &token_store, source_id).unwrap();
        assert_eq!(retrieved, "glpat-secret");
    }

    #[test]
    fn test_remove_source_deletes_row_and_token() {
        let (db, token_store) = test_setup();
        let source_id = "remove-test-source";
        let keychain_key = format!("skillhub-{}", source_id);

        // Insert source and store token
        {
            let conn = db.lock().unwrap();
            let source = Source {
                id: source_id.to_string(),
                name: "Remove Me".to_string(),
                source_type: "gitlab".to_string(),
                url: Some("https://gitlab.com/test/repo".to_string()),
                folder_id: None,
                keychain_key: Some(keychain_key.clone()),
                refresh_token_key: None,
                added_at: Some(0),
            };
            Source::insert(&conn, &source).unwrap();
        }
        token_store
            .store_token("skillhub", &keychain_key, "glpat-todelete")
            .unwrap();

        // Call remove_source
        {
            let conn = db.lock().unwrap();
            remove_source(&conn, &token_store, source_id).unwrap();
        }

        // Verify source row is gone
        {
            let conn = db.lock().unwrap();
            let result = Source::find_by_id(&conn, source_id);
            assert!(result.is_err(), "Source row should be deleted");
        }

        // Verify token is gone
        let token_result = token_store.get_token("skillhub", &keychain_key);
        assert!(token_result.is_err(), "Token should be removed from store");
    }

    #[test]
    fn test_list_sources_returns_all() {
        let (db, _token_store) = test_setup();

        {
            let conn = db.lock().unwrap();
            for i in 0..2 {
                let source = Source {
                    id: format!("source-{}", i),
                    name: format!("Source {}", i),
                    source_type: "gitlab".to_string(),
                    url: Some(format!("https://gitlab.com/test/repo{}", i)),
                    folder_id: None,
                    keychain_key: None,
                    refresh_token_key: None,
                    added_at: Some(0),
                };
                Source::insert(&conn, &source).unwrap();
            }
        }

        let conn = db.lock().unwrap();
        let sources = Source::all(&conn).unwrap();
        assert_eq!(sources.len(), 2);
    }
}

/// Add a new source, storing its token in the provided `TokenStore`.
///
/// Validates input, calls `validate_source_access` (async network check), inserts into DB,
/// stores the token, and returns the newly created `Source`. Rolls back the DB row if the
/// keychain/token-store write fails.
pub async fn add_source(
    db: &Arc<Mutex<Connection>>,
    token_store: &dyn TokenStore,
    name: &str,
    source_type: &str,
    url: Option<&str>,
    folder_id: Option<&str>,
    token: Option<&str>,
) -> Result<Source, AppError> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err(AppError::Conflict("Source name cannot be empty".into()));
    }

    match source_type {
        "gitlab" => {
            let has_url = url.map(|v| !v.trim().is_empty()).unwrap_or(false);
            let has_token = token.map(|v| !v.trim().is_empty()).unwrap_or(false);
            if !has_url {
                return Err(AppError::Conflict("GitLab source URL is required".into()));
            }
            if !has_token {
                return Err(AppError::Conflict(
                    "GitLab Personal Access Token is required".into(),
                ));
            }
            let repo_url = url.map(|v| v.trim()).unwrap_or_default();
            let access_token = token.map(|v| v.trim()).unwrap_or_default();
            crate::remote::gitlab::validate_source_access(repo_url, access_token).await?;
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
            .unwrap_or_default()
            .as_secs() as i64,
    );

    let source = Source {
        id: source_id.clone(),
        name: trimmed_name.to_string(),
        source_type: source_type.to_string(),
        url: url.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()),
        folder_id: folder_id
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty()),
        keychain_key: None,
        refresh_token_key: None,
        added_at,
    };

    {
        let conn = db
            .lock()
            .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
        Source::insert(&conn, &source)?;
    }

    if let Some(tok) = token
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
    {
        let keychain_key = format!("skillhub-{}", source_id);
        match token_store.store_token("skillhub", &keychain_key, &tok) {
            Ok(()) => {
                let conn = db
                    .lock()
                    .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
                Source::update_keychain_key(&conn, &source_id, &keychain_key)?;
            }
            Err(e) => {
                let conn = db
                    .lock()
                    .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
                let _ = Source::delete(&conn, &source_id);
                return Err(e);
            }
        }
    }

    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    Source::find_by_id(&conn, &source_id).map_err(AppError::from)
}

/// Remove a source and delete its keychain entry.
pub fn remove_source(
    conn: &Connection,
    token_store: &dyn TokenStore,
    source_id: &str,
) -> Result<(), AppError> {
    let source = Source::find_by_id(conn, source_id)?;
    Source::delete(conn, source_id)?;
    if let Some(ref key) = source.keychain_key {
        // Best-effort: ignore errors since the DB row is already deleted
        let _ = token_store.delete_token("skillhub", key);
    }
    Ok(())
}

/// Retrieve the access token for a source.
pub fn get_source_token(
    conn: &Connection,
    token_store: &dyn TokenStore,
    source_id: &str,
) -> Result<String, AppError> {
    let source = Source::find_by_id(conn, source_id)
        .map_err(|_| AppError::NotFound(format!("Source not found: {}", source_id)))?;
    match source.keychain_key {
        Some(ref key) => token_store.get_token("skillhub", key),
        None => Err(AppError::OAuth(
            "No token configured for this source".into(),
        )),
    }
}

/// Update the stored access token for a source.
pub fn update_source_token(
    conn: &Connection,
    token_store: &dyn TokenStore,
    source_id: &str,
    new_token: &str,
) -> Result<(), AppError> {
    let trimmed = new_token.trim();
    if trimmed.is_empty() {
        return Err(AppError::Conflict("Token cannot be empty".into()));
    }

    let source = Source::find_by_id(conn, source_id)
        .map_err(|_| AppError::NotFound(format!("Source not found: {}", source_id)))?;

    let keychain_key = match source.keychain_key {
        Some(key) => key,
        None => {
            // Source has no keychain_key yet — create one
            let key = format!("skillhub-{}", source_id);
            Source::update_keychain_key(conn, source_id, &key)?;
            key
        }
    };

    token_store.store_token("skillhub", &keychain_key, trimmed)?;
    Ok(())
}

/// List all skills from a remote source.
pub async fn browse_source(
    db: &Arc<Mutex<Connection>>,
    token_store: &dyn TokenStore,
    source_id: &str,
) -> Result<Vec<RemoteSkill>, AppError> {
    let (source_name, source_type, url) = {
        let conn = db
            .lock()
            .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
        let source = Source::find_by_id(&conn, source_id)
            .map_err(|_| AppError::NotFound(format!("Source not found: {}", source_id)))?;
        (
            source.name.clone(),
            source.source_type.clone(),
            source.url.clone(),
        )
    };

    let token = {
        let conn = db
            .lock()
            .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
        get_source_token(&conn, token_store, source_id)?
    };

    match source_type.as_str() {
        "gitlab" => {
            let repo_url =
                url.ok_or_else(|| AppError::Remote("GitLab source has no URL configured".into()))?;
            let mut skills = crate::remote::gitlab::list_skills(&repo_url, &token)
                .await
                .map_err(|e| match e {
                    AppError::TokenExpired(_) => AppError::TokenExpired(source_id.to_string()),
                    other => other,
                })?;
            for skill in &mut skills {
                skill.source_id = source_id.to_string();
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

/// Get the content of a remote skill's skill.md file.
pub async fn get_remote_skill_content(
    db: &Arc<Mutex<Connection>>,
    token_store: &dyn TokenStore,
    source_id: &str,
    folder_name: &str,
) -> Result<Option<String>, AppError> {
    let (source_type, url, token) = {
        let conn = db
            .lock()
            .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
        let source = Source::find_by_id(&conn, source_id)
            .map_err(|_| AppError::NotFound(format!("Source not found: {}", source_id)))?;
        let token = get_source_token(&conn, token_store, source_id)?;
        (source.source_type, source.url, token)
    };

    match source_type.as_str() {
        "gitlab" => {
            let repo_url =
                url.ok_or_else(|| AppError::Remote("GitLab source has no URL configured".into()))?;
            crate::remote::gitlab::get_skill_content(&repo_url, folder_name, &token)
                .await
                .map_err(|e| match e {
                    AppError::TokenExpired(_) => AppError::TokenExpired(source_id.to_string()),
                    other => other,
                })
        }
        other => Err(AppError::Remote(format!(
            "Unsupported source type: {}",
            other
        ))),
    }
}
