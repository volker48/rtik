---
phase: 03-search-filtering-export
plan: "01"
subsystem: cli
tags: [clap, serde, serde_json, cli, rust]

# Dependency graph
requires:
  - phase: 02-agent-coordination-dependencies
    provides: "working rtik binary with all commands through Deps"
provides:
  - "serde and serde_json dependencies available in Cargo.toml"
  - "Command aliases: new, ls, get, up, rm, rel, dep, deps, dump"
  - "ListArgs with filter fields: status, claimed, unclaimed, claimer, search"
  - "ExportArgs struct with filter fields plus --json flag"
  - "Commands::Export(ExportArgs) variant"
affects:
  - "03-02-PLAN (list filtering wiring)"
  - "03-03-PLAN (export wiring)"

# Tech tracking
tech-stack:
  added:
    - serde 1.0.228 (derive feature)
    - serde_json 1.0.149
  patterns:
    - "Clap #[command(alias)] attribute on Commands variants for short aliases"
    - "Vec<String> for repeatable --search flag (all terms must match)"
    - "Option<String> status on ListArgs passes through to DB without validation (invalid returns empty)"

key-files:
  created: []
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/cli.rs
    - src/lib.rs

key-decisions:
  - "ListArgs --status has no value_parser (no enum) — accepts any string, invalid values return empty results"
  - "Commands::Export stub match arm added to lib.rs to keep compilation clean before logic wired in Plan 02/03"
  - "Block command has no alias — not in the locked alias set from CONTEXT.md"
  - "Claim command has no alias — canonical name is already 'claim' per RESEARCH.md recommendation"

patterns-established:
  - "Alias pattern: #[command(alias = \"short\")] on each Commands variant"
  - "Repeatable flags: Vec<String> type with #[arg(long = \"search\")]"

requirements-completed:
  - CLI-01
  - CLI-02
  - CLI-03
  - EXPORT-01
  - QUERY-01
  - QUERY-02
  - QUERY-03
  - QUERY-04
  - QUERY-05
  - QUERY-06

# Metrics
duration: 2min
completed: 2026-02-23
---

# Phase 3 Plan 01: Search/Filtering/Export CLI Surface Summary

**Clap command aliases (9 total), ListArgs filter flags (status/claimed/unclaimed/claimer/search), ExportArgs struct, and serde/serde_json dependencies added as pure structural CLI change**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-23T22:51:07Z
- **Completed:** 2026-02-23T22:52:30Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added serde 1.0.228 (with derive) and serde_json 1.0.149 to Cargo.toml
- Added 9 command aliases to Commands enum (new, ls, get, up, rm, rel, dep, deps, dump)
- Extended ListArgs with 5 filter fields (status, claimed, unclaimed, claimer, search)
- Created ExportArgs struct with same filter fields plus --json flag
- Added Commands::Export(ExportArgs) variant with alias "dump"

## Task Commits

Each task was committed atomically:

1. **Task 1: Add serde dependencies to Cargo.toml** - `c2c8789` (chore)
2. **Task 2: Add command aliases, filter flags, and Export command to cli.rs** - `8d63efc` (feat)

**Plan metadata:** (docs commit below)

## Files Created/Modified
- `Cargo.toml` - Added serde (features = ["derive"]) and serde_json dependencies
- `Cargo.lock` - Updated with resolved dependency versions
- `src/cli.rs` - Added aliases on all Commands variants, new filter fields on ListArgs, ExportArgs struct, Commands::Export variant
- `src/lib.rs` - Added stub Commands::Export match arm to keep non-exhaustive match compiling

## Decisions Made
- ListArgs `--status` uses no `value_parser` — accepts any string, invalid values silently return empty results (consistent with plan's locked decision)
- `Block` and `Claim` commands have no aliases per the locked alias set in CONTEXT.md
- Stub match arm `Commands::Export(_args) => eprintln! + exit(1)` added to lib.rs so the codebase compiles cleanly before logic is wired in Plans 02/03

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added Commands::Export stub match arm to lib.rs**
- **Found during:** Task 2 (Add command aliases, filter flags, and Export command to cli.rs)
- **Issue:** Adding `Commands::Export(ExportArgs)` to the enum caused a non-exhaustive pattern error in lib.rs's match statement
- **Fix:** Added stub arm `Commands::Export(_args) => { eprintln!("export: not yet implemented"); std::process::exit(1); }` to keep compilation clean
- **Files modified:** src/lib.rs
- **Verification:** `cargo check --quiet` exits 0 with no errors
- **Committed in:** `8d63efc` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary to keep the codebase compilable. The stub will be replaced with real logic in Plan 02/03.

## Issues Encountered
None beyond the expected non-exhaustive match error (handled as Rule 3 deviation).

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- CLI surface complete for Plans 02 and 03 to wire in list filtering and export logic
- `ListFilter` struct in ticket.rs will need to accept the filter fields from ListArgs
- `Commands::Export` stub in lib.rs ready to be replaced with real export logic

---
*Phase: 03-search-filtering-export*
*Completed: 2026-02-23*
