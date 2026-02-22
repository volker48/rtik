# Project Research Summary

**Project:** rtik - CLI Ticketing System for LLM Agents
**Domain:** CLI Task Management / Agent Coordination
**Researched:** 2026-02-22
**Confidence:** HIGH

## Executive Summary

rtik is a CLI-based ticketing system optimized for LLM agent coordination. Research shows this domain is well-established (Taskwarrior, todo.txt) but existing tools lack agent-specific features like soft claiming, token-efficient exports, and informational dependency tracking. The recommended approach is a Rust-based single binary with SQLite storage, leveraging Rust's zero-dependency distribution and excellent CLI ecosystem (clap, rusqlite).

The architecture follows standard layered patterns: thin CLI interface over business logic over data access. Critical to success is getting SQLite concurrency right from day one (WAL mode, IMMEDIATE transactions) since multi-agent coordination is a distributed systems problem. The core insight from research is that agents need visibility into what others are doing without hard locks that create artificial bottlenecks.

Key risks center on concurrency (ticket claiming races, database lock starvation) and context management (agents losing track of work mid-task). Both are preventable with atomic operations, proper transaction handling, and append-only work logs. The research confidence is high because the technology stack is mature, the domain patterns are well-documented, and the pitfalls are known from distributed systems literature.

## Key Findings

### Recommended Stack

Research confirms Rust as the optimal choice for CLI tools requiring single-binary distribution and zero runtime dependencies. The rusqlite + clap + anyhow stack is industry standard, with rusqlite's bundled feature eliminating all external dependencies. This matters for agent deployment where environments may lack system libraries.

**Core technologies:**
- **Rust 1.90+**: Single binary distribution, memory safety guarantees, excellent CLI ecosystem
- **clap 4.5.x**: Industry standard CLI parsing with derive macros, auto-generated help, type-safe arguments
- **rusqlite 0.38.0+**: Lightweight SQLite bindings with bundled feature for zero external deps, full SQLite feature access
- **anyhow 2.0+**: Application-level error handling with context chains, works seamlessly with any Error type
- **serde 1.0.228+**: Serialization framework for export formats (JSON/plain text)

**Why rusqlite over diesel:** For single-table CRUD operations, rusqlite avoids migration overhead, compile-time query checking complexity, and schema.rs maintenance burden. Diesel would add significant complexity for minimal benefit. The bundled feature is critical—provides static linking to SQLite without system dependencies.

### Expected Features

Research identified 12 features for v1 MVP, with clear table stakes vs. differentiators. The key insight: agents don't need JIRA-level complexity; they need speed, clarity, and coordination primitives.

**Must have (table stakes):**
- Ticket CRUD operations (create, read, update, delete)
- Unique ticket IDs (auto-incrementing for reference)
- Status field (todo/WIP/blocked/done)
- Description field (task context)
- Timestamps (created_at, updated_at)
- List/filter tickets (query for available work)
- Search by name/description (find relevant tickets)
- SQLite storage (persistent, file-based, no server)
- Plain text export format (agent-optimized, token-efficient)

**Should have (competitive advantages):**
- **Soft claiming mechanism** (reassignable ownership for agent coordination)
- **Dependency tracking** (informational only, no hard enforcement)
- **Agent-optimized export** (minimizes token usage, only essential fields)
- **Short CLI aliases** (speed-optimized: new, ls, claim)
- **Virtual tags** (+BLOCKED, +BLOCKING computed from state)
- **Context system** (auto-filtering based on working context)

**Defer (v2+):**
- Git-based sync (requires conflict resolution, complex)
- Offline-first with sync (massive complexity for agents)
- Advanced analytics/reporting (wait for usage patterns)
- Integrations (GitHub, JIRA) (add when needs emerge)

**Anti-features to avoid:**
- Hard dependency enforcement (agents need flexibility to work out of order)
- Real-time collaboration/websockets (adds complexity without value for polling agents)
- Hierarchical tasks/subtasks (creates query complexity, use dependencies instead)
- Recurring tasks (edge case hell, use external scheduler)
- Time tracking built-in (feature bloat, many dedicated tools exist)

### Architecture Approach

Standard layered architecture with clear separation of concerns: CLI interface layer, business logic layer, data access layer, persistence layer. The key pattern is "thin main, rich library" where main.rs only handles CLI setup and lib.rs exposes testable modules. This enables comprehensive testing without spawning processes.

**Major components:**
1. **CLI Interface Layer** (src/cli/) — Clap-based argument parsing, command routing, output formatting. Each command is a separate module with dedicated args struct.
2. **Business Logic Layer** (src/domain/) — Ticket entity, validation, state machine for status transitions, dependency resolution. Pure Rust with no database/CLI dependencies for testability.
3. **Data Access Layer** (src/data/) — Repository pattern abstracts SQLite operations, query builders for search/filter, prepared statements for performance.
4. **Persistence Layer** — SQLite with rusqlite, schema migrations, connection management, transaction handling. WAL mode for concurrent reads during writes.

**Build order recommendation:** Start with persistence foundation (database config, migrations, basic repository), then domain core (ticket entity, validation, operations), then CLI interface (one command end-to-end), finally expand features. This enables vertical slices where one complete command works before adding others.

### Critical Pitfalls

Research identified six critical pitfalls that have sunk similar projects. All are preventable with correct patterns from day one.

1. **SQLite Concurrent Write Starvation** — Multiple agents competing for claims can starve unlucky processes. Enable WAL mode immediately, use IMMEDIATE transactions (not DEFERRED), set busy timeout to several seconds. Keep transactions under 100ms. Test with 10 parallel claim operations.

2. **Race Condition on Ticket Claiming** — Classic check-then-act: two agents query for available tickets, both see same unclaimed ticket, both attempt claim. Use single atomic UPDATE with WHERE conditions: `UPDATE tickets SET claimed_by = ? WHERE id = ? AND claimed_by IS NULL`. Check affected row count to verify success. Never SELECT then UPDATE.

3. **Lost Context Window Recovery** — Agent crashes mid-task but ticket description doesn't contain enough detail to resume. Support append-only work log (not just status changes). Agents write "Starting X" before X, "Completed X" after. Export must include full history, not just current snapshot.

4. **Circular Dependency Deadlock** — Ticket A blocks on B, B blocks on C, C blocks on A. All stuck in "blocked" status. Implement cycle detection on dependency insertion using depth-first search with path tracking. Reject if creates cycle with clear error message.

5. **Broken Pipe Silent Failure** — Agent pipes output (`rtik ls | head -5`) but CLI continues writing after consumer hangs up. Rust's println! ignores EPIPE by default and panics. Use explicit stdout handle with error handling, return early on EPIPE. Test with `rtik ls | head -1` should exit cleanly.

6. **Silent Agent Failures** — Agent claims ticket, crashes, ticket remains claimed forever with no timeout. Add claimed_at timestamp field, show stale claims in list view (>1 hour highlighted), provide explicit release/reset-claim command for recovery.

## Implications for Roadmap

Based on research, suggested phase structure follows dependency order from architecture analysis:

### Phase 1: Database Foundation & Core CRUD
**Rationale:** Cannot build anything without persistence working correctly. Database configuration must be right from day one—fixing concurrency issues later is painful. This phase establishes the foundation all other features depend on.

**Delivers:**
- SQLite database with proper configuration (WAL mode, IMMEDIATE transactions, busy timeout)
- Schema with tickets table (id, name, description, status, created_at, updated_at)
- Basic repository pattern with create, read, update, delete operations
- CLI interface skeleton with one working command (create)

**Addresses:**
- Ticket CRUD (table stakes from FEATURES.md)
- Unique IDs (table stakes)
- Persistent storage (table stakes)
- Single binary distribution (differentiator)

**Avoids:**
- SQLite concurrent write starvation (Pitfall #1)
- Broken pipe silent failure (Pitfall #5)

**Stack elements:** rusqlite with bundled feature, clap for CLI parsing, anyhow for error handling

### Phase 2: Status Tracking & Claiming
**Rationale:** Status field and claiming mechanism are coupled—both involve state transitions and concurrent access. Implementing together allows testing the critical claim race condition thoroughly. Builds on Phase 1's database foundation.

**Delivers:**
- Status enum (todo/WIP/blocked/done) with validation
- Status transition state machine in domain layer
- Soft claiming with atomic UPDATE and row count verification
- claimed_by and claimed_at fields
- List command with status filtering
- Claim and release commands

**Addresses:**
- Status field (table stakes)
- Soft claiming (differentiator)
- Timestamps (table stakes)

**Avoids:**
- Race condition on ticket claiming (Pitfall #2)
- Silent agent failures (Pitfall #6)

**Stack elements:** Domain-driven design patterns for state machine, transaction handling in rusqlite

### Phase 3: Dependency Tracking
**Rationale:** Dependencies are core to agent coordination but independent of claiming mechanism. After status and claiming work, dependency tracking adds relationship modeling. Must implement cycle detection from start—fixing circular deps after creation is messy.

**Delivers:**
- Dependencies table (ticket_id, depends_on_ticket_id)
- Cycle detection on dependency insertion (DFS with path tracking)
- Add/remove dependency commands
- List view shows dependency count
- Query for blocked tickets (has dependencies in todo/WIP state)

**Addresses:**
- Dependency tracking informational (differentiator from FEATURES.md)

**Avoids:**
- Circular dependency deadlock (Pitfall #4)

**Stack elements:** Graph algorithms in pure Rust, JOIN queries in rusqlite

### Phase 4: Search & Filtering
**Rationale:** Basic listing exists from Phase 2, but agents need powerful filtering to find relevant work. Search is independent of claiming and dependencies, can develop in parallel after core CRUD works.

**Delivers:**
- Full-text search on name/description fields
- Advanced filtering (by status, claimed_by, has-dependencies)
- Query builder for combining filters
- Short CLI aliases (ls for list, new for create)

**Addresses:**
- Search functionality (table stakes)
- List/filter tickets (table stakes)
- Short CLI aliases (differentiator)

**Avoids:**
- Performance trap: full table scan for search (add FTS5 index)

**Stack elements:** SQLite FTS5 for full-text search, query builder pattern

### Phase 5: Export Format & Context
**Rationale:** Export format depends on all data structures being finalized (status, dependencies, claims). Context system is enhancement over basic filtering. Both focused on agent consumption, makes sense to implement together.

**Delivers:**
- Plain text export format (token-efficient, essential fields only)
- JSON export option for programmatic parsing
- Export includes full work log history
- Context-aware filtering (auto-apply tags/filters based on working context)

**Addresses:**
- Plain text export (table stakes)
- Agent-optimized export format (differentiator)
- Context system (differentiator)

**Avoids:**
- Lost context window recovery (Pitfall #3)
- Performance trap: unbounded export (add --limit flag, default 100)

**Stack elements:** serde for JSON serialization, custom plain text formatter

### Phase 6: Polish & Hardening
**Rationale:** After features work, focus on production-readiness: error handling, validation, edge cases, documentation.

**Delivers:**
- Comprehensive error messages with context
- Input validation (length limits, SQL injection prevention)
- Database file permissions (0600)
- Stale claim detection and highlighting
- Exit code standards (0=success, 1=error, 2=usage)
- Integration tests for all commands

**Addresses:**
- All UX pitfalls (clear errors, proper exit codes, separation of stdout/stderr)
- Security mistakes (input validation, file permissions)

**Avoids:**
- All "looks done but isn't" checklist items

### Phase Ordering Rationale

- **Database first:** Nothing works without persistence. Fixing concurrency issues after initial implementation is expensive.
- **Status + claiming together:** Both involve state transitions and concurrent access. Testing claim races requires status filtering.
- **Dependencies after claiming:** Independent systems. Dependencies add complexity; get simpler claiming right first.
- **Search after core features:** Can't search until data structures are stable. Enhances existing list command.
- **Export last:** Needs all data structures finalized. Context system builds on filtering patterns established earlier.
- **Polish final:** Edge cases and error handling after feature set is complete.

Each phase delivers working functionality, not partial implementations. Vertical slices ensure something deployable exists after every phase.

### Research Flags

**Phases with standard patterns (skip research-phase):**
- **Phase 1-6:** All phases use well-documented patterns. Rust CLI development, SQLite concurrency, repository pattern, state machines, graph algorithms, and full-text search all have established best practices. Research has already identified the patterns to follow.

**No phases need deeper research** because this is a well-trodden domain. The research files provide comprehensive guidance on stack choices, architecture patterns, and pitfall avoidance. Implementation can proceed directly from research findings.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Rust + rusqlite + clap is industry standard for CLI tools. Versions verified against crates.io (clap 4.5.54, rusqlite 0.38.0, serde 1.0.228). Multiple high-quality sources confirm recommendations. |
| Features | HIGH | Feature analysis based on established CLI task managers (Taskwarrior, todo.txt, dstask) and agent coordination research. Clear separation of table stakes vs. differentiators. Anti-features well-documented in multiple sources. |
| Architecture | HIGH | Standard layered architecture with patterns from official Rust CLI Book, Martin Fowler's work on DDD layering, and proven examples. Repository pattern and thin-main-rich-library are established best practices. |
| Pitfalls | MEDIUM | Pitfalls identified from distributed systems literature, SQLite concurrency guides, and Rust CLI error handling sources. Medium confidence because some agent-specific pitfalls are inferred from general patterns rather than rtik-specific experience. |

**Overall confidence:** HIGH

The stack, features, and architecture have high confidence based on official documentation, established patterns, and multiple corroborating sources. Pitfalls are medium confidence—well-researched from general distributed systems and CLI tool development, but specific to agent coordination use case which is newer domain.

### Gaps to Address

**Agent coordination patterns validation:** While distributed systems research provides strong foundation for claim mechanisms and concurrency handling, the specific pattern of multiple LLM agents coordinating via CLI is newer. During implementation, watch for agent-specific edge cases not covered by traditional multi-user concurrency patterns. Validate with multi-agent stress testing early.

**Token efficiency measurement:** Research recommends token-efficient export format but doesn't quantify what "efficient" means. During Phase 5, measure actual token usage with different format options (full export vs. minimal fields) to validate optimization is meaningful. Consider agent context window size (8k, 32k, 200k) when designing format.

**Context window recovery completeness:** Research identifies append-only work log as solution to lost context, but doesn't specify what level of detail enables effective resume. During Phase 3 implementation, test with real context resets—can agent actually resume from export, or are additional fields needed?

**Stale claim threshold:** Research recommends highlighting stale claims but doesn't specify threshold (1 hour, 4 hours, 24 hours). This likely depends on agent execution patterns. Start with configurable default, adjust based on real usage.

## Sources

All sources aggregated from research files. Full citations available in individual research documents.

### Primary (HIGH confidence)
- Official Rust CLI Book — CLI structure, error handling patterns, argument parsing
- clap crates.io (v4.5.54) — Version verification, API patterns
- rusqlite crates.io (v0.38.0) — Version verification, bundled feature documentation
- SQLite official docs — Concurrency, WAL mode, transaction types
- Martin Fowler (Presentation Domain Data Layering) — Architecture patterns
- Anthropic AI Engineering Blog — Context engineering for AI agents

### Secondary (MEDIUM confidence)
- Taskwarrior documentation — Established CLI task manager patterns
- Multiple Rust CLI recommendations articles (2025-2026) — Stack validation
- Medium articles on SQLite concurrency (2025) — Four ways to handle concurrency, abusing SQLite patterns
- Distributed systems race condition articles — Claim mechanism patterns
- Todo.txt and dstask projects — Alternative approaches, feature comparisons

### Tertiary (sources for awareness)
- Various CLI UX guidelines — Output formatting, exit codes
- GitHub discussions on Rust CLI error handling — Broken pipe handling
- Blog posts on agent orchestration — Multi-agent coordination strategies

**Source quality notes:** Stack recommendations verified against official crates.io. Architecture patterns from authoritative sources (Rust Book, Martin Fowler). Pitfalls draw from mix of official SQLite docs (high confidence) and distributed systems articles (medium confidence). No low-confidence sources used for critical decisions.

---
*Research completed: 2026-02-22*
*Ready for roadmap: yes*
