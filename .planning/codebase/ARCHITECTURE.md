# Architecture

> Generated: 2026-03-25
> Focus: System architecture, component relationships, data flow, design patterns

## Pattern Overview

**Overall:** Tauri desktop application using a layered Rust backend with a React frontend, communicating via Tauri's IPC bridge.

**Key Characteristics:**
- Frontend (React/TypeScript) and backend (Rust) are isolated processes; communication happens only through `invoke()` calls and `emit()` events
- The Rust backend exposes a second interface: an HTTP MCP server (port 9800 by default) and an optional standalone stdio binary, both sharing the same `mcp/tools.rs` logic
- All persistent state lives in a single SQLite database at `<data_dir>/skillhub/skillhub.db`; shared across threads via `Arc<Mutex<Connection>>`
- The "skill" concept is a filesystem folder (e.g. `~/.claude/skills/my-skill/`) containing a `skill.md` file with YAML frontmatter

---

## Layers

**Frontend — React UI:**
- Purpose: User interface for browsing, managing, and configuring skills
- Location: `src/`
- Contains: React components, a thin typed API wrapper, i18n, error utilities
- Depends on: Tauri JS API (`@tauri-apps/api`) to call Rust commands
- Used by: End user directly

**IPC Bridge:**
- Purpose: Serialized communication between the frontend and Rust backend
- Mechanism: `invoke()` for commands (request/response), `emit()` for push events (`skills-changed`)
- Location: `src/lib/tauri.ts` (frontend side), `src-tauri/src/lib.rs` `invoke_handler` list (backend side)
- All commands are registered in `src-tauri/src/lib.rs` via `tauri::generate_handler!`

**Tauri Commands Layer:**
- Purpose: Entry points for every UI-initiated action; validates input, acquires the DB mutex, delegates to lower layers
- Location: `src-tauri/src/commands/`
- Contains: `skills.rs`, `sync_cmd.rs`, `sources.rs`, `install.rs`, `upload.rs`, `settings.rs`
- Depends on: `db`, `scanner`, `sync`, `remote` modules
- Used by: IPC bridge from frontend; also re-used by `mcp/tools.rs`

**MCP Tools Layer:**
- Purpose: Business logic implementations shared by both the HTTP MCP server and the stdio MCP binary
- Location: `src-tauri/src/mcp/tools.rs`
- Contains: Functions like `search_skills`, `install_skill_tool`, `upload_skill_tool`, etc.
- Depends on: `db`, `scanner`, `remote`, `sync` modules
- Used by: `mcp/router.rs` (HTTP server) and `src-tauri/src/bin/skillhub-mcp-stdio.rs`

**Domain / Data Layer:**
- Purpose: SQLite models and low-level operations; no business logic
- Location: `src-tauri/src/db/`
- Contains: `mod.rs` (schema migration, DB init), `models.rs` (Agent, Skill, SkillSync, Source with CRUD)
- Depends on: `rusqlite`
- Used by: commands layer, MCP tools layer, scanner, watcher

**Scanner:**
- Purpose: Filesystem scan — discovers skill folders in each enabled agent's `skills/` directory, identifies symlinks as sync relationships, prunes stale DB rows
- Location: `src-tauri/src/scanner/`
- Contains: `mod.rs` (scan logic), `frontmatter.rs` (YAML frontmatter parser)
- Depends on: `db/models`, `error`
- Used by: startup in `lib.rs`, watcher, several commands, MCP tools

**Watcher:**
- Purpose: OS filesystem event watcher (debounced, 500 ms); fires `scan_all` and emits `skills-changed` to the frontend on any change in a monitored skill directory
- Location: `src-tauri/src/watcher/mod.rs`
- Depends on: `notify_debouncer_mini`, `scanner`, Tauri `Emitter`
- Used by: `lib.rs` during app setup

**Sync Module:**
- Purpose: Creates and removes OS symlinks (or Windows junctions) that implement the "sync" relationship between agent skill directories
- Location: `src-tauri/src/sync/mod.rs`
- Depends on: `std::fs`, platform-specific symlink APIs
- Used by: `commands/sync_cmd.rs`, `commands/skills.rs` (delete cleanup)

**Remote Layer:**
- Purpose: Communicates with external skill repositories (currently GitLab only)
- Location: `src-tauri/src/remote/`
- Contains: `gitlab.rs` (list/download/upload/validate), `oauth.rs` (keychain token store/retrieve), `gdrive.rs` (stub, not implemented), `mod.rs` (RemoteSkill struct)
- Depends on: `reqwest`, `keyring`, `percent-encoding`
- Used by: `commands/sources.rs`, `commands/install.rs`, `commands/upload.rs`, MCP tools

**MCP HTTP Server:**
- Purpose: Exposes SkillHub tools as an HTTP JSON-RPC endpoint on `127.0.0.1:9800`; consumed by AI agents running locally
- Location: `src-tauri/src/mcp/` (`mod.rs`, `router.rs`, `tools.rs`)
- Depends on: `axum`, shared DB, `mcp/tools.rs`
- Used by: spawned at startup in `lib.rs` via `tauri::async_runtime::spawn`

**MCP Stdio Binary:**
- Purpose: Standalone MCP server using JSON-RPC 2.0 over stdin/stdout (LSP-style `Content-Length` framing); alternative to the HTTP server for CI/headless environments
- Location: `src-tauri/src/bin/skillhub-mcp-stdio.rs`
- Shares: Same `mcp/tools.rs` functions; opens same SQLite DB path by default
- Depends on: `skillhub_lib` (the main Rust crate exported as `pub mod`)

---

## Data Flow

**User Views Local Skills:**
1. `App.tsx` calls `api.listSkills()` on mount → `invoke("list_skills")`
2. `commands::skills::list_skills` locks DB, calls `Skill::all_with_syncs`
3. Returns serialized `Vec<Skill>` (with `synced_to` populated) to frontend
4. `SkillGrid.tsx` merges skills by `folder_name` using `mergeSkillsByFolder()` and renders cards

**Filesystem Change Detected:**
1. `watcher::start_watching` receives a debounced OS event
2. Calls `scanner::scan_all(&conn)` to reconcile DB with filesystem
3. Calls `app_handle.emit("skills-changed", ())` to push event to frontend
4. `App.tsx` listens for `"skills-changed"` via `listen()` and calls `loadSkills()`

**User Syncs a Skill to Another Agent:**
1. `api.syncSkill(skillId, targetAgent)` → `invoke("sync_skill")`
2. `commands::sync_cmd::sync_skill` resolves paths for origin and target agents
3. Calls `sync::create_sync_link(origin_path, target_path)` to create symlink
4. Inserts `SkillSync` row into DB
5. Next watcher cycle (or manual scan) confirms the symlink and populates `synced_to`

**User Installs a Remote Skill:**
1. `api.installSkill(sourceId, folderName, targetAgent)` → `invoke("install_skill")`
2. `commands::install::install_skill_with_db` looks up source, retrieves token from keychain
3. Calls `remote::gitlab::download_skill()` which hits GitLab API → downloads files to a `tempfile::TempDir`
4. Validates downloaded `skill.md` frontmatter
5. Moves the temp folder into the target agent's `skills/` directory
6. Calls `scanner::scan_all` to register the new skill in the DB

**AI Agent Uses MCP HTTP:**
1. AI agent sends `POST http://127.0.0.1:9800/mcp` with `{"method": "search_skills", "params": {...}}`
2. `mcp::router::handle_mcp` dispatches to `mcp::tools::search_skills`
3. Tools function queries DB and/or calls remote APIs, returns JSON result

**State Management (Frontend):**
- All state lives in `App.tsx` as `useState` hooks: `skills`, `sources`, `activeSource`, `remoteSkills`, `browsing`, modal visibility flags
- No global state library (no Redux, Zustand, etc.)
- Child components receive data via props and call callbacks passed down from `App`
- Remote skill polling: `useEffect` on `activeSource` runs `api.browseSource()` once on selection and then every 15 seconds

---

## Key Abstractions

**Skill ID:**
- Format: `"<agent_id>:<folder_name>"` e.g. `"claude:analyze-api"`
- Used as primary key in `skills` table and for all IPC calls referencing a specific skill
- Examples: `src-tauri/src/commands/skills.rs` line 56–58

**Agent:**
- Represents an AI agent CLI tool (claude, codex, gemini)
- Has a `skill_dir` config path (defaults to `~/.{agent_id}`) with backward-compatible resolution logic in `Agent::resolved_skill_dir()` and `Agent::resolved_config_dir()`
- Examples: `src-tauri/src/db/models.rs`

**SkillSync (cross-agent sharing):**
- A symlink from `~/<target_agent>/skills/<folder>` → `~/<origin_agent>/skills/<folder>`
- Tracked in `skill_syncs` table (`skill_id`, `agent`, `symlink_path`)
- Scanner discovers symlinks and creates/cleans `skill_syncs` rows automatically

**Source:**
- An external repository (currently GitLab) identified by UUID
- Token stored in OS keychain via `keyring`; only the keychain key is persisted in SQLite

**Frontmatter:**
- Every skill must have a `skill.md` (or `SKILL.md`) with YAML frontmatter containing at minimum `name` and `description`
- Parsed by `src-tauri/src/scanner/frontmatter.rs` using the `gray_matter` crate

---

## Entry Points

**Desktop App:**
- Location: `src-tauri/src/main.rs` → `src-tauri/src/lib.rs::run()`
- Triggers: User launches the desktop application
- Responsibilities: Init SQLite DB, run initial scan, start filesystem watcher, start MCP HTTP server, register Tauri commands, check for updates

**Frontend:**
- Location: `src/main.tsx` (Vite entry) → `src/App.tsx`
- Triggers: Tauri webview loads `index.html`
- Responsibilities: Render UI, load initial skills and sources, subscribe to `skills-changed` events

**MCP Stdio Binary:**
- Location: `src-tauri/src/bin/skillhub-mcp-stdio.rs::main()`
- Triggers: Invoked from the command line or by an AI agent's MCP config
- Responsibilities: Read JSON-RPC 2.0 messages from stdin, dispatch to MCP tools, write responses to stdout

---

## Error Handling

**Strategy:** Rust `Result<T, AppError>` propagated to the frontend as a serialized `{ kind: string, message: string }` object

**Patterns:**
- `AppError` enum (`src-tauri/src/error.rs`) covers: `Db`, `Io`, `Frontmatter`, `NotFound`, `Conflict`, `Remote`, `OAuth`
- `AppError` implements `Serialize` directly for IPC transmission; Tauri maps `Err` variants to rejected promises on the frontend
- Frontend `extractErrorMessage()` (`src/lib/error.ts`) normalizes error shapes from multiple possible sources
- Domain-specific formatters like `formatAddSourceError()` translate backend error messages to i18n keys

---

## Cross-Cutting Concerns

**Logging:** `log` crate macros (`log::info!`, `log::error!`) in Rust; no structured logging framework

**i18n:** Flat key-value map in `src/lib/i18n.ts`; only Traditional Chinese (`zh-TW`) is implemented; `t(key)` function used across components

**Authentication:** Tokens stored in OS keychain via `keyring` crate; keychain key stored in `sources.keychain_key` DB column; retrieved at runtime via `remote::oauth::get_token()`

**Auto-update:** `tauri-plugin-updater` polling GitHub Releases on startup; prompts user before downloading

**Auto-start:** `tauri-plugin-autostart` using macOS LaunchAgent

---

*Architecture analysis: 2026-03-25*
