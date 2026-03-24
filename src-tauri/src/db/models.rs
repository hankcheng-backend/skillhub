use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub enabled: bool,
    pub skill_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub folder_name: String,
    pub origin_agent: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub discovered_at: Option<i64>,
    pub updated_at: Option<i64>,
    #[serde(default)]
    pub synced_to: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSync {
    pub skill_id: String,
    pub agent: String,
    pub symlink_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub source_type: String,
    pub url: Option<String>,
    pub folder_id: Option<String>,
    pub keychain_key: Option<String>,
    pub refresh_token_key: Option<String>,
    pub added_at: Option<i64>,
}

impl Agent {
    pub fn all(conn: &Connection) -> Result<Vec<Agent>, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT id, enabled, skill_dir FROM agents")?;
        let agents = stmt
            .query_map([], |row| {
                Ok(Agent {
                    id: row.get(0)?,
                    enabled: row.get::<_, i32>(1)? != 0,
                    skill_dir: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(agents)
    }

    pub fn enabled(conn: &Connection) -> Result<Vec<Agent>, rusqlite::Error> {
        let mut stmt =
            conn.prepare("SELECT id, enabled, skill_dir FROM agents WHERE enabled = 1")?;
        let agents = stmt
            .query_map([], |row| {
                Ok(Agent {
                    id: row.get(0)?,
                    enabled: row.get::<_, i32>(1)? != 0,
                    skill_dir: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(agents)
    }

    pub fn update(
        conn: &Connection,
        id: &str,
        enabled: bool,
        skill_dir: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE agents SET enabled = ?1, skill_dir = ?2 WHERE id = ?3",
            params![enabled as i32, skill_dir, id],
        )?;
        Ok(())
    }

    fn is_legacy_skills_path(path: &std::path::Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.eq_ignore_ascii_case("skills"))
            .unwrap_or(false)
    }

    fn looks_like_legacy_skill_root(path: &std::path::Path) -> bool {
        if !path.is_dir() {
            return false;
        }

        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => return false,
        };

        for entry in entries.flatten() {
            let child = entry.path();
            if !child.is_dir() {
                continue;
            }

            if child.join("skill.md").is_file() || child.join("SKILL.md").is_file() {
                return true;
            }
        }

        false
    }

    /// Returns the agent config directory (e.g. ~/.codex).
    /// Backward compatibility: if old data stored ".../skills", return its parent.
    pub fn resolved_config_dir(&self) -> std::path::PathBuf {
        if let Some(ref dir) = self.skill_dir {
            let path = std::path::PathBuf::from(dir);
            if Self::is_legacy_skills_path(&path) {
                return path.parent().unwrap_or(&path).to_path_buf();
            }
            if Self::looks_like_legacy_skill_root(&path) {
                return path;
            }
            return path;
        }

        let home = dirs::home_dir().expect("cannot find home dir");
        home.join(format!(".{}", self.id))
    }

    /// Returns the skills folder path (config_dir/skills).
    /// Backward compatibility: if old data stored ".../skills", use it directly.
    pub fn resolved_skill_dir(&self) -> std::path::PathBuf {
        if let Some(ref dir) = self.skill_dir {
            let path = std::path::PathBuf::from(dir);
            if Self::is_legacy_skills_path(&path) || Self::looks_like_legacy_skill_root(&path) {
                return path;
            }
            return path.join("skills");
        }

        self.resolved_config_dir().join("skills")
    }

    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Agent, rusqlite::Error> {
        conn.query_row(
            "SELECT id, enabled, skill_dir FROM agents WHERE id = ?1",
            params![id],
            |row| {
                Ok(Agent {
                    id: row.get(0)?,
                    enabled: row.get::<_, i32>(1)? != 0,
                    skill_dir: row.get(2)?,
                })
            },
        )
    }
}

impl Skill {
    pub fn upsert(conn: &Connection, skill: &Skill) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO skills (id, folder_name, origin_agent, name, description, tags, notes, discovered_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                updated_at = excluded.updated_at",
            params![
                skill.id, skill.folder_name, skill.origin_agent,
                skill.name, skill.description, skill.tags, skill.notes,
                skill.discovered_at, skill.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn all_with_syncs(conn: &Connection) -> Result<Vec<Skill>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, folder_name, origin_agent, name, description, tags, notes, discovered_at, updated_at
             FROM skills
             ORDER BY folder_name ASC, origin_agent ASC, id ASC"
        )?;
        let mut skills: Vec<Skill> = stmt
            .query_map([], |row| {
                Ok(Skill {
                    id: row.get(0)?,
                    folder_name: row.get(1)?,
                    origin_agent: row.get(2)?,
                    name: row.get(3)?,
                    description: row.get(4)?,
                    tags: row.get(5)?,
                    notes: row.get(6)?,
                    discovered_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    synced_to: vec![],
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut sync_stmt =
            conn.prepare("SELECT skill_id, agent FROM skill_syncs ORDER BY skill_id, agent")?;
        let syncs: Vec<(String, String)> = sync_stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        for skill in &mut skills {
            skill.synced_to = syncs
                .iter()
                .filter(|(sid, _)| sid == &skill.id)
                .map(|(_, agent)| agent.clone())
                .collect();
        }
        Ok(skills)
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
        conn.execute("DELETE FROM skills WHERE id = ?1", params![id])?;
        Ok(())
    }
}

impl SkillSync {
    pub fn insert(conn: &Connection, sync: &SkillSync) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT OR REPLACE INTO skill_syncs (skill_id, agent, symlink_path) VALUES (?1, ?2, ?3)",
            params![sync.skill_id, sync.agent, sync.symlink_path],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, skill_id: &str, agent: &str) -> Result<(), rusqlite::Error> {
        conn.execute(
            "DELETE FROM skill_syncs WHERE skill_id = ?1 AND agent = ?2",
            params![skill_id, agent],
        )?;
        Ok(())
    }

    pub fn delete_all_for_skill(conn: &Connection, skill_id: &str) -> Result<(), rusqlite::Error> {
        conn.execute(
            "DELETE FROM skill_syncs WHERE skill_id = ?1",
            params![skill_id],
        )?;
        Ok(())
    }
}

impl Source {
    pub fn all(conn: &Connection) -> Result<Vec<Source>, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT id, name, type, url, folder_id, keychain_key, refresh_token_key, added_at FROM sources")?;
        let sources = stmt
            .query_map([], |row| {
                Ok(Source {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    source_type: row.get(2)?,
                    url: row.get(3)?,
                    folder_id: row.get(4)?,
                    keychain_key: row.get(5)?,
                    refresh_token_key: row.get(6)?,
                    added_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sources)
    }

    pub fn insert(conn: &Connection, source: &Source) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO sources (id, name, type, url, folder_id, keychain_key, refresh_token_key, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![source.id, source.name, source.source_type, source.url, source.folder_id, source.keychain_key, source.refresh_token_key, source.added_at],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
        conn.execute("DELETE FROM sources WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Source, rusqlite::Error> {
        conn.query_row(
            "SELECT id, name, type, url, folder_id, keychain_key, refresh_token_key, added_at FROM sources WHERE id = ?1",
            params![id],
            |row| {
                Ok(Source {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    source_type: row.get(2)?,
                    url: row.get(3)?,
                    folder_id: row.get(4)?,
                    keychain_key: row.get(5)?,
                    refresh_token_key: row.get(6)?,
                    added_at: row.get(7)?,
                })
            },
        )
    }

    pub fn update_keychain_key(
        conn: &Connection,
        id: &str,
        keychain_key: &str,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE sources SET keychain_key = ?1 WHERE id = ?2",
            params![keychain_key, id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod path_resolution_tests {
    use super::Agent;
    use tempfile::TempDir;

    fn make_agent_with_dir(path: &std::path::Path) -> Agent {
        Agent {
            id: "codex".to_string(),
            enabled: true,
            skill_dir: Some(path.to_string_lossy().to_string()),
        }
    }

    fn make_skill(root: &std::path::Path, folder_name: &str) {
        let skill_dir = root.join(folder_name);
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("skill.md"),
            "---\nname: test\ndescription: test\n---\nbody",
        )
        .unwrap();
    }

    #[test]
    fn resolves_legacy_suffix_skills_path() {
        let temp = TempDir::new().unwrap();
        let cfg = temp.path().join(".codex");
        let skills = cfg.join("skills");
        std::fs::create_dir_all(&skills).unwrap();

        let agent = make_agent_with_dir(&skills);
        assert_eq!(agent.resolved_config_dir(), cfg);
        assert_eq!(agent.resolved_skill_dir(), skills);
    }

    #[test]
    fn resolves_legacy_custom_skill_root_by_contents() {
        let temp = TempDir::new().unwrap();
        let custom_root = temp.path().join("legacy-skill-root");
        make_skill(&custom_root, "alpha");

        let agent = make_agent_with_dir(&custom_root);
        assert_eq!(agent.resolved_config_dir(), custom_root);
        assert_eq!(agent.resolved_skill_dir(), custom_root);
    }

    #[test]
    fn resolves_config_root_to_skills_subdir() {
        let temp = TempDir::new().unwrap();
        let cfg = temp.path().join("my-codex-config");
        let skills = cfg.join("skills");
        make_skill(&skills, "alpha");

        let agent = make_agent_with_dir(&cfg);
        assert_eq!(agent.resolved_config_dir(), cfg);
        assert_eq!(agent.resolved_skill_dir(), skills);
    }
}
