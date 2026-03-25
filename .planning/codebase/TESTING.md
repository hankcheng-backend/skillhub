# Testing Patterns

**Analysis Date:** 2026-03-25

## Test Framework

**Runner:**
- Rust built-in `cargo test` — no separate test runner crate required
- Config: `src-tauri/Cargo.toml`

**Assertion Library:**
- Rust standard `assert_eq!`, `assert!`, `assert!(…is_empty())` macros

**Test-only dependencies (`[dev-dependencies]`):**
- `tower = { version = "0.5", features = ["util"] }` — provides `ServiceExt::oneshot` for axum HTTP tests
- `tempfile = "3"` — listed in `[dependencies]` (used in production code paths too, available in tests)
- `tokio = { version = "1", features = ["full"] }` — async runtime, also in `[dependencies]`

**Run Commands:**
```bash
cd src-tauri && cargo test              # Run all tests
cd src-tauri && cargo test -- --nocapture  # Run all tests with stdout
cd src-tauri && cargo test <test_name>  # Run a single test by name
```

No frontend test runner is configured. There are no `vitest.config.*`, `jest.config.*`, or any `.test.*` / `.spec.*` files in `src/`.

## Test File Organization

**Location:** Co-located with source code — every test module lives inside the same `.rs` file as the production code it tests.

**Naming convention:** Test modules are gated with `#[cfg(test)]`. Module names vary: `tests`, `path_resolution_tests`, `config_tests`.

**Files with test modules:**
- `src-tauri/src/mcp/router.rs` — HTTP integration tests
- `src-tauri/src/remote/gitlab.rs` — URL parsing unit tests
- `src-tauri/src/db/mod.rs` — DB migration / seeding tests
- `src-tauri/src/db/models.rs` — Agent path resolution tests
- `src-tauri/src/scanner/mod.rs` — Filesystem scanner integration tests
- `src-tauri/src/scanner/frontmatter.rs` — Frontmatter parsing unit tests

## Test Structure

**Module declaration pattern:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // imports for test utilities
    #[test]
    fn descriptive_snake_case_name() { … }
}
```

**Async tests** use `#[tokio::test]`:
```rust
#[tokio::test]
async fn health_endpoint_exposes_cors_header_for_ui_polling() { … }
```

**Naming style:** Full sentence-style names describing the expected behavior, e.g.
`scan_removes_deleted_skill_rows`, `resolves_legacy_suffix_skills_path`.

## Mocking

**Framework:** None — the codebase avoids mocking libraries.

**Strategy:** Real implementations are used throughout. External HTTP calls (GitLab API, `reqwest`) are not mocked; tests that exercise GitLab logic only cover pure parsing functions (`parse_repo_url`, `encode_component`), not network-dependent paths.

**What is tested without mocks:**
- In-memory SQLite via `Connection::open_in_memory()` replaces the on-disk database
- Real filesystem operations on temporary directories via `tempfile::TempDir`
- Real axum `Router` invoked without a bound TCP socket via `tower::ServiceExt::oneshot`

**What is intentionally not tested (no mock available):**
- `reqwest` HTTP calls to GitLab endpoints
- `keyring` OS keychain access
- Tauri IPC command handlers

## Fixtures and Factories

**DB setup helpers:**

In `src-tauri/src/db/mod.rs` tests:
```rust
fn test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").unwrap();
    migrate(&conn).unwrap();
    conn
}
```

In `src-tauri/src/scanner/mod.rs` tests — uses a real on-disk DB in a TempDir:
```rust
fn setup_db(temp: &TempDir) -> rusqlite::Connection {
    let db_path = temp.path().join("skillhub-test.db");
    let (conn, _is_fresh) = crate::db::init_db(&db_path).unwrap();
    conn
}
```

**Filesystem helpers:**
```rust
fn make_skill(dir: &std::path::Path) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(
        dir.join("skill.md"),
        "---\nname: test\ndescription: test\n---\nbody",
    ).unwrap();
}
```

**TempDir lifecycle:** `TempDir` is bound to a local variable; the directory is deleted automatically when it drops at end of each test. Tests never share a TempDir across test functions.

## Coverage

**Requirements:** None enforced. No coverage thresholds are configured in `Cargo.toml` or CI.

**View Coverage:**
```bash
cd src-tauri && cargo llvm-cov  # Requires cargo-llvm-cov to be installed
```

## Test Types

**Unit Tests:**
- `src-tauri/src/scanner/frontmatter.rs` — pure parsing logic for YAML frontmatter (`parse_frontmatter`)
- `src-tauri/src/remote/gitlab.rs` — pure URL parsing and encoding (`parse_repo_url`, URL encoding with special characters and nested paths)

**Integration Tests (in-process):**
- `src-tauri/src/db/mod.rs` — database migration seeding and idempotency, using in-memory SQLite
- `src-tauri/src/db/models.rs` — `Agent` path resolution logic with real filesystem layouts in TempDir
- `src-tauri/src/scanner/mod.rs` — full `scan_all` round-trips: discovers skills on disk, upserts to DB, handles symlinks, cleans stale rows; uses real on-disk SQLite in TempDir
- `src-tauri/src/mcp/router.rs` — axum HTTP handler: builds the full `Router` with in-memory DB and fires a real `GET /health` request via `oneshot`, asserting status and CORS headers

**E2E Tests:** Not used.

## Common Patterns

**HTTP testing with oneshot:**
```rust
use tower::util::ServiceExt;

let app = create_router(db);
let response = app
    .oneshot(
        Request::builder()
            .uri("/health")
            .method("GET")
            .header("Origin", "http://localhost:1420")
            .body(Body::empty())
            .expect("build request"),
    )
    .await
    .expect("request health endpoint");

assert!(response.status().is_success());
```

**In-memory SQLite pattern:**
```rust
let db = Arc::new(Mutex::new(
    Connection::open_in_memory().expect("open in-memory DB"),
));
```

**TempDir filesystem pattern:**
```rust
let temp = TempDir::new().unwrap();
let skill_path = temp.path().join(".claude").join("skills").join("alpha");
make_skill(&skill_path);
// … test logic …
// temp dropped here → directory deleted
```

**Error path testing:** Errors are asserted with `assert!(result.is_err())`. There are no `#[should_panic]` tests.

**Cross-platform symlink tests** use conditional compilation:
```rust
#[cfg(unix)]
std::os::unix::fs::symlink(src, dst).unwrap();
#[cfg(windows)]
std::os::windows::fs::symlink_dir(src, dst).unwrap();
```

## Notable Coverage Gaps

**Zero frontend coverage:**
- `src/` contains no test files of any kind
- All React components, hooks, and frontend utilities are completely untested
- Affected files: all of `src/components/`, `src/lib/`, `src/pages/`

**Tauri commands layer untested:**
- All files under `src-tauri/src/commands/` (`skills.rs`, `sources.rs`, `sync_cmd.rs`, `install.rs`, `upload.rs`, `settings.rs`) have no `#[cfg(test)]` modules
- These handlers are the primary surface between the frontend and backend logic
- Risk: regressions in command argument handling or state mutations are not caught automatically

**MCP tools layer untested:**
- `src-tauri/src/mcp/tools.rs` has no tests; all tool dispatch functions (`search_skills`, `install_skill_tool`, `upload_skill_tool`, etc.) are exercised only indirectly through the router

**GitLab network paths untested:**
- `src-tauri/src/remote/gitlab.rs` tests cover only pure URL parsing
- All async functions that call GitLab APIs (`list_skills`, `download_skill`, `upload_skill`, `validate_source_access`) have no tests
- No HTTP mocking library (e.g. `wiremock`) is present

**Scanner helper functions:**
- `detect_origin_agent`, `normalize_path`, `find_skill_md`, `cleanup_stale_rows` are tested only via the higher-level `scan_all` integration tests, not in isolation

---

*Testing analysis: 2026-03-25*
