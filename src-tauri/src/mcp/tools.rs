use crate::db::models::{Agent, Skill, SkillSync, Source};
use crate::remote::oauth;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

type Db = Arc<Mutex<Connection>>;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn auto_scan(db: &Db) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    crate::scanner::scan_all(&conn)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn require_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, String> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("missing {}", key))
}

fn opt_str<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params.get(key).and_then(|v| v.as_str())
}

fn opt_bool(params: &Value, key: &str) -> bool {
    params
        .get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn skill_to_json(s: &Skill) -> Value {
    json!({
        "id": s.id,
        "folder_name": s.folder_name,
        "name": s.name,
        "description": s.description,
        "origin_agent": s.origin_agent,
        "synced_to": s.synced_to,
        "source": "local",
    })
}

fn matches_query(q: &str, fields: &[Option<&str>]) -> bool {
    if q.is_empty() {
        return true;
    }
    fields
        .iter()
        .any(|f| f.unwrap_or("").to_lowercase().contains(q))
}

async fn fetch_remote_skills(
    db: &Db,
    source_filter: Option<&str>,
    query: &str,
) -> (Vec<Value>, Vec<Value>) {
    let sources = {
        let conn = match db.lock() {
            Ok(c) => c,
            Err(_) => return (vec![], vec![json!({"error": "db lock failed"})]),
        };
        match Source::all(&conn) {
            Ok(all) => match source_filter {
                Some(id) => all.into_iter().filter(|s| s.id == id).collect::<Vec<_>>(),
                None => all,
            },
            Err(e) => return (vec![], vec![json!({"error": e.to_string()})]),
        }
    };

    let mut results = vec![];
    let mut errors = vec![];
    let q = query.to_lowercase();

    for source in &sources {
        if source.source_type != "gitlab" {
            continue;
        }
        let repo_url = match &source.url {
            Some(u) => u.clone(),
            None => continue,
        };
        let token = {
            let conn = match db.lock() {
                Ok(c) => c,
                Err(_) => continue,
            };
            crate::commands::sources::get_source_token(&conn, &source.id)
        };
        let token = match token {
            Ok(t) => t,
            Err(e) => {
                errors.push(json!({
                    "source_id": source.id,
                    "source_name": source.name,
                    "error": e.to_string(),
                }));
                continue;
            }
        };

        match crate::remote::gitlab::list_skills(&repo_url, &token).await {
            Ok(skills) => {
                for mut skill in skills {
                    skill.source_id = source.id.clone();
                    skill.source_name = source.name.clone();
                    if !matches_query(
                        &q,
                        &[
                            skill.name.as_deref(),
                            Some(&skill.folder_name),
                            skill.description.as_deref(),
                        ],
                    ) {
                        continue;
                    }
                    results.push(json!({
                        "folder_name": skill.folder_name,
                        "name": skill.name,
                        "description": skill.description,
                        "source_id": skill.source_id,
                        "source_name": skill.source_name,
                        "updated_at": skill.updated_at,
                        "updated_by": skill.updated_by,
                        "source": "remote",
                    }));
                }
            }
            Err(e) => {
                errors.push(json!({
                    "source_id": source.id,
                    "source_name": source.name,
                    "error": e.to_string(),
                }));
            }
        }
    }
    (results, errors)
}

// ---------------------------------------------------------------------------
// 1. search_skills — unified local + remote search
//    params: { query, scope?: "local"|"remote"|"all" (default "all"), source_id? }
// ---------------------------------------------------------------------------
pub async fn search_skills(db: &Db, params: Value) -> Result<Value, String> {
    let query = opt_str(&params, "query").unwrap_or("");
    let scope = opt_str(&params, "scope").unwrap_or("all");
    let q = query.to_lowercase();

    let mut all_results: Vec<Value> = vec![];
    let mut all_errors: Vec<Value> = vec![];

    // Local
    if scope == "local" || scope == "all" {
        auto_scan(db)?;
        let conn = db.lock().map_err(|e| e.to_string())?;
        let skills = Skill::all_with_syncs(&conn).map_err(|e| e.to_string())?;
        for s in &skills {
            if matches_query(
                &q,
                &[
                    s.name.as_deref(),
                    Some(&s.folder_name),
                    s.description.as_deref(),
                    s.tags.as_deref(),
                ],
            ) {
                all_results.push(skill_to_json(s));
            }
        }
    }

    // Remote
    if scope == "remote" || scope == "all" {
        let source_filter = opt_str(&params, "source_id");
        let (remote_results, remote_errors) =
            fetch_remote_skills(db, source_filter, query).await;
        all_results.extend(remote_results);
        all_errors.extend(remote_errors);
    }

    Ok(json!({
        "skills": all_results,
        "errors": all_errors,
    }))
}

// ---------------------------------------------------------------------------
// 2. list_local_skills — list all local skills with auto scan
//    params: { agent? }
// ---------------------------------------------------------------------------
pub fn list_local_skills(db: &Db, params: Value) -> Result<Value, String> {
    auto_scan(db)?;
    let conn = db.lock().map_err(|e| e.to_string())?;
    let agent_filter = opt_str(&params, "agent");

    let skills = Skill::all_with_syncs(&conn).map_err(|e| e.to_string())?;
    let filtered: Vec<Value> = skills
        .iter()
        .filter(|s| {
            agent_filter.map_or(true, |a| {
                s.origin_agent == a || s.synced_to.contains(&a.to_string())
            })
        })
        .map(|s| skill_to_json(s))
        .collect();

    Ok(json!(filtered))
}

// ---------------------------------------------------------------------------
// 3. get_skill_content — read skill.md from local or remote
//    Local:  { skill_id: "agent:folder_name" }
//    Remote: { source_id, folder_name }
// ---------------------------------------------------------------------------
pub async fn get_skill_content(db: &Db, params: Value) -> Result<Value, String> {
    // Remote path
    if let (Some(source_id), Some(folder_name)) =
        (opt_str(&params, "source_id"), opt_str(&params, "folder_name"))
    {
        let (source_type, url, token) = {
            let conn = db.lock().map_err(|e| e.to_string())?;
            let source = Source::find_by_id(&conn, source_id)
                .map_err(|_| format!("Source not found: {}", source_id))?;
            let token = crate::commands::sources::get_source_token(&conn, source_id)
                .map_err(|e| e.to_string())?;
            (source.source_type, source.url, token)
        };
        if source_type != "gitlab" {
            return Err(format!("Unsupported source type: {}", source_type));
        }
        let repo_url = url.ok_or("GitLab source has no URL configured")?;
        let content =
            crate::remote::gitlab::get_skill_content(&repo_url, folder_name, &token)
                .await
                .map_err(|e| e.to_string())?;
        return Ok(json!({
            "source": "remote",
            "source_id": source_id,
            "folder_name": folder_name,
            "content": content,
        }));
    }

    // Local path
    let skill_id = require_str(&params, "skill_id")?;
    let parts: Vec<&str> = skill_id.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid skill ID: {}", skill_id));
    }
    let (agent_id, folder_name) = (parts[0], parts[1]);

    let conn = db.lock().map_err(|e| e.to_string())?;
    let agent = Agent::find_by_id(&conn, agent_id)
        .map_err(|_| format!("Agent not found: {}", agent_id))?;
    let skill_dir = agent.resolved_skill_dir().join(folder_name);
    for name in &["skill.md", "SKILL.md"] {
        let path = skill_dir.join(name);
        if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            return Ok(json!({
                "source": "local",
                "skill_id": skill_id,
                "content": content,
            }));
        }
    }
    Err("skill.md not found".to_string())
}

// ---------------------------------------------------------------------------
// 4. sync_skill
//    params: { skill_id, target_agent }
// ---------------------------------------------------------------------------
pub fn sync_skill_tool(db: &Db, params: Value) -> Result<Value, String> {
    let skill_id = require_str(&params, "skill_id")?;
    let target_agent = require_str(&params, "target_agent")?;

    let conn = db.lock().map_err(|e| e.to_string())?;
    let skills = Skill::all_with_syncs(&conn).map_err(|e| e.to_string())?;
    let skill = skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or("skill not found")?;

    let agents = Agent::all(&conn).map_err(|e| e.to_string())?;
    let origin = agents
        .iter()
        .find(|a| a.id == skill.origin_agent)
        .ok_or("origin agent not found")?;
    let target = agents
        .iter()
        .find(|a| a.id == target_agent)
        .ok_or("target agent not found")?;
    if !target.enabled {
        return Err(format!("target agent '{}' is not enabled", target_agent));
    }

    let origin_path = origin.resolved_skill_dir().join(&skill.folder_name);
    let target_path = target.resolved_skill_dir().join(&skill.folder_name);

    crate::sync::create_sync_link(&origin_path, &target_path).map_err(|e| e.to_string())?;

    SkillSync::insert(
        &conn,
        &SkillSync {
            skill_id: skill_id.to_string(),
            agent: target_agent.to_string(),
            symlink_path: Some(target_path.to_string_lossy().to_string()),
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(json!({"status": "ok"}))
}

// ---------------------------------------------------------------------------
// 5. unsync_skill
//    params: { skill_id, agent }
// ---------------------------------------------------------------------------
pub fn unsync_skill_tool(db: &Db, params: Value) -> Result<Value, String> {
    let skill_id = require_str(&params, "skill_id")?;
    let agent = require_str(&params, "agent")?;

    let conn = db.lock().map_err(|e| e.to_string())?;
    let skills = Skill::all_with_syncs(&conn).map_err(|e| e.to_string())?;
    let skill = skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or("skill not found")?;

    let agents = Agent::all(&conn).map_err(|e| e.to_string())?;
    let target = agents
        .iter()
        .find(|a| a.id == agent)
        .ok_or("agent not found")?;
    let link_path = target.resolved_skill_dir().join(&skill.folder_name);

    crate::sync::remove_sync_link(&link_path).map_err(|e| e.to_string())?;
    SkillSync::delete(&conn, skill_id, agent).map_err(|e| e.to_string())?;

    Ok(json!({"status": "ok"}))
}

// ---------------------------------------------------------------------------
// 6. install_skill — download from remote and install locally
//    params: { source_id, folder_name, target_agent, force? }
// ---------------------------------------------------------------------------
pub async fn install_skill_tool(db: &Db, params: Value) -> Result<Value, String> {
    let source_id = require_str(&params, "source_id")?;
    let folder_name = require_str(&params, "folder_name")?;
    let target_agent = require_str(&params, "target_agent")?;
    let force = opt_bool(&params, "force");

    crate::commands::install::install_skill_with_db(
        db,
        source_id,
        folder_name,
        target_agent,
        force,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "ok",
        "skill_id": format!("{}:{}", target_agent, folder_name),
    }))
}

// ---------------------------------------------------------------------------
// 7. uninstall_skill — delete a local skill
//    params: { skill_id, agent?, confirm? }
// ---------------------------------------------------------------------------
pub fn uninstall_skill_tool(db: &Db, params: Value) -> Result<Value, String> {
    let skill_id = require_str(&params, "skill_id")?;
    let confirm = opt_bool(&params, "confirm");
    let agent = opt_str(&params, "agent");

    let conn = db.lock().map_err(|e| e.to_string())?;

    if let Some(agent_id) = agent {
        let skills = Skill::all_with_syncs(&conn).map_err(|e| e.to_string())?;
        let skill = skills
            .iter()
            .find(|s| s.id == skill_id)
            .ok_or("skill not found")?;

        if agent_id != skill.origin_agent && skill.synced_to.contains(&agent_id.to_string()) {
            let agents = Agent::all(&conn).map_err(|e| e.to_string())?;
            let target = agents
                .iter()
                .find(|a| a.id == agent_id)
                .ok_or("agent not found")?;
            let link_path = target.resolved_skill_dir().join(&skill.folder_name);
            let _ = crate::sync::remove_sync_link(&link_path);
            SkillSync::delete(&conn, skill_id, agent_id).map_err(|e| e.to_string())?;
            return Ok(json!({"status": "ok"}));
        }
    }

    if !confirm {
        return Ok(json!({
            "confirmation_required": true,
            "message": format!("This will permanently delete skill '{}' and all its sync links. Call again with confirm: true to proceed.", skill_id)
        }));
    }

    let skills = Skill::all_with_syncs(&conn).map_err(|e| e.to_string())?;
    let skill = skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or("skill not found")?;

    let agents = Agent::all(&conn).map_err(|e| e.to_string())?;
    for synced in &skill.synced_to {
        if let Some(a) = agents.iter().find(|a| &a.id == synced) {
            let link = a.resolved_skill_dir().join(&skill.folder_name);
            let _ = crate::sync::remove_sync_link(&link);
        }
    }
    SkillSync::delete_all_for_skill(&conn, skill_id).map_err(|e| e.to_string())?;

    if let Some(origin) = agents.iter().find(|a| a.id == skill.origin_agent) {
        let folder = origin.resolved_skill_dir().join(&skill.folder_name);
        if folder.exists() {
            std::fs::remove_dir_all(&folder).map_err(|e| e.to_string())?;
        }
    }

    Skill::delete(&conn, skill_id).map_err(|e| e.to_string())?;
    Ok(json!({"status": "ok"}))
}

// ---------------------------------------------------------------------------
// 8. upload_skill — push a local skill to a remote source
//    params: { source_id, skill_id ("agent:folder_name"), force? }
// ---------------------------------------------------------------------------
pub async fn upload_skill_tool(db: &Db, params: Value) -> Result<Value, String> {
    let source_id = require_str(&params, "source_id")?;
    let skill_id = require_str(&params, "skill_id")?;
    let force = opt_bool(&params, "force");

    let parts: Vec<&str> = skill_id.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid skill ID format: {}", skill_id));
    }
    let (agent_id, folder_name) = (parts[0], parts[1]);

    let (url, skill_dir, token) = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        let source = Source::find_by_id(&conn, source_id)
            .map_err(|_| format!("Source not found: {}", source_id))?;
        if source.source_type != "gitlab" {
            return Err(format!("Unsupported source type: {}", source.source_type));
        }
        let url = source
            .url
            .ok_or("GitLab source has no URL configured")?;
        let agent = Agent::find_by_id(&conn, agent_id)
            .map_err(|_| format!("Agent not found: {}", agent_id))?;
        let skill_dir = agent.resolved_skill_dir();
        let token = crate::commands::sources::get_source_token(&conn, source_id)
            .map_err(|e| e.to_string())?;
        (url, skill_dir, token)
    };

    let local_path = skill_dir.join(folder_name);
    if !local_path.exists() {
        return Err(format!("Skill folder '{}' not found on disk", folder_name));
    }

    crate::remote::gitlab::upload_skill(&url, folder_name, &token, &skill_dir, force)
        .await
        .map_err(|e| e.to_string())?;

    Ok(json!({"status": "ok"}))
}

// ---------------------------------------------------------------------------
// 9. list_sources
// ---------------------------------------------------------------------------
pub fn list_sources_tool(db: &Db) -> Result<Value, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let sources = Source::all(&conn).map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(sources).map_err(|e| e.to_string())?)
}

// ---------------------------------------------------------------------------
// 10. add_source — add a new GitLab source
//     params: { name, source_type, url, token }
// ---------------------------------------------------------------------------
pub fn add_source_tool(db: &Db, params: Value) -> Result<Value, String> {
    let name = require_str(&params, "name")?.trim();
    let source_type = require_str(&params, "source_type")?;
    let url = opt_str(&params, "url").map(|s| s.trim().to_string());
    let token = opt_str(&params, "token").map(|s| s.trim().to_string());

    if name.is_empty() {
        return Err("Source name cannot be empty".into());
    }
    match source_type {
        "gitlab" => {
            if url.as_ref().map_or(true, |u| u.is_empty()) {
                return Err("GitLab source URL is required".into());
            }
            if token.as_ref().map_or(true, |t| t.is_empty()) {
                return Err("GitLab Personal Access Token is required".into());
            }
        }
        other => return Err(format!("Unsupported source type: {}", other)),
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
        name: name.to_string(),
        source_type: source_type.to_string(),
        url: url.filter(|v| !v.is_empty()),
        folder_id: None,
        keychain_key: None,
        refresh_token_key: None,
        added_at,
    };

    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        Source::insert(&conn, &source).map_err(|e| e.to_string())?;
    }

    if let Some(tok) = token.filter(|v| !v.is_empty()) {
        let keychain_key = format!("skillhub-{}", source_id);
        match oauth::store_token("skillhub", &keychain_key, &tok) {
            Ok(()) => {
                let conn = db.lock().map_err(|e| e.to_string())?;
                Source::update_keychain_key(&conn, &source_id, &keychain_key)
                    .map_err(|e| e.to_string())?;
            }
            Err(e) => {
                let conn = db.lock().map_err(|e| e.to_string())?;
                let _ = Source::delete(&conn, &source_id);
                return Err(e.to_string());
            }
        }
    }

    let conn = db.lock().map_err(|e| e.to_string())?;
    let saved = Source::find_by_id(&conn, &source_id).map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(saved).map_err(|e| e.to_string())?)
}

// ---------------------------------------------------------------------------
// 11. remove_source
//     params: { source_id }
// ---------------------------------------------------------------------------
pub fn remove_source_tool(db: &Db, params: Value) -> Result<Value, String> {
    let source_id = require_str(&params, "source_id")?;

    let keychain_key = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        let source =
            Source::find_by_id(&conn, source_id).map_err(|_| "Source not found".to_string())?;
        Source::delete(&conn, source_id).map_err(|e| e.to_string())?;
        source.keychain_key
    };
    if let Some(ref key) = keychain_key {
        if let Ok(entry) = keyring::Entry::new("skillhub", key) {
            let _ = entry.delete_credential();
        }
    }

    Ok(json!({"status": "ok"}))
}

// ---------------------------------------------------------------------------
// 12. browse_source — list all skills in one remote source
//     params: { source_id }
// ---------------------------------------------------------------------------
pub async fn browse_source_tool(db: &Db, params: Value) -> Result<Value, String> {
    let source_id = require_str(&params, "source_id")?;

    let (results, errors) = fetch_remote_skills(db, Some(source_id), "").await;
    Ok(json!({
        "skills": results,
        "errors": errors,
    }))
}

// ---------------------------------------------------------------------------
// 13. get_agents — list all agents and their status
// ---------------------------------------------------------------------------
pub fn get_agents_tool(db: &Db) -> Result<Value, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let agents = Agent::all(&conn).map_err(|e| e.to_string())?;
    let result: Vec<Value> = agents
        .iter()
        .map(|a| {
            json!({
                "id": a.id,
                "enabled": a.enabled,
                "skill_dir": a.skill_dir,
                "config_dir": a.resolved_config_dir().to_string_lossy(),
                "skill_dir_resolved": a.resolved_skill_dir().to_string_lossy(),
            })
        })
        .collect();
    Ok(json!(result))
}
