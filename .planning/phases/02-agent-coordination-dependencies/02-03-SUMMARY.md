---
phase: 02-agent-coordination-dependencies
plan: 03
subsystem: database
tags: [rust, rusqlite, sqlite, dependency-graph, dfs, cycle-detection]

# Dependency graph
requires:
  - phase: 02-agent-coordination-dependencies
    provides: ticket_deps table with ON DELETE CASCADE, WAL mode, migration framework
  - phase: 01-foundation-core-operations
    provides: tickets table, AppError, ticket CRUD functions
provides:
  - add_dep with self-dep guard and DFS cycle detection reporting path
  - remove_dep with DepNotFound error
  - list_deps returning forward and reverse dependency sets
  - would_create_cycle function (in-process DFS, no new crates)
  - DepInfo struct with forward/reverse Vec<i64>
  - Dep CLI subcommand (add/remove)
  - Deps top-level CLI command (tree view)
  - Get output augmented with Depends on / Required by
  - List output augmented with [N deps] suffix
affects: [03-export-reporting]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - In-process DFS for cycle detection by loading full adjacency list from DB
    - CyclicDependency error carries formatted path string for user-facing messages

key-files:
  created: []
  modified:
    - src/ticket.rs
    - src/cli.rs
    - src/lib.rs

key-decisions:
  - "Self-dependency pre-checked in add_dep rather than relying on DB CHECK constraint to give consistent CyclicDependency error type"
  - "would_create_cycle loads full ticket_deps adjacency list into memory for DFS — correct for small workloads, no new crate needed"
  - "Deps (plural) is top-level read command; Dep (singular) is mutation subcommand — consistent with CLI conventions"
  - "format_name_with_deps truncates name to 35 chars when dep suffix present to keep list rows tidy"

patterns-established:
  - "Cycle detection: load adjacency list from DB, add hypothetical edge, DFS from target to source"
  - "DepInfo struct: forward = what this ticket depends on, reverse = what depends on this ticket"

requirements-completed: [COORD-04, COORD-05, COORD-06]

# Metrics
duration: 2min
completed: 2026-02-23
---

# Phase 2 Plan 3: Dependency Management Summary

**In-process DFS cycle detection with path reporting, add/remove/list dep operations, and dep info integrated into get/list/deps views — zero new crate dependencies.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-23T01:09:02Z
- **Completed:** 2026-02-23T01:10:38Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added would_create_cycle (DFS over in-memory adjacency list) and add_dep/remove_dep/list_deps to ticket.rs
- Added Dep (add/remove) and Deps (tree view) CLI commands with dispatch in lib.rs
- Updated get to show "Depends on" and "Required by", updated list to show "[N deps]" count suffix
- All 13 existing integration tests continue to pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Dependency functions in ticket.rs** - `da8da3f` (feat)
2. **Task 2: Dep/Deps commands and get/list display** - `3711a62` (feat)

**Plan metadata:** TBD (docs: complete plan)

## Files Created/Modified

- `src/ticket.rs` - Added DepInfo struct, would_create_cycle, dfs_finds_target, add_dep, remove_dep, list_deps; new CyclicDependency and DepNotFound error variants
- `src/cli.rs` - Added DepArgs, DepAction (Add/Remove), DepsArgs structs and Commands::Dep/Deps variants
- `src/lib.rs` - Wired Dep/Deps dispatch; updated Get to show deps; updated List with dep counts; added format_name_with_deps and load_dep_counts helpers

## Decisions Made

- Self-dependency is pre-checked in `add_dep` (before calling `would_create_cycle`) to return `CyclicDependency` rather than a DB constraint error — consistent error type.
- `would_create_cycle` loads the entire `ticket_deps` table into a `HashMap<i64, Vec<i64>>` adjacency list in-process, then does DFS. Correct for the expected scale of agent workloads, no crate overhead.
- `Deps` (plural, read-only tree view) is a separate top-level command from `Dep` (singular, mutation subcommand), consistent with the CONTEXT.md spec.
- Name column truncates to 35 chars when `[N deps]` suffix is present, 40 chars otherwise, keeping list rows within the 60-char rule line.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Dependency management complete: COORD-04, COORD-05, COORD-06 fulfilled
- ticket_deps table already has ON DELETE CASCADE from 02-01 migration
- Ready for Phase 3 (export/reporting) which can query deps for context export

---
*Phase: 02-agent-coordination-dependencies*
*Completed: 2026-02-23*
