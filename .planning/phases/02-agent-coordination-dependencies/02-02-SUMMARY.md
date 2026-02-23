---
phase: 02-agent-coordination-dependencies
plan: 02
subsystem: coordination
tags: [sqlite, rusqlite, claiming, state-machine, cli]

requires:
  - phase: 02-01
    provides: M2 schema with claimed_by, claimed_at, block_reason, ticket_deps columns

provides:
  - claim_ticket with IMMEDIATE transaction atomicity and unmet-dep warnings
  - release_ticket with ownership check and --force override
  - block_ticket with state-machine validation
  - validate_transition enforcing todo/in-progress/blocked/done state machine
  - Claim, Release, Block CLI subcommands with RTIK_AGENT env var resolution
  - update_ticket now validates transitions and auto-clears claim on done

affects:
  - 02-03 (block command now fully functional)
  - 02-04 (dep warning in claim already works; ticket_deps infra ready)

tech-stack:
  added: []
  patterns:
    - "IMMEDIATE transaction via transaction_with_behavior for atomic claiming"
    - "conn.changes() == 0 to detect lost-race vs not-found in claim"
    - "State machine in pure validate_transition function called from both update_ticket and block_ticket"
    - "conn shadowed as mut in lib.rs run() to avoid public signature change"
    - "Named struct variant for InvalidTransition to avoid thiserror positional-arg limitation"

key-files:
  created: []
  modified:
    - src/ticket.rs
    - src/cli.rs
    - src/lib.rs

key-decisions:
  - "Use named struct variant for InvalidTransition (thiserror cannot call .join() on positional tuple args)"
  - "conn shadowed as mut inside run() rather than changing public signature — avoids touching main.rs"
  - "validate_transition called in update_ticket with pre-fetch of current status (one extra query per status change)"
  - "block_ticket uses &Connection (not &mut) — no IMMEDIATE tx needed for single-statement update"

patterns-established:
  - "Atomic SQLite claiming via IMMEDIATE transaction + changes() == 0 check"
  - "Status state machine enforced at write layer, not in CLI"

requirements-completed: [STATE-04, COORD-01, COORD-02, COORD-03, TECH-03, TECH-04]

duration: 2min
completed: 2026-02-23
---

# Phase 2 Plan 02: Atomic Claiming and Status State Machine Summary

**Atomic claiming via IMMEDIATE SQLite transactions, status state machine enforcement, and Claim/Release/Block CLI subcommands wired through RTIK_AGENT env var**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-23T01:03:57Z
- **Completed:** 2026-02-23T01:05:57Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- claim_ticket uses IMMEDIATE transaction and WHERE claimed_by IS NULL for atomic race-free claiming; changes() == 0 distinguishes NotFound from AlreadyClaimed
- release_ticket checks ownership (or force-overrides with warning) before resetting to todo
- block_ticket validates transition via validate_transition before setting blocked status and block_reason
- validate_transition enforces state machine: todo->in-progress/blocked, in-progress->done/blocked/todo, blocked->in-progress/todo, done->in-progress
- update_ticket fetches current status and calls validate_transition before any write; auto-clears claimed_by/claimed_at when status becomes done
- Claim/Release/Block subcommands added to CLI; RTIK_AGENT env var read via resolve_agent() with clear error on missing
- All 13 existing integration tests still pass; no regressions

## Task Commits

1. **Task 1: Add claim, release, block, and validate_transition to ticket.rs** - `0fd03cd` (feat)
2. **Task 2: Add Claim/Release/Block CLI subcommands and dispatch in cli.rs + lib.rs** - `54bdb7a` (feat)

## Files Created/Modified

- `src/ticket.rs` - New AppError variants, validate_transition, claim_ticket, release_ticket, block_ticket; update_ticket enhanced with transition validation and done auto-release
- `src/cli.rs` - ClaimArgs, ReleaseArgs, BlockArgs structs; Claim, Release, Block added to Commands enum
- `src/lib.rs` - resolve_agent() function; conn shadowed as mut; Claim/Release/Block dispatch arms

## Decisions Made

- Named struct variant for InvalidTransition because thiserror cannot format Vec<String>.join() in positional tuple format string
- conn shadowed as mut inside run() to support &mut Connection for claim/release without changing the public function signature
- validate_transition called in update_ticket with a pre-fetch query (one extra SELECT per status-changing update — acceptable per plan)
- block_ticket uses &Connection not &mut because a single execute doesn't need IMMEDIATE isolation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed InvalidTransition variant syntax**
- **Found during:** Task 1 (first build attempt)
- **Issue:** Plan specified `#[error("from {0}, valid transitions are: {}", .1.join(", "))]` which thiserror cannot compile for tuple variants (ambiguous positional reference + method call on vec)
- **Fix:** Changed to named struct variant `InvalidTransition { from: String, valid: String }` with pre-joined string at construction
- **Files modified:** src/ticket.rs
- **Commit:** 0fd03cd

## Issues Encountered

None beyond the InvalidTransition syntax deviation auto-fixed above.

## User Setup Required

Set `RTIK_AGENT` environment variable before using claim or release commands:
```bash
export RTIK_AGENT=my-agent-name
rtik claim 1
```

## Next Phase Readiness

- Claim/release/block all functional and tested via smoke tests
- Status state machine enforced on all update paths
- RTIK_AGENT resolution in place for Phase 2 Plans 03 and 04

---
*Phase: 02-agent-coordination-dependencies*
*Completed: 2026-02-23*

## Self-Check: PASSED

- FOUND: src/ticket.rs
- FOUND: src/cli.rs
- FOUND: src/lib.rs
- FOUND: 02-02-SUMMARY.md
