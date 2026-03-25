# External Integrations

> Generated: 2026-03-25
> Focus: External services, APIs, remote sources, and OS integrations

## APIs & External Services

**GitLab (active):**
- Used for: browsing, installing, and uploading skills from/to remote repositories
- Implementation: `src-tauri/src/remote/gitlab.rs`
- API version: GitLab REST API v4 (`/api/v4/`)
- Auth: Personal Access Token sent as `PRIVATE-TOKEN` header
- Token storage: OS keychain via `keyring` crate; key format `skillhub-{source_id}`
- Supports: self-hosted GitLab instances (host parsed from repo URL)
- Operations: list tree, fetch raw file, fetch commits, create/update file via Commits API

**Google Drive (stub - not implemented):**
- Declared in: `src-tauri/src/remote/gdrive.rs`, `src-tauri/src/remote/oauth.rs`
- Status: All methods return `AppError::Remote("not yet implemented")`
- OAuth flow stub exists in `src-tauri/src/remote/oauth.rs` but returns unimplemented error
- `keyring` crate wired for token storage when implemented

**npm Registry (read-only):**
- Used for: fetching latest published versions of Claude Code, Codex CLI, and Gemini CLI
- Endpoint: `https://registry.npmjs.org/{package}/latest`
- Packages queried: `@anthropic-ai/claude-code`, `@openai/codex`, `@google/gemini-cli`
- Implementation: `src-tauri/src/commands/settings.rs` (`get_latest_versions` command)
- No auth; public registry read

## Data Storage

**Database:**
- Type: SQLite (embedded, bundled libsqlite3 via `rusqlite` bundled feature)
- Location: `{OS data dir}/skillhub/skillhub.db` (e.g., `~/Library/Application Support/skillhub/skillhub.db` on macOS)
- WAL journal mode + foreign keys enabled
- Schema defined in: `src-tauri/src/db/mod.rs`
- Tables: `agents`, `skills`, `skill_syncs`, `sources`, `app_config`

**File Storage:**
- Local filesystem only — skill folders are directories on disk within each agent's configured skills directory
- No cloud file storage (Google Drive planned but not implemented)
- Symlinks created for cross-agent sync (`src-tauri/src/sync/`)

**Caching:**
- None — all data read from SQLite or filesystem on demand

## Authentication & Identity

**Token storage:**
- All API tokens stored in OS native keychain via `keyring` crate
- macOS: Keychain (apple-native feature)
- Windows: Windows Credential Store (windows-native feature)
- Linux: Secret Service (sync-secret-service feature)
- Keychain service name: `"skillhub"`, key: `"skillhub-{source_uuid}"`
- Implementation: `src-tauri/src/remote/oauth.rs` (`store_token`, `get_token`)
- `keychain_key` column in `sources` table maps source records to keychain entries

**No user authentication:**
- SkillHub is a local desktop app with no user login or cloud account

## AI Agent Integrations

SkillHub manages skills for three AI coding agents detected and tracked by their config directory presence:

| Agent | CLI package | Default config dir | Skills subdir |
|-------|-------------|-------------------|---------------|
| Claude Code | `@anthropic-ai/claude-code` | `~/.claude` | `~/.claude/skills` |
| OpenAI Codex | `@openai/codex` | `~/.codex` | `~/.codex/skills` |
| Google Gemini CLI | `@google/gemini-cli` | `~/.gemini` | `~/.gemini/skills` |

- Version detection: spawns `claude --version`, `codex --version`, `gemini --version` via login shell
- PATH resolution: `src-tauri/src/commands/settings.rs` `shell_path()` function (spawns login shell to capture full PATH for macOS GUI apps)

## MCP (Model Context Protocol) Server

SkillHub exposes its features as two MCP server implementations:

**HTTP MCP server (embedded):**
- Starts automatically with the Tauri app
- Listens on `127.0.0.1:{mcp_port}` (default port 9800, configurable via `app_config` table)
- Endpoints: `GET /health`, `POST /mcp`
- Implementation: `src-tauri/src/mcp/router.rs`, `src-tauri/src/mcp/tools.rs`
- CORS: `Access-Control-Allow-Origin: *` on health endpoint

**stdio MCP server (standalone binary):**
- Binary: `skillhub-mcp-stdio` (`src-tauri/src/bin/skillhub-mcp-stdio.rs`)
- Protocol: JSON-RPC 2.0 over stdio with `Content-Length` framing (LSP-style)
- Can be configured as an external MCP server in Claude Code / Cursor / other MCP clients
- Shares same DB and tool implementations as the HTTP server

**MCP tools exposed:**
`search_skills`, `list_local_skills`, `get_skill_content`, `sync_skill`, `unsync_skill`, `install_skill`, `uninstall_skill`, `upload_skill`, `list_sources`, `add_source`, `remove_source`, `browse_source`, `get_agents`

## Auto-Update

**Provider:** GitHub Releases
- Endpoint: `https://github.com/hankcheng-backend/skillhub/releases/latest/download/latest.json`
- Config: `src-tauri/tauri.conf.json` under `plugins.updater`
- Signing: minisign public key embedded in `tauri.conf.json`
- Implementation: `src-tauri/src/lib.rs` — checks on app startup, prompts user with native dialog before installing
- Plugin: `tauri-plugin-updater` 2

## Filesystem Watching

- Uses `notify` 6 + `notify-debouncer-mini` 0.4 to watch agent skill directories
- Triggers re-scan when files change in monitored directories
- Implementation: `src-tauri/src/watcher/`
- Events emitted to frontend via Tauri's `app.emit("skills-changed", ())` on changes

## OS Integrations

**Autostart:**
- macOS: LaunchAgent via `tauri-plugin-autostart` (`MacosLauncher::LaunchAgent`)
- Windows: startup registry entry
- Commands: `get_autostart`, `set_autostart` in `src-tauri/src/commands/settings.rs`

**Native dialogs:**
- File/folder picker: `tauri-plugin-dialog` (`pick_agent_dir` command)
- Message dialogs: used for update confirmation prompts

**Open external links:**
- `tauri-plugin-opener` for opening URLs in the system browser

## Environment Configuration

**No `.env` files used.** All configuration is stored in:
- SQLite `app_config` table (runtime config like `mcp_port`)
- OS keychain (API tokens)
- Agent records in SQLite `agents` table (enable/disable, custom skill dir)

**No required environment variables** for normal operation. `TAURI_DEV_HOST` is optionally read by `vite.config.ts` for remote dev scenarios.

## Webhooks & Callbacks

**Incoming:** None

**Outgoing:** None — all communication is pull-based (polling GitLab API, npm registry, GitHub Releases on demand)

---

*Integration audit: 2026-03-25*
