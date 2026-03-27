use crate::db::models::{Agent, Skill, SkillSync};
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

    /// Set up an agent directory with a skill folder containing skill.md,
    /// enable the agent with a custom config dir, and insert the skill into the DB.
    fn setup_agent_with_skill(
        conn: &rusqlite::Connection,
        temp: &TempDir,
        agent_id: &str,
        folder_name: &str,
    ) {
        let agent_cfg = temp.path().join(format!(".{}", agent_id));
        let skill_dir = agent_cfg.join("skills");
        let skill_path = skill_dir.join(folder_name);
        std::fs::create_dir_all(&skill_path).unwrap();
        std::fs::write(
            skill_path.join("skill.md"),
            "---\nname: test\ndescription: test\n---\nbody",
        )
        .unwrap();

        let agent_cfg_str = agent_cfg.to_string_lossy().to_string();
        conn.execute(
            "UPDATE agents SET enabled = 1, skill_dir = ?1 WHERE id = ?2",
            rusqlite::params![agent_cfg_str, agent_id],
        )
        .unwrap();

        // Insert the skill row directly (scan_all requires enabled agent, but we
        // control the DB directly here for speed)
        let skill_id = format!("{}:{}", agent_id, folder_name);
        conn.execute(
            "INSERT OR IGNORE INTO skills (id, folder_name, origin_agent, name, description, tags, notes, discovered_at, updated_at)
             VALUES (?1, ?2, ?3, 'test', 'test', NULL, NULL, 0, 0)",
            rusqlite::params![skill_id, folder_name, agent_id],
        )
        .unwrap();
    }

    /// Enable a target agent with a config dir pointing to temp (does not create a skill).
    fn enable_agent(conn: &rusqlite::Connection, temp: &TempDir, agent_id: &str) {
        let agent_cfg = temp.path().join(format!(".{}", agent_id));
        std::fs::create_dir_all(&agent_cfg).unwrap();
        let agent_cfg_str = agent_cfg.to_string_lossy().to_string();
        conn.execute(
            "UPDATE agents SET enabled = 1, skill_dir = ?1 WHERE id = ?2",
            rusqlite::params![agent_cfg_str, agent_id],
        )
        .unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn test_sync_skill_creates_symlink() {
        let temp = TempDir::new().unwrap();
        let conn = test_db();

        setup_agent_with_skill(&conn, &temp, "claude", "alpha");
        enable_agent(&conn, &temp, "codex");

        sync_skill(&conn, "claude:alpha", "codex").unwrap();

        // Verify symlink exists at codex's skill dir
        let codex_cfg = temp.path().join(".codex");
        let symlink_path = codex_cfg.join("skills").join("alpha");
        let metadata = std::fs::symlink_metadata(&symlink_path).unwrap();
        assert!(metadata.is_symlink(), "Expected symlink at {:?}", symlink_path);

        // Verify SkillSync row exists in DB
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skill_syncs WHERE skill_id = 'claude:alpha' AND agent = 'codex'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Expected SkillSync row in DB");
    }

    #[test]
    fn test_sync_skill_not_found() {
        let temp = TempDir::new().unwrap();
        let conn = test_db();
        enable_agent(&conn, &temp, "codex");

        let result = sync_skill(&conn, "claude:nonexistent", "codex");
        assert!(result.is_err(), "Expected error for nonexistent skill");
        let msg = result.unwrap_err().to_string().to_lowercase();
        assert!(
            msg.contains("not found"),
            "Expected 'not found' in: {}",
            msg
        );
    }

    #[test]
    fn test_sync_skill_disabled_agent() {
        let temp = TempDir::new().unwrap();
        let conn = test_db();

        setup_agent_with_skill(&conn, &temp, "claude", "beta");
        // Do NOT enable codex — it remains disabled by default

        let result = sync_skill(&conn, "claude:beta", "codex");
        assert!(result.is_err(), "Expected error for disabled agent");
        let msg = result.unwrap_err().to_string().to_lowercase();
        assert!(
            msg.contains("not enabled") || msg.contains("disabled"),
            "Expected 'not enabled' or 'disabled' in: {}",
            msg
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_unsync_skill_removes_link() {
        let temp = TempDir::new().unwrap();
        let conn = test_db();

        setup_agent_with_skill(&conn, &temp, "claude", "gamma");
        enable_agent(&conn, &temp, "codex");

        // First sync
        sync_skill(&conn, "claude:gamma", "codex").unwrap();

        let codex_cfg = temp.path().join(".codex");
        let symlink_path = codex_cfg.join("skills").join("gamma");
        assert!(
            std::fs::symlink_metadata(&symlink_path).is_ok(),
            "Symlink should exist before unsync"
        );

        // Now unsync
        unsync_skill(&conn, "claude:gamma", "codex").unwrap();

        // Symlink should be gone
        assert!(
            std::fs::symlink_metadata(&symlink_path).is_err(),
            "Symlink should be removed after unsync"
        );

        // SkillSync row should be deleted
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skill_syncs WHERE skill_id = 'claude:gamma' AND agent = 'codex'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "SkillSync row should be deleted");
    }
}

/// Create a symlink from origin to target_agent and record it in the DB.
pub fn sync_skill(
    conn: &Connection,
    skill_id: &str,
    target_agent: &str,
) -> Result<(), AppError> {
    let skills = Skill::all_with_syncs(conn)?;
    let skill = skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| AppError::NotFound(format!("Skill not found: {}", skill_id)))?;

    let agents = Agent::all(conn)?;
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
        conn,
        &SkillSync {
            skill_id: skill_id.to_string(),
            agent: target_agent.to_string(),
            symlink_path: Some(target_path.to_string_lossy().to_string()),
        },
    )?;

    Ok(())
}

/// Remove a sync symlink and delete the DB record.
pub fn unsync_skill(conn: &Connection, skill_id: &str, agent: &str) -> Result<(), AppError> {
    let agents = Agent::all(conn)?;
    let skills = Skill::all_with_syncs(conn)?;
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

    SkillSync::delete(conn, skill_id, agent)?;
    Ok(())
}
