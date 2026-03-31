use crate::commands::skills::DbState;
use crate::db::models::Agent;
use crate::error::AppError;
use crate::scanner;
use crate::watcher;
use rusqlite;
use serde::Serialize;
use std::process::{Command, Output};
use std::sync::OnceLock;
use tauri::{Emitter, State};
use tauri_plugin_autostart::ManagerExt;

/// Returns the full PATH as seen by a login shell.
/// macOS GUI apps launched via Finder/Launchpad only inherit a minimal PATH
/// (e.g. /usr/bin:/bin:/usr/sbin:/sbin), so CLI tools installed via npm/nvm/Homebrew
/// are invisible. This function spawns a login shell once to capture the real PATH
/// and caches the result for the lifetime of the process.
fn shell_path() -> &'static str {
    static CACHED: OnceLock<String> = OnceLock::new();
    CACHED.get_or_init(|| {
        // Try the user's default shell first, fall back to /bin/zsh (macOS default)
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let output = Command::new(&shell)
            .args(["-ilc", "echo $PATH"])
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !path.is_empty() {
                    return path;
                }
            }
            _ => {}
        }
        // Fallback: current PATH + common bin locations
        let current = std::env::var("PATH").unwrap_or_default();
        let home = dirs::home_dir().map(|h| h.to_string_lossy().to_string()).unwrap_or_default();
        format!(
            "{}:/usr/local/bin:/opt/homebrew/bin:{}/.nvm/versions/node/current/bin:{}/bin:{}/.local/bin",
            current, home, home, home
        )
    })
}

/// Create a Command with the full shell PATH injected.
fn command_with_path(program: &str) -> std::process::Command {
    let mut cmd = Command::new(program);
    cmd.env("PATH", shell_path());
    cmd
}

fn output_text(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{}\n{}", stdout, stderr),
        (false, true) => stdout,
        (true, false) => stderr,
        (true, true) => String::new(),
    }
}

fn normalize_version_token(token: &str) -> Option<String> {
    let trimmed = token
        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-' && c != '+');
    let trimmed = trimmed.strip_prefix('v').unwrap_or(trimmed);

    let first = trimmed.chars().next()?;
    if !first.is_ascii_digit() || !trimmed.contains('.') {
        return None;
    }

    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'))
    {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn parse_version_text(text: &str) -> Option<String> {
    text.split_whitespace().find_map(normalize_version_token)
}

fn resolve_command_path(program: &str) -> Option<String> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let output = Command::new(&shell)
        .args(["-ilc", &format!("command -v {}", program)])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with('/'))
        .map(|line| line.to_string())
}

fn probe_version(program: &str) -> Option<String> {
    let output = command_with_path(program).arg("--version").output().ok()?;
    let text = output_text(&output);
    parse_version_text(&text)
}

fn detect_agent_cli(program: &str) -> (bool, Option<String>) {
    if let Some(version) = probe_version(program) {
        return (true, Some(version));
    }

    if let Some(path) = resolve_command_path(program) {
        return match probe_version(&path) {
            Some(version) => (true, Some(version)),
            None => (true, None),
        };
    }

    (false, None)
}

#[derive(Serialize)]
pub struct AgentVersion {
    pub id: String,
    pub installed: bool,
    pub current_version: Option<String>,
}

#[derive(Serialize)]
pub struct LatestVersions {
    pub claude: Option<String>,
    pub codex: Option<String>,
    pub gemini: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "status")]
pub enum AgentDirStatus {
    Ok { path: String },
    NotInstalled { install_cmd: String },
    DirMissing { path: String },
}

#[tauri::command]
pub fn get_agents(db: State<'_, DbState>) -> Result<Vec<Agent>, AppError> {
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    Agent::all(&conn).map_err(AppError::from)
}

#[tauri::command]
pub fn update_agent(
    db: State<'_, DbState>,
    watcher: State<'_, watcher::WatcherState>,
    app: tauri::AppHandle,
    agent_id: String,
    enabled: bool,
    skill_dir: Option<String>,
) -> Result<(), AppError> {
    {
        let conn = db
            .lock()
            .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
        Agent::update(&conn, &agent_id, enabled, skill_dir.as_deref())?;
    }

    {
        let conn = db
            .lock()
            .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
        let _ = scanner::scan_all(&conn)?;
    }

    // Dynamic watcher registration — add newly enabled agent's skill dir (D-11)
    if enabled {
        let conn = db
            .lock()
            .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
        let agent = Agent::find_by_id(&conn, &agent_id)?;
        let dir = agent.resolved_skill_dir();
        if dir.exists() {
            let _ = watcher::add_paths(&watcher, &[dir]);
        }
    }

    let _ = app.emit("skills-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn get_agent_versions() -> Result<Vec<AgentVersion>, AppError> {
    tokio::task::spawn_blocking(|| {
        let agents_cmds = [
            ("claude", "claude"),
            ("codex", "codex"),
            ("gemini", "gemini"),
        ];

        let mut results = vec![];
        for (id, cmd) in &agents_cmds {
            let (installed, current_version) = detect_agent_cli(cmd);
            results.push(AgentVersion {
                id: id.to_string(),
                installed,
                current_version,
            });
        }
        Ok(results)
    })
    .await
    .map_err(|e| AppError::Remote(e.to_string()))?
}

#[tauri::command]
pub async fn get_latest_versions() -> Result<LatestVersions, AppError> {
    let packages = [
        ("claude", "@anthropic-ai/claude-code"),
        ("codex", "@openai/codex"),
        ("gemini", "@google/gemini-cli"),
    ];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Remote(e.to_string()))?;

    let mut results: std::collections::HashMap<String, Option<String>> =
        std::collections::HashMap::new();

    for (id, pkg) in &packages {
        let url = format!("https://registry.npmjs.org/{}/latest", pkg);
        let version = match client.get(&url).send().await {
            Ok(resp) => match resp.json::<serde_json::Value>().await {
                Ok(json) => json
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                Err(_) => None,
            },
            Err(_) => None,
        };
        results.insert(id.to_string(), version);
    }

    Ok(LatestVersions {
        claude: results.remove("claude").flatten(),
        codex: results.remove("codex").flatten(),
        gemini: results.remove("gemini").flatten(),
    })
}

#[tauri::command]
pub fn get_config(db: State<'_, DbState>, key: String) -> Option<String> {
    let conn = db.lock().ok()?;
    conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get::<_, String>(0),
    )
    .ok()
}

#[tauri::command]
pub fn set_config(db: State<'_, DbState>, key: String, value: String) -> Result<(), AppError> {
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    conn.execute(
        "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

#[tauri::command]
pub fn get_autostart(app: tauri::AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
pub fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<(), AppError> {
    if enabled {
        app.autolaunch()
            .enable()
            .map_err(|e| AppError::Remote(e.to_string()))
    } else {
        app.autolaunch()
            .disable()
            .map_err(|e| AppError::Remote(e.to_string()))
    }
}

#[tauri::command]
pub async fn check_agent_dir(
    db: State<'_, DbState>,
    agent_id: String,
) -> Result<AgentDirStatus, AppError> {
    let agent = {
        let conn = db
            .lock()
            .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
        Agent::find_by_id(&conn, &agent_id)?
    };

    let config_root = agent.resolved_config_dir();

    if config_root.is_dir() {
        return Ok(AgentDirStatus::Ok {
            path: config_root.to_string_lossy().to_string(),
        });
    }

    // Config dir missing — check if the CLI is installed
    let install_cmds: std::collections::HashMap<&str, &str> = [
        ("claude", "npm install -g @anthropic-ai/claude-code"),
        ("codex", "npm install -g @openai/codex"),
        ("gemini", "npm install -g @google/gemini-cli"),
    ]
    .into_iter()
    .collect();

    let agent_id_clone = agent_id.clone();
    let is_installed = tokio::task::spawn_blocking(move || detect_agent_cli(&agent_id_clone).0)
        .await
        .unwrap_or(false);

    if !is_installed {
        let cmd = install_cmds
            .get(agent_id.as_str())
            .unwrap_or(&"")
            .to_string();
        Ok(AgentDirStatus::NotInstalled { install_cmd: cmd })
    } else {
        Ok(AgentDirStatus::DirMissing {
            path: config_root.to_string_lossy().to_string(),
        })
    }
}

#[tauri::command]
pub async fn pick_agent_dir(app: tauri::AppHandle) -> Result<Option<String>, AppError> {
    use tauri_plugin_dialog::DialogExt;

    let picked = app.dialog().file().blocking_pick_folder();

    Ok(picked.map(|p| p.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{normalize_version_token, parse_version_text};

    #[test]
    fn parses_plain_semver_output() {
        assert_eq!(
            parse_version_text("codex-cli 0.117.0"),
            Some("0.117.0".to_string())
        );
    }

    #[test]
    fn parses_semver_when_warning_precedes_version() {
        let output = "WARNING: proceeding, even though we could not update PATH: Operation not permitted (os error 1)\ncodex-cli 0.117.0";
        assert_eq!(parse_version_text(output), Some("0.117.0".to_string()));
    }

    #[test]
    fn ignores_non_version_number_tokens() {
        assert_eq!(normalize_version_token("1)"), None);
        assert_eq!(normalize_version_token("(123)"), None);
    }
}
