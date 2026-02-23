---
phase: 03-search-filtering-export
plan: "03"
subsystem: cli
tags: [rust, clap, serde_json, rusqlite, filter, export]

requires:
  - phase: 03-01-search-filtering-export
    provides: ListArgs and ExportArgs with filter flags in cli.rs, Export command alias stub
  - phase: 03-02-search-filtering-export
    provides: list_tickets_filtered, tickets_to_export, format_export_text, ListFilter, TicketExport in ticket.rs

provides:
  - Commands::List wired to list_tickets_filtered via build_filter_from_list helper
  - Commands::Export match arm with plain-text and --json output via build_filter_from_export helper
  - build_filter_from_list (ListArgs -> ListFilter) and build_filter_from_export (ExportArgs -> ListFilter) private helpers
  - Mutually exclusive --claimed/--unclaimed validation with process::exit(1) on conflict

affects:
  - 03-04-search-filtering-export
  - any future plan extending filter or export behavior

tech-stack:
  added: []
  patterns:
    - build_filter_from_* private helper pattern: converts CLI Args struct to ticket::ListFilter, validates mutual exclusion, exits on conflict
    - Export command follows Unix no-output-on-empty convention (empty loop/array, exit 0)

key-files:
  created: []
  modified:
    - src/lib.rs

key-decisions:
  - "build_filter_from_list returns Result<ListFilter, AppError> (unused Ok path) while build_filter_from_export returns ListFilter directly — both use process::exit(1) for mutual exclusion, consistent with existing Update handler pattern"
  - "serde_json already in Cargo.toml from phase setup — no import alias needed, fully-qualified serde_json::to_string_pretty used inline"

patterns-established:
  - "Mutual exclusion check: if args.claimed && args.unclaimed { eprintln!(...); process::exit(1); } pattern"
  - "Export empty output: no 'no results' message — Unix grep-like behavior, empty output on no matches"

requirements-completed:
  - QUERY-01
  - QUERY-02
  - QUERY-03
  - QUERY-04
  - QUERY-05
  - QUERY-06
  - EXPORT-01
  - EXPORT-02
  - EXPORT-03
  - EXPORT-04
  - CLI-01
  - CLI-02
  - CLI-03

duration: 1min
completed: 2026-02-23
---

# Phase 03 Plan 03: Wire Filter Flags and Export Command Summary

**CLI filter flags fully wired: Commands::List uses list_tickets_filtered with build_filter_from_list helper; Commands::Export delivers plain-text and --json output via serde_json**

## Performance

- **Duration:** ~1 min
- **Started:** 2026-02-23T23:32:07Z
- **Completed:** 2026-02-23T23:33:01Z
- **Tasks:** 2 (committed together — both modify src/lib.rs)
- **Files modified:** 1

## Accomplishments
- Replaced `list_tickets` stub call with `list_tickets_filtered` in Commands::List, enabling all filter flags (--status, --claimed, --unclaimed, --claimer, --search)
- Added Commands::Export match arm with plain-text (`T-{id} [{status}] {name}`) and JSON (`--json`) output branches
- Added `build_filter_from_list` and `build_filter_from_export` private helpers that convert CLI args to `ticket::ListFilter`
- --claimed/--unclaimed mutual exclusion validated with clear error message and exit code 1

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire filter flags into the List command handler** - `f7ca50b` (feat)
2. **Task 2: Add Export command match arm with plain-text and JSON output** - `f7ca50b` (feat, same commit — both tasks modify src/lib.rs)

## Files Created/Modified
- `src/lib.rs` - Replaced list_tickets with list_tickets_filtered in Commands::List; replaced Export stub with full implementation; added build_filter_from_list and build_filter_from_export private helpers

## Decisions Made
- Tasks 1 and 2 committed together since both exclusively modify `src/lib.rs` — splitting would have left the file in a non-compilable intermediate state between commits
- `build_filter_from_export` returns `ticket::ListFilter` (not `Result<..>`) since there is no fallible path beyond the process::exit on conflict

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None - cargo check and release build clean on first attempt. All smoke tests and success criteria verified.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All filter flags (--status, --unclaimed, --claimed, --claimer, --search) work end-to-end in `rtik ls`
- `rtik export` / `rtik dump` deliver plain-text and JSON output with full filter support
- Ready for Plan 04 (integration tests)

---
*Phase: 03-search-filtering-export*
*Completed: 2026-02-23*
