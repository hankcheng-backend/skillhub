# Codebase Concerns

**Analysis Date:** 2026-03-25

## Tech Debt

**Google Drive Integration is a Stub:**
- Issue: All GDrive functions return hardcoded `AppError::Remote("not yet implemented")` errors. The `gdrive` source type is present in the frontend type definition (`src/types.ts` line 23) but the backend rejects it immediately.
- Files: `src-tauri/src/remote/gdrive.rs`, `src-tauri/src/commands/sources.rs` (line 48–51)
- Impact: Any user or MCP tool attempt to add or use a `gdrive` source receives a visible error. The UI type system implies the feature is available.
- Fix approach: Either implement GDrive integration or remove `"gdrive"` from the `Source` type union in `src/types.ts` and gate it behind a feature flag.

**OAuth Flow is a Stub:**
- Issue: `start_oauth_flow` in `src-tauri/src/remote/oauth.rs` returns `AppError::OAuth("OAuth not yet implemented")`. Token storage via `keyring` works, but the browser-based OAuth handshake is missing.
- Files: `src-tauri/src/remote/oauth.rs` (lines 3–10)
- Impact: Any future non-PAT authentication (e.g., GDrive, GitHub OAuth) cannot be implemented until this is built.
- Fix approach: Implement PKCE OAuth flow using `tauri-plugin-shell` to open browser and a local loopback listener to capture the callback.

**Duplicate add_source Logic Between Tauri Command and MCP Tool:**
- Issue: The full source-creation logic (validation, UUID generation, DB insert, keychain store, rollback on keychain failure) is duplicated in `src-tauri/src/commands/sources.rs` (lines 14–105) and `src-tauri/src/mcp/tools.rs` (lines 494–562). Both paths must be kept in sync for correctness.
- Files: `src-tauri/src/commands/sources.rs`, `src-tauri/src/mcp/tools.rs`
- Impact: A bug fix in one code path will not automatically apply to the other, leading to divergent behaviour between the GUI and MCP consumers.
- Fix approach: Extract the business logic into a shared `add_source_internal(db, ...)` function in `commands/sources.rs` or a new `src-tauri/src/services/sources.rs`, and call it from both sites.

**Inline SQL in Command Handlers:**
- Issue: Raw SQL strings appear directly in command handlers rather than being encapsulated in model methods. Examples: `get_config` and `set_config` in `src-tauri/src/commands/settings.rs` (lines 181–197) execute SQL inline with no abstraction.
- Files: `src-tauri/src/commands/settings.rs`
- Impact: SQL logic is scattered; changes to the schema require hunting across multiple files.
- Fix approach: Add `AppConfig::get(conn, key)` and `AppConfig::set(conn, key, value)` methods to `src-tauri/src/db/models.rs`.

**No Database Migration System:**
- Issue: `src-tauri/src/db/mod.rs` uses a single `migrate()` function with all `CREATE TABLE IF NOT EXISTS` DDL in one `execute_batch`. There is no version tracking, so incremental migrations (adding columns, new tables) cannot be applied safely to existing installs.
- Files: `src-tauri/src/db/mod.rs`
- Impact: Adding a column to an existing table requires a workaround (e.g., `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`), which is not supported by all SQLite versions. Failed schema changes will silently succeed or cause runtime errors on user machines.
- Fix approach: Introduce a `schema_version` table and apply numbered migrations in sequence (e.g., using `rusqlite_migration` crate).

**Frontend Backward-Compatibility Shim for Legacy skill_dir Paths:**
- Issue: `SettingsGeneral.tsx` (line 63) applies a `.replace(/[\\/]+skills$/i, "")` regex to strip a legacy `…/skills` suffix from stored `skill_dir` values. The same normalization also exists in Rust (`db/models.rs` via `is_legacy_skills_path`).
- Files: `src/components/SettingsGeneral.tsx`, `src-tauri/src/db/models.rs`
- Impact: Two different normalisation points for the same concern. If a user edits the path in settings, only one branch may run.
- Fix approach: Remove the frontend shim once a one-time DB migration normalises all stored paths.

## Security Considerations

**MCP HTTP Server Has No Authentication:**
- Risk: The local MCP server (`127.0.0.1:<port>`) accepts any POST to `/mcp` with no token, session, or origin check. Any local process can read, install, delete, or upload skills.
- Files: `src-tauri/src/mcp/router.rs`, `src-tauri/src/mcp/mod.rs`
- Current mitigation: Bound to loopback only (`127.0.0.1`), so remote network access is blocked.
- Recommendations: Add a shared secret (generated at first launch, stored in DB) that callers must pass as a header or query param. Alternatively, validate that the `Origin` or `Referer` header matches the expected Tauri webview origin.

**MCP Health Endpoint Returns Wildcard CORS:**
- Risk: The `/health` endpoint responds with `Access-Control-Allow-Origin: *`. While only `GET` is allowed, this means any website opened in the user's browser can probe whether SkillHub is running and on which port.
- Files: `src-tauri/src/mcp/router.rs` (lines 39–47)
- Current mitigation: None.
- Recommendations: Restrict the CORS origin to `tauri://localhost` or `http://localhost:1420`.

**GitLab PAT Stored in System Keychain Without Rotation Support:**
- Risk: Personal Access Tokens are stored via `keyring` and never rotated. There is no `refresh_token_key` usage implemented despite the field existing in the `Source` model.
- Files: `src-tauri/src/remote/oauth.rs`, `src-tauri/src/db/models.rs` (line 41), `src-tauri/src/commands/sources.rs`
- Current mitigation: Tokens are stored in the OS keychain (not plain text).
- Recommendations: Implement token expiry detection and prompt the user to re-enter a PAT when a 401 is received from GitLab.

## Performance Bottlenecks

**`scan_all` is Called on Every MCP Tool Invocation:**
- Problem: `auto_scan(db)` is called at the start of `search_skills` and `list_local_skills` in `src-tauri/src/mcp/tools.rs` (lines 13–18, 159, 196). Each call performs a full filesystem walk of all enabled agent skill directories and DB reconciliation.
- Files: `src-tauri/src/mcp/tools.rs`, `src-tauri/src/scanner/mod.rs`
- Cause: There is no in-memory cache or dirty flag; the scanner always re-reads the filesystem.
- Improvement path: Add a timestamp-based scan cache (e.g., skip re-scan if the last scan was within 2 seconds), or rely solely on the file-system watcher events already delivered via `src-tauri/src/watcher/mod.rs`.

**Remote Skill Fetches Are Sequential Across Sources:**
- Problem: In `fetch_remote_skills` (`src-tauri/src/mcp/tools.rs` lines 56–143), GitLab sources are fetched sequentially in a `for source in &sources` loop. Each GitLab API call blocks the others.
- Files: `src-tauri/src/mcp/tools.rs`
- Cause: No concurrent fetching using `tokio::join!` or `futures::future::join_all`.
- Improvement path: Collect futures for each source and resolve them concurrently with `futures::future::join_all`.

## Fragile Areas

**File Watcher Lifetime is Managed via `Box::leak`:**
- Files: `src-tauri/src/lib.rs` (line 63)
- Why fragile: The `notify_debouncer_mini::Debouncer` is intentionally leaked (`Box::leak(Box::new(watcher))`) to keep it alive for the process lifetime. This prevents clean shutdown and makes the watcher impossible to restart without an application restart (e.g., when new agents are enabled in settings).
- Safe modification: Pass the watcher handle into managed Tauri state instead of leaking it, allowing it to be dropped and recreated when agent configuration changes.
- Test coverage: No tests cover watcher lifecycle.

**Watcher Does Not Re-Register New Agent Directories:**
- Files: `src-tauri/src/watcher/mod.rs`, `src-tauri/src/lib.rs`
- Why fragile: The watcher registers directories only at startup from `Agent::enabled`. If a user enables a new agent in Settings, the watcher never starts watching that agent's skill directory — file changes are missed until the app restarts.
- Safe modification: Emit a Tauri event when agents are updated and subscribe to it in the watcher to add new watch paths dynamically.
- Test coverage: None.

**Symlink Removal Silently Ignored During Bulk Uninstall:**
- Files: `src-tauri/src/mcp/tools.rs` (lines 419–423)
- Why fragile: `let _ = crate::sync::remove_sync_link(&link)` discards all errors when cleaning up sync symlinks during `uninstall_skill`. A broken or missing symlink is silently skipped, potentially leaving stale DB records or orphaned filesystem entries.
- Safe modification: Log errors and collect partial failures, reporting them in the response payload rather than silently discarding.
- Test coverage: Partial — `scan_all` tests cover detection of removed symlinks but not the uninstall path.

**`skill_md.unwrap()` After `is_none()` Check in Scanner:**
- Files: `src-tauri/src/scanner/mod.rs` (lines 85–88)
- Why fragile: The pattern `if skill_md.is_none() { continue; }` followed immediately by `skill_md.unwrap()` is logically correct but relies on no intervening mutation. If this code is ever refactored, the `unwrap` becomes a latent panic site.
- Safe modification: Replace with `let Some(skill_md) = skill_md else { continue; };`.
- Test coverage: Covered by scanner tests.

## Scaling Limits

**Single SQLite Connection Behind a `Mutex`:**
- Current capacity: All Tauri commands and the MCP server share one `Arc<Mutex<Connection>>`. Under normal desktop use this is fine.
- Limit: Any long-running DB operation (e.g., a large `scan_all` during an active MCP search request) will block all other DB access, causing latency spikes or timeouts observable as the UI freezing briefly.
- Scaling path: Use a connection pool (e.g., `r2d2-sqlite`) or separate read/write connections using SQLite WAL mode (already enabled).

**Agent Set is Hardcoded to Three Built-in Agents:**
- Current capacity: Supports `claude`, `codex`, `gemini` only. The agent rows are seeded in the DB migration (`src-tauri/src/db/mod.rs` lines 59–62) and the first-launch detection loop is hardcoded (`src-tauri/src/lib.rs` lines 31–38).
- Limit: Adding a fourth agent requires a DB migration, a new code path in `lib.rs`, and UI updates.
- Scaling path: Make agent seeding data-driven (read from a config file or allow user-defined agents with a custom `skill_dir`).

## Dependencies at Risk

**`notify_debouncer_mini` — Minimal Maintenance Activity:**
- Risk: This crate is a thin wrapper around `notify`. It provides less configurability than `notify` directly and its maintenance cadence is slower.
- Impact: Watcher behaviour on edge cases (rapid renames, cross-device moves) may not be fixed promptly.
- Migration plan: Migrate to `notify` directly with a custom debounce implementation, or switch to `notify-debouncer-full` which is more actively maintained.

## Missing Critical Features

**No Conflict Resolution for `install_skill` Over Existing Files:**
- Problem: When `force: false` (the default), installing a skill that already exists returns `AppError::Conflict`. When `force: true`, the existing directory is deleted and replaced with no backup. There is no merge or diff capability.
- Blocks: Safe upgrades of installed skills from a remote source.

**Watcher Does Not Watch Newly Enabled Agents:**
- Problem: Described above under Fragile Areas. After enabling a new agent in Settings, the user must restart the application for live updates to work.
- Blocks: Reliable real-time skill discovery after reconfiguration.

**No Pagination for Remote Skill Listing:**
- Problem: `gitlab::list_skills` fetches all tree items from the GitLab repository tree API without pagination. GitLab's default page size is 20 items; for repositories with more than 20 skill folders, only the first page is returned.
- Files: `src-tauri/src/remote/gitlab.rs`
- Blocks: Reliable use with large skill repositories.

## Test Coverage Gaps

**No Frontend Tests:**
- What's not tested: All React components (`src/components/`), the `api` wrapper (`src/lib/tauri.ts`), the `extractErrorMessage` / `formatAddSourceError` utilities (`src/lib/error.ts`), and i18n string coverage (`src/lib/i18n.ts`).
- Files: Entire `src/` directory.
- Risk: UI regressions (broken error messages, incorrect state transitions, missing i18n keys) are only caught manually.
- Priority: Medium

**No Integration Tests for Tauri Commands:**
- What's not tested: The Tauri command layer (`src-tauri/src/commands/`) is not covered by any integration test that exercises the full invoke path with a real (even in-memory) DB. Unit tests exist only for the scanner, DB models, and the MCP router health endpoint.
- Files: `src-tauri/src/commands/sources.rs`, `src-tauri/src/commands/install.rs`, `src-tauri/src/commands/upload.rs`, `src-tauri/src/commands/sync_cmd.rs`
- Risk: Regressions in command argument handling, permission checks, or keychain interactions go undetected.
- Priority: High

**MCP Tools Layer Has No Tests:**
- What's not tested: None of the functions in `src-tauri/src/mcp/tools.rs` have unit or integration tests. Only the router's `/health` endpoint has a test.
- Files: `src-tauri/src/mcp/tools.rs`
- Risk: MCP-path bugs (e.g., silent errors in `fetch_remote_skills`, wrong JSON keys in responses) can break agent integrations without any local signal.
- Priority: High

**GitLab Remote Module Has Minimal Tests:**
- What's not tested: `list_skills`, `download_skill`, `upload_skill`, `validate_source_access`, `get_skill_content`. Only `parse_repo_url` is tested.
- Files: `src-tauri/src/remote/gitlab.rs`
- Risk: API contract changes or pagination issues are not caught before release.
- Priority: Medium

---

*Concerns audit: 2026-03-25*
