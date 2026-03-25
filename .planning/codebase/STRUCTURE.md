# Codebase Structure

**Analysis Date:** 2026-03-25

## Directory Layout

```
skillhub/                          # Project root
├── index.html                     # Vite HTML entry point
├── package.json                   # Frontend dependencies & npm scripts
├── package-lock.json              # Frontend lockfile
├── tsconfig.json                  # TypeScript config (src)
├── tsconfig.node.json             # TypeScript config (vite.config)
├── vite.config.ts                 # Vite + Tauri plugin config
├── README.md                      # Project overview
├── .gitignore                     # Git ignore rules
│
├── src/                           # Frontend (React + TypeScript)
│   ├── main.tsx                   # React DOM entry point
│   ├── App.tsx                    # Root component, global state & routing
│   ├── types.ts                   # Shared TypeScript interface definitions
│   ├── vite-env.d.ts              # Vite ambient type declarations
│   ├── components/                # React UI components
│   │   ├── Layout.tsx             # App shell (header + sidebar + main area)
│   │   ├── Sidebar.tsx            # Source navigation list
│   │   ├── SkillGrid.tsx          # Main skill display grid
│   │   ├── SkillCard.tsx          # Individual local skill card
│   │   ├── SkillModal.tsx         # Skill detail/edit modal
│   │   ├── RemoteSkillCard.tsx    # Individual remote skill card
│   │   ├── RemoteSkillModal.tsx   # Remote skill detail/install modal
│   │   ├── SettingsGeneral.tsx    # Settings page (agents, sources, config)
│   │   ├── AddSourceDialog.tsx    # Dialog to add a new remote source
│   │   ├── SearchBar.tsx          # Skill search input
│   │   └── AgentIcons.tsx         # SVG icon components per agent
│   ├── lib/                       # Frontend utility modules
│   │   ├── tauri.ts               # Typed `api` object wrapping all invoke() calls
│   │   ├── i18n.ts                # i18n translation strings (zh-TW primary)
│   │   └── error.ts               # Error extraction and user-facing message formatting
│   ├── styles/
│   │   └── globals.css            # Global CSS (Tailwind-style utility classes, custom vars)
│   └── assets/
│       └── react.svg              # Static image assets
│
├── src-tauri/                     # Tauri / Rust backend
│   ├── Cargo.toml                 # Rust crate manifest & dependencies
│   ├── Cargo.lock                 # Rust lockfile
│   ├── tauri.conf.json            # Tauri app config (window, bundle, updater)
│   ├── build.rs                   # Tauri build script
│   ├── capabilities/
│   │   └── default.json           # Tauri capability / permission declarations
│   ├── icons/                     # App icons for bundler (PNG, ICNS, ICO)
│   ├── gen/
│   │   └── schemas/               # Auto-generated JSON schemas (Tauri tooling)
│   └── src/
│       ├── main.rs                # Rust binary entry point (calls lib::run())
│       ├── lib.rs                 # Tauri app bootstrap: DB init, plugins, invoke_handler
│       ├── error.rs               # AppError enum (Db, Io, Frontmatter, NotFound, Conflict, Remote, OAuth)
│       ├── commands/              # #[tauri::command] handlers (IPC surface)
│       │   ├── mod.rs             # Re-exports all command modules
│       │   ├── skills.rs          # list, scan, delete, get content, update meta
│       │   ├── sync_cmd.rs        # sync_skill, unsync_skill (symlink management)
│       │   ├── install.rs         # install_skill (download from remote source)
│       │   ├── upload.rs          # upload_skill (push local skill to remote)
│       │   ├── sources.rs         # list, add, remove, browse sources
│       │   └── settings.rs        # agents, app_config, autostart, agent dir picker
│       ├── db/                    # SQLite persistence layer
│       │   ├── mod.rs             # init_db(), migrate() (schema + seeds), DB tests
│       │   └── models.rs          # ORM structs + CRUD methods: Agent, Skill, SkillSync, Source
│       ├── mcp/                   # MCP HTTP server (axum, port 9800 default)
│       │   ├── mod.rs             # start_server() — binds TCP, serves axum app
│       │   ├── router.rs          # create_router(): /health GET, /mcp POST + tests
│       │   └── tools.rs           # All MCP tool handlers (mirrors commands/ surface)
│       ├── remote/                # Remote source adapters
│       │   ├── mod.rs             # RemoteSkill struct; re-exports adapters
│       │   ├── gitlab.rs          # GitLab API client (list tree, get file, commit info)
│       │   ├── gdrive.rs          # Google Drive stub (not yet implemented)
│       │   └── oauth.rs           # OAuth token helpers (keychain read/write)
│       ├── scanner/               # Filesystem skill discovery
│       │   ├── mod.rs             # scan_all(), scan_agent_dir() — walks agent skill dirs
│       │   └── frontmatter.rs     # YAML frontmatter parser for skill.md files
│       ├── sync/
│       │   └── mod.rs             # create_sync_link(), remove_sync_link() (symlink helpers)
│       ├── watcher/
│       │   └── mod.rs             # start_watching() — fs notify debouncer, emits skills-changed
│       └── bin/
│           └── skillhub-mcp-stdio.rs  # Standalone stdio MCP server binary (alternative transport)
│
├── dist/                          # Vite frontend build output (gitignored)
│   └── assets/                    # Bundled JS/CSS
│
├── doc/                           # Developer documentation (design, handoff, API reference)
│   ├── 2026-03-20-skillhub-design.md
│   ├── 2026-03-23-skillhub.md
│   ├── 2026-03-23-requirement-audit-and-bugfix.md
│   ├── HANDOFF.md
│   ├── mcp-api-reference.md
│   ├── release-guide.md
│   └── stdio-mcp-server.md
│
├── docs/
│   └── superpowers/               # GSD planning specs and plans
│       ├── plans/
│       └── specs/
│
├── .planning/
│   └── codebase/                  # GSD codebase analysis documents (this file)
│
└── .superpowers/                  # GSD session data
```

## Directory Purposes

**`src/`**
- Purpose: All React/TypeScript frontend code compiled by Vite
- Contains: React components, typed API bindings, i18n strings, global CSS
- Key files: `src/main.tsx` (entry), `src/App.tsx` (root), `src/types.ts` (shared types), `src/lib/tauri.ts` (IPC layer)

**`src/components/`**
- Purpose: React UI components, one file per component
- Contains: `.tsx` files only; no tests, no utilities
- Key files: `SettingsGeneral.tsx` (largest, ~650 lines — agents + sources + config), `SkillGrid.tsx` (primary content area), `SkillModal.tsx` (detail view)

**`src/lib/`**
- Purpose: Non-component frontend utilities
- Contains: `tauri.ts` (all `invoke()` calls centralized), `i18n.ts` (translation dictionary), `error.ts` (error message normalizer)
- Rule: All Tauri IPC calls must go through `src/lib/tauri.ts`; components never call `invoke()` directly

**`src-tauri/src/`**
- Purpose: Entire Rust backend for the Tauri application
- Contains: Entry point, all command handlers, DB layer, MCP server, remote adapters, scanner, sync utilities

**`src-tauri/src/commands/`**
- Purpose: All functions exposed to the frontend via `#[tauri::command]` and registered in `lib.rs`'s `invoke_handler![]`
- Contains: One file per domain area; `mod.rs` re-exports all submodules
- Rule: Each new command must be added to both its module file and the `invoke_handler![]` list in `lib.rs`

**`src-tauri/src/db/`**
- Purpose: SQLite persistence — schema migrations and model structs with CRUD
- Contains: `mod.rs` (migration SQL + `init_db()`), `models.rs` (Agent, Skill, SkillSync, Source structs with query methods)
- DB file location at runtime: `~/Library/Application Support/skillhub/skillhub.db` (macOS)

**`src-tauri/src/mcp/`**
- Purpose: HTTP MCP server that exposes skill management to AI agents (Claude, Codex, etc.)
- Contains: `mod.rs` (server startup), `router.rs` (axum routes), `tools.rs` (method handlers)
- Listens at `127.0.0.1:9800` by default (port stored in `app_config` table)

**`src-tauri/src/remote/`**
- Purpose: Adapters for remote skill repositories (GitLab implemented; Google Drive stubbed)
- Contains: `mod.rs` (shared `RemoteSkill` struct), one file per provider, `oauth.rs` for token management

**`src-tauri/src/scanner/`**
- Purpose: Discovers skills by walking agent skill directories on the filesystem
- Contains: `mod.rs` (scan logic), `frontmatter.rs` (parses YAML `---` headers from `skill.md`)

**`src-tauri/src/sync/`**
- Purpose: Symlink creation/removal for syncing a skill to additional agent directories
- Contains: `mod.rs` only — `create_sync_link()` and `remove_sync_link()`

**`src-tauri/src/watcher/`**
- Purpose: Filesystem change detection; re-scans and emits `skills-changed` Tauri event
- Contains: `mod.rs` only — `start_watching()` using `notify-debouncer-mini`

**`src-tauri/src/bin/`**
- Purpose: Additional Rust binaries compiled from the same crate
- Contains: `skillhub-mcp-stdio.rs` — a standalone stdio-transport MCP server for environments that cannot use HTTP

**`src-tauri/capabilities/`**
- Purpose: Tauri v2 permission/capability declarations
- Contains: `default.json` — declares which Tauri APIs the frontend window is allowed to call

**`doc/`**
- Purpose: Human-readable developer documentation: design notes, API reference, release guide
- Not consumed by build tooling; read by developers

**`dist/`**
- Purpose: Vite build output, referenced by `tauri.conf.json` as `frontendDist`
- Generated: Yes — do not commit or edit manually

## Key File Locations

**Entry Points:**
- `index.html`: Browser entry point (Vite uses this as HTML template)
- `src/main.tsx`: React DOM bootstrap (`ReactDOM.createRoot`)
- `src-tauri/src/main.rs`: Rust binary entry, calls `skillhub_lib::run()`
- `src-tauri/src/lib.rs`: Tauri application setup — DB init, plugin registration, full command registry

**Configuration:**
- `vite.config.ts`: Vite configuration with `@vitejs/plugin-react`
- `tsconfig.json`: TypeScript strictness settings for `src/`
- `src-tauri/tauri.conf.json`: Window dimensions, bundle targets, updater endpoint, CSP
- `src-tauri/Cargo.toml`: Rust dependencies
- `src-tauri/capabilities/default.json`: Tauri plugin permissions

**Core Logic — Frontend:**
- `src/lib/tauri.ts`: Single `api` object with typed wrappers for every backend command
- `src/types.ts`: Canonical TypeScript interfaces (`Skill`, `Agent`, `Source`, `RemoteSkill`, `AppError`, etc.)
- `src/App.tsx`: Root-level state management (`skills`, `sources`, `activeSource`, `showSettings`)

**Core Logic — Backend:**
- `src-tauri/src/db/models.rs`: All database model structs and their query methods
- `src-tauri/src/db/mod.rs`: Schema definition and migration (`migrate()`)
- `src-tauri/src/scanner/mod.rs`: `scan_all()` — primary skill discovery function
- `src-tauri/src/error.rs`: `AppError` enum used as the error type throughout the backend

**MCP API Surface:**
- `src-tauri/src/mcp/router.rs`: HTTP routes `/health` and `/mcp`
- `src-tauri/src/mcp/tools.rs`: All MCP method implementations

**Remote Integration:**
- `src-tauri/src/remote/gitlab.rs`: GitLab API client (tree listing, file fetch, commit metadata)
- `src-tauri/src/remote/oauth.rs`: Keychain token storage helpers

**Testing:**
- `src-tauri/src/db/mod.rs` (lines 75–139): SQLite migration tests using in-memory DB
- `src-tauri/src/scanner/frontmatter.rs` (lines 42–67): Frontmatter parse unit tests
- `src-tauri/src/mcp/router.rs` (lines 83–118): Axum router integration test

## Naming Conventions

**Frontend Files:**
- React components: `PascalCase.tsx` (e.g., `SkillModal.tsx`, `AddSourceDialog.tsx`)
- Utility/library modules: `camelCase.ts` (e.g., `tauri.ts`, `i18n.ts`, `error.ts`)
- Style files: `kebab-case.css` (e.g., `globals.css`)
- Type-only file: `types.ts` at the `src/` root

**Frontend Identifiers:**
- Component functions: `PascalCase` (matches filename)
- Hook-like state/effect code: inside component functions, no separate hook files currently
- API wrapper: single `api` object in `src/lib/tauri.ts`; keys are `camelCase` verb phrases (e.g., `listSkills`, `addSource`)
- TypeScript interfaces: `PascalCase` (e.g., `Skill`, `AgentDirStatus`)
- TypeScript type aliases: `PascalCase`

**Backend Files (Rust):**
- Module files: `snake_case` directory + `mod.rs` or `snake_case.rs`
- Command files named after domain: `skills.rs`, `sources.rs`, `settings.rs`, `sync_cmd.rs`
- Struct names: `PascalCase` (e.g., `Agent`, `SkillSync`, `RemoteSkill`)
- Function names: `snake_case` (e.g., `scan_all`, `init_db`, `start_watching`)
- Tauri command functions: `snake_case`; mapped to identical `snake_case` string in `invoke_handler![]`

**Rust Module Visibility:**
- Modules intended for cross-module use: `pub mod` in `lib.rs`
- Internal modules: `mod` (no `pub`) in `lib.rs` (e.g., `mod commands`, `mod remote`)
- `db` and `scanner` are `pub mod` so `mcp/tools.rs` and the stdio binary can access them

## Where to Add New Code

**New React Component:**
- Implementation: `src/components/NewComponent.tsx`
- Import in: whichever parent component renders it (typically `App.tsx` or another component)
- Export: named export (`export function NewComponent`)

**New Tauri Command:**
1. Add function with `#[tauri::command]` to the appropriate file in `src-tauri/src/commands/` (or create a new `new_domain.rs` file and add `pub mod new_domain;` to `src-tauri/src/commands/mod.rs`)
2. Add the command to the `invoke_handler![]` array in `src-tauri/src/lib.rs`
3. Add a typed wrapper to the `api` object in `src/lib/tauri.ts`
4. Add the TypeScript return type to `src/types.ts` if a new data shape is introduced

**New Database Model:**
- Add struct with `#[derive(Debug, Clone, Serialize, Deserialize)]` to `src-tauri/src/db/models.rs`
- Add `CREATE TABLE IF NOT EXISTS` SQL to the `migrate()` function in `src-tauri/src/db/mod.rs`
- Add query methods as `impl YourModel { ... }` in `models.rs`
- Mirror the struct as a TypeScript interface in `src/types.ts`

**New Remote Source Adapter (e.g., GitHub, Google Drive):**
1. Create `src-tauri/src/remote/github.rs` (or the provider name)
2. Add `pub mod github;` to `src-tauri/src/remote/mod.rs`
3. Implement at minimum: `validate_source_access()`, `list_skills()`, `get_file()` matching the pattern in `gitlab.rs`
4. Add the new `source_type` string match arm to `commands/sources.rs` in `add_source`, `browse_source`, `get_remote_skill_content`
5. Add the same arm to `mcp/tools.rs` in the parallel MCP tool functions

**New MCP Tool:**
1. Add a handler function `pub fn my_tool(db: &SharedDb, params: serde_json::Value) -> Result<serde_json::Value, String>` in `src-tauri/src/mcp/tools.rs`
2. Add a match arm `"my_tool" => tools::my_tool(&db, req.params)` in the `handle_mcp` function in `src-tauri/src/mcp/router.rs`
3. Update `doc/mcp-api-reference.md` with the new method signature

**New App Setting (stored in `app_config` table):**
1. Add `INSERT OR IGNORE INTO app_config (key, value) VALUES ('new_key', 'default')` to `migrate()` in `src-tauri/src/db/mod.rs`
2. The existing `get_config` / `set_config` Tauri commands and `api.getConfig` / `api.setConfig` frontend wrappers handle arbitrary keys — no new commands needed for simple string settings
3. Add a UI control to `src/components/SettingsGeneral.tsx`

**New i18n String:**
- Add key-value to the `translations` object in `src/lib/i18n.ts`
- Use via `t("myKey")` imported from `src/lib/i18n`
- Language is currently hardcoded to `zh-TW`

**Utilities:**
- Shared Rust helpers with no external dependencies: add to the most relevant existing module or create a new `src-tauri/src/utils/mod.rs` if a standalone module is warranted
- Shared frontend helpers: add to an existing file in `src/lib/` or create a new `src/lib/myUtil.ts`

## Special Directories

**`dist/`**
- Purpose: Compiled Vite frontend output consumed by Tauri bundler
- Generated: Yes (by `npm run build`)
- Committed: No (in `.gitignore`)

**`src-tauri/target/`**
- Purpose: Rust build artifacts
- Generated: Yes (by `cargo build` / `tauri build`)
- Committed: No (in `.gitignore`)

**`src-tauri/gen/`**
- Purpose: Auto-generated JSON schema files produced by Tauri CLI tooling
- Generated: Yes
- Committed: Yes (small, stable, needed for IDE tooling)

**`src-tauri/icons/`**
- Purpose: Application icon assets in multiple resolutions for all platforms
- Generated: No (manually provided)
- Committed: Yes

**`doc/`**
- Purpose: Human-readable developer notes, design decisions, API reference, release guide
- Generated: No
- Committed: Yes — these are the primary source of context for onboarding

**`.planning/codebase/`**
- Purpose: GSD codebase analysis documents consumed by `/gsd:plan-phase` and `/gsd:execute-phase`
- Generated: By GSD mapping agents
- Committed: Yes

**`node_modules/`**
- Purpose: npm package installations
- Generated: Yes (by `npm install`)
- Committed: No

---

*Structure analysis: 2026-03-25*
