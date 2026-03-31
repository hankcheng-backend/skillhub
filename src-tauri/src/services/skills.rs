use crate::db::models::Skill;
use crate::error::AppError;
use rusqlite::Connection;

#[cfg(test)]
mod tests {
    use super::*;
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

    fn test_db() -> rusqlite::Connection {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .unwrap();
        let migrations = Migrations::new(vec![M::up(SCHEMA)]);
        migrations.to_latest(&mut conn).unwrap();
        conn
    }

    #[test]
    fn test_list_skills_empty_db() {
        let conn = test_db();
        let skills = list_skills(&conn).unwrap();
        assert!(skills.is_empty(), "Expected empty skill list on fresh DB");
    }

    #[test]
    fn test_delete_skill_not_found() {
        let conn = test_db();
        let result = delete_skill(&conn, "fake:nonexistent");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string().to_lowercase();
        assert!(
            msg.contains("not found"),
            "Expected 'not found' in: {}",
            msg
        );
    }

    #[test]
    fn test_get_skill_content_reads_file() {
        let temp = TempDir::new().unwrap();
        let claude_cfg = temp.path().join(".claude");
        let skill_dir = claude_cfg.join("skills").join("my-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        let content = "---\nname: My Skill\ndescription: Test\n---\nHello World";
        std::fs::write(skill_dir.join("skill.md"), content).unwrap();

        let conn = test_db();
        let claude_cfg_str = claude_cfg.to_string_lossy().to_string();
        conn.execute(
            "UPDATE agents SET enabled = 1, skill_dir = ?1 WHERE id = 'claude'",
            rusqlite::params![claude_cfg_str],
        )
        .unwrap();

        let result = get_skill_content(&conn, "claude:my-skill").unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_update_skill_meta() {
        let conn = test_db();

        // Insert a skill row directly
        conn.execute(
            "INSERT INTO skills (id, folder_name, origin_agent, name, description, tags, notes, discovered_at, updated_at)
             VALUES ('claude:test-skill', 'test-skill', 'claude', 'Test', 'Desc', NULL, NULL, 0, 0)",
            [],
        )
        .unwrap();

        update_skill_meta(
            &conn,
            "claude:test-skill",
            Some("rust,cli"),
            Some("test note"),
        )
        .unwrap();

        let (tags, notes): (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT tags, notes FROM skills WHERE id = 'claude:test-skill'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(tags, Some("rust,cli".to_string()));
        assert_eq!(notes, Some("test note".to_string()));
    }
}

/// List all local skills with their sync relationships.
pub fn list_skills(conn: &Connection) -> Result<Vec<Skill>, AppError> {
    Skill::all_with_syncs(conn).map_err(AppError::from)
}

/// Delete a skill and all its sync links from the filesystem and DB.
pub fn delete_skill(conn: &Connection, skill_id: &str) -> Result<(), AppError> {
    let skills = Skill::all_with_syncs(conn)?;
    let skill = skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| AppError::NotFound(format!("Skill not found: {}", skill_id)))?;

    for synced_agent in &skill.synced_to {
        let agents = crate::db::models::Agent::all(conn)?;
        if let Some(agent) = agents.iter().find(|a| &a.id == synced_agent) {
            let link_path = agent.resolved_skill_dir().join(&skill.folder_name);
            let _ = crate::sync::remove_sync_link(&link_path);
        }
        crate::db::models::SkillSync::delete(conn, skill_id, synced_agent)?;
    }

    let origin_agents = crate::db::models::Agent::all(conn)?;
    if let Some(origin) = origin_agents.iter().find(|a| a.id == skill.origin_agent) {
        let folder_path = origin.resolved_skill_dir().join(&skill.folder_name);
        if folder_path.exists() {
            std::fs::remove_dir_all(&folder_path)?;
        }
    }

    Skill::delete(conn, skill_id)?;
    Ok(())
}

/// Read the skill.md content from disk for the given skill_id ("agent:folder_name").
pub fn get_skill_content(conn: &Connection, skill_id: &str) -> Result<String, AppError> {
    let parts: Vec<&str> = skill_id.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(AppError::NotFound(format!(
            "Invalid skill ID: {}",
            skill_id
        )));
    }
    let (agent_id, folder_name) = (parts[0], parts[1]);

    let agent = crate::db::models::Agent::find_by_id(conn, agent_id)
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

/// Update the tags and notes metadata for a skill.
pub fn update_skill_meta(
    conn: &Connection,
    skill_id: &str,
    tags: Option<&str>,
    notes: Option<&str>,
) -> Result<(), AppError> {
    conn.execute(
        "UPDATE skills SET tags = ?1, notes = ?2 WHERE id = ?3",
        rusqlite::params![tags, notes, skill_id],
    )?;
    Ok(())
}
