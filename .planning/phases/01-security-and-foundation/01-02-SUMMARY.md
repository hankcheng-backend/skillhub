---
phase: 01-security-and-foundation
plan: 02
subsystem: database
tags: [rusqlite_migration, sqlite, migration, backup, restore, error-dialog]

# Dependency graph
requires:
  - phase: 01-01
    provides: AppError enum and error handling foundation
provides:
  - versioned SQLite migration system via rusqlite_migration
  - DB backup before migration (skillhub.db.bak)
  - restore-on-failure with error dialog to user
  - AppError::Migration variant for migration errors
affects: [01-03, 01-04, db-schema-changes]

# Tech tracking
tech-stack:
  added: [rusqlite_migration 1.2.0]
  patterns:
    - MIGRATIONS constant holds all schema DDL as versioned M::up entries
    - Backup DB before opening connection to avoid WAL sidecar corruption
    - init_db returns (conn, is_fresh) plus migration_error propagated to setup dialog

key-files:
  created: []
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/error.rs
    - src-tauri/src/db/mod.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "Used rusqlite_migration 1.2.0 (not 2.5 as planned) because v2.5 requires rusqlite ^0.39 while project uses 0.31"
  - "M::up DDL uses CREATE TABLE IF NOT EXISTS for safety on existing databases"
  - "Backup created before Connection::open to avoid WAL sidecar file corruption"
  - "Migration failure restores backup, opens fallback connection, shows blocking dialog in setup()"

patterns-established:
  - "Future schema changes: add new M::up entry to MIGRATIONS constant"
  - "user_version PRAGMA managed automatically by rusqlite_migration"

requirements-completed: [DB-01, DB-02]

# Metrics
duration: 25min
completed: 2026-03-25
---

# Phase 01 Plan 02: Database Migration System Summary

**Versioned SQLite migration system with rusqlite_migration 1.2, DB backup before migration, restore-on-failure, and blocking error dialog**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-03-25T06:00:00Z
- **Completed:** 2026-03-25T06:25:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added rusqlite_migration 1.2.0 as a dependency (compatible with existing rusqlite 0.31)
- Added AppError::Migration(String) variant for migration-specific errors
- Replaced manual migrate() function with MIGRATIONS constant and Migrations::new().to_latest()
- Existing DDL preserved verbatim as migration 0 (backward-compatible schema)
- DB backup created to skillhub.db.bak before opening connection (D-10)
- Migration failure triggers backup restore and shows blocking error dialog in Tauri setup() (D-11)
- All 19 existing tests pass; 4 new config_tests + migrations_validate test added

## Task Commits

Each task was committed atomically:

1. **Task 1: Add rusqlite_migration dependency and Migration error variant** - `8ca474a` (feat)
2. **Task 2: Replace migrate() with rusqlite_migration system and backup/restore guard** - `ab0b754` (feat)

## Files Created/Modified
- `src-tauri/Cargo.toml` - Added rusqlite_migration 1.2 dependency
- `src-tauri/src/error.rs` - Added AppError::Migration(String) variant and Serialize arm
- `src-tauri/src/db/mod.rs` - Full migration system: MIGRATIONS const, init_db with backup/restore, updated tests
- `src-tauri/src/lib.rs` - Handle AppError::Migration at startup, show error dialog in setup()

## Decisions Made
- Used rusqlite_migration 1.2.0 instead of 2.5 (plan specified 2.5, which requires rusqlite ^0.39; project uses 0.31)
- DDL in migration 0 uses CREATE TABLE IF NOT EXISTS for safe re-runs on existing schemas
- Backup happens before Connection::open to avoid WAL sidecar file issues (research finding D-10)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Used rusqlite_migration 1.2 instead of 2.5**
- **Found during:** Task 1 (cargo check)
- **Issue:** Plan specified `rusqlite_migration = "2.5"` but v2.5 requires `rusqlite ^0.39.0`. The project uses `rusqlite 0.31`, causing a libsqlite3-sys link conflict.
- **Fix:** Used `rusqlite_migration = "1.2"` which specifies `rusqlite ^0.31.0` — compatible with the existing dependency. API is identical (`Migrations::new`, `M::up`, `to_latest`, `validate`). Note: 1.2 uses `Migrations::new(vec)` not `Migrations::from_slice` which does not exist in this version.
- **Files modified:** src-tauri/Cargo.toml, src-tauri/src/db/mod.rs
- **Verification:** cargo check exits 0; all 19 tests pass
- **Committed in:** 8ca474a, ab0b754 (part of task commits)

---

**Total deviations:** 1 auto-fixed (1 blocking dependency conflict)
**Impact on plan:** Required version downgrade to maintain compatibility. All objectives met. No scope creep.

## Issues Encountered
- rusqlite_migration 2.5 requires rusqlite 0.39 (libsqlite3-sys link conflict). Resolved by using 1.2.0.
- rusqlite_migration 1.2 uses `Migrations::new(Vec)` not `Migrations::from_slice` (which was added in a later version). Updated all call sites accordingly.

## Known Stubs
None — all migration functionality is fully wired.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Migration system operational; future schema additions go in MIGRATIONS as new M::up entries
- user_version PRAGMA managed automatically, no manual tracking needed
- Ready for Plan 03 (security audit / other foundation work)

---
*Phase: 01-security-and-foundation*
*Completed: 2026-03-25*
