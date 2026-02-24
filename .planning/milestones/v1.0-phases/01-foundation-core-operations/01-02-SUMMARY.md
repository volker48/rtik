---
phase: 01-foundation-core-operations
plan: "02"
subsystem: database
tags: [rust, rusqlite, sqlite, clap, crud]

# Dependency graph
requires:
  - phase: 01-01
    provides: Ticket struct, AppError enum, DB connection layer, CLI stubs, stub run()
provides:
  - Five CRUD functions in ticket.rs: create_ticket, get_ticket, list_tickets, delete_ticket, update_ticket
  - Complete run() dispatch in lib.rs wiring all 5 subcommands to DB functions
  - All CLI output formats: Created/Get/Updated/Deleted/List with aligned table and --timestamps flag
  - Status case normalization: input any case, stored/displayed lowercase
  - Name truncation at 40 chars with ellipsis in list output
affects: [01-03]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Dynamic named-params Vec for rusqlite execute to match SET clause exactly (avoid unmatched param error)
    - Pure std UTC timestamp helper using SystemTime + UNIX_EPOCH with Gregorian calendar calculation
    - fetch-then-delete pattern in Delete command to get name for confirmation output
    - Normalize Option<&str> status to Option<String> before building params vec (lifetime fix)

key-files:
  created: []
  modified:
    - src/ticket.rs
    - src/lib.rs
    - src/cli.rs

key-decisions:
  - "Dynamic params Vec in update_ticket: only params matching the SET clause are bound, avoiding rusqlite unmatched named parameter error"
  - "normalize_status as Option<String> declared before params Vec to ensure lifetime outlives the vec"
  - "fetch ticket before delete in Delete command to include name in confirmation output"
  - "parse_status made pub in cli.rs for external test access"

patterns-established:
  - "Pattern: update_ticket builds SET clause and params Vec together — sets.push() and params.push() in same if branch"
  - "Pattern: date display in get/list by splitting ISO timestamp on T and taking first part"

requirements-completed: [CRUD-01, CRUD-02, CRUD-03, CRUD-04, CRUD-05, STATE-01, STATE-02, STATE-03, CLI-04]

# Metrics
duration: 2min
completed: 2026-02-22
---

# Phase 1 Plan 2: CRUD Implementation Summary

**Five CRUD database functions in ticket.rs plus full run() CLI dispatch in lib.rs delivering working create, get, update, delete, and list commands with aligned table output and --timestamps flag**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-22T16:16:16Z
- **Completed:** 2026-02-22T16:18:28Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- All 5 CRUD functions implemented with correct rusqlite error mapping (NotFound on 0 rows affected)
- update_ticket builds named params Vec dynamically to exactly match the SET clause, avoiding rusqlite unmatched parameter errors
- Complete run() dispatch with all output formats matching CONTEXT.md specifications
- Status normalized to lowercase in both Clap value_parser (parse time) and update_ticket (storage)
- End-to-end smoke test: all 12 verification steps pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement ticket CRUD database functions** - `c6e90d9` (feat)
2. **Task 2: Wire CLI dispatch with output formatting** - `af3b8ca` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src/ticket.rs` - Five CRUD functions: create_ticket, get_ticket, list_tickets, delete_ticket, update_ticket; pure-std UTC timestamp helper
- `src/lib.rs` - Complete run() dispatch for all 5 subcommands with all output formats
- `src/cli.rs` - Made parse_status pub

## Decisions Made

- Dynamic params Vec in update_ticket: build `sets` and `params` in the same `if` branch so only params in the SQL are bound. Rusqlite returns an error if a named param is in the bind list but absent from the SQL.
- `normalized_status: Option<String>` declared before the params Vec so its lifetime covers the borrow. Using `Option<&str>` inside if-let blocks caused lifetime errors because the binding was dropped at block end.
- Fetch ticket before delete: the Delete command calls `get_ticket` first to retrieve the name, then `delete_ticket`. This matches the CONTEXT.md output spec `Deleted: #1 Buy milk`.
- `parse_status` made pub for future test access.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed `&str` to `&dyn ToSql` coercion in update_ticket params Vec**
- **Found during:** Task 1 (implement ticket CRUD database functions)
- **Issue:** Plan's code example used `n` (type `&str`) directly as `&dyn ToSql`. Rust requires `&&str` for this coercion. Initial `&n` fix introduced lifetime errors because `n` was bound in an `if let` block and dropped at block end while `params` still held a borrow.
- **Fix:** Declared `normalized_status: Option<String>` before the params Vec. Used `name.as_ref().unwrap()` and `desc.as_ref().unwrap()` (guarded by `.is_some()` checks) so the `&str` refs point into function parameters that live for the whole function, not into dropped if-let bindings.
- **Files modified:** `src/ticket.rs`
- **Verification:** `cargo check` passes; smoke tests confirm partial update (status-only) works without rusqlite named parameter error
- **Committed in:** `c6e90d9` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug in plan's example code)
**Impact on plan:** Fix required for correctness; no scope creep. Behavior matches plan spec exactly.

## Issues Encountered

The plan's code example for `update_ticket` had a Rust type coercion issue: pushing `n: &str` directly into `Vec<(&str, &dyn ToSql)>` fails because `&str` is unsized and `&&str` is needed for the `ToSql` coercion. A secondary lifetime error appeared when using `&n` because `n` was a temporary if-let binding. Resolved by restructuring to keep the `Option<String>` for normalized status and using the original function parameters (which have function-scope lifetime) for name and desc refs.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All 5 CRUD commands work correctly via the binary
- Output formats match CONTEXT.md specifications exactly
- Status normalization, error routing, and aligned table output all verified
- No blockers for Plan 03

## Self-Check: PASSED

All created/modified files verified to exist on disk. All task commits verified in git history.

- FOUND: src/ticket.rs
- FOUND: src/lib.rs
- FOUND: src/cli.rs
- FOUND: .planning/phases/01-foundation-core-operations/01-02-SUMMARY.md
- FOUND: commit c6e90d9 (Task 1)
- FOUND: commit af3b8ca (Task 2)

---
*Phase: 01-foundation-core-operations*
*Completed: 2026-02-22*
