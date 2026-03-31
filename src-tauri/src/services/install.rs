use crate::error::AppError;
use crate::services::token_store::TokenStore;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::Source;
    use crate::services::token_store::InMemoryTokenStore;
    use rusqlite_migration::{Migrations, M};
    use tempfile::TempDir;

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

    fn insert_source(conn: &rusqlite::Connection, source_id: &str) {
        let source = Source {
            id: source_id.to_string(),
            name: "Test Source".to_string(),
            source_type: "gitlab".to_string(),
            url: Some("https://gitlab.com/test/repo".to_string()),
            folder_id: None,
            keychain_key: Some(format!("skillhub-{}", source_id)),
            refresh_token_key: None,
            added_at: Some(0),
        };
        Source::insert(conn, &source).unwrap();
    }

    #[test]
    fn test_install_skill_source_not_found() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let (db, token_store) = test_setup();
            let result = install_skill(
                &db,
                &token_store,
                "nonexistent-source-id",
                "my-skill",
                "claude",
                false,
            )
            .await;
            assert!(result.is_err(), "Expected error for missing source");
            let msg = result.unwrap_err().to_string().to_lowercase();
            assert!(
                msg.contains("not found"),
                "Expected 'not found' in: {}",
                msg
            );
        });
    }

    #[test]
    fn test_install_skill_agent_disabled() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let (db, token_store) = test_setup();

            // Insert source and store its token
            {
                let conn = db.lock().unwrap();
                insert_source(&conn, "src-1");
            }
            token_store
                .store_token("skillhub", "skillhub-src-1", "glpat-test")
                .unwrap();

            // codex is disabled by default
            let result =
                install_skill(&db, &token_store, "src-1", "my-skill", "codex", false).await;
            assert!(result.is_err(), "Expected error for disabled agent");
            let msg = result.unwrap_err().to_string().to_lowercase();
            assert!(
                msg.contains("not enabled") || msg.contains("disabled"),
                "Expected 'not enabled' or 'disabled' in: {}",
                msg
            );
        });
    }

    #[test]
    fn test_install_skill_conflict_without_force() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let temp = TempDir::new().unwrap();
            let (db, token_store) = test_setup();

            // Insert source and store token
            {
                let conn = db.lock().unwrap();
                insert_source(&conn, "src-2");
            }
            token_store
                .store_token("skillhub", "skillhub-src-2", "glpat-test")
                .unwrap();

            // Enable claude with a real config dir
            let claude_cfg = temp.path().join(".claude");
            let skill_dir = claude_cfg.join("skills");
            // Pre-create the target folder to trigger the conflict
            let existing_folder = skill_dir.join("my-skill");
            std::fs::create_dir_all(&existing_folder).unwrap();

            {
                let conn = db.lock().unwrap();
                conn.execute(
                    "UPDATE agents SET enabled = 1, skill_dir = ?1 WHERE id = 'claude'",
                    rusqlite::params![claude_cfg.to_string_lossy().to_string()],
                )
                .unwrap();
            }

            let result =
                install_skill(&db, &token_store, "src-2", "my-skill", "claude", false).await;
            assert!(result.is_err(), "Expected Conflict error");
            let err = result.unwrap_err();
            assert!(
                matches!(err, crate::error::AppError::Conflict(_)),
                "Expected AppError::Conflict, got: {:?}",
                err
            );
        });
    }
}

/// Download a skill from a remote source and install it for the given agent.
pub async fn install_skill(
    db: &Arc<Mutex<Connection>>,
    token_store: &dyn TokenStore,
    source_id: &str,
    folder_name: &str,
    target_agent: &str,
    force: bool,
) -> Result<(), AppError> {
    use crate::db::models::{Agent, Source};
    use crate::scanner::frontmatter::parse_frontmatter;
    use crate::services::sources::get_source_token;

    let (source_type, repo_url, token, config_dir, skill_dir) = {
        let conn = db
            .lock()
            .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;

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

        let token = get_source_token(&conn, token_store, source_id)?;
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
        let conn = db
            .lock()
            .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
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
