# Roadmap: rtik

## Overview

rtik delivers in three focused phases: foundation (database + CRUD), coordination (claiming + dependencies), and discovery (search + export). Each phase builds on the previous, delivering complete functionality that agents can use immediately. The journey moves from basic ticket persistence to full multi-agent coordination with token-efficient exports.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundation & Core Operations** - Database persistence, CRUD operations, and CLI infrastructure
- [ ] **Phase 2: Agent Coordination & Dependencies** - Claiming mechanism, status transitions, and dependency tracking
- [ ] **Phase 3: Search, Filtering & Export** - Powerful queries, short aliases, and agent-optimized export formats

## Phase Details

### Phase 1: Foundation & Core Operations
**Goal**: Agents can persist and retrieve ticket data via CLI with zero runtime dependencies
**Depends on**: Nothing (first phase)
**Requirements**: CRUD-01, CRUD-02, CRUD-03, CRUD-04, CRUD-05, STATE-01, STATE-02, STATE-03, STATE-05, TECH-01, TECH-02, TECH-05, TECH-06, CLI-04, CLI-05, CLI-06
**Success Criteria** (what must be TRUE):
  1. User can create ticket with name and description, receives unique ID
  2. User can view ticket details by ID showing all fields
  3. User can update ticket fields and delete tickets
  4. User can list all tickets with status, timestamps, and IDs
  5. CLI provides helpful error messages and exits with standard codes (0=success, 1=error, 2=usage)
  6. SQLite database persists between CLI invocations with WAL mode enabled
**Plans**: 3 plans

Plans:
- [ ] 01-01-PLAN.md — Project scaffold and DB layer (Cargo setup, WAL, migrations, path resolution)
- [ ] 01-02-PLAN.md — Ticket CRUD functions and CLI dispatch with output formatting
- [ ] 01-03-PLAN.md — Integration tests and release binary verification

### Phase 2: Agent Coordination & Dependencies
**Goal**: Multiple agents coordinate work through atomic claiming and dependency tracking without deadlocks
**Depends on**: Phase 1
**Requirements**: STATE-04, COORD-01, COORD-02, COORD-03, COORD-04, COORD-05, COORD-06, TECH-03, TECH-04
**Success Criteria** (what must be TRUE):
  1. Agent can claim unclaimed ticket atomically (two agents claiming same ticket: one succeeds, one fails)
  2. Claimed ticket shows claimed_by and claimed_at timestamp
  3. Agent can release claimed ticket, making it available to others
  4. User can add dependency between tickets, see dependency count in lists
  5. System rejects circular dependencies with clear error message
  6. Status transitions validate state machine (cannot go from done to todo)
**Plans**: TBD

Plans:
- [ ] TBD after planning

### Phase 3: Search, Filtering & Export
**Goal**: Agents efficiently discover relevant work and export context in token-optimized formats
**Depends on**: Phase 2
**Requirements**: QUERY-01, QUERY-02, QUERY-03, QUERY-04, QUERY-05, QUERY-06, CLI-01, CLI-02, CLI-03, EXPORT-01, EXPORT-02, EXPORT-03, EXPORT-04
**Success Criteria** (what must be TRUE):
  1. User can filter tickets by status, claimed status, and claimer
  2. User can search tickets by name or description (substring match)
  3. User can combine multiple filters in single query
  4. Short aliases work: 'new' creates, 'ls' lists, 'claim' claims tickets
  5. Plain text export includes only essential fields (ID, name, description, dependencies)
  6. JSON export option available for programmatic parsing
**Plans**: TBD

Plans:
- [ ] TBD after planning

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation & Core Operations | 0/3 | Not started | - |
| 2. Agent Coordination & Dependencies | 0/TBD | Not started | - |
| 3. Search, Filtering & Export | 0/TBD | Not started | - |
