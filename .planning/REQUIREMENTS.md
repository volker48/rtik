# Requirements: rtik

**Defined:** 2026-02-22
**Core Value:** Agents never lose track of work when context resets - persistent, queryable task state that survives session restarts and enables multi-agent coordination.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### CRUD Operations

- [x] **CRUD-01**: User can create ticket with name and description via CLI
- [x] **CRUD-02**: User can view ticket details by ID
- [x] **CRUD-03**: User can update ticket fields (name, description, status)
- [x] **CRUD-04**: User can delete ticket by ID
- [x] **CRUD-05**: User can list all tickets

### State Management

- [x] **STATE-01**: Ticket has status field with values: todo, WIP, blocked, done
- [x] **STATE-02**: Ticket automatically tracks created_at timestamp on creation
- [x] **STATE-03**: Ticket automatically updates updated_at timestamp on modification
- [x] **STATE-04**: Status transitions are validated (e.g., cannot go from done to todo)
- [x] **STATE-05**: Ticket has unique auto-incrementing ID

### Agent Coordination

- [x] **COORD-01**: Agent can claim ticket (sets claimed_by field)
- [x] **COORD-02**: Claimed ticket records claimed_at timestamp
- [x] **COORD-03**: Agent can release/unclaim ticket (allows reassignment)
- [x] **COORD-04**: User can add dependency between tickets (ticket A depends on ticket B)
- [x] **COORD-05**: User can remove dependency between tickets
- [x] **COORD-06**: System detects and rejects circular dependencies

### Query & Search

- [ ] **QUERY-01**: User can filter tickets by status
- [ ] **QUERY-02**: User can filter tickets by claimed status (claimed vs unclaimed)
- [ ] **QUERY-03**: User can filter tickets by claimer (claimed_by value)
- [ ] **QUERY-04**: User can search tickets by name (substring match)
- [ ] **QUERY-05**: User can search tickets by description (substring match)
- [ ] **QUERY-06**: User can combine multiple filters in single query

### Export

- [ ] **EXPORT-01**: User can export tickets to plain text format
- [ ] **EXPORT-02**: Plain text export includes: ID, name, description, dependencies
- [ ] **EXPORT-03**: Plain text export is token-efficient (minimal verbosity)
- [ ] **EXPORT-04**: User can export tickets to JSON format

### CLI Interface

- [ ] **CLI-01**: CLI provides short alias 'new' for create command
- [ ] **CLI-02**: CLI provides short alias 'ls' for list command
- [ ] **CLI-03**: CLI provides short alias 'claim' for claim command
- [x] **CLI-04**: CLI provides helpful error messages with context
- [x] **CLI-05**: CLI exits with standard codes (0=success, 1=error, 2=usage)
- [x] **CLI-06**: CLI handles broken pipe gracefully (piping to head/grep)

### Technical Infrastructure

- [x] **TECH-01**: All data persists in SQLite database
- [x] **TECH-02**: SQLite uses WAL mode for concurrent reads during writes
- [x] **TECH-03**: Claim operations use atomic UPDATE to prevent race conditions
- [x] **TECH-04**: Database transactions use IMMEDIATE mode to prevent write starvation
- [x] **TECH-05**: CLI compiles to single binary with zero runtime dependencies
- [x] **TECH-06**: Database schema supports migrations for future changes

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Advanced Features

- **ADV-01**: Virtual tags computed from state (+BLOCKED, +BLOCKING)
- **ADV-02**: Context system for auto-filtering based on working context
- **ADV-03**: Work log with append-only history of agent actions
- **ADV-04**: Stale claim detection and highlighting (>1 hour)
- **ADV-05**: Bulk operations (update multiple tickets)
- **ADV-06**: Custom fields/metadata on tickets

### Analytics

- **ANAL-01**: Summary statistics (tickets by status, avg completion time)
- **ANAL-02**: Agent productivity metrics (tickets completed per agent)
- **ANAL-03**: Dependency graph visualization

### Integration

- **INTEG-01**: Import from CSV/JSON
- **INTEG-02**: Git-based sync for multi-machine coordination
- **INTEG-03**: Webhook support for external integrations

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Web UI or GUI | CLI-only tool; agents don't need visual interfaces |
| Hard dependency enforcement | Agents need flexibility to work out of order; informational tracking sufficient |
| User authentication | Single-user local tool; no multi-user concerns |
| Remote sync or cloud storage | Local SQLite only; complexity not justified for v1 |
| Real-time collaboration | Adds complexity; polling model sufficient for agent coordination |
| Hierarchical tasks/subtasks | Creates query complexity; use dependencies instead |
| Recurring tasks | Edge case complexity; use external scheduler if needed |
| Time tracking | Feature bloat; many dedicated tools exist |
| Rich text or markdown in descriptions | Plain text only; keeps parsing simple |
| File attachments | Text-based task tracking only |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CRUD-01 | Phase 1 | Complete |
| CRUD-02 | Phase 1 | Complete |
| CRUD-03 | Phase 1 | Complete |
| CRUD-04 | Phase 1 | Complete |
| CRUD-05 | Phase 1 | Complete |
| STATE-01 | Phase 1 | Complete |
| STATE-02 | Phase 1 | Complete |
| STATE-03 | Phase 1 | Complete |
| STATE-04 | Phase 2 | Complete |
| STATE-05 | Phase 1 | Complete (01-01) |
| COORD-01 | Phase 2 | Complete |
| COORD-02 | Phase 2 | Complete |
| COORD-03 | Phase 2 | Complete |
| COORD-04 | Phase 2 | Complete |
| COORD-05 | Phase 2 | Complete |
| COORD-06 | Phase 2 | Complete |
| QUERY-01 | Phase 3 | Pending |
| QUERY-02 | Phase 3 | Pending |
| QUERY-03 | Phase 3 | Pending |
| QUERY-04 | Phase 3 | Pending |
| QUERY-05 | Phase 3 | Pending |
| QUERY-06 | Phase 3 | Pending |
| EXPORT-01 | Phase 3 | Pending |
| EXPORT-02 | Phase 3 | Pending |
| EXPORT-03 | Phase 3 | Pending |
| EXPORT-04 | Phase 3 | Pending |
| CLI-01 | Phase 3 | Pending |
| CLI-02 | Phase 3 | Pending |
| CLI-03 | Phase 3 | Pending |
| CLI-04 | Phase 1 | Complete |
| CLI-05 | Phase 1 | Complete (01-01) |
| CLI-06 | Phase 1 | Complete (01-01) |
| TECH-01 | Phase 1 | Complete (01-01) |
| TECH-02 | Phase 1 | Complete (01-01) |
| TECH-03 | Phase 2 | Complete |
| TECH-04 | Phase 2 | Complete |
| TECH-05 | Phase 1 | Complete (01-01) |
| TECH-06 | Phase 1 | Complete (01-01) |

**Coverage:**
- v1 requirements: 38 total
- Mapped to phases: 38 (100%)
- Unmapped: 0

---
*Requirements defined: 2026-02-22*
*Last updated: 2026-02-22 after 01-01 completion*
