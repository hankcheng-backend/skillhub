# Requirements: SkillHub

**Defined:** 2026-03-25
**Core Value:** A developer's skills are reliably synced across all their agents and accessible via MCP — no crashes, no data loss, no silent failures.

## v1 Requirements

Requirements for public launch. Each maps to roadmap phases.

### Security

- [x] **SEC-01**: CORS on `/health` restricted to `tauri://localhost` and `http://localhost:1420` — no wildcard
- [ ] **SEC-02**: GitLab PAT 401 response detected and surfaced as "token expired — update it" with in-place re-entry (no source delete/re-add)
- [x] **SEC-03**: Shell injection audit complete — no user-controlled string passed as shell argument; Rust version >= 1.77.2 for CVE-2024-24576

### Database

- [ ] **DB-01**: Schema migrations managed by `rusqlite_migration` with existing DDL as migration 0; `user_version` PRAGMA validated
- [ ] **DB-02**: Migration tested with real v1 DB snapshot — existing user data survives upgrade

### Backend Structure

- [ ] **BE-01**: `add_source` business logic extracted to shared service function called by both commands and MCP tools — no duplication
- [ ] **BE-02**: All `.unwrap()` in non-test Rust code replaced with proper error propagation; only startup `expect()` allowed
- [ ] **BE-03**: File watcher moved from `Box::leak` to Tauri managed state; watcher handle accessible for dynamic path registration

### Testing

- [ ] **TEST-01**: Backend integration tests cover `add_source`, `install_skill`, `sync_skill` paths using in-memory SQLite
- [ ] **TEST-02**: Frontend test setup with Vitest + React Testing Library + mockIPC; critical UI flows tested
- [ ] **TEST-03**: CI runs both frontend and backend test suites on every push

### Polish

- [ ] **UX-01**: All `AppError` variants produce human-readable messages; MCP tools never leak raw Rust error strings
- [ ] **UX-02**: Empty states shown for skill list, sources list, and sync status on first launch with guidance text
- [ ] **UX-03**: i18n audit complete — no hardcoded user-visible strings in `.tsx` files; Rust update dialog strings use English fallback
- [ ] **UX-04**: Google Drive option in AddSourceDialog shows "Coming Soon" label and remains disabled; no backend changes
- [ ] **UX-05**: GitLab `list_skills` paginates through all pages — repositories with 20+ skills return complete results

## v2 Requirements

### Distribution

- **DIST-01**: macOS code signing and notarization via Apple Developer ID
- **DIST-02**: Windows code signing via OV/EV certificate

### Extensibility

- **EXT-01**: Data-driven agent seeding from config file (no code change to add agents)
- **EXT-02**: Skill install conflict resolution with diff/backup before overwrite

### Performance

- **PERF-01**: Concurrent remote fetches for multiple GitLab sources
- **PERF-02**: Scan cache / dirty flag to skip redundant filesystem walks on MCP calls

### Security

- **SEC-V2-01**: MCP HTTP server shared-secret token authentication (defense-in-depth against DNS rebinding)

### UX Enhancements

- **UXE-01**: "Check for Updates" button in Settings
- **UXE-02**: Watcher re-registration when new agents enabled (no restart required)

## Out of Scope

| Feature | Reason |
|---------|--------|
| OAuth browser flow | PKCE implementation is significant scope; PATs cover all current use cases |
| Google Drive integration | Requires OAuth; explicitly deferred per PROJECT.md |
| SQLite connection pool / WAL optimization | Not observable at current skill repository sizes |
| In-app skill editor | Developers use their own editor; file watcher handles live updates |
| Custom agent definitions | Requires DB migration system + significant design; post-v1 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SEC-01 | Phase 1 | Complete |
| SEC-02 | Phase 1 | Pending |
| SEC-03 | Phase 1 | Complete |
| DB-01 | Phase 1 | Pending |
| DB-02 | Phase 1 | Pending |
| BE-01 | Phase 2 | Pending |
| BE-02 | Phase 2 | Pending |
| BE-03 | Phase 2 | Pending |
| TEST-01 | Phase 2 | Pending |
| TEST-02 | Phase 2 | Pending |
| TEST-03 | Phase 2 | Pending |
| UX-01 | Phase 3 | Pending |
| UX-02 | Phase 3 | Pending |
| UX-03 | Phase 3 | Pending |
| UX-04 | Phase 3 | Pending |
| UX-05 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 16 total
- Mapped to phases: 16
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-25*
*Last updated: 2026-03-25 after roadmap creation*
