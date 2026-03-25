---
phase: 01-security-and-foundation
plan: "01"
subsystem: mcp-server
tags: [security, cors, shell-injection, audit]
dependency_graph:
  requires: []
  provides: [cors-free-health-endpoint, shell-injection-audit-confirmed]
  affects: [mcp-server, security]
tech_stack:
  added: []
  patterns: [axum-statuscode-only-response]
key_files:
  created: []
  modified:
    - src-tauri/src/mcp/router.rs
decisions:
  - "Health endpoint returns bare StatusCode::OK with no CORS headers (D-06 compliance)"
  - "Shell injection audit: only Command::new in settings.rs; agent_id validated via DB lookup before use as command name"
  - "Rust 1.94.0 confirmed >= 1.77.2 (CVE-2024-24576 mitigated)"
metrics:
  duration_seconds: 145
  completed_date: "2026-03-25T05:41:49Z"
  tasks_completed: 2
  files_modified: 1
requirements:
  - SEC-01
  - SEC-03
---

# Phase 01 Plan 01: Remove CORS Headers and Shell Injection Audit Summary

**One-liner:** Removed CORS wildcard headers from MCP /health endpoint and confirmed no shell injection vulnerabilities in Rust command usage.

## What Was Built

### Task 1: Remove CORS Headers from MCP Health Endpoint

Updated `src-tauri/src/mcp/router.rs` to:
- Remove `ACCESS_CONTROL_ALLOW_ORIGIN` and `ACCESS_CONTROL_ALLOW_METHODS` imports from axum
- Replace the multi-header tuple response in `health()` with `axum::http::StatusCode::OK`
- Rename test from `health_endpoint_exposes_cors_header_for_ui_polling` to `health_endpoint_returns_no_cors_headers`
- Updated test assertions to confirm absence of `access-control-allow-origin` and `access-control-allow-methods` headers
- Removed `Origin` request header from test (no longer relevant)

### Task 2: Shell Injection Audit (SEC-03, read-only)

Searched all `.rs` files under `src-tauri/src/` for `Command::new` and `std::process::Command`.

**Findings:**

| Location | Usage | User-controlled? | Risk |
|----------|-------|-----------------|------|
| `settings.rs:21` (`shell_path`) | `Command::new(&shell)` where `shell = env::var("SHELL")` | No — OS-set env var | None |
| `settings.rs:44-45` (`command_with_path`) | `Command::new(program)` | No — always called with hardcoded literals `"claude"`, `"codex"`, `"gemini"` | None |
| `settings.rs:247` (`check_agent_dir`) | `command_with_path(&agent_id_clone)` | Indirect — `agent_id` from frontend, but validated via `Agent::find_by_id` DB lookup first (returns error on unknown ID) | Effectively none |

**Rust version:** `rustc 1.94.0 (4a4ef493e 2026-03-02)` — well above CVE-2024-24576 threshold of 1.77.2.

**Result:** No shell injection vulnerabilities found. All `Command::new` calls use either OS-set env vars or values validated against a fixed set of known agent IDs.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1 | `9376176` | fix(01-01): remove CORS wildcard headers from MCP health endpoint |
| Task 2 | n/a | Read-only audit — no code changes |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None relevant to this plan's goals.

## Self-Check

- [x] `src-tauri/src/mcp/router.rs` exists and was modified
- [x] Commit `9376176` exists
- [x] `cargo test health_endpoint_returns_no_cors_headers` passed (1 test, 0 failures)
- [x] `grep "ACCESS_CONTROL_ALLOW" src-tauri/src/mcp/router.rs` returns no results
- [x] Rust 1.94.0 >= 1.77.2

## Self-Check: PASSED
