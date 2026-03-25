---
phase: 01-security-and-foundation
plan: 03
subsystem: backend-security
tags: [security, error-handling, gitlab, pat, token-expiry]
dependency_graph:
  requires: []
  provides: [AppError::TokenExpired, update_source_token command, TokenExpired re-wrapping in browse_source]
  affects: [src-tauri/src/error.rs, src-tauri/src/remote/gitlab.rs, src-tauri/src/commands/sources.rs, src-tauri/src/lib.rs, src/lib/tauri.ts]
tech_stack:
  added: []
  patterns: [TokenExpired sentinel pattern (gitlab.rs stateless), source_id re-wrapping in command layer]
key_files:
  created: []
  modified:
    - src-tauri/src/error.rs
    - src-tauri/src/remote/gitlab.rs
    - src-tauri/src/commands/sources.rs
    - src-tauri/src/lib.rs
    - src/lib/tauri.ts
decisions:
  - gitlab.rs returns TokenExpired("unauthorized") as sentinel; sources.rs re-wraps with actual source_id, keeping gitlab.rs stateless
  - update_source_token updates PAT in-place via existing keychain entry — no source UUID churn
metrics:
  duration: ~6 minutes
  completed: 2026-03-25
  tasks_completed: 2
  files_modified: 5
---

# Phase 01 Plan 03: GitLab PAT 401 Detection and Token Update Summary

**One-liner:** AppError::TokenExpired propagates GitLab 401s with source_id context, and update_source_token enables in-place PAT re-entry without deleting the source.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add AppError::TokenExpired variant and 401 detection in gitlab.rs | 6fbb132 | src-tauri/src/error.rs, src-tauri/src/remote/gitlab.rs |
| 2 | Add update_source_token command and re-wrap TokenExpired in browse_source | 594b8d0 | src-tauri/src/commands/sources.rs, src-tauri/src/lib.rs, src/lib/tauri.ts |

## What Was Built

### AppError::TokenExpired variant (error.rs)
- New `TokenExpired(String)` variant added after `OAuth`
- Serializes as `{ kind: "TokenExpired", message: "Token expired for source: {id}" }`
- Frontend (Plan 04) can match on `err.kind === "TokenExpired"` to trigger re-entry prompt

### 401 detection in gitlab.rs
All GitLab API response handlers now check for HTTP 401 UNAUTHORIZED before the generic `!is_success()` check:
- `get_default_branch` — covers `list_skills`, `validate_source_access`, `download_skill`, `upload_skill` (all call this first)
- `list_skills` tree listing response
- `fetch_skill_md` — covers `get_skill_content` and per-folder skill.md fetch in `list_skills`
- `download_skill` tree listing and per-file download responses
- `upload_skill` tree check response (within `if/else if/else` pattern) and commit response
- All return `AppError::TokenExpired("unauthorized")` as sentinel — stateless, no source_id needed here

### TokenExpired re-wrapping in sources.rs
- `browse_source`: wraps `gitlab::list_skills` result — `TokenExpired(_) => TokenExpired(source_id.clone())`
- `get_remote_skill_content`: wraps `gitlab::get_skill_content` result — same pattern
- The command layer injects the actual `source_id` so the frontend knows which source to prompt for re-entry

### update_source_token command
- Validates new token is non-empty
- Looks up `keychain_key` from DB for the source; creates one (`skillhub-{source_id}`) if missing
- Calls `oauth::store_token("skillhub", &keychain_key, trimmed)` to update in-place
- No source deletion, no UUID change — existing source data and sync relationships preserved
- Registered in `lib.rs` `generate_handler![]` after `browse_source`

### Frontend API wrapper (tauri.ts)
- `updateSourceToken(sourceId, newToken)` calls `invoke<void>("update_source_token", { sourceId, newToken })`
- Available for Plan 04 to wire into the token re-entry UI flow

## Deviations from Plan

None - plan executed exactly as written.

## Verification Results

- `cargo check`: PASS (4 unrelated dead code warnings, no errors)
- `cargo test`: PASS (18 tests, 0 failures)
- `npx tsc --noEmit`: PASS (no output, no errors)
- `grep "TokenExpired" error.rs`: shows variant and serialize arm
- `grep -c "UNAUTHORIZED" gitlab.rs`: 7 occurrences across all API response handlers
- `grep "update_source_token" sources.rs`: shows command function
- `grep "updateSourceToken" tauri.ts`: shows API wrapper

## Known Stubs

None. All implementations are complete and wired end-to-end from GitLab response to frontend API wrapper.

## Self-Check: PASSED

Files created/modified:
- FOUND: src-tauri/src/error.rs (TokenExpired variant)
- FOUND: src-tauri/src/remote/gitlab.rs (UNAUTHORIZED checks)
- FOUND: src-tauri/src/commands/sources.rs (update_source_token, re-wrapping)
- FOUND: src-tauri/src/lib.rs (update_source_token registered)
- FOUND: src/lib/tauri.ts (updateSourceToken wrapper)

Commits:
- FOUND: 6fbb132
- FOUND: 594b8d0
