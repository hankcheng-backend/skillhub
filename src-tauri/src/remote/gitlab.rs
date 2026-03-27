use super::RemoteSkill;
use crate::error::AppError;
use crate::scanner::frontmatter::parse_frontmatter;
use percent_encoding::percent_decode_str;
use serde::Deserialize;

#[derive(Deserialize)]
struct TreeItem {
    name: String,
    #[serde(rename = "type")]
    item_type: String,
    path: String,
}

#[derive(Deserialize)]
struct ProjectInfo {
    default_branch: Option<String>,
}

#[derive(Deserialize)]
struct CommitInfo {
    authored_date: Option<String>,
    author_name: Option<String>,
}

fn parse_repo_url(repo_url: &str) -> Result<(String, String), AppError> {
    let parsed = reqwest::Url::parse(repo_url.trim())
        .map_err(|_| AppError::Remote(format!("Invalid repo URL: {}", repo_url)))?;
    let scheme = parsed.scheme();
    if scheme != "https" && scheme != "http" {
        return Err(AppError::Remote(format!("Invalid repo URL: {}", repo_url)));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| AppError::Remote(format!("Invalid repo URL: {}", repo_url)))?;
    let host_with_port = if let Some(port) = parsed.port() {
        format!("{}:{}", host, port)
    } else {
        host.to_string()
    };

    let segments = parsed
        .path_segments()
        .ok_or_else(|| AppError::Remote(format!("Invalid repo URL: {}", repo_url)))?
        .filter(|s| !s.is_empty())
        .map(encode_component)
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return Err(AppError::Remote(format!("Invalid repo URL: {}", repo_url)));
    }
    let encoded_path = segments.join("%2F");

    Ok((host_with_port, encoded_path))
}

fn encode_component(value: &str) -> String {
    let decoded = percent_decode_str(value).decode_utf8_lossy();
    let mut url = reqwest::Url::parse("https://example.com").expect("valid static URL");
    {
        let mut segments = url.path_segments_mut().expect("base URL can be a base");
        segments.push(decoded.as_ref());
    }
    url.path().trim_start_matches('/').to_string()
}

fn encode_repo_path(path: &str) -> String {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(encode_component)
        .collect::<Vec<_>>()
        .join("%2F")
}

fn build_client(token: &str) -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "PRIVATE-TOKEN",
                reqwest::header::HeaderValue::from_str(token)
                    .map_err(|e| AppError::Remote(format!("Invalid token: {}", e)))?,
            );
            headers
        })
        .build()
        .map_err(|e| AppError::Remote(e.to_string()))
}

async fn get_default_branch(
    client: &reqwest::Client,
    host: &str,
    project: &str,
) -> Result<Option<String>, AppError> {
    let url = format!("https://{}/api/v4/projects/{}", host, project);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Remote(format!("GitLab API error: {}", e)))?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::TokenExpired("unauthorized".into()));
    }
    if !resp.status().is_success() {
        return Err(AppError::Remote(format!(
            "GitLab project info failed ({}): {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        )));
    }

    let info: ProjectInfo = resp
        .json()
        .await
        .map_err(|e| AppError::Remote(format!("Failed to parse project info: {}", e)))?;
    Ok(info.default_branch)
}

async fn fetch_last_commit(
    client: &reqwest::Client,
    host: &str,
    project: &str,
    branch: &str,
    path: &str,
) -> Result<Option<CommitInfo>, AppError> {
    let branch_encoded = encode_component(branch);
    let path_encoded = encode_component(path);
    let url = format!(
        "https://{}/api/v4/projects/{}/repository/commits?ref_name={}&path={}&per_page=1",
        host, project, branch_encoded, path_encoded
    );
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Remote(format!("GitLab API error: {}", e)))?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let commits: Vec<CommitInfo> = resp
        .json()
        .await
        .map_err(|e| AppError::Remote(format!("Failed to parse commits: {}", e)))?;
    Ok(commits.into_iter().next())
}

async fn fetch_skill_md(
    client: &reqwest::Client,
    host: &str,
    project: &str,
    branch: &str,
    folder: &str,
) -> Result<Option<String>, AppError> {
    let branch_encoded = encode_component(branch);
    for filename in &["skill.md", "SKILL.md"] {
        let file_path = encode_repo_path(&format!("{}/{}", folder, filename));
        let url = format!(
            "https://{}/api/v4/projects/{}/repository/files/{}/raw?ref={}",
            host, project, file_path, branch_encoded
        );
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Remote(format!("GitLab API error: {}", e)))?;

        if resp.status().is_success() {
            let text = resp
                .text()
                .await
                .map_err(|e| AppError::Remote(format!("Failed to read response: {}", e)))?;
            return Ok(Some(text));
        }
        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AppError::TokenExpired("unauthorized".into()));
        }
        if resp.status().as_u16() != 404 {
            return Err(AppError::Remote(format!(
                "GitLab file fetch failed ({})",
                resp.status()
            )));
        }
    }
    Ok(None)
}

pub async fn get_skill_content(
    repo_url: &str,
    folder_name: &str,
    token: &str,
) -> Result<Option<String>, AppError> {
    let (host, project) = parse_repo_url(repo_url)?;
    let client = build_client(token)?;
    let branch = get_default_branch(&client, &host, &project)
        .await?
        .ok_or_else(|| AppError::Remote("Repository is empty".into()))?;
    fetch_skill_md(&client, &host, &project, &branch, folder_name).await
}

pub async fn validate_source_access(repo_url: &str, token: &str) -> Result<(), AppError> {
    let (host, project) = parse_repo_url(repo_url)?;
    let client = build_client(token)?;
    // Project info endpoint validates both repository visibility and token access.
    let _ = get_default_branch(&client, &host, &project).await?;
    Ok(())
}

pub async fn list_skills(repo_url: &str, token: &str) -> Result<Vec<RemoteSkill>, AppError> {
    let (host, project) = parse_repo_url(repo_url)?;
    let client = build_client(token)?;

    let branch = match get_default_branch(&client, &host, &project).await? {
        Some(b) => b,
        None => return Ok(vec![]), // empty repo has no skills
    };
    let branch_encoded = encode_component(&branch);

    // Pagination loop — fetch all tree pages (D-11), cap at 500 items / 5 pages (D-12)
    let max_pages = 5;
    let mut all_items: Vec<TreeItem> = Vec::new();
    let mut page = 1u32;

    loop {
        let tree_url = format!(
            "https://{}/api/v4/projects/{}/repository/tree?per_page=100&ref={}&page={}",
            host, project, branch_encoded, page
        );
        let resp = client
            .get(&tree_url)
            .send()
            .await
            .map_err(|e| AppError::Remote(format!("GitLab API error: {}", e)))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AppError::TokenExpired("unauthorized".into()));
        }
        if !resp.status().is_success() {
            return Err(AppError::Remote(format!(
                "GitLab tree listing failed ({})",
                resp.status()
            )));
        }

        // Read X-Next-Page header before consuming the response body
        let next_page: Option<u32> = resp
            .headers()
            .get("x-next-page")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let items: Vec<TreeItem> = resp
            .json()
            .await
            .map_err(|e| AppError::Remote(format!("Failed to parse tree: {}", e)))?;

        all_items.extend(items);

        match next_page {
            Some(np) if np > 0 && page < max_pages as u32 => {
                page = np;
            }
            _ => break,
        }
    }

    let capped = page >= max_pages as u32;
    if capped {
        log::warn!(
            "GitLab source {}: reached {} skill page limit — repository may have more skills",
            repo_url,
            max_pages
        );
    }

    let folders: Vec<&TreeItem> = all_items
        .iter()
        .filter(|item| item.item_type == "tree")
        .collect();

    let mut skills = Vec::new();
    for folder in folders {
        if let Some(content) =
            fetch_skill_md(&client, &host, &project, &branch, &folder.name).await?
        {
            let fm = parse_frontmatter(&content).unwrap_or_default();
            let commit = fetch_last_commit(&client, &host, &project, &branch, &folder.name).await?;
            let (updated_at, updated_by) = match commit {
                Some(c) => (c.authored_date, c.author_name),
                None => (None, None),
            };
            skills.push(RemoteSkill {
                folder_name: folder.name.clone(),
                name: fm.name,
                description: fm.description,
                source_id: String::new(),
                source_name: String::new(),
                updated_at,
                updated_by,
            });
        }
    }

    Ok(skills)
}

pub async fn download_skill(
    repo_url: &str,
    folder_name: &str,
    token: &str,
    dest: &std::path::Path,
) -> Result<(), AppError> {
    let (host, project) = parse_repo_url(repo_url)?;
    let client = build_client(token)?;
    let branch = get_default_branch(&client, &host, &project)
        .await?
        .ok_or_else(|| AppError::Remote("Repository is empty, nothing to download".into()))?;
    let branch_encoded = encode_component(&branch);
    let folder_encoded = encode_component(folder_name);

    let tree_url = format!(
        "https://{}/api/v4/projects/{}/repository/tree?path={}&recursive=true&per_page=100&ref={}",
        host, project, folder_encoded, branch_encoded
    );
    let resp = client
        .get(&tree_url)
        .send()
        .await
        .map_err(|e| AppError::Remote(format!("GitLab API error: {}", e)))?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::TokenExpired("unauthorized".into()));
    }
    if !resp.status().is_success() {
        return Err(AppError::Remote(format!(
            "GitLab tree listing for '{}' failed ({})",
            folder_name,
            resp.status()
        )));
    }

    let items: Vec<TreeItem> = resp
        .json()
        .await
        .map_err(|e| AppError::Remote(format!("Failed to parse tree: {}", e)))?;

    let files: Vec<&TreeItem> = items
        .iter()
        .filter(|item| item.item_type == "blob")
        .collect();

    for file in files {
        let file_path_encoded = encode_repo_path(&file.path);
        let raw_url = format!(
            "https://{}/api/v4/projects/{}/repository/files/{}/raw?ref={}",
            host, project, file_path_encoded, branch_encoded
        );
        let resp =
            client.get(&raw_url).send().await.map_err(|e| {
                AppError::Remote(format!("Failed to download {}: {}", file.path, e))
            })?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AppError::TokenExpired("unauthorized".into()));
        }
        if !resp.status().is_success() {
            return Err(AppError::Remote(format!(
                "Failed to download {} ({})",
                file.path,
                resp.status()
            )));
        }

        let content = resp
            .bytes()
            .await
            .map_err(|e| AppError::Remote(format!("Failed to read {}: {}", file.path, e)))?;

        let dest_file = dest.join(&file.path);
        if let Some(parent) = dest_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest_file, &content)?;
    }

    Ok(())
}

pub async fn upload_skill(
    repo_url: &str,
    folder_name: &str,
    token: &str,
    skill_dir: &std::path::Path,
    force: bool,
) -> Result<(), AppError> {
    let (host, project) = parse_repo_url(repo_url)?;
    let client = build_client(token)?;
    let maybe_branch = get_default_branch(&client, &host, &project).await?;
    let repo_empty = maybe_branch.is_none();
    let branch = maybe_branch.unwrap_or_else(|| "main".to_string());
    let branch_encoded = encode_component(&branch);
    let folder_encoded = encode_component(folder_name);

    // For empty repos, skip tree checks — nothing exists remotely
    let (folder_exists, existing_paths) = if repo_empty {
        (false, std::collections::HashSet::new())
    } else {
        // Check if remote folder already exists
        let check_url = format!(
            "https://{}/api/v4/projects/{}/repository/tree?path={}&per_page=1&ref={}",
            host, project, folder_encoded, branch_encoded
        );
        let resp = client
            .get(&check_url)
            .send()
            .await
            .map_err(|e| AppError::Remote(format!("GitLab API error: {}", e)))?;

        let exists = if resp.status().is_success() {
            let items: Vec<TreeItem> = resp
                .json()
                .await
                .map_err(|e| AppError::Remote(format!("Failed to parse tree: {}", e)))?;
            !items.is_empty()
        } else if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AppError::TokenExpired("unauthorized".into()));
        } else if resp.status().as_u16() == 404 {
            false
        } else {
            return Err(AppError::Remote(format!(
                "GitLab tree check failed ({})",
                resp.status()
            )));
        };

        if exists && !force {
            return Err(AppError::Conflict(format!(
                "Skill folder '{}' already exists on remote",
                folder_name
            )));
        }

        // Build set of existing remote file paths (for create vs update)
        let paths: std::collections::HashSet<String> = if exists {
            let tree_url = format!(
                "https://{}/api/v4/projects/{}/repository/tree?path={}&recursive=true&per_page=100&ref={}",
                host, project, folder_encoded, branch_encoded
            );
            let resp = client
                .get(&tree_url)
                .send()
                .await
                .map_err(|e| AppError::Remote(format!("GitLab API error: {}", e)))?;
            if resp.status().is_success() {
                let items: Vec<TreeItem> = resp
                    .json()
                    .await
                    .map_err(|e| AppError::Remote(format!("Failed to parse tree: {}", e)))?;
                items
                    .into_iter()
                    .filter(|i| i.item_type == "blob")
                    .map(|i| i.path)
                    .collect()
            } else {
                std::collections::HashSet::new()
            }
        } else {
            std::collections::HashSet::new()
        };

        (exists, paths)
    };

    // Read local files recursively
    let local_dir = skill_dir.join(folder_name);
    let mut actions = Vec::new();
    collect_files_recursive(&local_dir, folder_name, &existing_paths, &mut actions)?;

    if actions.is_empty() {
        return Err(AppError::NotFound(format!(
            "No files found in skill folder '{}'",
            folder_name
        )));
    }

    let commit_message = if folder_exists {
        format!("Update skill: {}", folder_name)
    } else {
        format!("Upload skill: {}", folder_name)
    };

    let payload = serde_json::json!({
        "branch": branch,
        "commit_message": commit_message,
        "actions": actions,
    });

    let commit_url = format!(
        "https://{}/api/v4/projects/{}/repository/commits",
        host, project
    );
    let resp = client
        .post(&commit_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::Remote(format!("GitLab commit API error: {}", e)))?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::TokenExpired("unauthorized".into()));
    }
    if !resp.status().is_success() {
        return Err(AppError::Remote(format!(
            "GitLab commit failed ({}): {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        )));
    }

    Ok(())
}

fn is_text_file(path: &std::path::Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "md" | "txt"
                | "yaml"
                | "yml"
                | "json"
                | "toml"
                | "xml"
                | "html"
                | "css"
                | "js"
                | "ts"
                | "jsx"
                | "tsx"
                | "py"
                | "rs"
                | "go"
                | "sh"
                | "bat"
                | "cfg"
                | "ini"
                | "env"
                | "gitignore"
                | "dockerignore"
                | "csv"
        ),
        None => false,
    }
}

fn collect_files_recursive(
    dir: &std::path::Path,
    prefix: &str,
    existing_paths: &std::collections::HashSet<String>,
    actions: &mut Vec<serde_json::Value>,
) -> Result<(), AppError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let file_path = format!("{}/{}", prefix, name);

        if path.is_dir() {
            collect_files_recursive(&path, &file_path, existing_paths, actions)?;
        } else {
            let action = if existing_paths.contains(&file_path) {
                "update"
            } else {
                "create"
            };

            let (content, encoding) = if is_text_file(&path) {
                // Fall back to base64 if the file isn't valid UTF-8
                match std::fs::read_to_string(&path) {
                    Ok(text) => (text, "text".to_string()),
                    Err(_) => {
                        use base64::Engine;
                        let bytes = std::fs::read(&path)?;
                        (
                            base64::engine::general_purpose::STANDARD.encode(&bytes),
                            "base64".to_string(),
                        )
                    }
                }
            } else {
                use base64::Engine;
                let bytes = std::fs::read(&path)?;
                (
                    base64::engine::general_purpose::STANDARD.encode(&bytes),
                    "base64".to_string(),
                )
            };

            actions.push(serde_json::json!({
                "action": action,
                "file_path": file_path,
                "content": content,
                "encoding": encoding,
            }));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo_url_simple() {
        let (host, project) = parse_repo_url("https://gitlab.com/team/skills-repo").unwrap();
        assert_eq!(host, "gitlab.com");
        assert_eq!(project, "team%2Fskills-repo");
    }

    #[test]
    fn test_parse_repo_url_nested() {
        let (host, project) = parse_repo_url("https://gitlab.company.com/org/group/repo/").unwrap();
        assert_eq!(host, "gitlab.company.com");
        assert_eq!(project, "org%2Fgroup%2Frepo");
    }

    #[test]
    fn test_parse_repo_url_invalid() {
        assert!(parse_repo_url("not-a-url").is_err());
    }

    #[test]
    fn test_parse_repo_url_with_special_chars() {
        let (host, project) =
            parse_repo_url("https://gitlab.com/team/data science/技能庫").unwrap();
        assert_eq!(host, "gitlab.com");
        assert_eq!(
            project,
            "team%2Fdata%20science%2F%E6%8A%80%E8%83%BD%E5%BA%AB"
        );
    }
}
