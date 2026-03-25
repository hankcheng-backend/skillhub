# Technology Stack

> Generated: 2026-03-25
> Focus: Languages, frameworks, runtimes, build tools, and key dependencies

## Languages

**Primary:**
- TypeScript 5.8.3 - Frontend UI (`src/`)
- Rust (edition 2021) - Backend/system layer (`src-tauri/src/`)

**Secondary:**
- HTML/JSX - Component templates via React JSX transform

## Runtime

**Environment:**
- Node.js - Frontend dev server and build toolchain
- Native OS process - Rust backend embedded in the Tauri app bundle

**Package Manager:**
- npm
- Lockfile: `package-lock.json` (present)

## Frameworks

**Core:**
- Tauri 2 - Desktop app shell bridging React frontend and Rust backend. Config: `src-tauri/tauri.conf.json`
- React 19.1.0 - UI component framework (`src/`)
- Axum 0.7 - Embedded HTTP server for the local MCP endpoint (`src-tauri/src/mcp/`)

**Async Runtime:**
- Tokio 1 (full features) - Async executor for Rust backend, used for HTTP calls, filesystem ops, and MCP server

**Build/Dev:**
- Vite 7.0.4 - Frontend bundler and dev server. Config: `vite.config.ts`; fixed port 1420
- `@vitejs/plugin-react` 4.6.0 - React Fast Refresh plugin for Vite
- `tauri-build` 2 - Build script for Tauri integration (`src-tauri/build.rs`)

## TypeScript Configuration

**Compiler target:** ES2020
**Module resolution:** bundler mode
**Strict mode:** enabled (`strict`, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`)
**JSX:** `react-jsx` (no explicit React import needed)
**Config files:** `tsconfig.json`, `tsconfig.node.json`

## Key Rust Dependencies

**Data persistence:**
- `rusqlite` 0.31 (bundled feature) - SQLite via bundled libsqlite3. DB path: `{data_dir}/skillhub/skillhub.db`. Schema in `src-tauri/src/db/mod.rs`

**HTTP client:**
- `reqwest` 0.12 (json feature) - Async HTTP client for GitLab API calls and npm registry queries (`src-tauri/src/remote/gitlab.rs`, `src-tauri/src/commands/settings.rs`)

**Serialization:**
- `serde` 1 (derive) + `serde_json` 1 - All data serialization for Tauri commands and API payloads

**Filesystem watching:**
- `notify` 6 + `notify-debouncer-mini` 0.4 - Watch agent skill directories for live changes (`src-tauri/src/watcher/`)

**Secrets/keychain:**
- `keyring` 3 (apple-native, windows-native, sync-secret-service) - OS keychain storage for API tokens. Used in `src-tauri/src/remote/oauth.rs`

**Frontmatter parsing:**
- `gray_matter` 0.2 - Parse YAML frontmatter from `skill.md` files. Used in `src-tauri/src/scanner/frontmatter.rs`

**Utilities:**
- `uuid` 1 (v4) - Generate source IDs
- `dirs` 5 - Resolve `home_dir()` and `data_dir()` per OS
- `thiserror` 1 - Derive `AppError` enum (`src-tauri/src/error.rs`)
- `base64` 0.22.1 - Encode binary file content when uploading to GitLab
- `percent-encoding` 2.3 - URL-encode GitLab repo paths
- `log` 0.4 + `env_logger` 0.11 - Logging facade and env-driven log levels
- `tempfile` 3 - Temp dirs used in install flows

**Key frontend dependencies:**
- `@tauri-apps/api` 2 - `invoke()` IPC bridge to Rust commands. Wrapped in `src/lib/tauri.ts`
- `@tauri-apps/plugin-opener` 2 - Open URLs in default browser
- `react-markdown` 10.1.0 - Render `skill.md` content in `SkillModal.tsx` / `RemoteSkillModal.tsx`

## Tauri Plugins Used

| Plugin | Purpose |
|--------|---------|
| `tauri-plugin-dialog` 2 | Native file/folder picker dialogs and message dialogs |
| `tauri-plugin-autostart` 2 | macOS LaunchAgent / Windows startup registration |
| `tauri-plugin-updater` 2 | Auto-update check against GitHub Releases |
| `tauri-plugin-opener` 2 | Open external URLs |

## Binary Targets

- **Main app** (`src-tauri/src/main.rs`) - Tauri desktop app with embedded MCP HTTP server
- **`skillhub-mcp-stdio`** (`src-tauri/src/bin/skillhub-mcp-stdio.rs`) - Standalone CLI binary implementing MCP protocol over stdio (JSON-RPC 2.0) for use as an external MCP server by AI agents

## Configuration

**App config stored in SQLite:**
- `mcp_port` key in `app_config` table (default: `9800`)

**Build:**
- `src-tauri/tauri.conf.json` - Main Tauri config (product name, bundle targets, updater endpoints, window size, CSP)
- `src-tauri/capabilities/default.json` - Permission declarations for Tauri plugins

**CSP policy (production):**
- `default-src 'self'`
- `connect-src 'self' http://127.0.0.1:*` (allows frontend to reach local MCP server)

## Platform Requirements

**Development:**
- Node.js + npm (for Vite dev server and Tauri CLI)
- Rust toolchain (stable)
- `tauri-cli` v2 (`npm run tauri`)

**Production:**
- macOS, Windows, Linux (Tauri `targets: "all"`)
- Auto-update artifacts uploaded to GitHub Releases (`src-tauri/tauri.conf.json` updater endpoint)
- macOS autostart via LaunchAgent

---

*Stack analysis: 2026-03-25*
