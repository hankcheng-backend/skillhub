---
phase: 04-sidebar-context-menu
plan: 01
subsystem: ui
tags: [react, typescript, context-menu, sidebar, i18n]

# Dependency graph
requires: []
provides:
  - Right-click context menu on remote source items in Sidebar
  - ConfirmDeleteSourceDialog component for safe source deletion
  - handleDeleteSource flow in App.tsx: removes source, refreshes list, resets to local if deleted
affects: [sidebar, sources, app-state]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Context menu via fixed-positioned div with backdrop div for outside-click dismissal
    - Escape key dismissal via useEffect + document keydown listener
    - Confirmation dialog reusing existing confirm-dialog + dialog-overlay CSS classes
    - onDeleteSource prop pattern: Sidebar owns UI state, App owns data mutation

key-files:
  created:
    - src/components/ConfirmDeleteSourceDialog.tsx
  modified:
    - src/lib/i18n.ts
    - src/styles/globals.css
    - src/components/Sidebar.tsx
    - src/App.tsx

key-decisions:
  - "ConfirmDeleteSourceDialog rendered by Sidebar (not App) because Sidebar owns the source context and context menu state"
  - "Context menu uses fixed positioning with clientX/clientY for accurate cursor-relative placement"
  - "Backdrop div at z-index 299 (below menu at 300) handles outside-click dismissal without blocking page interaction"

patterns-established:
  - "Context menu pattern: backdrop div + fixed menu div, Escape key via useEffect, outside click via backdrop onClick"
  - "Confirmation dialog: reuse .dialog-overlay + .confirm-dialog classes; override confirm-dialog positioning with inline style when inside flexbox overlay"

requirements-completed: [CTX-01, CTX-02, CTX-03, CTX-04, CTX-05]

# Metrics
duration: 2min
completed: 2026-03-30
---

# Phase 4 Plan 01: Sidebar Context Menu Summary

**Right-click context menu on remote source items with ConfirmDeleteSourceDialog, wired to api.removeSource with active-source reset**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-30T10:39:56Z
- **Completed:** 2026-03-30T10:41:46Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Right-clicking any remote source in the Sidebar now shows a positioned context menu with a "Delete Source" option styled in danger color
- ConfirmDeleteSourceDialog component created, reusing existing confirm-dialog and dialog-overlay CSS patterns with correct flexbox positioning override
- handleDeleteSource in App.tsx calls api.removeSource, refreshes the source list, and resets activeSource to "local" (clearing remoteSkills) when the deleted source was active
- Context menu dismisses on outside click (backdrop) or Escape key

## Task Commits

Each task was committed atomically:

1. **Task 1: Add i18n keys, context menu CSS, and ConfirmDeleteSourceDialog** - `f99c390` (feat)
2. **Task 2: Wire context menu into Sidebar and delete handler into App** - `bb62ab9` (feat)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified
- `src/components/ConfirmDeleteSourceDialog.tsx` - New component: confirmation dialog for source deletion with loading state
- `src/lib/i18n.ts` - Added deleteSource, deleteSourceConfirmMsg, deleteSourceConfirmSuffix keys in both zh-TW and en locales
- `src/styles/globals.css` - Added .sidebar-context-menu, .sidebar-context-menu-item, .sidebar-context-menu-backdrop CSS classes
- `src/components/Sidebar.tsx` - Added onDeleteSource prop, contextMenu/confirmingDeleteSource state, onContextMenu handler, context menu JSX, ConfirmDeleteSourceDialog integration
- `src/App.tsx` - Added handleDeleteSource function and onDeleteSource prop to Sidebar JSX

## Decisions Made
- ConfirmDeleteSourceDialog is rendered by Sidebar (not App) because Sidebar already owns the source name context and the context menu state that triggers it. This keeps the delete UI flow self-contained in the sidebar layer.
- Context menu uses `position: fixed` with `clientX`/`clientY` for accurate cursor-relative placement regardless of scroll position.
- The backdrop div at `z-index: 299` (one below the menu at `300`) provides outside-click dismissal without requiring a global event listener.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Right-click delete source flow is fully wired end-to-end
- All strings go through t() with keys in both locales
- TypeScript compiles with zero errors
- Ready for the next plan in this phase

## Self-Check: PASSED

- FOUND: src/components/ConfirmDeleteSourceDialog.tsx
- FOUND: src/components/Sidebar.tsx
- FOUND: src/App.tsx
- FOUND: .planning/phases/04-sidebar-context-menu/04-01-SUMMARY.md
- FOUND commit: f99c390 (Task 1)
- FOUND commit: bb62ab9 (Task 2)

---
*Phase: 04-sidebar-context-menu*
*Completed: 2026-03-30*
