use serde::Deserialize;
use serde_json::{json, Value};
use skillhub_lib::db;
use skillhub_lib::db::models::Agent;
use skillhub_lib::mcp::tools;
use skillhub_lib::scanner;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

type SharedDb = Arc<Mutex<rusqlite::Connection>>;

#[derive(Debug)]
struct CliOptions {
    db_path: PathBuf,
    no_initial_scan: bool,
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

enum HandleAction {
    Respond(Value),
    Ignore,
    Exit,
}

fn print_help() {
    println!(
        "\
skillhub-mcp-stdio

A standalone MCP server over stdio for SkillHub tools.

Usage:
  skillhub-mcp-stdio [--db-path <path>] [--no-initial-scan]

Options:
  --db-path <path>      Override SQLite DB path (default: <data_dir>/skillhub/skillhub.db)
  --no-initial-scan     Skip scanner::scan_all on startup
  -h, --help            Show this help message
"
    );
}

fn parse_args() -> Result<Option<CliOptions>, String> {
    let mut db_path: Option<PathBuf> = None;
    let mut no_initial_scan = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--db-path" => {
                let path = args
                    .next()
                    .ok_or_else(|| "--db-path requires a value".to_string())?;
                db_path = Some(PathBuf::from(path));
            }
            "--no-initial-scan" => {
                no_initial_scan = true;
            }
            "-h" | "--help" => {
                print_help();
                return Ok(None);
            }
            other => {
                return Err(format!("Unknown argument: {}", other));
            }
        }
    }

    let resolved_db_path = match db_path {
        Some(v) => v,
        None => {
            let base =
                dirs::data_dir().ok_or_else(|| "Cannot resolve OS data directory".to_string())?;
            base.join("skillhub").join("skillhub.db")
        }
    };

    Ok(Some(CliOptions {
        db_path: resolved_db_path,
        no_initial_scan,
    }))
}

fn init_shared_db(db_path: &PathBuf, no_initial_scan: bool) -> Result<SharedDb, String> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let (conn, is_fresh) = db::init_db(db_path).map_err(|e| e.to_string())?;

    if is_fresh {
        let home = dirs::home_dir().ok_or_else(|| "cannot find home dir".to_string())?;
        for agent_id in &["claude", "codex", "gemini"] {
            let _ = Agent::update(&conn, agent_id, false, None);
            let config_dir = home.join(format!(".{}", agent_id));
            if config_dir.is_dir() {
                let _ = Agent::update(&conn, agent_id, true, None);
            }
        }
    }

    if !no_initial_scan {
        let _ = scanner::scan_all(&conn).map_err(|e| e.to_string())?;
    }

    Ok(Arc::new(Mutex::new(conn)))
}

fn tools_catalog() -> Vec<Value> {
    vec![
        json!({
            "name": "search_skills",
            "description": "Search local and remote skills.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "scope": { "type": "string", "enum": ["local", "remote", "all"] },
                    "source_id": { "type": "string" }
                }
            }
        }),
        json!({
            "name": "list_local_skills",
            "description": "List all local skills. Optional filter by agent.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "agent": { "type": "string", "enum": ["claude", "codex", "gemini"] }
                }
            }
        }),
        json!({
            "name": "get_skill_content",
            "description": "Read local skill.md by skill_id, or remote skill.md by source_id + folder_name.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string" },
                    "source_id": { "type": "string" },
                    "folder_name": { "type": "string" }
                }
            }
        }),
        json!({
            "name": "sync_skill",
            "description": "Sync one local skill to another agent.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string" },
                    "target_agent": { "type": "string", "enum": ["claude", "codex", "gemini"] }
                },
                "required": ["skill_id", "target_agent"]
            }
        }),
        json!({
            "name": "unsync_skill",
            "description": "Remove one skill sync link from an agent.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string" },
                    "agent": { "type": "string", "enum": ["claude", "codex", "gemini"] }
                },
                "required": ["skill_id", "agent"]
            }
        }),
        json!({
            "name": "install_skill",
            "description": "Install remote skill into target agent.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source_id": { "type": "string" },
                    "folder_name": { "type": "string" },
                    "target_agent": { "type": "string", "enum": ["claude", "codex", "gemini"] },
                    "force": { "type": "boolean" }
                },
                "required": ["source_id", "folder_name", "target_agent"]
            }
        }),
        json!({
            "name": "uninstall_skill",
            "description": "Delete local skill (requires confirm: true) or remove one sync by agent.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string" },
                    "agent": { "type": "string", "enum": ["claude", "codex", "gemini"] },
                    "confirm": { "type": "boolean" }
                },
                "required": ["skill_id"]
            }
        }),
        json!({
            "name": "upload_skill",
            "description": "Upload local skill to remote source.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source_id": { "type": "string" },
                    "skill_id": { "type": "string" },
                    "force": { "type": "boolean" }
                },
                "required": ["source_id", "skill_id"]
            }
        }),
        json!({
            "name": "list_sources",
            "description": "List all configured remote sources.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "add_source",
            "description": "Add remote source. GitLab token is validated immediately.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "source_type": { "type": "string", "enum": ["gitlab"] },
                    "url": { "type": "string" },
                    "token": { "type": "string" }
                },
                "required": ["name", "source_type", "url", "token"]
            }
        }),
        json!({
            "name": "remove_source",
            "description": "Remove one remote source and keychain token.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source_id": { "type": "string" }
                },
                "required": ["source_id"]
            }
        }),
        json!({
            "name": "browse_source",
            "description": "List all skills from one source.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source_id": { "type": "string" }
                },
                "required": ["source_id"]
            }
        }),
        json!({
            "name": "get_agents",
            "description": "List all agents and their status.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
    ]
}

async fn dispatch_tool(db: &SharedDb, name: &str, args: Value) -> Result<Value, String> {
    match name {
        "search_skills" => tools::search_skills(db, args).await,
        "list_local_skills" => tools::list_local_skills(db, args),
        "get_skill_content" => tools::get_skill_content(db, args).await,
        "sync_skill" => tools::sync_skill_tool(db, args),
        "unsync_skill" => tools::unsync_skill_tool(db, args),
        "install_skill" => tools::install_skill_tool(db, args).await,
        "uninstall_skill" => tools::uninstall_skill_tool(db, args),
        "upload_skill" => tools::upload_skill_tool(db, args).await,
        "list_sources" => tools::list_sources_tool(db),
        "add_source" => tools::add_source_tool(db, args).await,
        "remove_source" => tools::remove_source_tool(db, args),
        "browse_source" => tools::browse_source_tool(db, args).await,
        "get_agents" => tools::get_agents_tool(db),
        other => Err(format!("Unknown tool: {}", other)),
    }
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn tool_ok(payload: Value) -> Value {
    let text = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ],
        "structuredContent": payload
    })
}

fn tool_err(message: &str) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": message
            }
        ],
        "isError": true
    })
}

async fn handle_request(db: &SharedDb, req: JsonRpcRequest) -> HandleAction {
    if req.method == "notifications/initialized" {
        return HandleAction::Ignore;
    }

    if req.method == "exit" {
        return HandleAction::Exit;
    }

    let id = match req.id {
        Some(v) => v,
        None => return HandleAction::Ignore,
    };

    match req.method.as_str() {
        "initialize" => {
            let requested_protocol = req
                .params
                .get("protocolVersion")
                .and_then(|v| v.as_str())
                .unwrap_or("2024-11-05");
            HandleAction::Respond(success_response(
                id,
                json!({
                    "protocolVersion": requested_protocol,
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "skillhub-mcp-stdio",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            ))
        }
        "ping" => HandleAction::Respond(success_response(id, json!({}))),
        "tools/list" => HandleAction::Respond(success_response(
            id,
            json!({
                "tools": tools_catalog()
            }),
        )),
        "tools/call" => {
            let name = match req.params.get("name").and_then(|v| v.as_str()) {
                Some(v) if !v.trim().is_empty() => v,
                _ => {
                    return HandleAction::Respond(error_response(
                        id,
                        -32602,
                        "tools/call missing name",
                    ));
                }
            };
            let args = req
                .params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            if !args.is_object() {
                return HandleAction::Respond(error_response(
                    id,
                    -32602,
                    "tools/call arguments must be an object",
                ));
            }

            let result = match dispatch_tool(db, name, args).await {
                Ok(v) => tool_ok(v),
                Err(e) => tool_err(&e),
            };
            HandleAction::Respond(success_response(id, result))
        }
        "shutdown" => HandleAction::Respond(success_response(id, json!({}))),
        _ => HandleAction::Respond(error_response(
            id,
            -32601,
            &format!("Method not found: {}", req.method),
        )),
    }
}

fn read_stdio_message<R: BufRead>(reader: &mut R) -> io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;

    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Ok(None);
        }

        if line == "\r\n" || line == "\n" {
            break;
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                let parsed = value.trim().parse::<usize>().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid Content-Length: {}", e),
                    )
                })?;
                content_length = Some(parsed);
            }
        }
    }

    let len = content_length.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Missing Content-Length in stdio message",
        )
    })?;

    let mut buf = vec![0_u8; len];
    reader.read_exact(&mut buf)?;

    let msg = serde_json::from_slice::<Value>(&buf).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid JSON payload: {}", e),
        )
    })?;

    Ok(Some(msg))
}

fn write_stdio_message<W: Write>(writer: &mut W, value: &Value) -> io::Result<()> {
    let payload = serde_json::to_vec(value)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    write!(writer, "Content-Length: {}\r\n\r\n", payload.len())?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(())
}

fn main() {
    let options = match parse_args() {
        Ok(Some(v)) => v,
        Ok(None) => return,
        Err(e) => {
            eprintln!("Argument error: {}", e);
            std::process::exit(2);
        }
    };

    let db = match init_shared_db(&options.db_path, options.no_initial_scan) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to initialize DB: {}", e);
            std::process::exit(1);
        }
    };

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to start Tokio runtime: {}", e);
            std::process::exit(1);
        }
    };

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());

    loop {
        let raw = match read_stdio_message(&mut reader) {
            Ok(Some(v)) => v,
            Ok(None) => break,
            Err(e) => {
                eprintln!("Failed to read stdio message: {}", e);
                break;
            }
        };

        let req = match serde_json::from_value::<JsonRpcRequest>(raw) {
            Ok(v) => v,
            Err(e) => {
                let resp = error_response(Value::Null, -32700, &format!("Parse error: {}", e));
                let _ = write_stdio_message(&mut writer, &resp);
                continue;
            }
        };

        match runtime.block_on(handle_request(&db, req)) {
            HandleAction::Respond(resp) => {
                if write_stdio_message(&mut writer, &resp).is_err() {
                    break;
                }
            }
            HandleAction::Ignore => {}
            HandleAction::Exit => break,
        }
    }
}
