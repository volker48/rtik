---
phase: 03-search-filtering-export
plan: "02"
subsystem: database
tags: [rust, rusqlite, serde, filtering, export, sqlite, dynamic-sql]

# Dependency graph
requires:
  - phase: 03-search-filtering-export/03-01
    provides: "CLI ListArgs/ExportArgs structs and serde dependency"
  - phase: 02-agent-coordination-dependencies
    provides: "ticket_deps table, list_deps function, Ticket struct, AppError"
provides:
  - "ListFilter struct with status/claimed/claimer/search fields"
  - "list_tickets_filtered with dynamic WHERE clause using positional params"
  - "TicketExport struct with serde::Serialize derive"
  - "tickets_to_export function enriching tickets with dependency IDs"
  - "format_export_text producing T-{id} [{status}] {name} [deps:...] lines"
affects: [03-03-wiring, 03-04-tests]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Dynamic SQL WHERE using positional ? params with Vec<Box<dyn ToSql>>"
    - "N+1 list_deps per ticket for export enrichment (acceptable at small scale)"
    - "list_tickets refactored as zero-filter wrapper over list_tickets_filtered"

key-files:
  created: []
  modified:
    - src/ticket.rs

key-decisions:
  - "Ticket struct gained claimed_by: Option<String> field and all queries updated — required for export to surface claimer without extra queries"
  - "list_tickets_filtered uses positional ? params (not named params) because dynamic param counts are incompatible with named param binding in rusqlite"
  - "list_tickets refactored as wrapper calling list_tickets_filtered with empty filter — removes code duplication"
  - "N+1 list_deps pattern in tickets_to_export: one query per ticket, acceptable given small-scale tool usage"

patterns-established:
  - "Dynamic WHERE builder: conditions Vec<String> + params Vec<Box<dyn ToSql>>, joined with AND, empty = no WHERE clause"
  - "Export text format: T-{id} [{status}] {name} deps:T-x,T-y (T- prefix for export, # prefix for interactive)"

requirements-completed: [QUERY-01, QUERY-02, QUERY-03, QUERY-04, QUERY-05, QUERY-06, EXPORT-01, EXPORT-02, EXPORT-03, EXPORT-04]

# Metrics
duration: 35min
completed: 2026-02-23
---

# Phase 3 Plan 02: Search Filtering Export — Ticket Library Layer Summary

**Dynamic SQL filtering with ListFilter/list_tickets_filtered plus TicketExport/format_export_text for token-efficient plain-text export**

## Performance

- **Duration:** 35 min
- **Started:** 2026-02-23T22:54:22Z
- **Completed:** 2026-02-23T23:29:49Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- `ListFilter` struct with 4 fields: status (exact), claimed (IS NULL / IS NOT NULL), claimer (exact), search (LIKE per term, AND-composed)
- `list_tickets_filtered` builds dynamic WHERE clause via positional `?` params — handles zero to many filter conditions correctly
- `TicketExport` with `#[derive(Serialize)]` and `tickets_to_export` enriching each ticket with dependency IDs via `list_deps`
- `format_export_text` producing `T-{id} [{status}] {name}` with optional ` deps:T-x,T-y` suffix
- All 33 existing tests pass after adding `claimed_by` field to `Ticket` struct

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ListFilter and list_tickets_filtered** - `168149f` (feat)
2. **Task 2: Add TicketExport, tickets_to_export, format_export_text** - `32fd194` (feat)

## Files Created/Modified

- `src/ticket.rs` - Added ListFilter struct, list_tickets_filtered function, TicketExport struct, tickets_to_export, format_export_text; added claimed_by to Ticket struct

## Decisions Made

- `Ticket` struct gained `claimed_by: Option<String>` — needed so `tickets_to_export` can surface claimer without extra per-ticket queries
- Used positional `?` params (not named params) — rusqlite named param binding is incompatible with dynamic param counts
- `list_tickets` refactored as a wrapper calling `list_tickets_filtered` with empty filter — eliminates code duplication

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added claimed_by field to Ticket struct and updated all queries**
- **Found during:** Task 2 (TicketExport/tickets_to_export implementation)
- **Issue:** `tickets_to_export` referenced `t.claimed_by` but `Ticket` struct only had `id, name, description, status, created_at, updated_at` — code would not compile
- **Fix:** Added `claimed_by: Option<String>` to `Ticket`; updated `get_ticket` and `list_tickets_filtered` SQL to SELECT claimed_by and map column index 4 for claimed_by, 5/6 for created_at/updated_at
- **Files modified:** src/ticket.rs
- **Verification:** `cargo check` passes with zero errors; all 33 tests pass
- **Committed in:** `32fd194` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - compilation bug)
**Impact on plan:** Necessary fix — plan spec implied claimed_by was available on Ticket but the struct predated the claim feature. No scope creep.

## Issues Encountered

None beyond the auto-fixed deviation above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All library types (`ListFilter`, `list_tickets_filtered`, `TicketExport`, `tickets_to_export`, `format_export_text`) are ready for Plan 03 wiring into `lib.rs` command handlers
- Plan 03 can now wire `Commands::List` to call `list_tickets_filtered` with a `ListFilter` built from `ListArgs`, and `Commands::Export` to call `tickets_to_export` then either `serde_json::to_string_pretty` or `format_export_text`

---
*Phase: 03-search-filtering-export*
*Completed: 2026-02-23*

## Self-Check: PASSED

- FOUND: src/ticket.rs
- FOUND: .planning/phases/03-search-filtering-export/03-02-SUMMARY.md
- FOUND commit: 168149f (feat(03-02): add ListFilter struct and list_tickets_filtered)
- FOUND commit: 32fd194 (feat(03-02): add TicketExport struct, tickets_to_export, format_export_text)
