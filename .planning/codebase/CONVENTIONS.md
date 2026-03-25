# Coding Conventions
> Generated: 2026-03-25
> Focus: Naming patterns, code style, TypeScript/React conventions, Rust conventions

## Naming Patterns

**TypeScript/React Files:**
- React component files: PascalCase, `.tsx` extension — e.g., `SkillCard.tsx`, `AddSourceDialog.tsx`, `RemoteSkillModal.tsx`
- Library/utility files: camelCase, `.ts` extension — e.g., `tauri.ts`, `i18n.ts`, `error.ts`
- Type definition files: camelCase, `.ts` extension — e.g., `types.ts`
- Style files: kebab-case, `.css` extension — e.g., `globals.css`

**Rust Files:**
- Module files use snake_case: `mod.rs`, `models.rs`, `frontmatter.rs`, `sync_cmd.rs`
- All Rust file names are lowercase snake_case

**Functions (TypeScript):**
- Event handler functions: `handle` prefix — e.g., `handleSubmit`, `handleDelete`, `handleClose`, `handleInstall`
- Load functions: `load` prefix — e.g., `loadSkills`, `loadSources`, `loadVersions`
- Toggle functions: `toggle` prefix — e.g., `toggleAgent`, `toggleAutostart`
- Boolean state: `is` prefix — e.g., `isRemote`, `isOrigin`, `isSynced`, `isActive`, `isLatest`, `isOutdated`
- Show/hide state: `show` prefix — e.g., `showSettings`, `showAddSource`, `showUploadDialog`

**Functions (Rust):**
- Public functions use snake_case: `scan_all`, `init_db`, `parse_frontmatter`, `resolve_skill_dir`
- Private helpers use snake_case: `scan_agent_dir`, `upsert_local_skill`, `cleanup_stale_rows`
- Tauri command functions match their invoke names exactly: `list_skills`, `delete_skill`, `browse_source`

**Types and Interfaces (TypeScript):**
- Interfaces use PascalCase with `Props` suffix for component props: `SkillCardProps`, `AddSourceDialogProps`, `LayoutProps`
- Standalone interfaces use PascalCase: `MergedSkillEntry`, `VersionCardData`
- Type aliases use PascalCase: `AgentDirStatus`, `DbState`
- Union type members use string literals: `"gitlab" | "gdrive"`, `"general" | "versions" | "mcp"`

**Types (Rust):**
- Structs use PascalCase: `Agent`, `Skill`, `SkillSync`, `Source`, `AppError`, `SkillFrontmatter`
- Enums use PascalCase with variant names in PascalCase: `AppError::Db`, `AppError::NotFound`, `AppError::Conflict`
- Type aliases use PascalCase: `DbState`, `SharedDb`

**CSS Classes:**
- Component-scoped BEM-style kebab-case: `skill-card`, `skill-card-header`, `skill-card-name`, `skill-card-desc`
- Modifier classes with double-dash: `agent-badge--origin`, `agent-badge--inactive`, `mcp-status-dot--running`
- State classes use descriptive names: `active`, `open`, `spin`

**Constants:**
- TypeScript: SCREAMING_SNAKE_CASE for module-level constants — e.g., `SOURCE_AUTO_REFRESH_SECONDS`, `AGENT_PRIORITY`, `AGENTS_ORDER`
- Rust: standard Rust does not use module-level constants in this codebase; config is stored in DB

## Code Style

**TypeScript Formatting:**
- No standalone formatter config found (no `.prettierrc`, `biome.json`, or `eslint.config.*`)
- Indentation: 2 spaces
- String quotes: double quotes in JSX attributes (`className="btn-primary"`), double quotes in TS strings
- Trailing commas: present in multi-line object/array literals
- Semicolons: used throughout

**TypeScript Strictness (enforced via `tsconfig.json`):**
- `strict: true` — all strict checks active
- `noUnusedLocals: true` — unused local variables are errors
- `noUnusedParameters: true` — unused parameters are errors
- `noFallthroughCasesInSwitch: true`
- `isolatedModules: true`
- Target: `ES2020`

**Rust Style:**
- Standard Rust formatting (`rustfmt` conventions, though no explicit config file found)
- Long lines occasionally in SQL strings — otherwise standard line length

## Import Organization

**TypeScript Import Order (observed pattern):**

1. React hooks from `react` — e.g., `import { useEffect, useState } from "react"`
2. Tauri APIs — e.g., `import { listen } from "@tauri-apps/api/event"`
3. Internal lib modules — e.g., `import { api } from "./lib/tauri"`, `import { t } from "../lib/i18n"`
4. Internal type imports (with `import type`) — e.g., `import type { Skill, Source } from "../types"`
5. Component imports — e.g., `import { Layout } from "./components/Layout"`
6. Style imports — e.g., `import "./styles/globals.css"`

**`import type` usage:**
- Used consistently for type-only imports to keep bundles clean:
  ```typescript
  import type { RemoteSkill, Skill, Source } from "./types";
  import type { Agent, AgentDirStatus } from "../types";
  ```

**Rust Import Style:**
- Grouped `use` declarations at top of file
- Imports from `crate::` for internal modules, then external crates
- Re-exports via `pub mod` and `pub use` at module boundaries

## Error Handling

**TypeScript — Frontend:**
- Catch blocks always type `e` as `unknown`, then narrow: `catch (e: unknown)`
- Error extraction through `extractErrorMessage(error: unknown)` in `src/lib/error.ts`
- Domain-specific formatting through `formatAddSourceError(error: unknown)` in `src/lib/error.ts`
- In-component error state: `const [error, setError] = useState<string | null>(null)`
- Toast messages for transient errors: `showToast(msg)` — clears after 3 seconds

**TypeScript — Error shape from Rust:**
- Rust serializes errors as `{ kind: string; message: string }` — matched as `{ kind?: string; message?: string }` in TS
- Kind-based branching: `if (err.kind === "Conflict") { ... }` in `src/components/SkillModal.tsx`
- Defined in `src/types.ts`:
  ```typescript
  export interface AppError {
    kind: string;
    message: string;
  }
  ```

**Rust — Backend:**
- All commands return `Result<T, AppError>` — never panic in command handlers
- `thiserror` crate used for ergonomic error derivation
- `From` implementations auto-convert `rusqlite::Error` and `std::io::Error`
- `AppError` serializes to `{ kind, message }` so frontend can inspect type
- Pattern: `Err(AppError::NotFound(format!("Skill not found: {}", skill_id)))`

## Async Patterns

**TypeScript:**
- `async/await` throughout; no Promise `.then()` chains in component logic (only in fire-and-forget init)
- `void` prefix used for fire-and-forget async in JSX handlers: `onClick={() => { void handleDelete(); }}`
- `void` prefix used for floating async calls: `void refreshRemoteSkills()`, `void loadSkills()`
- Cancellation via `cancelled` flag in `useEffect` cleanup:
  ```typescript
  let cancelled = false;
  // ... in async function: if (cancelled) return;
  return () => { cancelled = true; clearInterval(timer); };
  ```

**Rust:**
- `async fn` used for network operations (HTTP calls via `reqwest`)
- Sync functions for DB operations (SQLite via `rusqlite` with `Mutex<Connection>`)
- `tokio::spawn` via `tauri::async_runtime::spawn` for background tasks

## State Management

**React State:**
- All state is local component state via `useState` — no global state library
- Data loading: `useEffect` with `api.*` calls, results stored in `useState`
- Derived/filtered data: `useMemo` for expensive derivations
  ```typescript
  const selectedEntry = useMemo(() => {
    if (!selectedFolder) return null;
    return mergedLocal.find(entry => entry.skill.folder_name === selectedFolder) ?? null;
  }, [selectedFolder, mergedLocal]);
  ```
- State lifting: `App.tsx` owns top-level `skills`, `sources`, `activeSource`; passes down via props

## Component Design

**Component Export:**
- Named exports only — no default exports except `App` in `src/App.tsx`
  ```typescript
  export function SkillCard({ ... }: SkillCardProps) { ... }
  export function Layout({ ... }: LayoutProps) { ... }
  ```

**Props Interface:**
- Every component defines an explicit `interface XxxProps` directly above the component function
- Optional props typed with `?` suffix: `originAgents?: string[]`, `sources?: Source[]`

**Sub-components:**
- Small helper components defined in the same file as the parent when tightly coupled:
  - `LoadingSkeletons` in `src/components/SkillGrid.tsx`
  - `Toggle`, `VersionCard`, `VersionsTab`, `McpTab` in `src/components/SettingsGeneral.tsx`

**CSS Classes:**
- Styling via CSS class names in `src/styles/globals.css` — no CSS modules, no Tailwind
- Design tokens via CSS custom properties: `var(--accent)`, `var(--bg-card)`, `var(--radius-md)`
- Inline styles used only for dynamic values: `style={{ color: 'var(--text-muted)' }}`, `style={{ background: display.color }}`
- CSS custom properties passed as inline style for component-level theming:
  ```tsx
  style={{ '--badge-color': cfg.color } as React.CSSProperties}
  ```

## i18n Pattern

- All user-facing strings go through `t(key)` from `src/lib/i18n.ts`
- String keys are camelCase identifiers typed as `Key = keyof typeof translations["zh-TW"]`
- Template strings with `{placeholder}` replaced via `.replace("{agent}", agentId)` at call site
- Default locale is `zh-TW`; English (`en`) is alternative
- Language stored in `localStorage.getItem("lang")`

## Rust Module Structure

- Commands expose flat functions decorated with `#[tauri::command]`
- DB access via `State<'_, DbState>` where `DbState = Arc<Mutex<Connection>>`
- Models defined in `src-tauri/src/db/models.rs` with `impl` blocks for DB CRUD
- Each model implements operations as associated functions: `Agent::all()`, `Skill::upsert()`, `Source::find_by_id()`

## Logging

**Rust:**
- `log` crate macros: `log::error!`, `log::info!` — initialized with `env_logger`
- Used in `src-tauri/src/lib.rs` for startup and update events
- Example: `log::error!("MCP server error: {}", e)`

**TypeScript:**
- `console.error` used as a fallback when no `onError` callback is provided:
  ```typescript
  console.error("Sync operation failed:", err);
  ```
- Otherwise errors surface via toast messages to the user

## Comments

**Rust:**
- Doc comments (`///`) used for public functions explaining purpose and backward compatibility
- Inline comments (`//`) for non-obvious logic, DB query intent, and mutex scope explanation
- Example: `// Read port from DB (braces required — MutexGuard is !Send, must drop before tokio::spawn)`

**TypeScript:**
- Minimal comments; code is self-documenting through naming
- Section comments in `src/lib/i18n.ts` to group translation keys: `// Layout`, `// Sidebar`
- No JSDoc blocks observed
