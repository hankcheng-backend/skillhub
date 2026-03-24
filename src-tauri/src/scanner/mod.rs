pub mod frontmatter;

use crate::db::models::{Agent, Skill, SkillSync};
use crate::error::AppError;
use rusqlite::Connection;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct PendingSync {
    skill_id: String,
    agent: String,
    symlink_path: String,
}

pub fn scan_all(conn: &Connection) -> Result<Vec<Skill>, AppError> {
    let agents = Agent::enabled(conn)?;
    let all_agents = Agent::all(conn)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let agent_roots = all_agents
        .iter()
        .map(|a| (a.id.clone(), normalize_path(&a.resolved_skill_dir())))
        .collect::<Vec<_>>();

    let mut seen_skill_ids = HashSet::new();
    let mut seen_sync_pairs = HashSet::new();
    let mut pending_syncs = Vec::new();

    for agent in &agents {
        let skill_dir = agent.resolved_skill_dir();
        if !skill_dir.exists() {
            continue;
        }
        scan_agent_dir(
            conn,
            agent,
            &skill_dir,
            now,
            &agent_roots,
            &mut seen_skill_ids,
            &mut pending_syncs,
        )?;
    }

    materialize_pending_syncs(conn, &pending_syncs, &mut seen_sync_pairs)?;

    cleanup_stale_rows(conn, &agents, &seen_skill_ids, &seen_sync_pairs)?;

    let skills = Skill::all_with_syncs(conn)?;
    Ok(skills)
}

fn scan_agent_dir(
    conn: &Connection,
    agent: &Agent,
    skill_dir: &Path,
    now: i64,
    agent_roots: &[(String, PathBuf)],
    seen_skill_ids: &mut HashSet<String>,
    pending_syncs: &mut Vec<PendingSync>,
) -> Result<(), AppError> {
    let entries = fs::read_dir(skill_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let folder_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let metadata = fs::symlink_metadata(&path)?;
        let is_symlink = metadata.is_symlink();

        let skill_md = find_skill_md(&path);
        if skill_md.is_none() {
            continue;
        }
        let skill_md = skill_md.unwrap();

        if is_symlink {
            let target = fs::read_link(&path)?;
            let origin_agent = detect_origin_agent(&target, &path, agent_roots);
            if let Some(origin) = origin_agent {
                let skill_id = format!("{}:{}", origin, folder_name);
                pending_syncs.push(PendingSync {
                    skill_id,
                    agent: agent.id.clone(),
                    symlink_path: path.to_string_lossy().to_string(),
                });
            }
        } else {
            upsert_local_skill(conn, agent, &folder_name, &skill_md, now, seen_skill_ids)?;
        }
    }
    Ok(())
}

fn upsert_local_skill(
    conn: &Connection,
    agent: &Agent,
    folder_name: &str,
    skill_md: &Path,
    now: i64,
    seen_skill_ids: &mut HashSet<String>,
) -> Result<(), AppError> {
    let skill_id = format!("{}:{}", agent.id, folder_name);
    let content = fs::read_to_string(skill_md).unwrap_or_default();
    let fm = frontmatter::parse_frontmatter(&content).unwrap_or_default();

    Skill::upsert(
        conn,
        &Skill {
            id: skill_id.clone(),
            folder_name: folder_name.to_string(),
            origin_agent: agent.id.clone(),
            name: fm.name,
            description: fm.description,
            tags: None,
            notes: None,
            discovered_at: Some(now),
            updated_at: Some(now),
            synced_to: vec![],
        },
    )?;
    seen_skill_ids.insert(skill_id);
    Ok(())
}

fn materialize_pending_syncs(
    conn: &Connection,
    pending_syncs: &[PendingSync],
    seen_sync_pairs: &mut HashSet<(String, String)>,
) -> Result<(), AppError> {
    for pending in pending_syncs {
        if !skill_exists(conn, &pending.skill_id)? {
            continue;
        }

        SkillSync::insert(
            conn,
            &SkillSync {
                skill_id: pending.skill_id.clone(),
                agent: pending.agent.clone(),
                symlink_path: Some(pending.symlink_path.clone()),
            },
        )?;
        seen_sync_pairs.insert((pending.skill_id.clone(), pending.agent.clone()));
    }

    Ok(())
}

fn skill_exists(conn: &Connection, skill_id: &str) -> Result<bool, AppError> {
    let exists: i64 = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM skills WHERE id = ?1)",
        rusqlite::params![skill_id],
        |row| row.get(0),
    )?;
    Ok(exists == 1)
}

fn find_skill_md(dir: &Path) -> Option<std::path::PathBuf> {
    let candidates = ["skill.md", "SKILL.md"];
    for name in &candidates {
        let p = dir.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn detect_origin_agent(
    target: &Path,
    symlink_path: &Path,
    agent_roots: &[(String, PathBuf)],
) -> Option<String> {
    let resolved_target = if target.is_absolute() {
        normalize_path(target)
    } else {
        let parent = symlink_path.parent()?;
        normalize_path(&parent.join(target))
    };

    for (agent_id, root) in agent_roots {
        if resolved_target.starts_with(root) {
            return Some(agent_id.clone());
        }
    }
    None
}

fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(canon) = std::fs::canonicalize(path) {
        return canon;
    }

    if path.is_absolute() {
        return path.to_path_buf();
    }

    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

fn cleanup_stale_rows(
    conn: &Connection,
    enabled_agents: &[Agent],
    seen_skill_ids: &HashSet<String>,
    seen_sync_pairs: &HashSet<(String, String)>,
) -> Result<(), AppError> {
    let enabled_ids = enabled_agents
        .iter()
        .map(|a| a.id.clone())
        .collect::<HashSet<_>>();

    let mut stale_skills = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT id, origin_agent FROM skills")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (skill_id, origin_agent) = row?;
            if !enabled_ids.contains(&origin_agent) || !seen_skill_ids.contains(&skill_id) {
                stale_skills.push(skill_id);
            }
        }
    }

    for skill_id in stale_skills {
        SkillSync::delete_all_for_skill(conn, &skill_id)?;
        Skill::delete(conn, &skill_id)?;
    }

    let mut stale_syncs = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT skill_id, agent FROM skill_syncs")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let pair = row?;
            if !enabled_ids.contains(&pair.1) || !seen_sync_pairs.contains(&pair) {
                stale_syncs.push(pair);
            }
        }
    }

    for (skill_id, agent) in stale_syncs {
        SkillSync::delete(conn, &skill_id, &agent)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_skill(dir: &Path) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("skill.md"),
            "---\nname: test\ndescription: test\n---\nbody",
        )
        .unwrap();
    }

    fn setup_db(temp: &TempDir) -> rusqlite::Connection {
        let db_path = temp.path().join("skillhub-test.db");
        let (conn, _is_fresh) = crate::db::init_db(&db_path).unwrap();
        conn
    }

    #[test]
    fn scan_removes_deleted_skill_rows() {
        let temp = TempDir::new().unwrap();
        let claude_cfg = temp.path().join(".claude");
        let claude_skills = claude_cfg.join("skills");
        make_skill(&claude_skills.join("alpha"));

        let conn = setup_db(&temp);
        conn.execute(
            "UPDATE agents SET enabled = 1, skill_dir = ?1 WHERE id = 'claude'",
            rusqlite::params![claude_cfg.to_string_lossy().to_string()],
        )
        .unwrap();
        conn.execute(
            "UPDATE agents SET enabled = 0 WHERE id IN ('codex', 'gemini')",
            [],
        )
        .unwrap();

        let first = scan_all(&conn).unwrap();
        assert_eq!(first.len(), 1);

        std::fs::remove_dir_all(claude_skills.join("alpha")).unwrap();
        let second = scan_all(&conn).unwrap();
        assert!(second.is_empty());
    }

    #[test]
    fn scan_detects_and_cleans_symlinks_for_custom_dirs() {
        let temp = TempDir::new().unwrap();
        let claude_cfg = temp.path().join(".claude");
        let codex_cfg = temp.path().join(".codex");
        let claude_skills = claude_cfg.join("skills");
        let codex_skills = codex_cfg.join("skills");
        std::fs::create_dir_all(&codex_skills).unwrap();
        make_skill(&claude_skills.join("beta"));

        #[cfg(unix)]
        std::os::unix::fs::symlink(claude_skills.join("beta"), codex_skills.join("beta")).unwrap();

        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(claude_skills.join("beta"), codex_skills.join("beta"))
            .unwrap();

        let conn = setup_db(&temp);
        conn.execute(
            "UPDATE agents SET enabled = 1, skill_dir = ?1 WHERE id = 'claude'",
            rusqlite::params![claude_cfg.to_string_lossy().to_string()],
        )
        .unwrap();
        conn.execute(
            "UPDATE agents SET enabled = 1, skill_dir = ?1 WHERE id = 'codex'",
            rusqlite::params![codex_cfg.to_string_lossy().to_string()],
        )
        .unwrap();
        conn.execute("UPDATE agents SET enabled = 0 WHERE id = 'gemini'", [])
            .unwrap();

        let scanned = scan_all(&conn).unwrap();
        let skill = scanned.iter().find(|s| s.id == "claude:beta").unwrap();
        assert_eq!(skill.synced_to, vec!["codex".to_string()]);

        #[cfg(unix)]
        std::fs::remove_file(codex_skills.join("beta")).unwrap();

        #[cfg(windows)]
        std::fs::remove_dir(codex_skills.join("beta")).unwrap();
        let rescanned = scan_all(&conn).unwrap();
        let skill = rescanned.iter().find(|s| s.id == "claude:beta").unwrap();
        assert!(skill.synced_to.is_empty());
    }

    #[test]
    fn scan_removes_skills_when_agent_disabled() {
        let temp = TempDir::new().unwrap();
        let claude_cfg = temp.path().join(".claude");
        let claude_skills = claude_cfg.join("skills");
        make_skill(&claude_skills.join("alpha"));

        let conn = setup_db(&temp);
        conn.execute(
            "UPDATE agents SET enabled = 1, skill_dir = ?1 WHERE id = 'claude'",
            rusqlite::params![claude_cfg.to_string_lossy().to_string()],
        )
        .unwrap();
        conn.execute(
            "UPDATE agents SET enabled = 0 WHERE id IN ('codex', 'gemini')",
            [],
        )
        .unwrap();

        let first = scan_all(&conn).unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].id, "claude:alpha");

        // Disable claude agent
        conn.execute("UPDATE agents SET enabled = 0 WHERE id = 'claude'", [])
            .unwrap();

        let second = scan_all(&conn).unwrap();
        assert!(second.is_empty(), "disabled agent skills should be removed");
    }

    #[test]
    fn scan_handles_symlink_before_origin_is_upserted() {
        let temp = TempDir::new().unwrap();
        let claude_cfg = temp.path().join(".claude");
        let codex_cfg = temp.path().join(".codex");
        let claude_skills = claude_cfg.join("skills");
        let codex_skills = codex_cfg.join("skills");
        std::fs::create_dir_all(&claude_skills).unwrap();
        make_skill(&codex_skills.join("delta"));

        // Symlink sits in claude dir, while origin physical folder lives in codex dir.
        #[cfg(unix)]
        std::os::unix::fs::symlink(codex_skills.join("delta"), claude_skills.join("delta"))
            .unwrap();

        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(codex_skills.join("delta"), claude_skills.join("delta"))
            .unwrap();

        let conn = setup_db(&temp);
        conn.execute(
            "UPDATE agents SET enabled = 1, skill_dir = ?1 WHERE id = 'claude'",
            rusqlite::params![claude_cfg.to_string_lossy().to_string()],
        )
        .unwrap();
        conn.execute(
            "UPDATE agents SET enabled = 1, skill_dir = ?1 WHERE id = 'codex'",
            rusqlite::params![codex_cfg.to_string_lossy().to_string()],
        )
        .unwrap();
        conn.execute("UPDATE agents SET enabled = 0 WHERE id = 'gemini'", [])
            .unwrap();

        let scanned = scan_all(&conn).unwrap();
        let origin = scanned.iter().find(|s| s.id == "codex:delta").unwrap();
        assert_eq!(origin.synced_to, vec!["claude".to_string()]);
    }
}
