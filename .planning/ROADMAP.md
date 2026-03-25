# Roadmap: SkillHub

## Overview

SkillHub has a working v0.1.0 codebase but is not ready for public distribution. This roadmap hardens the app in three phases: first securing the architecture and establishing a versioned database foundation; then refactoring the codebase for structural correctness and adding test coverage; then polishing the user-facing experience for launch. Each phase delivers a verifiable capability and the ordering is driven by hard dependencies — tests cannot be trusted before the services refactor, and frontend cleanup cannot ship safely without the DB migration system.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Security and Foundation** - Authenticated MCP server, restricted CORS, GitLab 401 handling, and versioned DB schema
- [ ] **Phase 2: Structural Correctness and Testing** - Services layer refactor, watcher lifecycle fix, unwrap elimination, and test suites covering critical paths
- [ ] **Phase 3: Polish and Cleanup** - Meaningful errors, empty states, i18n completeness, Google Drive "Coming Soon" label, and GitLab pagination

## Phase Details

### Phase 1: Security and Foundation
**Goal**: Users can trust the app with their credentials and data — MCP server is authenticated, CORS is locked down, GitLab PAT errors surface actionably, and the database schema is versioned for safe future upgrades
**Depends on**: Nothing (first phase)
**Requirements**: SEC-01, SEC-02, SEC-03, DB-01, DB-02
**Success Criteria** (what must be TRUE):
  1. A browser page on localhost cannot fingerprint or call the MCP `/health` endpoint due to CORS wildcard
  2. When a GitLab PAT expires, the app shows an in-place "token expired — update it" prompt without requiring source deletion and re-addition
  3. No user-controlled string is passed as a shell argument anywhere in the codebase; Rust version is >= 1.77.2
  4. Existing user databases (SQLite) survive app upgrade — schema migrations run without data loss
  5. The DB migration system tracks `user_version` and `rusqlite_migration` manages all future schema changes
**Plans:** 4 plans

Plans:
- [x] 01-01-PLAN.md — CORS removal and shell injection audit (SEC-01, SEC-03)
- [ ] 01-02-PLAN.md — Database migration system with rusqlite_migration (DB-01, DB-02)
- [ ] 01-03-PLAN.md — Backend PAT 401 detection and update_source_token command (SEC-02 backend)
- [ ] 01-04-PLAN.md — Frontend PAT expiry UX: badge, modal, auto-retry, i18n (SEC-02 frontend)

### Phase 2: Structural Correctness and Testing
**Goal**: The codebase is non-panicking, structurally sound, and covered by tests — `add_source` logic is deduplicated, the file watcher is properly lifecycle-managed, all `.unwrap()` calls in production paths are replaced, and CI runs test suites on every push
**Depends on**: Phase 1
**Requirements**: BE-01, BE-02, BE-03, TEST-01, TEST-02, TEST-03
**Success Criteria** (what must be TRUE):
  1. A bug fix to `add_source` only needs to be made in one place; commands and MCP tools call the same service function
  2. The app does not panic under any user-reachable code path — all `.unwrap()` in non-test Rust code is replaced with proper error propagation
  3. The file watcher can be registered with new paths dynamically without restarting the app
  4. Backend integration tests cover `add_source`, `install_skill`, and `sync_skill` using in-memory SQLite
  5. Frontend critical flows (source add, skill install, sync) are covered by Vitest + React Testing Library tests
  6. CI runs both frontend and backend test suites and passes on every push
**Plans**: TBD

### Phase 3: Polish and Cleanup
**Goal**: The app is ready for public release — all failure paths produce human-readable messages, first-launch experience has guidance, no hardcoded strings remain in the UI, the Google Drive stub is clearly marked as unavailable, and GitLab repositories with 20+ skills return complete results
**Depends on**: Phase 2
**Requirements**: UX-01, UX-02, UX-03, UX-04, UX-05
**Success Criteria** (what must be TRUE):
  1. Every user-visible error message is human-readable — no raw Rust error strings appear in the UI or in MCP tool responses
  2. On first launch with no skills or sources, the app shows guidance text instead of blank lists
  3. All user-visible strings in `.tsx` files pass through the i18n system — no hardcoded English strings remain
  4. The Google Drive option in AddSourceDialog is visible with a "Coming Soon" label and is disabled — it does not imply the feature is available
  5. A GitLab source with more than 20 skills returns all skills, not just the first page
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Security and Foundation | 0/4 | Planning complete | - |
| 2. Structural Correctness and Testing | 0/TBD | Not started | - |
| 3. Polish and Cleanup | 0/TBD | Not started | - |
