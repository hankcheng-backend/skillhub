# SkillHub

## What This Is

SkillHub is a Tauri desktop app for managing AI agent skills across multiple agents (Claude, Cursor, Windsurf, etc.). It lets users browse, sync, install, and upload skills via a local GUI and an MCP server that AI tools can call directly. Skills are filesystem folders containing a `skill.md` file — SkillHub tracks them in SQLite and syncs them as symlinks between agents.

## Core Value

A developer's skills are reliably synced across all their agents and accessible via MCP — no crashes, no data loss, no silent failures.

## Requirements

### Validated

- ✓ SQLite-backed skill database with schema migration — existing
- ✓ Filesystem scanner discovers skills from agent directories — existing
- ✓ File watcher auto-updates skills on filesystem changes — existing
- ✓ GitLab remote source: list, install, upload, validate skills — existing
- ✓ PAT-based authentication stored in OS keychain — existing
- ✓ Symlink-based sync between agent skill directories — existing
- ✓ MCP HTTP server on port 9800 (search, install, upload, list) — existing
- ✓ Standalone stdio MCP binary (`skillhub-mcp-stdio`) — existing
- ✓ Tauri desktop UI: source management, skill browser, settings — existing
- ✓ i18n infrastructure (English strings in `src/lib/i18n.ts`) — existing
- ✓ Multiple agent support (Claude, Cursor, Windsurf, Zed, Copilot, etc.) — existing

### Active

- [ ] Frontend test coverage — zero tests exist; pre-launch requires confidence in UI flows
- [ ] Backend Rust test coverage — commands layer and MCP tools untested
- [ ] Security: MCP server authentication (shared secret or origin validation)
- [ ] Security: CORS wildcard removed from `/health` endpoint
- [ ] Security: GitLab PAT 401 detection and re-entry prompt
- [ ] Security: Full security audit (systematic pass beyond known issues)
- [ ] Bug: Duplicate `add_source` logic unified into shared function
- [ ] Bug: Shell injection risks identified and patched
- [ ] Polish: Error messages meaningful across all failure paths
- [ ] Polish: Empty states for skill lists, sources, sync status
- [ ] Polish: i18n completeness audit — no hardcoded strings in UI
- [ ] Polish: Edge cases in scanner, watcher, and sync flows
- [ ] Cleanup: Google Drive type removed from frontend type system
- [ ] Cleanup: Legacy `skill_dir` path normalization consolidated to one place
- [ ] DB: Migration versioning system (schema_version table)

### Out of Scope

- Google Drive integration — stub only; will not be implemented in this milestone (ship without it, remove the UI-implied availability)
- OAuth browser flow — `start_oauth_flow` is a stub; non-PAT auth deferred to future milestone
- Performance optimizations (scan caching, concurrent remote fetches) — not blocking launch, can be post-launch

## Context

SkillHub has a working v1 codebase with one previous release commit. The core architecture is solid (Tauri + Rust backend + React frontend + SQLite). The gap before public launch is quality and safety:

- **Zero frontend test coverage** — all UI logic is untested
- **Commands layer untested** — Rust Tauri commands have no direct tests
- **Known security issues** — MCP server has no auth, CORS is too permissive, PAT error handling is incomplete
- **Tech debt** — duplicate business logic, inline SQL, `Box::leak` for file watcher, pervasive `.unwrap()` in non-test Rust code
- **Stub features in type system** — `gdrive` source type exists in `src/types.ts` but backend rejects it; misleads users

Distribution target: public release (GitHub releases / similar).

## Constraints

- **Tech stack**: Tauri 2 + Rust + React/TypeScript + SQLite — no stack changes
- **No breaking changes**: Existing user data (SQLite DB, keychained PATs) must survive any schema or code changes
- **Compatibility**: App must work on macOS (primary), Windows, Linux

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Ship without Google Drive | Stub would confuse users; full implementation is out of scope for launch | — Pending |
| Remove gdrive from type system | Prevents UI from implying a non-functional feature | — Pending |
| Full security audit (not just known fixes) | Public release raises the bar; systematic review prevents surprises | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-25 after initialization*
